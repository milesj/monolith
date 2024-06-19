use ci_env::get_environment;
use moon_action::ActionStatus;
use moon_api::graphql::{
    self, add_job_to_run, create_run, update_job, update_run, AddJobToRun, CreateRun, GraphQLQuery,
    UpdateJob, UpdateRun,
};
use moon_api::Moonbase;
use moon_app_context::AppContext;
use moon_common::is_ci;
use moon_emitter::{Event, EventFlow, Subscriber};
use moon_logger::{debug, error, map_list, warn};
use moon_utils::async_trait;
use rustc_hash::FxHashMap;
use starbase_styles::color;
use std::env;
use tokio::task::JoinHandle;

const LOG_TARGET: &str = "moonbase";

pub struct MoonbaseSubscriber {
    // Mapping of actions to job IDs
    job_ids: FxHashMap<String, i64>,

    // Upstream database record ID
    run_id: Option<i64>,

    // In-flight requests
    requests: Vec<JoinHandle<()>>,
}

impl MoonbaseSubscriber {
    pub fn new() -> Self {
        MoonbaseSubscriber {
            job_ids: FxHashMap::default(),
            run_id: None,
            requests: vec![],
        }
    }

    async fn update_run(
        &self,
        run_id: &i64,
        auth_token: &str,
        input: update_run::UpdateRunInput,
    ) -> miette::Result<()> {
        fn log_failure(id: &i64, message: String) {
            warn!(
                target: LOG_TARGET,
                "Failed to update CI run {}. Failure: {}",
                id,
                color::muted_light(message)
            );
        }

        let Ok(response) = graphql::post_mutation::<update_run::ResponseData>(
            UpdateRun::build_query(update_run::Variables { id: *run_id, input }),
            Some(auth_token),
        )
        .await
        else {
            return Ok(());
        };

        match (response.data, response.errors) {
            (_, Some(errors)) => {
                log_failure(run_id, map_list(&errors, |e| e.message.to_owned()));
            }
            (Some(data), _) => {
                if !data.update_run.user_errors.is_empty() {
                    log_failure(
                        run_id,
                        map_list(&data.update_run.user_errors, |e| e.message.to_owned()),
                    );
                }
            }
            _ => {}
        };

        Ok(())
    }
}

