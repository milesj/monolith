use crate::action_graph::ActionGraph;
use moon_action::{
    ActionNode, InstallProjectDepsNode, InstallWorkspaceDepsNode, RunTaskNode, SetupToolchainNode,
    SyncProjectNode,
};
use moon_action_context::{ActionContext, TargetState};
use moon_affected::{AffectedBy, AffectedTracker, DownstreamScope, UpstreamScope};
use moon_common::path::WorkspaceRelativePathBuf;
use moon_common::{color, Id};
use moon_config::{PlatformType, TaskDependencyConfig};
use moon_platform::{PlatformManager, Runtime};
use moon_project::Project;
use moon_query::{build_query, Criteria};
use moon_task::{Target, TargetError, TargetLocator, TargetScope, Task};
use moon_task_args::parse_task_args;
use moon_workspace_graph::{tasks::TaskGraphError, GraphConnections, WorkspaceGraph};
use petgraph::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::mem;
use tracing::{debug, instrument, trace};

#[derive(Default)]
pub struct RunRequirements {
    pub ci: bool,         // Are we in a CI environment
    pub ci_check: bool,   // Check the `runInCI` option
    pub dependents: bool, // Run dependent tasks as well
    pub interactive: bool,
    pub skip_affected: bool, // Temporary until we support task dependents properly
    pub target_locators: FxHashSet<TargetLocator>,

    // Used to mark affected states
    pub upstream_target: Option<Target>,
    pub downstream_target: Option<Target>,
}

impl RunRequirements {
    pub fn has_target(&self, target: &Target) -> bool {
        self.target_locators.iter().any(|loc| loc == target)
    }
}

pub struct ActionGraphBuilder<'app> {
    all_query: Option<Criteria<'app>>,
    graph: DiGraph<ActionNode, ()>,
    indices: FxHashMap<ActionNode, NodeIndex>,
    platform_manager: &'app PlatformManager,
    workspace_graph: &'app WorkspaceGraph,

    // Affected states
    affected: Option<AffectedTracker<'app>>,
    touched_files: Option<&'app FxHashSet<WorkspaceRelativePathBuf>>,

    // Target tracking
    initial_targets: FxHashSet<Target>,
    passthrough_targets: FxHashSet<Target>,
    primary_targets: FxHashSet<Target>,
}

impl<'app> ActionGraphBuilder<'app> {
    pub fn new(workspace_graph: &'app WorkspaceGraph) -> miette::Result<Self> {
        ActionGraphBuilder::with_platforms(PlatformManager::read(), workspace_graph)
    }

    pub fn with_platforms(
        platform_manager: &'app PlatformManager,
        workspace_graph: &'app WorkspaceGraph,
    ) -> miette::Result<Self> {
        debug!("Building action graph");

        Ok(ActionGraphBuilder {
            all_query: None,
            affected: None,
            graph: DiGraph::new(),
            indices: FxHashMap::default(),
            initial_targets: FxHashSet::default(),
            passthrough_targets: FxHashSet::default(),
            platform_manager,
            primary_targets: FxHashSet::default(),
            workspace_graph,
            touched_files: None,
        })
    }

    pub fn build(self) -> ActionGraph {
        ActionGraph::new(self.graph)
    }

    pub fn build_context(&mut self) -> ActionContext {
        let mut context = ActionContext {
            affected: self.affected.take().map(|affected| affected.build()),
            ..ActionContext::default()
        };

        if !self.initial_targets.is_empty() {
            context.initial_targets = mem::take(&mut self.initial_targets);
        }

        if !self.passthrough_targets.is_empty() {
            for target in mem::take(&mut self.passthrough_targets) {
                context.set_target_state(target, TargetState::Passthrough);
            }
        }

        if !self.primary_targets.is_empty() {
            context.primary_targets = mem::take(&mut self.primary_targets);
        }

        if let Some(files) = self.touched_files.take() {
            context.touched_files = files.to_owned();
        }

        context
    }

    pub fn get_index_from_node(&self, node: &ActionNode) -> Option<&NodeIndex> {
        self.indices.get(node)
    }

    pub fn get_runtime(
        &self,
        project: &Project,
        platform_type: PlatformType,
        allow_override: bool,
    ) -> Runtime {
        if let Some(platform) = self
            .platform_manager
            .find(|p| p.get_type() == platform_type)
        {
            return platform.get_runtime_from_config(if allow_override {
                Some(&project.config)
            } else {
                None
            });
        }

        Runtime::system()
    }

