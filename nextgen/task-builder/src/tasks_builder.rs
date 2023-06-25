#![allow(dead_code)]

use miette::IntoDiagnostic;
use moon_args::split_args;
use moon_common::{color, Id};
use moon_config::{
    InheritedTasksConfig, InputPath, PlatformType, ProjectConfig,
    ProjectWorkspaceInheritedTasksConfig, TaskCommandArgs, TaskConfig, TaskMergeStrategy,
    TaskOutputStyle, TaskType,
};
use moon_target::Target;
use moon_task::{Task, TaskOptions};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::path::Path;
use tracing::{debug, trace, warn};

pub type PlatformDetector = dyn Fn(&str) -> PlatformType;

pub struct TasksBuilder<'proj> {
    project_id: &'proj str,
    project_env: FxHashMap<&'proj str, &'proj str>,
    project_platform: &'proj PlatformType,
    project_source: &'proj str,
    workspace_root: &'proj Path,

    // Global settings for tasks to inherit
    implicit_deps: Vec<&'proj Target>,
    implicit_inputs: Vec<&'proj InputPath>,
    platform_detector: Option<Box<PlatformDetector>>,

    // Tasks to merge and build
    task_ids: FxHashSet<&'proj Id>,
    global_tasks: FxHashMap<&'proj Id, &'proj TaskConfig>,
    local_tasks: FxHashMap<&'proj Id, &'proj TaskConfig>,
}

impl<'proj> TasksBuilder<'proj> {
    pub fn new(
        project_id: &'proj str,
        project_source: &'proj str,
        project_platform: &'proj PlatformType,
        workspace_root: &'proj Path,
    ) -> Self {
        Self {
            project_id,
            project_env: FxHashMap::default(),
            project_platform,
            project_source,
            workspace_root,
            platform_detector: None,
            implicit_deps: vec![],
            implicit_inputs: vec![],
            task_ids: FxHashSet::default(),
            global_tasks: FxHashMap::default(),
            local_tasks: FxHashMap::default(),
        }
    }

    /// Register a function to detect a task's platform when unknown.
    pub fn detect_platform<F>(&mut self, detector: F) -> &mut Self
    where
        F: Fn(&str) -> PlatformType + 'static,
    {
        self.platform_detector = Some(Box::new(detector));
        self
    }

    pub fn inherit_global_tasks(
        &mut self,
        global_config: &'proj InheritedTasksConfig,
        global_filters: Option<&'proj ProjectWorkspaceInheritedTasksConfig>,
    ) -> &mut Self {
        let mut include_all = true;
        let mut include_set = FxHashSet::default();
        let mut exclude = vec![];
        let mut rename = FxHashMap::default();

        if let Some(filters) = global_filters {
            exclude.extend(&filters.exclude);
            rename.extend(&filters.rename);

            if let Some(include_config) = &filters.include {
                include_all = false;
                include_set.extend(include_config);
            }
        }

        debug!(project_id = self.project_id, "Filtering global tasks");

        for (task_id, task_config) in &global_config.tasks {
            let target = Target::new(self.project_id, task_id).unwrap();

            // None = Include all
            // [] = Include none
            // ["a"] = Include "a"
            if !include_all {
                if include_set.is_empty() {
                    debug!(
                        target = target.as_str(),
                        "Not inheriting any global tasks, empty include filter",
                    );

                    break;
                } else if !include_set.contains(task_id) {
                    debug!(
                        target = target.as_str(),
                        "Not inheriting global task {}, not included",
                        color::id(task_id)
                    );

                    continue;
                }
            }

            // None, [] = Exclude none
            // ["a"] = Exclude "a"
            if !exclude.is_empty() && exclude.contains(&task_id) {
                debug!(
                    target = target.as_str(),
                    "Not inheriting global task {}, excluded",
                    color::id(task_id)
                );

                continue;
            }

            let task_key = if let Some(renamed_task_id) = rename.get(task_id) {
                debug!(
                    target = target.as_str(),
                    "Inheriting global task {} and renaming to {}",
                    color::id(task_id),
                    color::id(renamed_task_id)
                );

                renamed_task_id
            } else {
                debug!(
                    target = target.as_str(),
                    "Inheriting global task {}",
                    color::id(task_id),
                );

                task_id
            };

            self.global_tasks.insert(task_key, task_config);
            self.task_ids.insert(task_key);
        }

        self.implicit_deps.extend(&global_config.implicit_deps);
        self.implicit_inputs.extend(&global_config.implicit_inputs);
        self
    }

