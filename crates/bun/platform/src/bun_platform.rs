use crate::actions;
use moon_action_context::ActionContext;
use moon_bun_lang::BUNPM;
use moon_bun_tool::BunTool;
use moon_common::Id;
use moon_config::{
    BunConfig, DependencyConfig, DependencyScope, DependencySource, HasherConfig, PlatformType,
    ProjectConfig, ProjectsAliasesMap, ProjectsSourcesMap, TypeScriptConfig, UnresolvedVersionSpec,
};
use moon_hash::{ContentHasher, DepsHash};
use moon_logger::{debug, warn};
use moon_node_lang::{node::get_package_manager_workspaces, PackageJson};
use moon_platform::{Platform, Runtime, RuntimeReq};
use moon_process::Command;
use moon_project::Project;
use moon_task::Task;
use moon_tool::{Tool, ToolManager};
use moon_typescript_platform::TypeScriptTargetHash;
use moon_utils::async_trait;
use proto_core::ProtoEnvironment;
use rustc_hash::FxHashMap;
use starbase_styles::color;
use starbase_utils::glob::GlobSet;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

const LOG_TARGET: &str = "moon:bun-platform";

pub struct BunPlatform {
    pub config: BunConfig,

    package_names: FxHashMap<String, Id>,

    proto_env: Arc<ProtoEnvironment>,

    toolchain: ToolManager<BunTool>,

    typescript_config: Option<TypeScriptConfig>,

    #[allow(dead_code)]
    pub workspace_root: PathBuf,
}

impl BunPlatform {
    pub fn new(
        config: &BunConfig,
        typescript_config: &Option<TypeScriptConfig>,
        workspace_root: &Path,
        proto_env: Arc<ProtoEnvironment>,
    ) -> Self {
        BunPlatform {
            config: config.to_owned(),
            package_names: FxHashMap::default(),
            proto_env,
            toolchain: ToolManager::new(Runtime::new(PlatformType::Bun, RuntimeReq::Global)),
            typescript_config: typescript_config.to_owned(),
            workspace_root: workspace_root.to_path_buf(),
        }
    }
}

#[async_trait]
impl Platform for BunPlatform {
    fn get_type(&self) -> PlatformType {
        PlatformType::Bun
    }

    fn get_runtime_from_config(&self, project_config: Option<&ProjectConfig>) -> Runtime {
        if let Some(config) = &project_config {
            if let Some(bun_config) = &config.toolchain.bun {
                if let Some(version) = &bun_config.version {
                    return Runtime::new_override(
                        PlatformType::Bun,
                        RuntimeReq::Toolchain(version.to_owned()),
                    );
                }
            }
        }

        if let Some(version) = &self.config.version {
            return Runtime::new(PlatformType::Bun, RuntimeReq::Toolchain(version.to_owned()));
        }

        Runtime::new(PlatformType::Bun, RuntimeReq::Global)
    }

    fn matches(&self, platform: &PlatformType, runtime: Option<&Runtime>) -> bool {
        if matches!(platform, PlatformType::Bun) {
            return true;
        }

        if let Some(runtime) = &runtime {
            return matches!(runtime.platform, PlatformType::Bun);
        }

        false
    }

    // PROJECT GRAPH

    fn is_project_in_dependency_workspace(&self, project_source: &str) -> miette::Result<bool> {
        let mut in_workspace = false;

        // Root package is always considered within the workspace
        if project_source.is_empty() || project_source == "." {
            return Ok(true);
        }

        if let Some(globs) = get_package_manager_workspaces(self.workspace_root.to_owned())? {
            in_workspace = GlobSet::new(&globs)?.matches(project_source);
        }

        Ok(in_workspace)
    }

    fn load_project_graph_aliases(
        &mut self,
        projects_map: &ProjectsSourcesMap,
        aliases_map: &mut ProjectsAliasesMap,
    ) -> miette::Result<()> {
        debug!(
            target: LOG_TARGET,
            "Loading names (aliases) from project {}'s",
            color::file(BUNPM.manifest)
        );

        for (project_id, project_source) in projects_map {
            if let Some(package_json) =
                PackageJson::read(project_source.to_path(&self.workspace_root))?
            {
                if let Some(package_name) = package_json.name {
                    let alias = package_name.clone();

                    self.package_names
                        .insert(package_name.clone(), project_id.to_owned());

                    if let Some(existing_source) = projects_map.get(&alias) {
                        if existing_source != project_source {
                            warn!(
                                target: LOG_TARGET,
                                "A project already exists with the ID {} ({}), skipping alias of the same name ({})",
                                color::id(&alias),
                                color::file(existing_source),
                                color::file(project_source)
                            );

                            continue;
                        }
                    }

                    if aliases_map.contains_key(&alias) {
                        // Ignore warning here since the duplicate may have come
                        // from the Node.js platform!
                        continue;
                    }

                    aliases_map.insert(alias, project_id.to_owned());
                }
            }
        }

        Ok(())
    }

