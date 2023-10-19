use crate::action_graph::ActionGraph;
use crate::action_node::ActionNode;
use moon_common::{color, path::WorkspaceRelativePathBuf};
use moon_platform::{PlatformManager, Runtime};
use moon_project::Project;
use moon_project_graph::ProjectGraph;
use moon_query::{build_query, Criteria};
use moon_task::{Target, TargetError, TargetLocator, TargetScope, Task};
use petgraph::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::{debug, trace};

type TouchedFilePaths = FxHashSet<WorkspaceRelativePathBuf>;

pub struct ActionGraphBuilder<'app> {
    all_query: Option<Criteria>,
    dependents: bool,
    graph: DiGraph<ActionNode, ()>,
    indices: FxHashMap<ActionNode, NodeIndex>,
    interactive: bool,
    platform_manager: &'app PlatformManager,
    project_graph: &'app ProjectGraph,
}

impl<'app> ActionGraphBuilder<'app> {
    pub fn new(project_graph: &'app ProjectGraph) -> miette::Result<Self> {
        ActionGraphBuilder::with_platforms(PlatformManager::read(), project_graph)
    }

    pub fn with_platforms(
        platform_manager: &'app PlatformManager,
        project_graph: &'app ProjectGraph,
    ) -> miette::Result<Self> {
        debug!("Building action graph");

        Ok(ActionGraphBuilder {
            all_query: None,
            graph: DiGraph::new(),
            dependents: false,
            indices: FxHashMap::default(),
            interactive: false,
            platform_manager,
            project_graph,
        })
    }

    pub fn build(self) -> miette::Result<ActionGraph> {
        Ok(ActionGraph::new(self.graph))
    }

    pub fn get_index_from_node(&self, node: &ActionNode) -> Option<&NodeIndex> {
        self.indices.get(node)
    }

    pub fn get_runtime(
        &self,
        project: &Project,
        task: Option<&Task>,
        allow_override: bool,
    ) -> Runtime {
        if let Some(platform) = self.platform_manager.find(|p| match task {
            Some(task) => p.matches(&task.platform, None),
            None => p.matches(&project.platform, None),
        }) {
            return platform.get_runtime_from_config(if allow_override {
                Some(&project.config)
            } else {
                None
            });
        }

        Runtime::system()
    }

    pub fn force_interactive(&mut self) {
        self.interactive = true;
    }

    pub fn include_dependents(&mut self) {
        self.dependents = true;
    }

    pub fn set_query(&mut self, input: &str) -> miette::Result<()> {
        self.all_query = Some(build_query(input)?);

        Ok(())
    }

    // ACTIONS

    pub fn install_deps(
        &mut self,
        project: &Project,
        task: Option<&Task>,
    ) -> miette::Result<Option<NodeIndex>> {
        let mut in_project = false;

        // If project is NOT in the package manager workspace, then we should
        // install dependencies in the project, not the workspace root.
        if let Ok(platform) = self.platform_manager.get(project.language.clone()) {
            if !platform.is_project_in_dependency_workspace(project.source.as_str())? {
                in_project = true;

                debug!(
                    "Project {} is not within the dependency manager workspace, dependencies will be installed within the project instead of the root",
                    color::id(&project.id),
                );
            }
        }

        let node = if in_project {
            ActionNode::InstallProjectDeps {
                project: project.id.to_owned(),
                runtime: self.get_runtime(project, task, true),
            }
        } else {
            ActionNode::InstallDeps {
                runtime: self.get_runtime(project, task, false),
            }
        };

        if node.get_runtime().platform.is_system() {
            return Ok(None);
        }

        if let Some(index) = self.get_index_from_node(&node) {
            return Ok(Some(*index));
        }

        // Before we install deps, we must ensure the language has been installed
        let setup_tool_index = self.setup_tool(node.get_runtime());
        let index = self.insert_node(node);

        self.link_requirements(index, vec![setup_tool_index]);

        Ok(Some(index))
    }

    pub fn run_task(
        &mut self,
        project: &Project,
        task: &Task,
        touched_files: Option<&TouchedFilePaths>,
    ) -> miette::Result<Option<NodeIndex>> {
        let node = ActionNode::RunTask {
            interactive: task.is_interactive(),
            persistent: task.is_persistent(),
            runtime: self.get_runtime(project, Some(task), true),
            target: task.target.to_owned(),
        };

        if let Some(index) = self.get_index_from_node(&node) {
            return Ok(Some(*index));
        }

        // Compare against touched files if provided
        if let Some(touched) = touched_files {
            if !task.is_affected(touched)? {
                return Ok(None);
            }
        }

        // We should install deps & sync projects *before* running targets
        let mut reqs = vec![];

        if let Some(install_deps_index) = self.install_deps(project, Some(task))? {
            reqs.push(install_deps_index);
        }

        reqs.push(self.sync_project(project)?);

        let index = self.insert_node(node);

        // And we also need to create edges for task dependencies
        if !task.deps.is_empty() {
            trace!(
                task = task.target.as_str(),
                deps = ?task.deps.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
                "Linking dependencies for task",
            );

            reqs.extend(self.run_task_dependencies(task)?);
        }

        self.link_requirements(index, reqs);

        // And possibly dependents
        if self.dependents {
            self.run_task_dependents(task)?;
        }

        Ok(Some(index))
    }