    pub fn set_affected_scopes(
        &mut self,
        upstream: UpstreamScope,
        downstream: DownstreamScope,
    ) -> miette::Result<()> {
        // If we require dependents, then we must load all projects into the
        // graph so that the edges are created!
        if downstream != DownstreamScope::None {
            debug!("Force loading all projects to determine downstream relationships");

            self.workspace_graph.get_all_projects()?;
        }

        self.affected
            .as_mut()
            .expect("Affected tracker not set!")
            .with_scopes(upstream, downstream);

        Ok(())
    }

    pub fn set_query(&mut self, input: &'app str) -> miette::Result<()> {
        self.all_query = Some(build_query(input)?);

        Ok(())
    }

    pub fn set_touched_files(
        &mut self,
        touched_files: &'app FxHashSet<WorkspaceRelativePathBuf>,
    ) -> miette::Result<()> {
        self.touched_files = Some(touched_files);
        self.affected = Some(AffectedTracker::new(self.workspace_graph, touched_files));

        Ok(())
    }

    // ACTIONS

    #[instrument(skip_all)]
    pub fn install_deps(
        &mut self,
        project: &Project,
        task: Option<&Task>,
    ) -> miette::Result<Option<NodeIndex>> {
        let mut in_project = false;
        let mut platform_type = task.map(|t| t.platform).unwrap_or_else(|| project.platform);

        // If project is NOT in the package manager workspace, then we should
        // install dependencies in the project, not the workspace root.
        if let Ok(platform) = self.platform_manager.get(platform_type) {
            if !platform.is_project_in_dependency_workspace(project.source.as_str())? {
                in_project = true;

                debug!(
                    "Project {} is not within the dependency manager workspace, dependencies will be installed within the project instead of the root",
                    color::id(&project.id),
                );
            }
        }

        // If Bun and Node.js are enabled, they will both attempt to install
        // dependencies in the target root. We need to avoid this problem,
        // so always prefer Node.js instead. Revisit in the future.
        if matches!(platform_type, PlatformType::Bun)
            && self
                .platform_manager
                .enabled()
                .any(|enabled_platform| matches!(enabled_platform, PlatformType::Node))
        {
            debug!(
                "Already installing dependencies with {}, skipping a conflicting install from {}",
                PlatformType::Node,
                platform_type,
            );

            platform_type = PlatformType::Node;
        }

        let node = if in_project {
            ActionNode::install_project_deps(InstallProjectDepsNode {
                project: project.id.to_owned(),
                runtime: self.get_runtime(project, platform_type, true),
            })
        } else {
            ActionNode::install_workspace_deps(InstallWorkspaceDepsNode {
                runtime: self.get_runtime(project, platform_type, false),
            })
        };

        if node.get_runtime().platform.is_system() {
            return Ok(None);
        }

        if let Some(index) = self.get_index_from_node(&node) {
            return Ok(Some(*index));
        }

        // Before we install deps, we must ensure the language has been installed
        let setup_tool_index = self.setup_toolchain(node.get_runtime());
        let index = self.insert_node(node);

        self.link_requirements(index, vec![setup_tool_index]);

        Ok(Some(index))
    }

    pub fn run_task(
        &mut self,
        project: &Project,
        task: &Task,
        reqs: &RunRequirements,
    ) -> miette::Result<Option<NodeIndex>> {
        self.run_task_with_config(project, task, reqs, None)
    }

