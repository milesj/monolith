use crate::target_hasher::DenoTargetHasher;
use crate::{actions, bins_hasher::DenoBinsHasher};
use moon_action_context::ActionContext;
use moon_common::{color, is_ci, Id};
use moon_config::{
    BinEntry, DenoConfig, DependencyConfig, HasherConfig, HasherOptimization, PlatformType,
    ProjectConfig, TypeScriptConfig,
};
use moon_deno_lang::{load_lockfile_dependencies, DenoJson, DENO_DEPS};
use moon_deno_tool::DenoTool;
use moon_hasher::{DepsHasher, HashSet};
use moon_logger::{debug, map_list};
use moon_platform::{Platform, Runtime, Version};
use moon_process::Command;
use moon_project::Project;
use moon_task::Task;
use moon_terminal::{print_checkpoint, Checkpoint};
use moon_tool::{Tool, ToolManager};
use moon_typescript_platform::TypeScriptTargetHasher;
use moon_utils::async_trait;
use proto::{get_sha256_hash_of_file, Proto};
use rustc_hash::FxHashMap;
use std::sync::Arc;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

const LOG_TARGET: &str = "moon:deno-platform";

#[derive(Debug)]
pub struct DenoPlatform {
    config: DenoConfig,

    toolchain: ToolManager<DenoTool>,

    typescript_config: Option<TypeScriptConfig>,

    workspace_root: PathBuf,
}

impl DenoPlatform {
    pub fn new(
        config: &DenoConfig,
        typescript_config: &Option<TypeScriptConfig>,
        workspace_root: &Path,
    ) -> Self {
        DenoPlatform {
            config: config.to_owned(),
            toolchain: ToolManager::new(Runtime::Deno(Version::new_global())),
            typescript_config: typescript_config.to_owned(),
            workspace_root: workspace_root.to_path_buf(),
        }
    }
}

#[async_trait]
impl Platform for DenoPlatform {
    fn get_type(&self) -> PlatformType {
        PlatformType::Deno
    }

    fn get_runtime_from_config(&self, _project_config: Option<&ProjectConfig>) -> Runtime {
        Runtime::Deno(Version::new_global())
    }

    fn matches(&self, platform: &PlatformType, runtime: Option<&Runtime>) -> bool {
        if matches!(platform, PlatformType::Deno) {
            return true;
        }

        if let Some(runtime) = &runtime {
            return matches!(runtime, Runtime::Deno(_));
        }

        false
    }

    // PROJECT GRAPH

    fn load_project_implicit_dependencies(
        &self,
        _project_id: &str,
        _project_source: &str,
    ) -> miette::Result<Vec<DependencyConfig>> {
        let implicit_deps = vec![];

        Ok(implicit_deps)
    }

    // TOOLCHAIN

    fn is_toolchain_enabled(&self) -> miette::Result<bool> {
        Ok(false)
    }

    fn get_tool(&self) -> miette::Result<Box<&dyn Tool>> {
        let tool = self.toolchain.get()?;

        Ok(Box::new(tool))
    }

    fn get_tool_for_version(&self, version: Version) -> miette::Result<Box<&dyn Tool>> {
        let tool = self.toolchain.get_for_version(&version)?;

        Ok(Box::new(tool))
    }

    fn get_dependency_configs(&self) -> miette::Result<Option<(String, String)>> {
        Ok(Some((
            DENO_DEPS.lockfile.to_owned(),
            self.config.deps_file.to_owned(),
        )))
    }

    async fn setup_toolchain(&mut self) -> miette::Result<()> {
        // let version = match &self.config.version {
        //     Some(v) => Version::new(v),
        //     None => Version::new_global(),
        // };

        let version = Version::new_global();
        let mut last_versions = FxHashMap::default();

        if !self.toolchain.has(&version) {
            self.toolchain.register(
                &version,
                DenoTool::new(&Proto::new()?, &self.config, &version)?,
            );
        }

        self.toolchain.setup(&version, &mut last_versions).await?;

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
        last_versions: &mut FxHashMap<String, String>,
    ) -> miette::Result<u8> {
        let version = runtime.version();

        if !self.toolchain.has(&version) {
            self.toolchain.register(
                &version,
                DenoTool::new(&Proto::new()?, &self.config, &version)?,
            );
        }

        Ok(self.toolchain.setup(&version, last_versions).await?)
    }

