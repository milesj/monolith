use crate::errors::ProjectGraphError;
use crate::graph_hasher::GraphHasher;
use crate::helpers::detect_projects_with_globs;
use crate::project_graph::{GraphType, IndicesType, ProjectGraph, LOG_TARGET};
use crate::token_resolver::{TokenContext, TokenResolver};
use moon_common::path::WorkspaceRelativePathBuf;
use moon_common::{consts, Id};
use moon_config::{InputPath, ProjectsAliasesMap, ProjectsSourcesMap, WorkspaceProjects};
use moon_enforcer::{enforce_project_type_relationships, enforce_tag_relationships};
use moon_hasher::{convert_paths_to_strings, to_hash};
use moon_logger::{debug, map_list, trace, warn};
use moon_platform_detector::{detect_project_language, detect_task_platform};
use moon_project::Project;
use moon_project_builder::{ProjectBuilder, ProjectBuilderError};
use moon_target::{Target, TargetScope};
use moon_task::Task;
use moon_utils::regex::ENV_VAR_SUBSTITUTE;
use moon_utils::{path, time};
use moon_workspace::Workspace;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use rustc_hash::{FxHashMap, FxHashSet};
use starbase_styles::color;
use starbase_utils::glob;
use std::collections::BTreeMap;
use std::env;
use std::mem;

pub struct ProjectGraphBuilder<'ws> {
    workspace: &'ws mut Workspace,

    aliases: ProjectsAliasesMap,
    graph: GraphType,
    indices: IndicesType,
    sources: ProjectsSourcesMap,

    // Project and its dependencies being created.
    // We use this to prevent circular dependencies.
    created: FxHashSet<Id>,

    pub is_cached: bool,
    pub hash: String,
}