    #[instrument(skip_all)]
    pub fn run_task_with_config(
        &mut self,
        project: &Project,
        task: &Task,
        reqs: &RunRequirements,
        config: Option<&TaskDependencyConfig>,
    ) -> miette::Result<Option<NodeIndex>> {
        // Only apply checks when requested. This applies to `moon ci`,
        // but not `moon run`, since the latter should be able to
        // manually run local tasks in CI (deploys, etc).
        if reqs.ci && reqs.ci_check && !task.should_run_in_ci() {
            self.passthrough_targets.insert(task.target.clone());

            debug!(
                task = task.target.as_str(),
                "Not running task {} because {} is false",
                color::label(&task.target.id),
                color::property("runInCI"),
            );

            // Dependents may still want to run though,
            // but only if this task was affected
            if reqs.dependents {
                if let Some(affected) = &mut self.affected {
                    trace!(
                        task = task.target.as_str(),
                        "But will run all dependent tasks if affected"
                    );

                    if let Some(by) = affected.is_task_affected(task)? {
                        affected.mark_task_affected(task, by)?;

                        self.run_task_dependents(task, reqs)?;
                    }
                }
            }

            return Ok(None);
        }

        // These tasks shouldn't actually run, so filter them out
        if self.passthrough_targets.contains(&task.target) {
            trace!(
                task = task.target.as_str(),
                "Not adding task {} to graph because it has been marked as passthrough",
                color::label(&task.target.id),
            );

            return Ok(None);
        }

        let mut args = vec![];
        let mut env = FxHashMap::default();

        if let Some(config) = config {
            args.extend(parse_task_args(&config.args)?);
            env.extend(config.env.clone());
        }

        let node = ActionNode::run_task(RunTaskNode {
            args,
            env,
            interactive: task.is_interactive() || reqs.interactive,
            persistent: task.is_persistent(),
            runtime: self.get_runtime(project, task.platform, true),
            target: task.target.to_owned(),
            id: None,
        });

        if let Some(index) = self.get_index_from_node(&node) {
            return Ok(Some(*index));
        }

        // Compare against touched files if provided
        if let Some(affected) = &mut self.affected {
            if let Some(downstream) = &reqs.downstream_target {
                affected
                    .mark_task_affected(task, AffectedBy::DownstreamTask(downstream.to_owned()))?;
            }

            if let Some(upstream) = &reqs.upstream_target {
                affected.mark_task_affected(task, AffectedBy::UpstreamTask(upstream.to_owned()))?;
            }

            if !reqs.skip_affected {
                match affected.is_task_affected(task)? {
                    Some(by) => {
                        affected.mark_task_affected(task, by)?;
                        affected.mark_project_affected(
                            project,
                            AffectedBy::Task(task.target.clone()),
                        )?;
                    }
                    None => return Ok(None),
                };
            }
        }

        // We should install deps & sync projects *before* running targets
        let mut edges = vec![];

        if let Some(install_deps_index) = self.install_deps(project, Some(task))? {
            edges.push(install_deps_index);
        }

        edges.push(self.sync_project(project)?);

        let index = self.insert_node(node);

        // And we also need to create edges for task dependencies
        if !task.deps.is_empty() {
            trace!(
                task = task.target.as_str(),
                deps = ?task.deps.iter().map(|d| d.target.as_str()).collect::<Vec<_>>(),
                "Linking dependencies for task",
            );

            edges.extend(self.run_task_dependencies(task)?);
        }

        self.link_requirements(index, edges);

        // And possibly dependents
        if reqs.dependents {
            self.run_task_dependents(task, reqs)?;
        }

        Ok(Some(index))
    }

    // We don't pass touched files to dependencies, because if the parent
    // task is affected/going to run, then so should all of these!
    #[instrument(skip_all)]
    pub fn run_task_dependencies(&mut self, task: &Task) -> miette::Result<Vec<NodeIndex>> {
        let parallel = task.options.run_deps_in_parallel;
        let reqs = RunRequirements {
            downstream_target: if self.affected.is_some() {
                Some(task.target.clone())
            } else {
                None
            },
            ..Default::default()
        };
        let mut indices = vec![];
        let mut previous_target_index = None;

        for dep in &task.deps {
            let (_, dep_indices) =
                self.run_task_by_target_with_config(&dep.target, &reqs, Some(dep))?;

            for dep_index in dep_indices {
                // When parallel, parent depends on child
                if parallel {
                    indices.push(dep_index);

                    // When serial, next child depends on previous child
                } else if let Some(prev) = previous_target_index {
                    self.link_requirements(dep_index, vec![prev]);
                }

                previous_target_index = Some(dep_index);
            }
        }

        if !parallel {
            if let Some(index) = previous_target_index {
                indices.push(index);
            }
        }

        Ok(indices)
    }

