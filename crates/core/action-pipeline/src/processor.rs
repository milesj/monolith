use crate::actions::install_deps::install_deps;
use crate::actions::run_task::run_task;
use crate::actions::setup_tool::setup_tool;
use crate::actions::sync_project::sync_project;
use crate::actions::sync_workspace::sync_workspace;
use moon_action::{Action, ActionNode};
use moon_action_context::ActionContext;
use moon_emitter::{Emitter, Event};
use moon_logger::trace;
use moon_project_graph::ProjectGraph;
use moon_workspace::Workspace;
use starbase_styles::color;
use std::sync::Arc;
use tokio::sync::RwLock;

fn extract_error<T>(result: &miette::Result<T>) -> Option<String> {
    match result {
        Ok(_) => None,
        Err(error) => Some(error.to_string()),
    }
}

pub async fn process_action(
    mut action: Action,
    context: Arc<RwLock<ActionContext>>,
    emitter: Arc<RwLock<Emitter>>,
    workspace: Arc<RwLock<Workspace>>,
    project_graph: Arc<RwLock<ProjectGraph>>,
) -> miette::Result<Action> {
    action.start();

    let node = action.node.take().unwrap();
    let log_action_label = color::muted_light(&action.label);

    trace!(
        target: &action.log_target,
        "Processing action {}",
        log_action_label
    );

    let local_emitter = Arc::clone(&emitter);
    let local_emitter = local_emitter.read().await;

    let local_project_graph = Arc::clone(&project_graph);
    let local_project_graph = local_project_graph.read().await;

    local_emitter
        .emit(Event::ActionStarted {
            action: &action,
            node: &node,
        })
        .await?;

    let result = match &node {
        // Setup and install the specific tool
        ActionNode::SetupTool { runtime } => {
            local_emitter
                .emit(Event::ToolInstalling { runtime })
                .await?;

            let setup_result = setup_tool(&mut action, context, workspace, runtime).await;

            local_emitter
                .emit(Event::ToolInstalled {
                    error: extract_error(&setup_result),
                    runtime,
                })
                .await?;

            setup_result
        }

        // Install dependencies in the workspace root
        ActionNode::InstallDeps { runtime } => {
            local_emitter
                .emit(Event::DependenciesInstalling {
                    project: None,
                    runtime,
                })
                .await?;

            let install_result = install_deps(&mut action, context, workspace, runtime, None).await;

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
        ActionNode::InstallProjectDeps {
            runtime,
            project: project_id,
        } => {
            let project = local_project_graph.get(project_id)?;

            local_emitter
                .emit(Event::DependenciesInstalling {
                    project: Some(&project),
                    runtime,
                })
                .await?;

            let install_result =
                install_deps(&mut action, context, workspace, runtime, Some(&project)).await;

            local_emitter
                .emit(Event::DependenciesInstalled {
                    error: extract_error(&install_result),
                    project: Some(&project),
                    runtime,
                })
                .await?;

            install_result
        }

        // Sync a project within the graph
        ActionNode::SyncProject {
            runtime,
            project: project_id,
        } => {
            let project = local_project_graph.get(project_id)?;

            local_emitter
                .emit(Event::ProjectSyncing {
                    project: &project,
                    runtime,
                })
                .await?;

            let sync_result = sync_project(
                &mut action,
                context,
                workspace,
                project_graph,
                &project,
                runtime,
            )
            .await;

            local_emitter
                .emit(Event::ProjectSynced {
                    error: extract_error(&sync_result),
                    project: &project,
                    runtime,
                })
                .await?;

            sync_result
        }

        // Sync the workspace
        ActionNode::SyncWorkspace => {
            local_emitter.emit(Event::WorkspaceSyncing).await?;

            let sync_result = sync_workspace(&mut action, context, workspace, project_graph).await;

            local_emitter
                .emit(Event::WorkspaceSynced {
                    error: extract_error(&sync_result),
                })
                .await?;

            sync_result
        }

        // Run a task within a project
        ActionNode::RunTask {
            runtime, target, ..
        } => {
            let project = local_project_graph.get(target.get_project_id().unwrap())?;

            local_emitter.emit(Event::TargetRunning { target }).await?;

            let run_result = run_task(
                &mut action,
                context,
                emitter,
                workspace,
                &project,
                target,
                runtime,
            )
            .await;

            local_emitter
                .emit(Event::TargetRan {
                    error: extract_error(&run_result),
                    target,
                })
                .await?;

            run_result
        }
    };

    let error_message = extract_error(&result);

    match result {
        Ok(status) => {
            action.finish(status);
        }
        Err(error) => {
            action.fail(error);
        }
    };

    if action.has_failed() {
        // If these fail, we should abort instead of trying to continue
        if matches!(
            node,
            ActionNode::SetupTool { .. } | ActionNode::InstallDeps { .. }
        ) {
            action.abort();
        }
    }

    local_emitter
        .emit(Event::ActionFinished {
            action: &action,
            error: error_message,
            node: &node,
        })
        .await?;

    if action.has_failed() {
        trace!(
            target: &action.log_target,
            "Failed to process action {} in {:?}",
            log_action_label,
            action.duration.unwrap()
        );
    } else {
        trace!(
            target: &action.log_target,
            "Processed action {} in {:?}",
            log_action_label,
            action.duration.unwrap()
        );
    }

    // Reassign the node for reuse
    action.node = Some(node);

    Ok(action)
}
