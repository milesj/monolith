use crate::actions::install_deps::install_deps;
use crate::actions::run_target::run_target;
use crate::actions::setup_tool::setup_tool;
use crate::actions::sync_project::sync_project;
use crate::errors::PipelineError;
use moon_action::{Action, ActionNode, ActionStatus};
use moon_action_context::ActionContext;
use moon_emitter::{Emitter, Event};
use moon_logger::{color, debug, error, trace};
use moon_project_graph::ProjectGraph;
use moon_task::Target;
use moon_workspace::Workspace;
use std::sync::Arc;
use tokio::sync::RwLock;

fn extract_error<T>(result: &Result<T, PipelineError>) -> Option<String> {
    match result {
        Ok(_) => None,
        Err(error) => Some(error.to_string()),
    }
}

pub async fn process_action(
    action: &mut Action,
    context: Arc<RwLock<ActionContext>>,
    emitter: Arc<RwLock<Emitter>>,
    workspace: Arc<RwLock<Workspace>>,
    project_graph: Arc<RwLock<ProjectGraph>>,
) -> Result<(), PipelineError> {
    trace!(
        target: &action.log_target,
        "Running action {}",
        color::muted_light(&action.label)
    );

    let local_emitter = Arc::clone(&emitter);
    let local_emitter = local_emitter.read().await;

    let local_project_graph = Arc::clone(&project_graph);
    let local_project_graph = local_project_graph.read().await;

    let node = action.node.take().unwrap();
    let result = match &node {
        // Setup and install the specific tool
        ActionNode::SetupTool(runtime) => {
            local_emitter
                .emit(Event::ToolInstalling { runtime })
                .await?;

            let setup_result = setup_tool(action, context, workspace, runtime).await;

            local_emitter
                .emit(Event::ToolInstalled {
                    error: extract_error(&setup_result),
                    runtime,
                })
                .await?;

            setup_result
        }

        // Install dependencies in the workspace root
        ActionNode::InstallDeps(runtime) => {
            local_emitter
                .emit(Event::DependenciesInstalling {
                    project: None,
                    runtime,
                })
                .await?;

            let install_result = install_deps(action, context, workspace, runtime, None).await;

            local_emitter
                .emit(Event::DependenciesInstalled {
                    error: extract_error(&install_result),
                    project: None,
                    runtime,
                })
                .await?;

            install_result
        }

        // Install dependencies in the project root
        ActionNode::InstallProjectDeps(runtime, project_id) => {
            let project = local_project_graph.get(project_id)?;

            local_emitter
                .emit(Event::DependenciesInstalling {
                    project: Some(project),
                    runtime,
                })
                .await?;

            let install_result =
                install_deps(action, context, workspace, runtime, Some(project)).await;

            local_emitter
                .emit(Event::DependenciesInstalled {
                    error: extract_error(&install_result),
                    project: Some(project),
                    runtime,
                })
                .await?;

            install_result
        }

        // Sync a project within the graph
        ActionNode::SyncProject(runtime, project_id) => {
            let project = local_project_graph.get(project_id)?;

            local_emitter
                .emit(Event::ProjectSyncing { project, runtime })
                .await?;

            let sync_result =
                sync_project(action, context, workspace, project_graph, project, runtime).await;

            local_emitter
                .emit(Event::ProjectSynced {
                    error: extract_error(&sync_result),
                    project,
                    runtime,
                })
                .await?;

            sync_result
        }

        // Run a task within a project
        ActionNode::RunTarget(target_id) => {
            let target = Target::parse(target_id)?;
            let project = local_project_graph.get(target.project_id.as_ref().unwrap())?;

            local_emitter
                .emit(Event::TargetRunning { target: &target })
                .await?;

            let run_result = run_target(action, context, workspace, project, &target).await;

            local_emitter
                .emit(Event::TargetRan {
                    error: extract_error(&run_result),
                    target: &target,
                })
                .await?;

            run_result
        }
    };

    match result {
        Ok(status) => {
            action.done(status);
        }
        Err(error) => {
            action.fail(error.to_string());

            // If these fail, we should abort instead of trying to continue
            if matches!(node, ActionNode::SetupTool(_))
                || matches!(node, ActionNode::InstallDeps(_))
            {
                action.abort();
            }
        }
    }

    Ok(())
}