    // This is costly, is there a better way to do this?
    #[instrument(skip_all)]
    pub fn run_task_dependents(
        &mut self,
        task: &Task,
        parent_reqs: &RunRequirements,
    ) -> miette::Result<Vec<NodeIndex>> {
        let mut indices = vec![];

        // Create a new requirements object as we only want direct
        // dependents, and shouldn't recursively create.
        let reqs = RunRequirements {
            ci: parent_reqs.ci,
            interactive: parent_reqs.interactive,
            skip_affected: true,
            upstream_target: if self.affected.is_some() {
                Some(task.target.clone())
            } else {
                None
            },
            ..Default::default()
        };

        if let TargetScope::Project(project_locator) = &task.target.scope {
            let mut projects_to_build = vec![];

            // From self project
            let self_project = self.workspace_graph.get_project(project_locator)?;

            projects_to_build.push(self_project.clone());

            // From other projects
            for dependent_id in self.workspace_graph.projects.dependents_of(&self_project) {
                projects_to_build.push(self.workspace_graph.get_project(&dependent_id)?);
            }

            for project in projects_to_build {
                // Don't skip internal tasks, since they are a dependency of the parent
                // task, and must still run! They just can't be ran manually.
                for dep_task in self.workspace_graph.get_tasks_from_project(&project.id)? {
                    // But do skip persistent tasks!
                    if dep_task.is_persistent() {
                        continue;
                    }

                    // Since these are transient tasks, we should always filter out tasks
                    // that should not run in CI, as we don't know what side-effects it
                    // will cause. This applies to both `moon ci` and `moon run`.
                    if parent_reqs.ci && !dep_task.should_run_in_ci() {
                        self.passthrough_targets.insert(dep_task.target.clone());
                        continue;
                    }

                    if dep_task.deps.iter().any(|dep| dep.target == task.target) {
                        if let Some(index) = self.run_task(&project, &dep_task, &reqs)? {
                            indices.push(index);
                        }
                    }
                }
            }
        }

        Ok(indices)
    }

    pub fn run_task_by_target<T: AsRef<Target>>(
        &mut self,
        target: T,
        reqs: &RunRequirements,
    ) -> miette::Result<(FxHashSet<Target>, FxHashSet<NodeIndex>)> {
        self.run_task_by_target_with_config(target, reqs, None)
    }

    pub fn run_task_by_target_with_config<T: AsRef<Target>>(
        &mut self,
        target: T,
        reqs: &RunRequirements,
        config: Option<&TaskDependencyConfig>,
    ) -> miette::Result<(FxHashSet<Target>, FxHashSet<NodeIndex>)> {
        let target = target.as_ref();
        let mut inserted_targets = FxHashSet::default();
        let mut inserted_indices = FxHashSet::default();

        match &target.scope {
            // :task
            TargetScope::All => {
                let mut projects = vec![];

                if let Some(all_query) = &self.all_query {
                    projects.extend(self.workspace_graph.query_projects(all_query)?);
                } else {
                    projects.extend(self.workspace_graph.get_all_projects()?);
                };

                for project in projects {
                    // Don't error if the task does not exist
                    if let Ok(task) = self
                        .workspace_graph
                        .get_task_from_project(&project.id, &target.task_id)
                    {
                        if task.is_internal() {
                            continue;
                        }

                        if let Some(index) =
                            self.run_task_with_config(&project, &task, reqs, config)?
                        {
                            inserted_targets.insert(task.target.clone());
                            inserted_indices.insert(index);
                        }
                    }
                }
            }
            // ^:task
            TargetScope::Deps => {
                return Err(TargetError::NoDepsInRunContext.into());
            }
            // project:task
            TargetScope::Project(project_locator) => {
                let project = self.workspace_graph.get_project(project_locator)?;
                let task = self
                    .workspace_graph
                    .get_task_from_project(&project.id, &target.task_id)?;

                // Don't allow internal tasks to be ran
                if task.is_internal() && reqs.has_target(&task.target) {
                    return Err(TaskGraphError::UnconfiguredTarget(task.target.clone()).into());
                }

                if let Some(index) = self.run_task_with_config(&project, &task, reqs, config)? {
                    inserted_targets.insert(task.target.to_owned());
                    inserted_indices.insert(index);
                }
            }
            // #tag:task
            TargetScope::Tag(tag) => {
                let projects = self
                    .workspace_graph
                    .query_projects(build_query(format!("tag={}", tag).as_str())?)?;

                for project in projects {
                    // Don't error if the task does not exist
                    if let Ok(task) = self
                        .workspace_graph
                        .get_task_from_project(&project.id, &target.task_id)
                    {
                        if task.is_internal() {
                            continue;
                        }

                        if let Some(index) =
                            self.run_task_with_config(&project, &task, reqs, config)?
                        {
                            inserted_targets.insert(task.target.clone());
                            inserted_indices.insert(index);
                        }
                    }
                }
            }
            // ~:task
            TargetScope::OwnSelf => {
                return Err(TargetError::NoSelfInRunContext.into());
            }
        };

        Ok((inserted_targets, inserted_indices))
    }

