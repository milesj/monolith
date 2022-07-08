use crate::dep_graph::DepGraph;
use crate::errors::{ActionRunnerError, DepGraphError};
use crate::node::Node;
use moon_action::{
    install_node_deps, run_target, setup_toolchain, sync_node_project, Action, ActionContext,
    ActionStatus,
};
use moon_lang::SupportedLanguage;
use moon_logger::{color, debug, error, trace};
use moon_workspace::Workspace;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::task;

const LOG_TARGET: &str = "moon:action-runner";

async fn run_action(
    node: &Node,
    action: &mut Action,
    context: &ActionContext,
    workspace: Arc<RwLock<Workspace>>,
) -> Result<(), ActionRunnerError> {
    let result = match node {
        Node::InstallDeps(lang) => match lang {
            SupportedLanguage::Node => install_node_deps(action, context, workspace).await,
            _ => Ok(ActionStatus::Passed),
        },
        Node::RunTarget(target_id) => run_target(action, context, workspace, target_id).await,
        Node::SetupToolchain => setup_toolchain(action, context, workspace).await,
        Node::SyncProject(lang, project_id) => match lang {
            SupportedLanguage::Node => {
                sync_node_project(action, context, workspace, project_id).await
            }
            _ => Ok(ActionStatus::Passed),
        },
    };

    match result {
        Ok(status) => {
            action.pass(status);
        }
        Err(error) => {
            action.fail(error.to_string());

            // If these fail, we should abort instead of trying to continue
            if matches!(node, Node::SetupToolchain) || matches!(node, Node::InstallDeps(_)) {
                action.abort();
            }
        }
    }

    Ok(())
}

pub struct ActionRunner {
    bail: bool,

    pub duration: Option<Duration>,

    workspace: Arc<RwLock<Workspace>>,
}

impl ActionRunner {
    pub fn new(workspace: Workspace) -> Self {
        debug!(target: LOG_TARGET, "Creating action runner");

        ActionRunner {
            bail: false,
            duration: None,
            workspace: Arc::new(RwLock::new(workspace)),
        }
    }

    pub fn bail_on_error(&mut self) -> &mut Self {
        self.bail = true;
        self
    }

    pub async fn cleanup(&self) -> Result<(), ActionRunnerError> {
        let workspace = self.workspace.read().await;

        // Delete all previously created runfiles
        trace!(target: LOG_TARGET, "Deleting stale runfiles");

        workspace.cache.delete_runfiles().await?;

        Ok(())
    }

    pub async fn run(
        &mut self,
        graph: DepGraph,
        context: ActionContext,
    ) -> Result<Vec<Action>, ActionRunnerError> {
        let start = Instant::now();
        let node_count = graph.graph.node_count();
        let batches = graph.sort_batched_topological()?;
        let batches_count = batches.len();
        let graph = Arc::new(RwLock::new(graph));
        let context = Arc::new(context);

        // Clean the runner state *before* running actions instead of after,
        // so that failing or broken builds can dig into and debug the state!
        self.cleanup().await?;

        debug!(
            target: LOG_TARGET,
            "Running {} actions across {} batches", node_count, batches_count
        );

        let mut results: Vec<Action> = vec![];

        for (b, batch) in batches.into_iter().enumerate() {
            let batch_count = b + 1;
            let batch_target_name = format!("{}:batch:{}", LOG_TARGET, batch_count);
            let actions_count = batch.len();

            trace!(
                target: &batch_target_name,
                "Running {} actions",
                actions_count
            );

            let mut action_handles = vec![];

            for (i, node_index) in batch.into_iter().enumerate() {
                let action_count = i + 1;
                let graph_clone = Arc::clone(&graph);
                let context_clone = Arc::clone(&context);
                let workspace_clone = Arc::clone(&self.workspace);

                action_handles.push(task::spawn(async move {
                    let mut action = Action::new(node_index.index(), None);
                    let own_graph = graph_clone.read().await;

                    if let Some(node) = own_graph.get_node_from_index(&node_index) {
                        action.label = Some(node.label());

                        let log_target_name =
                            format!("{}:batch:{}:{}", LOG_TARGET, batch_count, action_count);
                        let log_action_label = color::muted_light(&node.label());

                        trace!(
                            target: &log_target_name,
                            "Running action {}",
                            log_action_label
                        );

                        run_action(node, &mut action, &context_clone, workspace_clone).await?;

                        if action.has_failed() {
                            trace!(
                                target: &log_target_name,
                                "Failed to run action {} in {:?}",
                                log_action_label,
                                action.duration.unwrap()
                            );
                        } else {
                            trace!(
                                target: &log_target_name,
                                "Ran action {} in {:?}",
                                log_action_label,
                                action.duration.unwrap()
                            );
                        }
                    } else {
                        action.status = ActionStatus::Invalid;

                        return Err(ActionRunnerError::DepGraph(DepGraphError::UnknownNode(
                            node_index.index(),
                        )));
                    }

                    Ok(action)
                }));
            }

            // Wait for all actions in this batch to complete,
            // while also handling and propagating errors
            for handle in action_handles {
                match handle.await {
                    Ok(Ok(result)) => {
                        if result.should_abort() {
                            error!(
                                target: &batch_target_name,
                                "Encountered a critical error, aborting the action runner"
                            );
                        }

                        if self.bail && result.error.is_some() || result.should_abort() {
                            return Err(ActionRunnerError::Failure(result.error.unwrap()));
                        }

                        results.push(result);
                    }
                    Ok(Err(e)) => {
                        return Err(e);
                    }
                    Err(e) => {
                        return Err(ActionRunnerError::Failure(e.to_string()));
                    }
                }
            }
        }

        self.duration = Some(start.elapsed());

        debug!(
            target: LOG_TARGET,
            "Finished running {} actions in {:?}",
            node_count,
            self.duration.unwrap()
        );

        Ok(results)
    }
}
