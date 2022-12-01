use moon_config::PlatformType;
use moon_dep_graph::DepGraphBuilder;
use moon_node_platform::NodePlatform;
use moon_project_graph::{ProjectError, ProjectGraph, ProjectGraphBuilder};
use moon_system_platform::SystemPlatform;
use moon_workspace::{Workspace, WorkspaceError};
use rustc_hash::FxHashMap;
use strum::IntoEnumIterator;

/// Loads the workspace and registers all available platforms!
pub async fn load_workspace() -> Result<Workspace, WorkspaceError> {
    let mut workspace = Workspace::load().await?;

    workspace.register_platform(Box::new(SystemPlatform::default()));

    if let Some(node_config) = &workspace.toolchain.config.node {
        workspace.register_platform(Box::new(NodePlatform::new(node_config, &workspace.root)));
    }

    workspace.signin_to_moonbase().await?;

    Ok(workspace)
}

// Some commands require the toolchain to exist, but don't use
// the action runner. This is a simple flow to wire up the tools.
pub async fn load_workspace_with_toolchain() -> Result<Workspace, WorkspaceError> {
    let mut workspace = load_workspace().await?;
    let mut last_versions = FxHashMap::default();

    // Use exhaustive checks so we don't miss a platform
    for platform in PlatformType::iter() {
        match platform {
            PlatformType::Node => {
                if let Some(node_config) = &workspace.toolchain.config.node {
                    workspace
                        .toolchain
                        .node
                        .setup(&node_config.version, &mut last_versions)
                        .await?;
                }
            }
            PlatformType::System | PlatformType::Unknown => {}
        }
    }

    Ok(workspace)
}

pub fn build_dep_graph<'g>(
    workspace: &'g Workspace,
    project_graph: &'g ProjectGraph,
) -> DepGraphBuilder<'g> {
    DepGraphBuilder::new(&workspace.platforms, project_graph)
}

pub async fn generate_project_graph(
    workspace: &mut Workspace,
) -> Result<ProjectGraph, ProjectError> {
    let mut builder = ProjectGraphBuilder {
        cache: &workspace.cache,
        config: &workspace.projects_config,
        platforms: &mut workspace.platforms,
        workspace_config: &workspace.config,
        workspace_root: &workspace.root,
    };

    Ok(builder.build().await?)
}