    #[instrument(skip_all)]
    pub fn run_from_requirements(
        &mut self,
        reqs: RunRequirements,
    ) -> miette::Result<Vec<NodeIndex>> {
        let mut inserted_nodes = vec![];

        for locator in reqs.target_locators.clone() {
            let target = match locator {
                TargetLocator::Qualified(target) => target,
                TargetLocator::TaskFromWorkingDir(task_id) => Target::new(
                    &self.workspace_graph.get_project_from_path(None)?.id,
                    task_id,
                )?,
            };

            // Track the qualified as an initial target
            self.initial_targets.insert(target.clone());

            // Run the target
            let (inserted_targets, inserted_indices) = self.run_task_by_target(target, &reqs)?;

            // Track the primary targets
            self.primary_targets.extend(inserted_targets);

            inserted_nodes.extend(inserted_indices);
        }

        Ok(inserted_nodes)
    }

    #[instrument(skip_all)]
    pub fn setup_toolchain(&mut self, runtime: &Runtime) -> NodeIndex {
        let node = ActionNode::setup_toolchain(SetupToolchainNode {
            runtime: runtime.to_owned(),
        });

        if let Some(index) = self.get_index_from_node(&node) {
            return *index;
        }

        let sync_workspace_index = self.sync_workspace();
        let index = self.insert_node(node);

        self.link_requirements(index, vec![sync_workspace_index]);

        index
    }

    #[instrument(skip_all)]
    pub fn sync_project(&mut self, project: &Project) -> miette::Result<NodeIndex> {
        self.internal_sync_project(project, &mut FxHashSet::default())
    }

    fn internal_sync_project(
        &mut self,
        project: &Project,
        cycle: &mut FxHashSet<Id>,
    ) -> miette::Result<NodeIndex> {
        let node = ActionNode::sync_project(SyncProjectNode {
            project: project.id.clone(),
            runtime: self.get_runtime(project, project.platform, true),
        });

        if let Some(index) = self.get_index_from_node(&node) {
            return Ok(*index);
        }

        cycle.insert(project.id.clone());

        // Determine affected state
        if let Some(affected) = &mut self.affected {
            if let Some(by) = affected.is_project_affected(project) {
                affected.mark_project_affected(project, by)?;
            }
        }

        // Syncing requires the language's tool to be installed
        let setup_tool_index = self.setup_toolchain(node.get_runtime());
        let index = self.insert_node(node);
        let mut edges = vec![setup_tool_index];

        // And we should also depend on other projects
        for dep_project_id in self.workspace_graph.projects.dependencies_of(project) {
            if cycle.contains(&dep_project_id) {
                continue;
            }

            let dep_project = self.workspace_graph.get_project(&dep_project_id)?;
            let dep_project_index = self.internal_sync_project(&dep_project, cycle)?;

            if index != dep_project_index {
                edges.push(dep_project_index);
            }
        }

        self.link_requirements(index, edges);

        Ok(index)
    }

    pub fn sync_workspace(&mut self) -> NodeIndex {
        let node = ActionNode::sync_workspace();

        if let Some(index) = self.get_index_from_node(&node) {
            return *index;
        }

        self.insert_node(node)
    }

    // PRIVATE

    fn link_requirements(&mut self, index: NodeIndex, edges: Vec<NodeIndex>) {
        trace!(
            index = index.index(),
            requires = ?edges.iter().map(|i| i.index()).collect::<Vec<_>>(),
            "Linking requirements for index"
        );

        for edge in edges {
            self.graph.add_edge(index, edge, ());
        }
    }

    fn insert_node(&mut self, node: ActionNode) -> NodeIndex {
        let index = self.graph.add_node(node.clone());

        debug!(
            index = index.index(),
            "Adding {} to graph",
            color::muted_light(node.label())
        );

        self.indices.insert(node, index);

        index
    }
}