impl<'ws> ProjectGraphBuilder<'ws> {
    pub async fn new(workspace: &'ws mut Workspace) -> miette::Result<ProjectGraphBuilder<'ws>> {
        debug!(target: LOG_TARGET, "Creating project graph");

        let mut graph = ProjectGraphBuilder {
            aliases: FxHashMap::default(),
            created: FxHashSet::default(),
            graph: DiGraph::new(),
            hash: String::new(),
            indices: FxHashMap::default(),
            is_cached: false,
            sources: FxHashMap::default(),
            workspace,
        };

        graph.preload().await?;

        Ok(graph)
    }

    pub fn build(&mut self) -> miette::Result<ProjectGraph> {
        self.enforce_constraints()?;

        Ok(ProjectGraph::new(
            mem::take(&mut self.graph),
            mem::take(&mut self.indices),
            mem::take(&mut self.sources),
            mem::take(&mut self.aliases),
        ))
    }

    pub fn load(&mut self, alias_or_id: &str) -> miette::Result<&Self> {
        self.internal_load(alias_or_id)?;

        Ok(self)
    }

    pub fn load_all(&mut self) -> miette::Result<&Self> {
        // TODO: Don't clone data here, but satisfying the borrow checker
        // is almost impossible here without a major refactor!
        let ids = self
            .sources
            .keys()
            .map(|k| k.to_owned())
            .collect::<Vec<Id>>();

        for id in ids {
            self.internal_load(&id)?;
        }

        Ok(self)
    }

    /// Create a project with the provided ID and file path source. Based on the project's
    /// configured language, detect and infer implicit dependencies and tasks for the
    /// matching platform. Do *not* expand tasks until after dependents have been created.
    fn create_project(&self, id: &Id, source: &str) -> miette::Result<Project> {
        let mut builder = ProjectBuilder::new(id, source, &self.workspace.root)?;

        builder.detect_language(detect_project_language);
        builder.detect_platform(detect_task_platform, &self.workspace.toolchain_config);
        builder.load_local_config()?;
        builder.inherit_global_config(&self.workspace.tasks_config)?;

        if let Ok(platform) = self.workspace.platforms.get(builder.language.clone()) {
            // Inherit implicit dependencies
            for dep_config in
                platform.load_project_implicit_dependencies(id, source, &self.aliases)?
            {
                builder.extend_with_dependency(dep_config);
            }

            // Inherit platform specific tasks
            for (task_id, task_config) in platform.load_project_tasks(id, source)? {
                builder.extend_with_task(task_id, task_config);
            }
        }

        let mut project = builder.build()?;

        // Collect all aliases for the current project ID
        for (alias, project_id) in &self.aliases {
            if project_id == id {
                project.alias = Some(alias.to_owned());
            }
        }

        Ok(project)
    }

    fn enforce_constraints(&self) -> miette::Result<()> {
        let type_relationships = self
            .workspace
            .config
            .constraints
            .enforce_project_type_relationships;
        let tag_relationships = &self.workspace.config.constraints.tag_relationships;

        for project in self.graph.node_weights() {
            let deps: Vec<_> = self
                .graph
                .neighbors_directed(*self.indices.get(&project.id).unwrap(), Direction::Outgoing)
                .map(|idx| self.graph.node_weight(idx).unwrap())
                .collect();

            // Enforce project constraints and boundaries.
            for dep in deps {
                if type_relationships {
                    enforce_project_type_relationships(project, dep)?;
                }

                for (source_tag, required_tags) in tag_relationships {
                    enforce_tag_relationships(project, source_tag, dep, required_tags)?;
                }
            }

            // Validate non-persistent tasks dont depend on persistent tasks
            for task in project.tasks.values() {
                for dep_target in &task.deps {
                    let TargetScope::Project(maybe_project_id) = &dep_target.scope else {
                        continue;
                    };
                    let project_id = self.resolve_id(maybe_project_id);

                    let dep_task = if project_id == project.id {
                        project.tasks.get(&dep_target.task_id)
                    } else {
                        let Some(dep_index) = self.indices.get(&project_id) else {
                            // Our tests are wonky, fix later...
                            continue;
                        };

                        let dep_project = self.graph.node_weight(*dep_index).unwrap();

                        dep_project.tasks.get(&dep_target.task_id)
                    };

                    if let Some(dep_task) = dep_task {
                        if !task.is_persistent() && dep_task.is_persistent() {
                            return Err(ProjectGraphError::PersistentDepRequirement(
                                task.target.to_string(),
                                dep_task.target.to_string(),
                            )
                            .into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Expand all tasks within a project, by expanding data and resolving any tokens.
    /// This must run *after* dependent projects have been created, as we require them
    /// to resolve "parent" relations.
    fn expand_project(&mut self, project: &mut Project) -> miette::Result<()> {
        let mut tasks = BTreeMap::new();

        // Use `mem::take` so that we can mutably borrow the project and tasks in parallel
        for (task_id, mut task) in mem::take(&mut project.tasks) {
            // Resolve in this order!
            self.expand_task_env(project, &mut task)?;
            self.expand_task_deps(project, &mut task)?;
            self.expand_task_inputs(project, &mut task)?;
            self.expand_task_outputs(project, &mut task)?;
            self.expand_task_args(project, &mut task)?;
            self.expand_task_command(project, &mut task)?;

            tasks.insert(task_id, task);
        }

        project.tasks.extend(tasks);

        Ok(())
    }

    pub fn expand_task_command(
        &self,
        project: &mut Project,
        task: &mut Task,
    ) -> miette::Result<()> {
        task.command = TokenResolver::new(TokenContext::Command, project, &self.workspace.root)
            .resolve_command(task)?;

        Ok(())
    }

    /// Expand the args list to resolve tokens, relative to the project root.
    pub fn expand_task_args(&self, project: &mut Project, task: &mut Task) -> miette::Result<()> {
        if task.args.is_empty() {
            return Ok(());
        }

        let mut args: Vec<String> = vec![];

        // When running within a project:
        //  - Project paths are relative and start with "./"
        //  - Workspace paths are relative up to the root
        // When running from the workspace:
        //  - All paths are absolute
        let handle_path =
            |path: WorkspaceRelativePathBuf, is_glob: bool| -> miette::Result<String> {
                let arg = path::to_virtual_string(
                    path::relative_from(
                        path.to_path(&self.workspace.root),
                        if task.options.run_from_workspace_root {
                            &self.workspace.root
                        } else {
                            &project.root
                        },
                    )
                    .unwrap(),
                )?;

                let arg = if arg.starts_with("..") {
                    arg
                } else {
                    format!("./{}", arg)
                };

                if is_glob {
                    return Ok(glob::normalize(arg)?);
                }

                Ok(arg)
            };

        // We cant use `TokenResolver.resolve` as args are a mix of strings,
        // strings with tokens, and file paths when tokens are resolved.
        let token_resolver = TokenResolver::new(TokenContext::Args, project, &self.workspace.root);

        for arg in &task.args {
            if token_resolver.has_token_func(arg) {
                let (paths, globs) = token_resolver.resolve_func(arg, task)?;

                for path in paths {
                    args.push(handle_path(path, false)?);
                }

                for glob in globs {
                    args.push(handle_path(glob, true)?);
                }
            } else if token_resolver.has_token_var(arg) {
                args.push(token_resolver.resolve_vars(arg, task)?);
            } else {
                args.push(arg.clone());
            }
        }

        task.args = args;

        Ok(())
    }

    /// Expand the deps list and resolve parent/self scopes.
    pub fn expand_task_deps(&self, project: &mut Project, task: &mut Task) -> miette::Result<()> {
        if task.deps.is_empty() {
            return Ok(());
        }

        let mut dep_targets: Vec<Target> = vec![];

        // Dont use a `HashSet` as we want to preserve order
        let mut push_target = |dep: Target| {
            if !dep_targets.contains(&dep) {
                dep_targets.push(dep);
            }
        };

        for dep_target in &task.deps {
            match &dep_target.scope {
                // ^:task
                TargetScope::Deps => {
                    for dep_id in project.get_dependency_ids() {
                        let dep_index = self.indices.get(dep_id).unwrap();
                        let dep_project = self.graph.node_weight(*dep_index).unwrap();

                        if let Some(dep_task) = dep_project.tasks.get(&dep_target.task_id) {
                            push_target(dep_task.target.clone());
                        }
                    }
                }
                // ~:task
                TargetScope::OwnSelf => {
                    if dep_target.task_id == task.id {
                        // Avoid circular references
                    } else {
                        push_target(Target::new(&project.id, &dep_target.task_id)?);
                    }
                }
                // project:task
                TargetScope::Project(project_id) => {
                    if project_id == &project.id && dep_target.task_id == task.id {
                        // Avoid circular references
                    } else {
                        push_target(dep_target.clone());
                    }
                }
                // :task
                // #tag:task
                _ => {
                    return Err(ProjectGraphError::PersistentDepRequirement(
                        dep_target.to_string(),
                        task.target.to_string(),
                    )
                    .into());
                }
            };
        }

        task.deps = dep_targets;

        Ok(())
    }

    /// Expand environment variables by loading a `.env` file if configured.
    pub fn expand_task_env(&self, _project: &mut Project, task: &mut Task) -> miette::Result<()> {
        task.env.iter_mut().for_each(|(_, value)| {
            while let Some(matches) = ENV_VAR_SUBSTITUTE.captures(value) {
                let sub = matches.get(0).unwrap().as_str();
                let sub_key = matches.get(1).unwrap().as_str();
                let sub_value = env::var(sub_key).unwrap_or_default();

                *value = value.replace(sub, &sub_value);
            }
        });

        Ok(())
    }

    /// Expand the inputs list to a set of absolute file paths, while resolving tokens.
    pub fn expand_task_inputs(&self, project: &mut Project, task: &mut Task) -> miette::Result<()> {
        task.inputs.retain(|input| {
            if let InputPath::EnvVar(var) = input {
                task.input_vars.insert(var.to_owned());
                false
            } else {
                true
            }
        });

        let mut inputs_to_resolve = vec![];
        inputs_to_resolve.extend(&task.inputs);

        if inputs_to_resolve.is_empty() {
            return Ok(());
        }

        let token_resolver =
            TokenResolver::new(TokenContext::Inputs, project, &self.workspace.root);
        let (paths, globs) = token_resolver.resolve_inputs(&inputs_to_resolve, task)?;

        task.input_paths.extend(paths);
        task.input_globs.extend(globs);

        Ok(())
    }

    /// Expand the outputs list to a set of absolute file paths, while resolving tokens.
    pub fn expand_task_outputs(
        &self,
        project: &mut Project,
        task: &mut Task,
    ) -> miette::Result<()> {
        if task.outputs.is_empty() {
            return Ok(());
        }

        let token_resolver =
            TokenResolver::new(TokenContext::Outputs, project, &self.workspace.root);
        let (paths, globs) = token_resolver.resolve_outputs(&task.outputs, task)?;

        task.output_globs.extend(globs);

        for path in paths {
            // Inputs must not consider outputs as a source
            if task.input_paths.contains(&path) {
                task.input_paths.remove(&path);
            }

            task.output_paths.insert(path);
        }

        Ok(())
    }

    fn internal_load(&mut self, alias_or_id: &str) -> miette::Result<NodeIndex> {
        let id = self.resolve_id(alias_or_id);

        // Already loaded, abort early
        if let Some(index) = self.indices.get(&id) {
            trace!(
                target: LOG_TARGET,
                "Project {} already exists in the project graph",
                color::id(id),
            );

            return Ok(*index);
        }

        trace!(
            target: LOG_TARGET,
            "Project {} does not exist in the project graph, attempting to load",
            color::id(&id),
        );

        // Create the current project
        let Some(source) = self.sources.get(&id) else {
            return Err(ProjectBuilderError::UnconfiguredID(id.to_string()).into());
        };

        let mut project = self.create_project(&id, source)?;

        self.created.insert(id.clone());

        // Create dependent projects
        let mut dep_indices = FxHashSet::default();

        for dep_id in project.get_dependency_ids() {
            if self.created.contains(dep_id) {
                warn!(
                    target: LOG_TARGET,
                    "Found a cycle between {} and {}, and will disconnect nodes to avoid recursion",
                    color::id(&id),
                    color::id(dep_id),
                );
            } else {
                dep_indices.insert(self.internal_load(dep_id)?);
            }
        }

        // Expand tasks for the current project
        self.expand_project(&mut project)?;

        // Insert into the graph and connect edges
        let index = self.graph.add_node(project);

        self.indices.insert(id, index);

        for dep_index in dep_indices {
            self.graph.add_edge(index, dep_index, ());
        }

        // Reset for the next project
        self.created.clear();

        Ok(index)
    }

    async fn preload(&mut self) -> miette::Result<()> {
        let mut globs = vec![];
        let mut sources: ProjectsSourcesMap = FxHashMap::default();
        let mut aliases: ProjectsAliasesMap = FxHashMap::default();
        let mut cache = self.workspace.cache.cache_projects_state()?;

        let mut add_sources = |map: &FxHashMap<Id, String>| -> miette::Result<()> {
            for (id, source) in map {
                sources.insert(id.to_owned(), path::standardize_separators(source));
            }

            Ok(())
        };

        // Load project sources
        match &self.workspace.config.projects {
            WorkspaceProjects::Sources(map) => {
                add_sources(map)?;
            }
            WorkspaceProjects::Globs(list) => {
                globs.extend(list.clone());
            }
            WorkspaceProjects::Both {
                globs: list,
                sources: map,
            } => {
                globs.extend(list.clone());
                add_sources(map)?;
            }
        };

        if !globs.is_empty() {
            debug!(
                target: LOG_TARGET,
                "Finding projects with globs: {}",
                map_list(&globs, |g| color::file(g))
            );

            detect_projects_with_globs(
                &self.workspace.root,
                &globs,
                &mut sources,
                Some(&self.workspace.vcs),
            )?;

            cache.last_glob_time = time::now_millis();
        }

        // Load project aliases
        for platform in self.workspace.platforms.list_mut() {
            platform.load_project_graph_aliases(&sources, &mut aliases)?;
        }

        // Update the cache
        let hash = self.generate_hash(&sources, &aliases).await?;

        if !hash.is_empty() {
            self.is_cached = cache.last_hash == hash;
            self.hash = hash.clone();

            debug!(
                target: LOG_TARGET,
                "Generated hash {} for project graph",
                color::hash(&hash),
            );
        }

        self.aliases.extend(aliases.clone());
        self.sources.extend(sources.clone());

        cache.last_hash = hash;
        cache.globs = globs;
        cache.projects = sources;
        cache.save()?;

        if self.is_cached {
            debug!(
                target: LOG_TARGET,
                "Loading project graph with {} projects from cache",
                self.sources.len(),
            );
        } else {
            debug!(
                target: LOG_TARGET,
                "Creating project graph with {} projects",
                self.sources.len(),
            );
        }

        Ok(())
    }

    async fn generate_hash(
        &self,
        sources: &ProjectsSourcesMap,
        aliases: &ProjectsAliasesMap,
    ) -> miette::Result<String> {
        if !self.workspace.vcs.is_enabled() {
            return Ok(String::new());
        }

        let mut hasher = GraphHasher::new();

        // Hash aliases and sources as-is as they're very explicit
        hasher.hash_aliases(aliases);
        hasher.hash_sources(sources);

        // Hash all project-level config files, as a single change in any of
        // these files would invalidate the entire project graph cache!
        // TODO: handle extended config files?
        let project_configs = convert_paths_to_strings(
            &FxHashSet::from_iter(sources.values().map(|source| {
                if source == "." {
                    self.workspace.root.join(consts::CONFIG_PROJECT_FILENAME)
                } else {
                    self.workspace
                        .root
                        .join(source)
                        .join(consts::CONFIG_PROJECT_FILENAME)
                }
            })),
            &self.workspace.root,
        )?;

        // Hash all workspace-level config files for the same reason!
        let workspace_configs = convert_paths_to_strings(
            &FxHashSet::from_iter(glob::walk(
                self.workspace.root.join(consts::CONFIG_DIRNAME),
                ["*.yml", "tasks/*.yml"],
            )?),
            &self.workspace.root,
        )?;

        // Hash all the configs!
        let mut configs = Vec::with_capacity(project_configs.len() + workspace_configs.len());
        configs.extend(project_configs);
        configs.extend(workspace_configs);

        let config_hashes = self
            .workspace
            .vcs
            .get_file_hashes(&configs, false, 100)
            .await?;

        hasher.hash_configs(&config_hashes);

        // Generate the hash
        let hash = to_hash(&hasher);

        self.workspace.cache.create_hash_manifest(&hash, &hasher)?;

        Ok(hash)
    }

    fn resolve_id(&self, alias_or_id: &str) -> Id {
        Id::raw(match self.aliases.get(alias_or_id) {
            Some(project_id) => project_id,
            None => alias_or_id,
        })
    }
}
