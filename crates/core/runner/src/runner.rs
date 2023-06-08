use crate::target_hasher::TargetHasher;
use crate::{errors::RunnerError, inputs_collector};
use console::Term;
use moon_action::{ActionStatus, Attempt};
use moon_action_context::ActionContext;
use moon_cache::RunTargetState;
use moon_config::{TaskOptionAffectedFiles, TaskOutputStyle};
use moon_emitter::{Emitter, Event, EventFlow};
use moon_error::MoonError;
use moon_hasher::HashSet;
use moon_logger::{debug, warn};
use moon_platform_runtime::Runtime;
use moon_process::{args, output_to_error, output_to_string, Command, Output};
use moon_project::Project;
use moon_target::{TargetError, TargetScope};
use moon_task::{Task, TaskError};
use moon_terminal::{label_checkpoint, Checkpoint};
use moon_utils::{is_ci, is_test_env, path, time};
use moon_workspace::Workspace;
use rustc_hash::FxHashMap;
use starbase_styles::color;
use starbase_utils::glob;
use tokio::{
    task,
    time::{sleep, Duration},
};

const LOG_TARGET: &str = "moon:runner";

pub enum HydrateFrom {
    LocalCache,
    PreviousOutput,
    RemoteCache,
}

pub struct Runner<'a> {
    pub cache: RunTargetState,

    emitter: &'a Emitter,

    project: &'a Project,

    stderr: Term,

    stdout: Term,

    task: &'a Task,

    workspace: &'a Workspace,
}