#[async_trait]
impl Subscriber for MoonbaseSubscriber {
    async fn on_emit<'a>(
        &mut self,
        event: &Event<'a>,
        app_context: &AppContext,
    ) -> miette::Result<EventFlow> {
        let Some(moonbase) = Moonbase::session() else {
            return Ok(EventFlow::Continue);
        };

        // CI INSIGHTS

        if moonbase.ci_insights_enabled && is_ci() {
            match event {
                // We must wait for this request to finish before firing off other requests,
                // as we require the run ID from the record saved upstream!
                Event::PipelineStarted {
                    actions_count,
                    context,
                } => {
                    debug!(
                        target: LOG_TARGET,
                        "Pipeline started, attempting to create CI run in moonbase"
                    );

                    fn log_failure(message: String) {
                        error!(
                            target: LOG_TARGET,
                            "Failed to create CI run in moonbase, will not track running jobs. Failure: {}",
                            color::muted_light(message)
                        );
                    }

                    let mut branch = env::var("MOONBASE_CI_BRANCH").unwrap_or_default();
                    let mut revision = env::var("MOONBASE_CI_REVISION").unwrap_or_default();
                    let mut request_number = env::var("MOONBASE_CI_REQUEST_NUMBER").ok();

                    if let Some(ci) = get_environment() {
                        if branch.is_empty() {
                            branch = ci.branch;
                        }

                        if revision.is_empty() {
                            revision = ci.revision;
                        }

                        if request_number.is_none() {
                            request_number = ci.request_id;
                        }
                    }

                    if branch.is_empty() {
                        branch = (*app_context.vcs.get_local_branch().await?).clone();
                    }

                    if revision.is_empty() {
                        revision = (*app_context.vcs.get_local_branch_revision().await?).clone();
                    }

                    let affected_targets = context
                        .primary_targets
                        .iter()
                        .map(|t| t.id.clone())
                        .collect::<Vec<_>>();

                    let touched_files = context
                        .touched_files
                        .iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<_>>();

                    let response = match graphql::post_mutation::<create_run::ResponseData>(
                        CreateRun::build_query(create_run::Variables {
                            input: create_run::CreateRunInput {
                                affected_targets: if affected_targets.is_empty() {
                                    None
                                } else {
                                    Some(affected_targets)
                                },
                                branch,
                                job_count: *actions_count as i64,
                                repository_id: moonbase.repository_id as i64,
                                request_number,
                                revision: Some(revision),
                                touched_files: if touched_files.is_empty() {
                                    None
                                } else {
                                    Some(touched_files)
                                },
                            },
                        }),
                        Some(&moonbase.auth_token),
                    )
                    .await
                    {
                        Ok(res) => res,

                        // If the request fails, dont crash the entire pipeline!
                        Err(error) => {
                            log_failure(error.to_string());

                            return Ok(EventFlow::Continue);
                        }
                    };

                    match (response.data, response.errors) {
                        (_, Some(errors)) => {
                            log_failure(map_list(&errors, |e| e.message.to_owned()));
                        }
                        (Some(data), _) => {
                            if data.create_run.user_errors.is_empty() {
                                let id = data.create_run.run.unwrap().id;

                                debug!(
                                    target: LOG_TARGET,
                                    "CI run created in moonbase (id = {})", id,
                                );

                                self.run_id = Some(id);
                            } else {
                                log_failure(map_list(&data.create_run.user_errors, |e| {
                                    e.message.to_owned()
                                }));
                            }
                        }
                        _ => {}
                    };
                }

                // Update the status and duration when the pipeline finishes!
                Event::PipelineFinished {
                    baseline_duration,
                    duration,
                    failed_count,
                    ..
                } => {
                    if let Some(run_id) = &self.run_id {
                        self.update_run(
                            run_id,
                            &moonbase.auth_token,
                            update_run::UpdateRunInput {
                                comparison_duration: Some(baseline_duration.as_millis() as i64),
                                duration: Some(duration.as_millis() as i64),
                                status: Some(if *failed_count > 0 {
                                    update_run::RunStatus::FAILED
                                } else {
                                    update_run::RunStatus::PASSED
                                }),
                            },
                        )
                        .await?
                    }
                }

                // Update the status when the pipeline aborts!
                Event::PipelineAborted { .. } => {
                    if let Some(run_id) = &self.run_id {
                        self.update_run(
                            run_id,
                            &moonbase.auth_token,
                            update_run::UpdateRunInput {
                                comparison_duration: None,
                                duration: None,
                                status: Some(update_run::RunStatus::ABORTED),
                            },
                        )
                        .await?
                    }
                }

                // Actions map to jobs in moonbase, so create a job record for each action.
                // We also need to wait for these requests so that we can extract the job ID.
                Event::ActionStarted { action, .. } => {
                    fn log_failure(message: String) {
                        warn!(
                            target: LOG_TARGET,
                            "Failed to create job for CI run. Failure: {}",
                            color::muted_light(message)
                        );
                    }

                    if let Some(run_id) = &self.run_id {
                        let Ok(response) = graphql::post_mutation::<add_job_to_run::ResponseData>(
                            AddJobToRun::build_query(add_job_to_run::Variables {
                                input: add_job_to_run::CreateJobInput {
                                    run_id: *run_id,
                                    action: action.label.clone(),
                                    started_at: action
                                        .started_at
                                        .expect("Missing start time for action!"),
                                },
                            }),
                            Some(&moonbase.auth_token),
                        )
                        .await
                        else {
                            return Ok(EventFlow::Continue);
                        };

                        match (response.data, response.errors) {
                            (_, Some(errors)) => {
                                log_failure(map_list(&errors, |e| e.message.to_owned()));
                            }
                            (Some(data), _) => {
                                if data.add_job_to_run.user_errors.is_empty() {
                                    self.job_ids.insert(
                                        action.label.clone(),
                                        data.add_job_to_run.job.unwrap().id,
                                    );
                                } else {
                                    log_failure(map_list(&data.add_job_to_run.user_errors, |e| {
                                        e.message.to_owned()
                                    }));
                                }
                            }
                            _ => {}
                        };
                    }
                }

                // When an action finishes, update the job with the final state!
                Event::ActionFinished { action, .. } => {
                    fn log_failure(message: String) {
                        warn!(
                            target: LOG_TARGET,
                            "Failed to update job for CI run. Failure: {}",
                            color::muted_light(message)
                        );
                    }

                    if let Some(job_id) = self.job_ids.get(&action.label) {
                        let mut input = update_job::UpdateJobInput {
                            attempts: None,
                            duration: action.duration.map(|d| d.as_millis() as i64),
                            finished_at: Some(
                                action.finished_at.expect("Missing finish time for action!"),
                            ),
                            status: Some(map_status(&action.status)),
                        };

                        let attempts = action
                            .operations
                            .iter()
                            .filter(|op| op.meta.is_task_execution())
                            .collect::<Vec<_>>();

                        if !attempts.is_empty() {
                            input.attempts = Some(
                                attempts
                                    .iter()
                                    .map(|at| update_job::JobAttemptInput {
                                        duration: at
                                            .duration
                                            .map(|d| d.as_millis() as i64)
                                            .unwrap_or_default(),
                                        finished_at: at
                                            .finished_at
                                            .expect("Missing finish time for attempt!"),
                                        started_at: at.started_at,
                                        status: map_status(&at.status),
                                    })
                                    .collect::<Vec<_>>(),
                            );
                        }

                        let variables = update_job::Variables { id: *job_id, input };
                        let auth_token = moonbase.auth_token.clone();

                        // Run the update in a background thread!
                        self.requests.push(tokio::spawn(async move {
                            if let Ok(response) =
                                graphql::post_mutation::<update_job::ResponseData>(
                                    UpdateJob::build_query(variables),
                                    Some(&auth_token),
                                )
                                .await
                            {
                                match (response.data, response.errors) {
                                    (_, Some(errors)) => {
                                        log_failure(map_list(&errors, |e| e.message.to_owned()));
                                    }
                                    (Some(data), _) => {
                                        if !data.update_job.user_errors.is_empty() {
                                            log_failure(map_list(
                                                &data.update_job.user_errors,
                                                |e| e.message.to_owned(),
                                            ));
                                        }
                                    }
                                    _ => {}
                                };
                            }
                        }));
                    }
                }

                Event::TargetRunning { action, target } => {
                    // Temporary, pass this data to the moonbase instance
                    if let Some(job_id) = self.job_ids.get(&action.label) {
                        moonbase
                            .job_ids
                            .write()
                            .await
                            .insert(target.to_string(), *job_id);
                    }
                }

                _ => {}
            }
        }

        // For the last event, we want to ensure that all requests have been completed!
        if event.is_end() {
            for future in self.requests.drain(0..) {
                let _ = future.await;
            }

            moonbase.wait_for_requests().await;
        }

        Ok(EventFlow::Continue)
    }
}

fn map_status(status: &ActionStatus) -> update_job::JobStatus {
    match status {
        ActionStatus::Cached | ActionStatus::CachedFromRemote => update_job::JobStatus::CACHED,
        ActionStatus::Aborted | ActionStatus::Failed | ActionStatus::TimedOut => {
            update_job::JobStatus::FAILED
        }
        ActionStatus::Invalid | ActionStatus::Passed => update_job::JobStatus::PASSED,
        ActionStatus::Running => update_job::JobStatus::RUNNING,
        ActionStatus::Skipped => update_job::JobStatus::SKIPPED,
    }
}
