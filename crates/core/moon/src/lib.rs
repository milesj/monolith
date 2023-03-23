use moon_deno_platform::DenoPlatform;
use moon_dep_graph::DepGraphBuilder;
use moon_error::MoonError;
use moon_node_platform::NodePlatform;
use moon_project_graph::{ProjectGraph, ProjectGraphBuilder, ProjectGraphError};
use moon_system_platform::SystemPlatform;
use moon_utils::{is_test_env, json};
use moon_workspace::{Workspace, WorkspaceError};
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static TELEMETRY: AtomicBool = AtomicBool::new(true);

pub fn is_telemetry_enabled() -> bool {
    TELEMETRY.load(Ordering::Relaxed)
}

pub fn register_platforms(workspace: &mut Workspace) -> Result<(), WorkspaceError> {
    if let Some(deno_config) = &workspace.toolchain_config.deno {
        workspace.register_platform(Box::new(DenoPlatform::new(
            deno_config,
            &workspace.toolchain_config.typescript,
            &workspace.root,
        )));
    }

    if let Some(node_config) = &workspace.toolchain_config.node {
        workspace.register_platform(Box::new(NodePlatform::new(
            node_config,
            &workspace.toolchain_config.typescript,
            &workspace.root,
        )));
    }

    // Should be last since it's the most common
    workspace.register_platform(Box::<SystemPlatform>::default());

    Ok(())
}

async fn setup_workspace(mut workspace: Workspace) -> Result<Workspace, WorkspaceError> {
    if !workspace.config.telemetry {
        TELEMETRY.store(false, Ordering::Relaxed);
    }

    register_platforms(&mut workspace)?;

    if !is_test_env() {
        if workspace.vcs.is_enabled() {
            if let Ok(slug) = workspace.vcs.get_repository_slug().await {
                env::set_var("MOON_REPO_SLUG", slug);
            }
        }

        workspace.signin_to_moonbase().await?;
    }

    Ok(workspace)
}

/// Loads the workspace from the current working directory.
pub async fn load_workspace() -> Result<Workspace, WorkspaceError> {
    setup_workspace(Workspace::load()?).await
}

/// Loads the workspace from a provided directory.
pub async fn load_workspace_from(path: &Path) -> Result<Workspace, WorkspaceError> {
    setup_workspace(Workspace::load_from(path)?).await
}

// Some commands require the toolchain to exist, but don't use
// the action pipeline. This is a simple flow to wire up the tools.
pub async fn load_workspace_with_toolchain() -> Result<Workspace, WorkspaceError> {
    let mut workspace = load_workspace().await?;

    for platform in workspace.platforms.list_mut() {
        platform
            .setup_toolchain()
            .await
            .map_err(|e| WorkspaceError::Moon(MoonError::Generic(e.to_string())))?;
    }

    Ok(workspace)
}

pub fn build_dep_graph<'g>(
    workspace: &'g Workspace,
    project_graph: &'g ProjectGraph,
) -> DepGraphBuilder<'g> {
    DepGraphBuilder::new(&workspace.platforms, project_graph)
}

pub async fn build_project_graph(
    workspace: &mut Workspace,
) -> Result<ProjectGraphBuilder, ProjectGraphError> {
    ProjectGraphBuilder::new(workspace).await
}

pub async fn generate_project_graph(
    workspace: &mut Workspace,
) -> Result<ProjectGraph, ProjectGraphError> {
    let cache_path = workspace.cache.get_state_path("projectGraph.json");
    let mut builder = build_project_graph(workspace).await?;

    if builder.is_cached && cache_path.exists() {
        let graph: ProjectGraph = json::read(&cache_path)?;

        return Ok(graph);
    }

    builder.load_all()?;

    let graph = builder.build()?;

    if !builder.hash.is_empty() {
        json::write(&cache_path, &graph, false)?;
    }

    Ok(graph)
}
