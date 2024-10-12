use moon_config::PythonConfig;
use moon_python_lang::pip_requirements::{load_lockfile_dependencies};
use moon_python_lang::{LockfileDependencyVersions};
use moon_console::{Checkpoint, Console};
// use moon_logger::debug;
use moon_utils::get_workspace_root;
use moon_process::Command;
use moon_tool::{
    async_trait, get_proto_paths, load_tool_plugin, prepend_path_env_var, use_global_tool_on_path,
    Tool, get_proto_version_env, get_proto_env_vars, DependencyManager
};
use moon_toolchain::RuntimeReq;
use proto_core::flow::install::InstallOptions;
use proto_core::{Id, ProtoEnvironment, Tool as ProtoTool, UnresolvedVersionSpec};
use rustc_hash::FxHashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::{ffi::OsStr, path::Path};
use starbase_utils::fs;
use tracing::instrument;
use starbase_styles::color;
use moon_logger::{debug, map_list};

const LOG_TARGET: &str = "moon:python-tool";


pub fn get_python_tool_paths(python_tool: &PythonTool) -> Vec<PathBuf> {
    // let mut paths = get_proto_paths(proto_env);
    // let mut paths:Vec<PathBuf> = [];

    let paths = python_tool.tool.get_globals_dirs()
        .iter()
        .cloned()                
        .collect::<Vec<PathBuf>>();

    paths
}

pub fn get_python_env_paths(proto_env: &ProtoEnvironment) -> Vec<PathBuf> {
    let paths = get_proto_paths(proto_env);
    
    paths
}

pub struct PythonTool {
    pub config: PythonConfig,

    pub global: bool,

    pub tool: ProtoTool,

    console: Arc<Console>,

    proto_env: Arc<ProtoEnvironment>,
}

impl PythonTool {
    pub async fn new(
        proto_env: Arc<ProtoEnvironment>,
        console: Arc<Console>,
        config: &PythonConfig,
        req: &RuntimeReq,
    ) -> miette::Result<PythonTool> {
        let mut python = PythonTool {
            config: config.to_owned(),
            global: false,
            tool: load_tool_plugin(
                &Id::raw("python"),
                &proto_env,
                config.plugin.as_ref().unwrap(),
            )
            .await?,
            proto_env,
            console,
        };

        if use_global_tool_on_path("python") || req.is_global() {
            python.global = true;
            python.config.version = None;
        } else {
            python.config.version = req.to_spec();
        };

        Ok(python)
    }

    #[instrument(skip_all)]
    pub async fn exec_python<I, S>(&self, args: I, working_dir: &Path) -> miette::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new("python")
            .args(args)
            .envs(get_proto_env_vars())
            .env(
                "PATH",
                prepend_path_env_var(get_python_env_paths(&self.proto_env)),
            )            
            .cwd(working_dir)
            .with_console(self.console.clone())
            .create_async()
            .exec_stream_output()
            .await?;

        Ok(())


        // let mut cmd = Command::new("python");
        // cmd.with_console(self.console.clone());
        // cmd.envs(get_proto_env_vars());       
        // cmd.envs(get_proto_env_vars());        
        // if !self.global {
        //     cmd.env(
        //         "PATH",
        //         prepend_path_env_var(get_python_env_paths(&self.proto_env)),
        //     );
        // } 
    }

}