    pub fn load_local_tasks(&mut self, local_config: &'proj ProjectConfig) -> &mut Self {
        for (key, value) in &local_config.env {
            self.project_env.insert(key, value);
        }

        self.local_tasks.extend(&local_config.tasks);

        for id in local_config.tasks.keys() {
            self.task_ids.insert(id);
        }

        self
    }

    #[tracing::instrument(name = "task", skip_all)]
    pub fn build(self) -> miette::Result<BTreeMap<Id, Task>> {
        let mut tasks = BTreeMap::new();

        for id in &self.task_ids {
            tasks.insert((*id).to_owned(), self.build_task(id)?);
        }

        Ok(tasks)
    }

    fn build_task(&self, id: &Id) -> miette::Result<Task> {
        let target = Target::new(self.project_id, id)?;

        debug!(target = target.as_str(), "Building task");

        let mut task = Task::default();
        let mut configs = vec![];

        if let Some(config) = self.global_tasks.get(id) {
            configs.push(*config);
        }

        if let Some(config) = self.local_tasks.get(id) {
            configs.push(*config);
        }

        // Determine command and args before building options and the task,
        // as we need to figure out if we're running in local mode or not.
        let mut is_local = id == "dev" || id == "serve" || id == "start";
        let mut args_sets = vec![];

        for config in &configs {
            let (command, base_args) = self.get_command_and_args(config)?;

            if let Some(command) = command {
                task.command = command;
            }

            // Add to task later after we have a merge strategy
            args_sets.push(base_args);

            if let Some(local) = config.local {
                is_local = local;
            }
        }

        trace!(target = target.as_str(), "Marking task as local");

        task.options = self.build_task_options(id, is_local)?;
        task.flags.local = is_local;

        // Aggregate all values that are inherited from the global task configs,
        // and should always be included in the task, regardless of merge strategy.
        let global_deps = self.build_global_deps(&target)?;
        let mut global_inputs = self.build_global_inputs(&target, &task.options)?;

        // Aggregate all values that that are inherited from the project,
        // and should be set on the task first, so that merge strategies can be applied.
        for args in args_sets {
            if !args.is_empty() {
                task.args = self.merge_vec(task.args, args, task.options.merge_args, false);
            }
        }

        task.env = self.build_env(&target, &task.options)?;

        // Finally build the task itself, while applying our complex merge logic!
        let mut configured_inputs = 0;
        let mut has_configured_inputs = false;

        for config in configs {
            if !config.deps.is_empty() {
                task.deps = self.merge_vec(
                    task.deps,
                    config.deps.to_owned(),
                    task.options.merge_deps,
                    true,
                );
            }

            if !config.env.is_empty() {
                task.env = self.merge_map(task.env, config.env.to_owned(), task.options.merge_env);
            }

            // Inherit global inputs as normal inputs, but do not consider them a configured input
            if !config.global_inputs.is_empty() {
                global_inputs.extend(config.global_inputs.to_owned());
            }

            // Inherit local inputs, which are user configured, and keep track of the total
            if let Some(inputs) = &config.inputs {
                configured_inputs += inputs.len();
                has_configured_inputs = true;

                task.inputs = self.merge_vec(
                    task.inputs,
                    inputs.to_owned(),
                    task.options.merge_inputs,
                    true,
                );
            }

            if let Some(outputs) = &config.outputs {
                task.outputs = self.merge_vec(
                    task.outputs,
                    outputs.to_owned(),
                    task.options.merge_outputs,
                    true,
                );
            }

            if !config.platform.is_unknown() {
                task.platform = config.platform;
            }
        }

        // Inputs are tricky, as they come from many sources. We need to ensure that user configured
        // inputs are handled explicitly, while globally inherited sources are handled implicitly.
        if configured_inputs == 0 {
            if has_configured_inputs {
                debug!(
                    target = target.as_str(),
                    "Task has explicitly disabled inputs",
                );

                task.flags.empty_inputs = true;
            } else {
                debug!(
                    target = target.as_str(),
                    "No inputs configured, defaulting to {} (from project)",
                    color::file("**/*"),
                );

                task.inputs.push(InputPath::ProjectGlob("**/*".into()));
            }
        }

        // And lastly, before we return the task and options, we should finalize
        // all necessary fields and populate/calculate with values.
        if task.command.is_empty() {
            task.command = "noop".into();
        }

        if !global_deps.is_empty() {
            task.deps = self.merge_vec(task.deps, global_deps, TaskMergeStrategy::Append, true);
        }

        task.id = id.to_owned();

        if !global_inputs.is_empty() {
            task.inputs =
                self.merge_vec(task.inputs, global_inputs, TaskMergeStrategy::Append, true);
        }

        if task.platform.is_unknown() {
            if let Some(detector) = &self.platform_detector {
                task.platform = detector(&task.command);
            } else {
                task.platform = self.project_platform.to_owned();
            }

            if task.platform.is_unknown() {
                task.platform = PlatformType::System;
            }
        }

        task.target = target;

        task.type_of = if !task.outputs.is_empty() {
            TaskType::Build
        } else if is_local {
            TaskType::Run
        } else {
            TaskType::Test
        };

        Ok(task)
    }