impl<'a> Runner<'a> {
    pub fn new(
        emitter: &'a Emitter,
        workspace: &'a Workspace,
        project: &'a Project,
        task: &'a Task,
    ) -> Result<Runner<'a>, MoonError> {
        Ok(Runner {
            cache: workspace.cache.cache_run_target_state(&task.target)?,
            emitter,
            project,
            stderr: Term::buffered_stderr(),
            stdout: Term::buffered_stdout(),
            task,
            workspace,
        })
    }

    /// Cache outputs to the `.moon/cache/outputs` folder and to the cloud,
    /// so that subsequent builds are faster, and any local outputs
    /// can be hydrated easily.
    pub async fn archive_outputs(&self) -> Result<(), RunnerError> {
        let hash = &self.cache.hash;

        if hash.is_empty() || !self.is_archivable()? {
            return Ok(());
        }

        // Check that outputs actually exist
        if !self.task.outputs.is_empty() && !self.has_outputs()? {
            return Err(RunnerError::Task(TaskError::MissingOutput(
                self.task.target.id.clone(),
            )));
        }

        // If so, then cache the archive
        if let EventFlow::Return(archive_path) = self
            .emitter
            .emit(Event::TargetOutputArchiving {
                cache: &self.cache,
                hash,
                project: self.project,
                target: &self.task.target,
                task: self.task,
            })
            .await?
        {
            self.emitter
                .emit(Event::TargetOutputArchived {
                    archive_path: archive_path.into(),
                    hash,
                    project: self.project,
                    target: &self.task.target,
                    task: self.task,
                })
                .await?;
        }

        Ok(())
    }

    pub async fn hydrate(&self, from: HydrateFrom) -> Result<ActionStatus, RunnerError> {
        // Only hydrate when the hash is different from the previous build,
        // as we can assume the outputs from the previous build still exist?
        if matches!(from, HydrateFrom::LocalCache) || matches!(from, HydrateFrom::RemoteCache) {
            self.hydrate_outputs().await?;
        }

        let mut comments = vec![match from {
            HydrateFrom::LocalCache => "cached",
            HydrateFrom::RemoteCache => "cached from remote",
            HydrateFrom::PreviousOutput => "cached from previous run",
        }];

        if self.should_print_short_hash() {
            comments.push(self.get_short_hash());
        }

        self.print_checkpoint(Checkpoint::RunPassed, &comments)?;
        self.print_cache_item()?;
        self.flush_output()?;

        Ok(if matches!(from, HydrateFrom::RemoteCache) {
            ActionStatus::CachedFromRemote
        } else {
            ActionStatus::Cached
        })
    }

    /// If we are cached (hash match), hydrate the project with the
    /// cached task outputs found in the hashed archive.
    pub async fn hydrate_outputs(&self) -> Result<(), RunnerError> {
        let hash = &self.cache.hash;

        if hash.is_empty() {
            return Ok(());
        }

        // Hydrate outputs from the cache
        if let EventFlow::Return(archive_path) = self
            .emitter
            .emit(Event::TargetOutputHydrating {
                cache: &self.cache,
                hash,
                project: self.project,
                target: &self.task.target,
                task: self.task,
            })
            .await?
        {
            self.emitter
                .emit(Event::TargetOutputHydrated {
                    archive_path: archive_path.into(),
                    hash,
                    project: self.project,
                    target: &self.task.target,
                    task: self.task,
                })
                .await?;
        }

        // Update the run state with the new hash
        self.cache.save()?;

        Ok(())
    }

    /// Create a hasher that is shared amongst all platforms.
    /// Primarily includes task information.
    pub async fn hash_common_target(
        &self,
        context: &ActionContext,
        hashset: &mut HashSet,
    ) -> Result<(), RunnerError> {
        let vcs = &self.workspace.vcs;
        let task = &self.task;
        let project = &self.project;
        let workspace = &self.workspace;
        let mut hasher = TargetHasher::new();

        hasher.hash_project_deps(self.project.get_dependency_ids());
        hasher.hash_task(task);
        hasher.hash_task_deps(task, &context.target_hashes)?;

        if context.should_inherit_args(&task.target) {
            hasher.hash_args(&context.passthrough_args);
        }

        hasher.hash_inputs(
            inputs_collector::collect_and_hash_inputs(
                vcs,
                task,
                &project.root,
                &workspace.root,
                &workspace.config.hasher,
            )
            .await?,
        );

        hashset.hash(hasher);

        Ok(())
    }

    pub async fn create_command(
        &self,
        context: &ActionContext,
        runtime: &Runtime,
    ) -> Result<Command, RunnerError> {
        let workspace = &self.workspace;
        let project = &self.project;
        let task = &self.task;
        let working_dir = if task.options.run_from_workspace_root {
            &workspace.root
        } else {
            &project.root
        };

        debug!(
            target: LOG_TARGET,
            "Creating {} command (in working directory {})",
            color::label(&task.target),
            color::path(working_dir)
        );

        let mut command = self
            .workspace
            .platforms
            .get(task.platform)?
            .create_run_target_command(context, project, task, runtime, working_dir)
            .await?;

        command
            .cwd(working_dir)
            .envs(self.create_env_vars().await?)
            // We need to handle non-zero's manually
            .set_error_on_nonzero(false);

        // Wrap in a shell
        if task.platform.is_system() && task.options.shell {
            command.with_shell();
        }

        // Passthrough args
        if context.should_inherit_args(&self.task.target) {
            command.args(&context.passthrough_args);
        }

        // Terminal colors
        if self.workspace.config.runner.inherit_colors_for_piped_tasks {
            command.inherit_colors();
        }

        // Affected files (must be last args)
        if let Some(check_affected) = &self.task.options.affected_files {
            let mut affected_files = if context.affected_only {
                self.task
                    .get_affected_files(&context.touched_files, &self.project.source)?
            } else {
                Vec::with_capacity(0)
            };

            affected_files.sort();

            if matches!(
                check_affected,
                TaskOptionAffectedFiles::Env | TaskOptionAffectedFiles::Enabled(true)
            ) {
                command.env(
                    "MOON_AFFECTED_FILES",
                    if affected_files.is_empty() {
                        ".".into()
                    } else {
                        affected_files
                            .iter()
                            .map(|f| f.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(",")
                    },
                );
            }

            if matches!(
                check_affected,
                TaskOptionAffectedFiles::Args | TaskOptionAffectedFiles::Enabled(true)
            ) {
                if affected_files.is_empty() {
                    command.arg_if_missing(".");
                } else {
                    command.args(affected_files);
                }
            }
        }

        Ok(command)
    }

    pub async fn create_env_vars(&self) -> Result<FxHashMap<String, String>, MoonError> {
        let mut env_vars = FxHashMap::default();

        env_vars.insert(
            "MOON_CACHE_DIR".to_owned(),
            path::to_string(&self.workspace.cache.dir)?,
        );
        env_vars.insert("MOON_PROJECT_ID".to_owned(), self.project.id.to_string());
        env_vars.insert(
            "MOON_PROJECT_ROOT".to_owned(),
            path::to_string(&self.project.root)?,
        );
        env_vars.insert(
            "MOON_PROJECT_SOURCE".to_owned(),
            self.project.source.clone(),
        );
        env_vars.insert("MOON_TARGET".to_owned(), self.task.target.id.to_string());
        env_vars.insert(
            "MOON_TOOLCHAIN_DIR".to_owned(),
            path::to_string(&self.workspace.toolchain_root)?,
        );
        env_vars.insert(
            "MOON_WORKSPACE_ROOT".to_owned(),
            path::to_string(&self.workspace.root)?,
        );
        env_vars.insert(
            "MOON_WORKING_DIR".to_owned(),
            path::to_string(&self.workspace.working_dir)?,
        );
        env_vars.insert(
            "MOON_PROJECT_RUNFILE".to_owned(),
            path::to_string(
                self.workspace
                    .cache
                    .get_state_path(&self.project.id)
                    .join("runfile.json"),
            )?,
        );
        // env_vars.insert("PROTO_SKIP_USED_AT".to_owned(), "true".to_owned());

        Ok(env_vars)
    }

    pub fn flush_output(&self) -> Result<(), MoonError> {
        self.stdout.flush()?;
        self.stderr.flush()?;

        Ok(())
    }

    pub fn get_short_hash(&self) -> &str {
        if self.cache.hash.is_empty() {
            "" // Empty when cache is disabled
        } else {
            &self.cache.hash[0..8]
        }
    }

    pub fn has_outputs(&self) -> Result<bool, MoonError> {
        // Check paths first since they are literal
        for output in &self.task.output_paths {
            if !output.to_path(&self.workspace.root).exists() {
                return Ok(false);
            }
        }

        // Check globs last, as they are costly
        if !self.task.output_globs.is_empty() {
            let outputs = glob::walk_files(&self.workspace.root, &self.task.output_globs)?;

            return Ok(!outputs.is_empty());
        }

        Ok(true)
    }

    /// Determine if the current task can be archived.
    pub fn is_archivable(&self) -> Result<bool, TargetError> {
        let task = self.task;

        if task.is_build_type() {
            return Ok(true);
        }

        for target in &self.workspace.config.runner.archivable_targets {
            match &target.scope {
                TargetScope::All => {
                    if task.target.task_id == target.task_id {
                        return Ok(true);
                    }
                }
                TargetScope::Project(project_id) => {
                    if let Some(owner_id) = &task.target.scope_id {
                        if owner_id == project_id && task.target.task_id == target.task_id {
                            return Ok(true);
                        }
                    }
                }
                TargetScope::Tag(_) => todo!(),
                TargetScope::Deps => return Err(TargetError::NoDepsInRunContext),
                TargetScope::OwnSelf => return Err(TargetError::NoSelfInRunContext),
            };
        }

        Ok(false)
    }

    /// Hash the target based on all current parameters and return early
    /// if this target hash has already been cached. Based on the state
    /// of the target and project, determine the hydration strategy as well.
    pub async fn is_cached(
        &mut self,
        context: &mut ActionContext,
        runtime: &Runtime,
    ) -> Result<Option<HydrateFrom>, RunnerError> {
        let mut hashset = HashSet::default();

        self.hash_common_target(context, &mut hashset).await?;

        self.workspace
            .platforms
            .get(self.task.platform)?
            .hash_run_target(
                self.project,
                runtime,
                &mut hashset,
                &self.workspace.config.hasher,
            )
            .await?;

        let hash = hashset.generate();

        debug!(
            target: LOG_TARGET,
            "Generated hash {} for target {}",
            color::hash(&hash),
            color::id(&self.task.target)
        );

        context
            .target_hashes
            .insert(self.task.target.clone(), hash.clone());

        // Hash is the same as the previous build, so simply abort!
        // However, ensure the outputs also exist, otherwise we should hydrate
        if self.cache.hash == hash && self.has_outputs()? {
            debug!(
                target: LOG_TARGET,
                "Cache hit for hash {}, reusing previous build",
                color::hash(&hash),
            );

            return Ok(Some(HydrateFrom::PreviousOutput));
        }

        self.cache.hash = hash.clone();

        // Refresh the hash manifest
        self.workspace.cache.create_hash_manifest(&hash, &hashset)?;

        // Check if that hash exists in the cache
        if let EventFlow::Return(value) = self
            .emitter
            .emit(Event::TargetOutputCacheCheck {
                hash: &hash,
                target: &self.task.target,
            })
            .await?
        {
            match value.as_ref() {
                "local-cache" => {
                    debug!(
                        target: LOG_TARGET,
                        "Cache hit for hash {}, hydrating from local cache",
                        color::hash(&hash),
                    );

                    return Ok(Some(HydrateFrom::LocalCache));
                }
                "remote-cache" => {
                    debug!(
                        target: LOG_TARGET,
                        "Cache hit for hash {}, hydrating from remote cache",
                        color::hash(&hash),
                    );

                    return Ok(Some(HydrateFrom::RemoteCache));
                }
                _ => {}
            }
        }

        debug!(
            target: LOG_TARGET,
            "Cache miss for hash {}, continuing run",
            color::hash(&hash),
        );

        Ok(None)
    }

    /// Run the command as a child process and capture its output. If the process fails
    /// and `retry_count` is greater than 0, attempt the process again in case it passes.
    pub async fn run_command(
        &mut self,
        context: &ActionContext,
        command: &mut Command,
    ) -> Result<Vec<Attempt>, RunnerError> {
        let attempt_total = self.task.options.retry_count + 1;
        let mut attempt_index = 1;
        let mut attempts = vec![];
        let primary_longest_width = context.primary_targets.iter().map(|t| t.id.len()).max();
        let is_primary = context.primary_targets.contains(&self.task.target);
        let is_real_ci = is_ci() && !is_test_env();
        let is_persistent = self.task.options.persistent;
        let output;

        // When a task is configured as local (no caching), or the interactive flag is passed,
        // we don't "capture" stdout/stderr (which breaks stdin) and let it stream natively.
        let is_interactive =
            (!self.task.options.cache && context.primary_targets.len() == 1) || context.interactive;

        // When the primary target, always stream the output for a better developer experience.
        // However, transitive targets can opt into streaming as well.
        let should_stream_output = if let Some(output_style) = &self.task.options.output_style {
            matches!(output_style, TaskOutputStyle::Stream)
        } else {
            is_primary || is_real_ci
        };

        // Transitive targets may run concurrently, so differentiate them with a prefix.
        let stream_prefix = if is_real_ci || !is_primary || context.primary_targets.len() > 1 {
            Some(&self.task.target.id)
        } else {
            None
        };

        // For long-running process, log a message every 30 seconds to indicate it's still running
        let interval_target = self.task.target.clone();
        let interval_handle = task::spawn(async move {
            if is_persistent {
                return;
            }

            let mut secs = 0;

            loop {
                sleep(Duration::from_secs(30)).await;
                secs += 30;

                println!(
                    "{} {}",
                    label_checkpoint(&interval_target, Checkpoint::RunStart),
                    color::muted(format!("running for {}s", secs))
                );
            }
        });

        loop {
            let mut attempt = Attempt::new(attempt_index);

            self.print_target_label(Checkpoint::RunStart, &attempt, attempt_total)?;
            self.print_target_command(context, command)?;
            self.flush_output()?;

            let possible_output = if should_stream_output {
                if let Some(prefix) = stream_prefix {
                    command.set_prefix(prefix, primary_longest_width);
                }

                if is_interactive {
                    command.create_async().exec_stream_output().await
                } else {
                    command
                        .create_async()
                        .exec_stream_and_capture_output()
                        .await
                }
            } else {
                command.create_async().exec_capture_output().await
            };

            match possible_output {
                // zero and non-zero exit codes
                Ok(out) => {
                    attempt.done(if out.status.success() {
                        ActionStatus::Passed
                    } else {
                        ActionStatus::Failed
                    });

                    if should_stream_output {
                        self.handle_streamed_output(&attempt, attempt_total, &out)?;
                    } else {
                        self.handle_captured_output(&attempt, attempt_total, &out)?;
                    }

                    attempts.push(attempt);

                    if out.status.success() {
                        output = out;
                        break;
                    } else if attempt_index >= attempt_total {
                        interval_handle.abort();

                        return Err(RunnerError::Process(output_to_error(
                            self.task.command.clone(),
                            &out,
                            false,
                        )));
                    } else {
                        attempt_index += 1;

                        warn!(
                            target: LOG_TARGET,
                            "Target {} failed, running again with attempt {}",
                            color::label(&self.task.target),
                            attempt_index
                        );
                    }
                }
                // process itself failed
                Err(error) => {
                    attempt.done(ActionStatus::Failed);
                    attempts.push(attempt);

                    interval_handle.abort();

                    return Err(RunnerError::Process(error));
                }
            }
        }

        interval_handle.abort();

        // Write the cache with the result and output
        self.cache.exit_code = output.status.code().unwrap_or(0);
        self.cache.save_output_logs(
            output_to_string(&output.stdout),
            output_to_string(&output.stderr),
        )?;

        Ok(attempts)
    }

    pub async fn create_and_run_command(
        &mut self,
        context: &ActionContext,
        runtime: &Runtime,
    ) -> Result<Vec<Attempt>, RunnerError> {
        let attempts = if self.task.is_no_op() {
            debug!(
                target: LOG_TARGET,
                "Target {} is a no operation, skipping",
                color::label(&self.task.target),
            );

            self.print_target_label(Checkpoint::RunPassed, &Attempt::new(0), 0)?;
            self.flush_output()?;

            vec![]
        } else {
            let mut command = self.create_command(context, runtime).await?;

            self.run_command(context, &mut command).await?
        };

        self.cache.last_run_time = time::now_millis();
        self.cache.save()?;

        Ok(attempts)
    }

    pub fn print_cache_item(&self) -> Result<(), MoonError> {
        let item = &self.cache;
        let (stdout, stderr) = item.load_output_logs()?;

        self.print_output_with_style(&stdout, &stderr, item.exit_code != 0)?;

        Ok(())
    }

    pub fn print_checkpoint<T: AsRef<str>>(
        &self,
        checkpoint: Checkpoint,
        comments: &[T],
    ) -> Result<(), MoonError> {
        let label = label_checkpoint(&self.task.target, checkpoint);

        if comments.is_empty() {
            self.stdout.write_line(&label)?;
        } else {
            self.stdout.write_line(&format!(
                "{} {}",
                label,
                color::muted(format!(
                    "({})",
                    comments
                        .iter()
                        .map(|c| c.as_ref())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            ))?;
        }

        Ok(())
    }

    pub fn print_output_with_style(
        &self,
        stdout: &str,
        stderr: &str,
        failed: bool,
    ) -> Result<(), MoonError> {
        let print_stdout = || -> Result<(), MoonError> {
            if !stdout.is_empty() {
                self.stdout.write_line(stdout)?;
            }

            Ok(())
        };

        let print_stderr = || -> Result<(), MoonError> {
            if !stderr.is_empty() {
                self.stderr.write_line(stderr)?;
            }

            Ok(())
        };

        match self.task.options.output_style {
            // Only show output on failure
            Some(TaskOutputStyle::BufferOnlyFailure) => {
                if failed {
                    print_stdout()?;
                    print_stderr()?;
                }
            }
            // Only show the hash
            Some(TaskOutputStyle::Hash) => {
                let hash = &self.cache.hash;

                if !hash.is_empty() {
                    // Print to stderr so it can be captured
                    self.stderr.write_line(hash)?;
                }
            }
            // Show nothing
            Some(TaskOutputStyle::None) => {}
            // Show output on both success and failure
            _ => {
                print_stdout()?;
                print_stderr()?;
            }
        };

        Ok(())
    }

    pub fn print_target_command(
        &self,
        context: &ActionContext,
        command: &Command,
    ) -> Result<(), MoonError> {
        if !self.workspace.config.runner.log_running_command {
            return Ok(());
        }

        let task = &self.task;
        let mut args = vec![&task.command];
        args.extend(&task.args);

        if context.should_inherit_args(&task.target) {
            args.extend(&context.passthrough_args);
        }

        let command_line = args::join_args(args);

        let message = color::muted_light(command.inspect().format_command(
            &command_line,
            &self.workspace.root,
            Some(if task.options.run_from_workspace_root {
                &self.workspace.root
            } else {
                &self.project.root
            }),
        ));

        self.stdout.write_line(&message)?;

        Ok(())
    }

    pub fn print_target_label(
        &self,
        checkpoint: Checkpoint,
        attempt: &Attempt,
        attempt_total: u8,
    ) -> Result<(), MoonError> {
        let mut comments = vec![];

        if self.task.is_no_op() {
            comments.push("no op".to_owned());
        } else if attempt.index > 1 {
            comments.push(format!("{}/{}", attempt.index, attempt_total));
        }

        if let Some(duration) = attempt.duration {
            comments.push(time::elapsed(duration));
        }

        if self.should_print_short_hash() && attempt.finished_at.is_some() {
            comments.push(self.get_short_hash().to_owned());
        }

        self.print_checkpoint(checkpoint, &comments)?;

        Ok(())
    }

    // Print label *after* output has been captured, so parallel tasks
    // aren't intertwined and the labels align with the output.
    fn handle_captured_output(
        &self,
        attempt: &Attempt,
        attempt_total: u8,
        output: &Output,
    ) -> Result<(), MoonError> {
        self.print_target_label(
            if output.status.success() {
                Checkpoint::RunPassed
            } else {
                Checkpoint::RunFailed
            },
            attempt,
            attempt_total,
        )?;

        let stdout = output_to_string(&output.stdout);
        let stderr = output_to_string(&output.stderr);

        self.print_output_with_style(&stdout, &stderr, !output.status.success())?;
        self.flush_output()?;

        Ok(())
    }

    // Only print the label when the process has failed,
    // as the actual output has already been streamed to the console.
    fn handle_streamed_output(
        &self,
        attempt: &Attempt,
        attempt_total: u8,
        output: &Output,
    ) -> Result<(), MoonError> {
        self.print_target_label(
            if output.status.success() {
                Checkpoint::RunPassed
            } else {
                Checkpoint::RunFailed
            },
            attempt,
            attempt_total,
        )?;

        self.flush_output()?;

        Ok(())
    }

    fn should_print_short_hash(&self) -> bool {
        // Do not include the hash while testing, as the hash
        // constantly changes and breaks our local snapshots
        !is_test_env() && self.task.options.cache && !self.cache.hash.is_empty()
    }
}