    async fn install_deps(
        &self,
        _context: &ActionContext,
        runtime: &Runtime,
        working_dir: &Path,
    ) -> miette::Result<()> {
        if !self.config.lockfile {
            return Ok(());
        }

        let tool = self.toolchain.get_for_version(runtime.version())?;

        debug!(target: LOG_TARGET, "Installing dependencies");

        print_checkpoint("deno cache", Checkpoint::Setup);

        Command::new(tool.get_bin_path()?)
            .args([
                "cache",
                "--lock",
                DENO_DEPS.lockfile,
                "--lock-write",
                &self.config.deps_file,
            ])
            .cwd(working_dir)
            .create_async()
            .exec_stream_output()
            .await?;

        // Then attempt to install binaries
        if !self.config.bins.is_empty() {
            print_checkpoint("deno install", Checkpoint::Setup);

            debug!(
                target: LOG_TARGET,
                "Installing Deno binaries: {}",
                map_list(&self.config.bins, |b| color::label(b.get_name()))
            );

            for bin in &self.config.bins {
                let mut args = vec![
                    "install",
                    "--allow-net",
                    "--allow-read",
                    "--no-prompt",
                    "--lock",
                    DENO_DEPS.lockfile,
                ];

                match bin {
                    BinEntry::Name(name) => args.push(name),
                    BinEntry::Config(cfg) => {
                        if cfg.local && is_ci() {
                            continue;
                        }

                        if cfg.force {
                            args.push("--force");
                        }

                        if let Some(name) = &cfg.name {
                            args.push("--name");
                            args.push(name);
                        }

                        args.push(&cfg.bin);
                    }
                };

                Command::new(tool.get_bin_path()?)
                    .args(args)
                    .cwd(working_dir)
                    .create_async()
                    .exec_stream_output()
                    .await?;
            }
        }

        Ok(())
    }

    async fn sync_project(
        &self,
        _context: &ActionContext,
        project: &Project,
        dependencies: &FxHashMap<Id, Arc<Project>>,
    ) -> miette::Result<bool> {
        let modified = actions::sync_project(
            project,
            dependencies,
            &self.workspace_root,
            &self.config,
            &self.typescript_config,
        )
        .await?;

        Ok(modified)
    }

    async fn hash_manifest_deps(
        &self,
        manifest_path: &Path,
        hashset: &mut HashSet,
        _hasher_config: &HasherConfig,
    ) -> miette::Result<()> {
        if !self.config.bins.is_empty() {
            hashset.hash(DenoBinsHasher {
                bins: self.config.bins.clone(),
            });
        }

        let mut hasher = DepsHasher::new("deno".into());
        let project_root = manifest_path.parent().unwrap();

        if let Ok(Some(deno_json)) = DenoJson::read(manifest_path) {
            if let Some(imports) = &deno_json.imports {
                hasher.hash_deps(imports);
            }

            if let Some(import_map_path) = &deno_json.import_map {
                if let Ok(Some(import_map)) = DenoJson::read(project_root.join(import_map_path)) {
                    if let Some(imports) = &import_map.imports {
                        hasher.hash_deps(imports);
                    }
                }
            }

            if let Some(scopes) = &deno_json.scopes {
                hasher.hash_aliases(scopes);
            }
        }

        // We can't parse TS files right now, so hash the file contents
        let deps_path = project_root.join(&self.config.deps_file);

        if deps_path.exists() {
            hasher.hash_dep(&self.config.deps_file, get_sha256_hash_of_file(deps_path)?);
        }

        hashset.hash(hasher);

        Ok(())
    }

    async fn hash_run_target(
        &self,
        project: &Project,
        _runtime: &Runtime,
        hashset: &mut HashSet,
        hasher_config: &HasherConfig,
    ) -> miette::Result<()> {
        let mut deno_hasher = DenoTargetHasher::new(None);

        if matches!(hasher_config.optimization, HasherOptimization::Accuracy)
            && self.config.lockfile
        {
            let resolved_dependencies =
                load_lockfile_dependencies(project.root.join(DENO_DEPS.lockfile))?;

            deno_hasher.hash_deps(BTreeMap::from_iter(resolved_dependencies));
        };

        hashset.hash(deno_hasher);

        if let Ok(Some(deno_json)) = DenoJson::read(&project.root) {
            if let Some(compiler_options) = &deno_json.compiler_options {
                let mut ts_hasher = TypeScriptTargetHasher::default();
                ts_hasher.hash_compiler_options(compiler_options);

                hashset.hash(ts_hasher);
            }
        }

        // Do we need this if we're using compiler options from deno.json?
        if let Some(typescript_config) = &self.typescript_config {
            let ts_hasher = TypeScriptTargetHasher::generate(
                typescript_config,
                &self.workspace_root,
                &project.root,
            )?;

            hashset.hash(ts_hasher);
        }

        Ok(())
    }

    async fn create_run_target_command(
        &self,
        _context: &ActionContext,
        _project: &Project,
        task: &Task,
        _runtime: &Runtime,
        working_dir: &Path,
    ) -> miette::Result<Command> {
        let mut command = Command::new(&task.command);

        command.args(&task.args).envs(&task.env).cwd(working_dir);

        Ok(command)
    }
}