    fn build_task_options(&self, id: &Id, is_local: bool) -> miette::Result<TaskOptions> {
        let mut options = TaskOptions {
            cache: !is_local,
            output_style: is_local.then_some(TaskOutputStyle::Stream),
            persistent: is_local,
            run_in_ci: !is_local,
            ..TaskOptions::default()
        };

        let mut configs = vec![];

        if let Some(config) = self.global_tasks.get(id) {
            configs.push(&config.options);
        }

        if let Some(config) = self.local_tasks.get(id) {
            configs.push(&config.options);
        }

        for config in configs {
            if let Some(affected_files) = &config.affected_files {
                options.affected_files = Some(affected_files.to_owned());
            }

            if let Some(cache) = &config.cache {
                options.cache = *cache;
            }

            if let Some(env_file) = &config.env_file {
                options.env_file = env_file.to_input_path();
            }

            if let Some(merge_args) = &config.merge_args {
                options.merge_args = *merge_args;
            }

            if let Some(merge_deps) = &config.merge_deps {
                options.merge_deps = *merge_deps;
            }

            if let Some(merge_env) = &config.merge_env {
                options.merge_env = *merge_env;
            }

            if let Some(merge_inputs) = &config.merge_inputs {
                options.merge_inputs = *merge_inputs;
            }

            if let Some(merge_outputs) = &config.merge_outputs {
                options.merge_outputs = *merge_outputs;
            }

            if let Some(output_style) = &config.output_style {
                options.output_style = Some(*output_style);
            }

            if let Some(persistent) = &config.persistent {
                options.persistent = *persistent;
            }

            if let Some(retry_count) = &config.retry_count {
                options.retry_count = *retry_count;
            }

            if let Some(run_deps_in_parallel) = &config.run_deps_in_parallel {
                options.run_deps_in_parallel = *run_deps_in_parallel;
            }

            if let Some(run_in_ci) = &config.run_in_ci {
                options.run_in_ci = *run_in_ci;
            }

            if let Some(run_from_workspace_root) = &config.run_from_workspace_root {
                options.run_from_workspace_root = *run_from_workspace_root;
            }

            if let Some(shell) = &config.shell {
                options.shell = *shell;
            }
        }

        Ok(options)
    }

    fn build_global_deps(&self, target: &Target) -> miette::Result<Vec<Target>> {
        let global_deps = self
            .implicit_deps
            .iter()
            .map(|d| (*d).to_owned())
            .collect::<Vec<_>>();

        if !global_deps.is_empty() {
            trace!(
                target = target.as_str(),
                deps = ?global_deps.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
                "Inheriting global implicit deps",
            );
        }

        Ok(global_deps)
    }