    fn load_project_implicit_dependencies(
        &self,
        project_id: &str,
        project_source: &str,
    ) -> miette::Result<Vec<DependencyConfig>> {
        let mut implicit_deps = vec![];

        debug!(
            target: LOG_TARGET,
            "Scanning {} for implicit dependency relations",
            color::id(project_id),
        );

        if let Some(package_json) = PackageJson::read(self.workspace_root.join(project_source))? {
            let mut find_implicit_relations =
                |package_deps: &BTreeMap<String, String>, scope: &DependencyScope| {
                    for dep_name in package_deps.keys() {
                        if let Some(dep_project_id) = self.package_names.get(dep_name) {
                            implicit_deps.push(DependencyConfig {
                                id: dep_project_id.to_owned(),
                                scope: *scope,
                                source: DependencySource::Implicit,
                                via: Some(dep_name.clone()),
                            });
                        }
                    }
                };

            if let Some(dependencies) = &package_json.dependencies {
                find_implicit_relations(dependencies, &DependencyScope::Production);
            }

            if let Some(dev_dependencies) = &package_json.dev_dependencies {
                find_implicit_relations(dev_dependencies, &DependencyScope::Development);
            }

            if let Some(peer_dependencies) = &package_json.peer_dependencies {
                find_implicit_relations(peer_dependencies, &DependencyScope::Peer);
            }
        }

        Ok(implicit_deps)
    }

    // TOOLCHAIN

    fn is_toolchain_enabled(&self) -> miette::Result<bool> {
        Ok(self.config.version.is_some())
    }

    fn get_tool(&self) -> miette::Result<Box<&dyn Tool>> {
        let tool = self.toolchain.get()?;

        Ok(Box::new(tool))
    }

    fn get_tool_for_version(&self, req: RuntimeReq) -> miette::Result<Box<&dyn Tool>> {
        let tool = self.toolchain.get_for_version(&req)?;

        Ok(Box::new(tool))
    }

    fn get_dependency_configs(&self) -> miette::Result<Option<(String, String)>> {
        Ok(Some((BUNPM.lockfile.to_owned(), BUNPM.manifest.to_owned())))
    }

    async fn setup_toolchain(&mut self) -> miette::Result<()> {
        let req = match &self.config.version {
            Some(v) => RuntimeReq::Toolchain(v.to_owned()),
            None => RuntimeReq::Global,
        };

        let mut last_versions = FxHashMap::default();

        if !self.toolchain.has(&req) {
            self.toolchain.register(
                &req,
                BunTool::new(&self.proto_env, &self.config, &req).await?,
            );
        }

        self.toolchain.setup(&req, &mut last_versions).await?;

        Ok(())
    }

    async fn teardown_toolchain(&mut self) -> miette::Result<()> {
        self.toolchain.teardown_all().await?;

        Ok(())
    }

    // ACTIONS

    async fn setup_tool(
        &mut self,
        _context: &ActionContext,
        runtime: &Runtime,
        last_versions: &mut FxHashMap<String, UnresolvedVersionSpec>,
    ) -> miette::Result<u8> {
        let req = &runtime.requirement;

        if !self.toolchain.has(req) {
            self.toolchain
                .register(req, BunTool::new(&self.proto_env, &self.config, req).await?);
        }

        Ok(self.toolchain.setup(req, last_versions).await?)
    }

    async fn install_deps(
        &self,
        _context: &ActionContext,
        runtime: &Runtime,
        working_dir: &Path,
    ) -> miette::Result<()> {
        actions::install_deps(
            self.toolchain.get_for_version(&runtime.requirement)?,
            working_dir,
        )
        .await?;

        Ok(())
    }

    async fn sync_project(
        &self,
        _context: &ActionContext,
        _project: &Project,
        _dependencies: &FxHashMap<Id, Arc<Project>>,
    ) -> miette::Result<bool> {
        Ok(false)
    }

    async fn hash_manifest_deps(
        &self,
        manifest_path: &Path,
        hasher: &mut ContentHasher,
        _hasher_config: &HasherConfig,
    ) -> miette::Result<()> {
        if let Ok(Some(package)) = PackageJson::read(manifest_path) {
            let name = package.name.unwrap_or_else(|| "unknown".into());
            let mut hash = DepsHash::new(name);

            if let Some(peer_deps) = &package.peer_dependencies {
                hash.add_deps(peer_deps);
            }

            if let Some(dev_deps) = &package.dev_dependencies {
                hash.add_deps(dev_deps);
            }

            if let Some(deps) = &package.dependencies {
                hash.add_deps(deps);
            }

            hasher.hash_content(hash)?;
        }

        Ok(())
    }

    async fn hash_run_target(
        &self,
        project: &Project,
        runtime: &Runtime,
        hasher: &mut ContentHasher,
        hasher_config: &HasherConfig,
    ) -> miette::Result<()> {
        let node_hash = actions::create_target_hasher(
            self.toolchain.get_for_version(&runtime.requirement).ok(),
            project,
            &self.workspace_root,
            hasher_config,
        )
        .await?;

        hasher.hash_content(node_hash)?;

        if let Some(typescript_config) = &self.typescript_config {
            let ts_hash = TypeScriptTargetHash::generate(
                typescript_config,
                &self.workspace_root,
                &project.root,
            )?;

            hasher.hash_content(ts_hash)?;
        }
        Ok(())
    }

    async fn create_run_target_command(
        &self,
        _context: &ActionContext,
        project: &Project,
        task: &Task,
        runtime: &Runtime,
        working_dir: &Path,
    ) -> miette::Result<Command> {
        let command = if self.is_toolchain_enabled()? {
            actions::create_target_command(
                self.toolchain.get_for_version(&runtime.requirement)?,
                project,
                task,
                working_dir,
            )?
        } else {
            actions::create_target_command_without_tool(project, task, working_dir)?
        };

        Ok(command)
    }
}