#[async_trait]
impl Tool for PythonTool {
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }

    #[instrument(skip_all)]
    async fn setup(
        &mut self,
        last_versions: &mut FxHashMap<String, UnresolvedVersionSpec>,
    ) -> miette::Result<u8> {
        let mut installed = 0;

        let Some(version) = &self.config.version else {
            return Ok(installed);
        };

        if self.global {
            debug!("Using global binary in PATH");
        } else if self.tool.is_setup(version).await? {
            debug!("Python has already been setup");

            // When offline and the tool doesn't exist, fallback to the global binary
        } else if proto_core::is_offline() {
            debug!(
                "No internet connection and Python has not been setup, falling back to global binary in PATH"
            );

            self.global = true;

            // Otherwise try and install the tool
        } else {
            let setup = match last_versions.get("python") {
                Some(last) => version != last,
                None => true,
            };

            if setup || !self.tool.get_product_dir().exists() {
                self.console
                    .out
                    .print_checkpoint(Checkpoint::Setup, format!("installing python {version}"))?;

                if self.tool.setup(version, InstallOptions::default()).await? {
                    last_versions.insert("python".into(), version.to_owned());
                    installed += 1;
                }
            }
        }
        self.tool.locate_globals_dirs().await?;





        Ok(installed)
    }

    async fn teardown(&mut self) -> miette::Result<()> {
        self.tool.teardown().await?;

        Ok(())
    }
}


#[async_trait]
impl DependencyManager<PythonTool> for PythonTool {
    fn create_command(&self, python: &PythonTool) -> miette::Result<Command> {
        let mut cmd = Command::new("python");
        cmd.with_console(self.console.clone());
        cmd.envs(get_proto_env_vars());        
        if !self.global {
            cmd.env(
                "PATH",
                prepend_path_env_var(get_python_env_paths(&self.proto_env)),
            );
        }

        if let Some(version) = get_proto_version_env(&self.tool) {
            cmd.env("PROTO_PYTHON_VERSION", version);
        }

        if let Some(version) = get_proto_version_env(&python.tool) {
            cmd.env("PROTO_PYTHON_VERSION", version);
        }

        Ok(cmd)
    }

    #[instrument(skip_all)]
    async fn dedupe_dependencies(
        &self,
        _python: &PythonTool,
        _working_dir: &Path,
        _log: bool,
    ) -> miette::Result<()> {
        // Not supported!

        Ok(())
    }

    fn get_lock_filename(&self) -> String {
        String::from("requirements.txt")
    }

    fn get_manifest_filename(&self) -> String {
        String::from("requirements.txt")
    }

    #[instrument(skip_all)]
    async fn get_resolved_dependencies(
        &self,
        project_root: &Path,
    ) -> miette::Result<LockfileDependencyVersions> {
        let Some(lockfile_path) =
            fs::find_upwards_until("requirements.txt", project_root, get_workspace_root())
        else {
            return Ok(FxHashMap::default());
        };

        Ok(load_lockfile_dependencies(lockfile_path)?)
    }
    #[instrument(skip_all)]
    async fn install_dependencies(
        &self,
        python: &PythonTool,
        working_dir: &Path,
        log: bool,
    ) -> miette::Result<()> {
        let mut cmd = self.create_command(python)?;
        
        // TODO: DONT KNOW IF CORRECT LOCATION, BECAUSE IT IS HANDLING THE TOOL INSTALLATION
        if let Some(pip_config) = &self.config.pip {

            cmd.args(["install"])
            // .args(&args)
            .cwd(working_dir)
            .set_print_command(log);

            //TODO: only read from root, but ready for sub virtual environments
            if let Some(requirements_path) = fs::find_upwards_until("requirements.txt", get_workspace_root(), get_workspace_root()) {                
                cmd.args(["-r", &requirements_path.as_os_str().to_str().unwrap()]);                
            }
            if let Some(install_args) = &pip_config.install_args {
                cmd.args(install_args);                
            }
            
            let mut cmd = cmd.create_async();
            if env::var("MOON_TEST_HIDE_INSTALL_OUTPUT").is_ok() {
                cmd.exec_capture_output().await?;
            } else {
                cmd.exec_stream_output().await?;
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn install_focused_dependencies(
        &self,
        _python: &PythonTool,
        _packages: &[String],
        _production_only: bool,
    ) -> miette::Result<()> {
        // TODO: Implement for docker purposes
        Ok(())
    }
}