    fn build_global_inputs(
        &self,
        target: &Target,
        options: &TaskOptions,
    ) -> miette::Result<Vec<InputPath>> {
        let mut global_inputs = self
            .implicit_inputs
            .iter()
            .map(|d| (*d).to_owned())
            .collect::<Vec<_>>();

        global_inputs.push(InputPath::WorkspaceGlob(".moon/*.yml".into()));

        if let Some(env_file) = &options.env_file {
            global_inputs.push(env_file.to_owned());
        }

        if !global_inputs.is_empty() {
            trace!(
                target = target.as_str(),
                inputs = ?global_inputs.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
                "Inheriting global implicit inputs",
            );
        }

        Ok(global_inputs)
    }

    /// Build environment variables for the task. The precedence is as follows.
    ///     - 1st - project-level `env`
    ///     - 2nd - task `env_file` (when enabled)
    ///     - 3rd - task-level `env`
    fn build_env(
        &self,
        target: &Target,
        options: &TaskOptions,
    ) -> miette::Result<FxHashMap<String, String>> {
        let mut env = self
            .project_env
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect::<FxHashMap<_, _>>();

        if !env.is_empty() {
            trace!(
                target = target.as_str(),
                env_vars = ?self.project_env,
                "Inheriting project env vars",
            );
        }

        if let Some(env_file) = &options.env_file {
            let env_path = env_file
                .to_workspace_relative(self.project_source)
                .to_path(self.workspace_root);

            trace!(
                target = target.as_str(),
                env_file = ?env_path,
                "Loading env vars from dotfile",
            );

            // The `.env` file may not have been committed, so avoid crashing
            if env_path.exists() {
                let env_file_vars = dotenvy::from_path_iter(&env_path)
                    .into_diagnostic()?
                    .flatten()
                    .collect::<FxHashMap<_, _>>();

                env = self.merge_map(env, env_file_vars, options.merge_env);
            } else {
                warn!(
                    target = target.as_str(),
                    env_file = ?env_path,
                    "The {} option is enabled but file doesn't exist, skipping as this may be intentional",
                    color::id("envFile"),
                );
            }
        }

        Ok(env)
    }

    fn get_command_and_args(
        &self,
        config: &TaskConfig,
    ) -> miette::Result<(Option<String>, Vec<String>)> {
        let mut command = None;
        let mut args = vec![];

        let mut cmd_list = match &config.command {
            TaskCommandArgs::None => vec![],
            TaskCommandArgs::String(cmd_string) => split_args(cmd_string)?,
            TaskCommandArgs::List(cmd_args) => cmd_args.to_owned(),
        };

        if !cmd_list.is_empty() {
            command = Some(cmd_list.remove(0));
            args.extend(cmd_list);
        }

        match &config.args {
            TaskCommandArgs::None => {}
            TaskCommandArgs::String(args_string) => args.extend(split_args(args_string)?),
            TaskCommandArgs::List(args_list) => args.extend(args_list.to_owned()),
        };

        Ok((command, args))
    }

    fn merge_map<K, V>(
        &self,
        base: FxHashMap<K, V>,
        next: FxHashMap<K, V>,
        strategy: TaskMergeStrategy,
    ) -> FxHashMap<K, V>
    where
        K: Eq + Hash,
    {
        match strategy {
            TaskMergeStrategy::Append => {
                let mut map = FxHashMap::default();
                map.extend(base);
                map.extend(next);
                map
            }
            TaskMergeStrategy::Prepend => {
                let mut map = FxHashMap::default();
                map.extend(next);
                map.extend(base);
                map
            }
            TaskMergeStrategy::Replace => next,
        }
    }

    fn merge_vec<T: Eq>(
        &self,
        base: Vec<T>,
        next: Vec<T>,
        strategy: TaskMergeStrategy,
        dedupe: bool,
    ) -> Vec<T> {
        let mut list: Vec<T> = vec![];

        // Dedupe while merging vectors. We can't use a set here because
        // we need to preserve the insertion order. Revisit if this is costly!
        let mut append = |items: Vec<T>, force: bool| {
            for item in items {
                #[allow(clippy::nonminimal_bool)]
                if force || !dedupe || (dedupe && !list.contains(&item)) {
                    list.push(item);
                }
            }
        };

        match strategy {
            TaskMergeStrategy::Append => {
                append(base, true);
                append(next, false);
            }
            TaskMergeStrategy::Prepend => {
                append(next, true);
                append(base, false);
            }
            TaskMergeStrategy::Replace => {
                list.extend(next);
            }
        }

        list
    }
}