    // We don't pass touched files to dependencies, because if the parent
    // task is affected/going to run, then so should all of these!
    pub fn run_task_dependencies(&mut self, task: &Task) -> miette::Result<Vec<NodeIndex>> {
        let parallel = task.options.run_deps_in_parallel;
        let mut indices = vec![];
        let mut previous_target_index = None;

        for dep_target in &task.deps {
            let (_, dep_indices) = self.run_task_by_target(dep_target, None)?;

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
            indices.push(previous_target_index.unwrap());
        }

        Ok(indices)
    }

    // This is costly, is there a better way to do this?
    pub fn run_task_dependents(&mut self, task: &Task) -> miette::Result<Vec<NodeIndex>> {
        let mut indices = vec![];

        if let TargetScope::Project(project_locator) = &task.target.scope {
            let project = self.project_graph.get(project_locator)?;

            // From self project
            for dep_task in project.tasks.values() {
                if dep_task.deps.contains(&task.target) {
                    if dep_task.is_persistent() {
                        continue;
                    }

                    if let Some(index) = self.run_task(&project, dep_task, None)? {
                        indices.push(index);
                    }
                }
            }

            // From other projects
            for dependent_id in self.project_graph.dependents_of(&project)? {
                let dep_project = self.project_graph.get(dependent_id)?;

                for dep_task in dep_project.tasks.values() {
                    if dep_task.is_persistent() {
                        continue;
                    }

                    if dep_task.deps.contains(&task.target) {
                        if let Some(index) = self.run_task(&dep_project, dep_task, None)? {
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
        touched_files: Option<&TouchedFilePaths>,
    ) -> miette::Result<(FxHashSet<Target>, FxHashSet<NodeIndex>)> {
        let target = target.as_ref();
        let mut inserted_targets = FxHashSet::default();
        let mut inserted_indices = FxHashSet::default();

        match &target.scope {
            // :task
            TargetScope::All => {
                let mut projects = vec![];

                if let Some(all_query) = &self.all_query {
                    projects.extend(self.project_graph.query(all_query)?);
                } else {
                    projects.extend(self.project_graph.get_all()?);
                };

                for project in projects {
                    // Don't error if the task does not exist
                    if let Ok(task) = project.get_task(&target.task_id) {
                        if let Some(index) = self.run_task(&project, task, touched_files)? {
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
                let project = self.project_graph.get(project_locator)?;
                let task = project.get_task(&target.task_id)?;

                if let Some(index) = self.run_task(&project, task, touched_files)? {
                    inserted_targets.insert(task.target.to_owned());
                    inserted_indices.insert(index);
                }
            }
            // #tag:task
            TargetScope::Tag(tag) => {
                let projects = self
                    .project_graph
                    .query(build_query(format!("tag={}", tag))?)?;

                for project in projects {
                    // Don't error if the task does not exist
                    if let Ok(task) = project.get_task(&target.task_id) {
                        if let Some(index) = self.run_task(&project, task, touched_files)? {
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

    pub fn run_task_by_target_locator<T: AsRef<TargetLocator>>(
        &mut self,
        target_locator: T,
        touched_files: Option<&TouchedFilePaths>,
    ) -> miette::Result<(FxHashSet<Target>, FxHashSet<NodeIndex>)> {
        match target_locator.as_ref() {
            TargetLocator::Qualified(target) => self.run_task_by_target(target, touched_files),
            TargetLocator::TaskFromWorkingDir(task_id) => self.run_task_by_target(
                Target::new(&self.project_graph.get_from_path(None)?.id, task_id)?,
                touched_files,
            ),
        }
    }

    pub fn setup_tool(&mut self, runtime: &Runtime) -> NodeIndex {
        let node = ActionNode::SetupTool {
            runtime: runtime.to_owned(),
        };

        if let Some(index) = self.get_index_from_node(&node) {
            return *index;
        }

        let sync_workspace_index = self.sync_workspace();
        let index = self.insert_node(node);

        self.link_requirements(index, vec![sync_workspace_index]);

        index
    }

    pub fn sync_project(&mut self, project: &Project) -> miette::Result<NodeIndex> {
        let node = ActionNode::SyncProject {
            project: project.id.clone(),
            runtime: self.get_runtime(project, None, true),
        };

        if let Some(index) = self.get_index_from_node(&node) {
            return Ok(*index);
        }

        // Syncing requires the language's tool to be installed
        let setup_tool_index = self.setup_tool(node.get_runtime());
        let index = self.insert_node(node);
        let mut reqs = vec![setup_tool_index];

        // And we should also depend on other projects
        for dep_project_id in self.project_graph.dependencies_of(project)? {
            let dep_project = self.project_graph.get(dep_project_id)?;
            let dep_project_index = self.sync_project(&dep_project)?;

            if index != dep_project_index {
                reqs.push(dep_project_index);
            }
        }

        self.link_requirements(index, reqs);

        Ok(index)
    }

    pub fn sync_workspace(&mut self) -> NodeIndex {
        let node = ActionNode::SyncWorkspace;

        if let Some(index) = self.get_index_from_node(&node) {
            return *index;
        }

        self.insert_node(node)
    }

    // PRIVATE

    fn link_requirements(&mut self, index: NodeIndex, reqs: Vec<NodeIndex>) {
        trace!(
            index = index.index(),
            requires = ?reqs.iter().map(|i| i.index()).collect::<Vec<_>>(),
            "Linking requirements for index"
        );

        for req in reqs {
            self.graph.add_edge(index, req, ());
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