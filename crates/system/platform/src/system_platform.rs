use crate::target_hash::SystemTargetHash;
use crate::tool::SystemToolStub;
use moon_action_context::ActionContext;
use moon_config::{HasherConfig, PlatformType, ProjectConfig};
use moon_hash::ContentHasher;
use moon_platform::{Platform, Runtime, RuntimeReq};
use moon_process::Command;
use moon_project::Project;
use moon_task::Task;
use moon_tool::{get_proto_paths, prepend_path_env_var, Tool};
use moon_utils::async_trait;
use proto_core::ProtoEnvironment;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct SystemPlatform {
    tool: SystemToolStub,

    proto_env: Arc<ProtoEnvironment>,

    _workspace_root: PathBuf,
}

impl SystemPlatform {
    pub fn new(workspace_root: &Path, proto_env: Arc<ProtoEnvironment>) -> Self {
        SystemPlatform {
            tool: SystemToolStub::default(),
            proto_env,
            _workspace_root: workspace_root.to_path_buf(),
        }
    }
}

#[async_trait]
impl Platform for SystemPlatform {
    fn get_type(&self) -> PlatformType {
        PlatformType::System
    }

    fn get_runtime_from_config(&self, _project_config: Option<&ProjectConfig>) -> Runtime {
        Runtime::system()
    }

    fn matches(&self, platform: &PlatformType, runtime: Option<&Runtime>) -> bool {
        if matches!(platform, PlatformType::System) {
            return true;
        }

        if let Some(runtime) = &runtime {
            return matches!(runtime.platform, PlatformType::System);
        }

        false
    }

    // TOOLCHAIN

    fn is_toolchain_enabled(&self) -> miette::Result<bool> {
        Ok(false)
    }

    fn get_tool(&self) -> miette::Result<Box<&dyn Tool>> {
        Ok(Box::new(&self.tool))
    }

    fn get_tool_for_version(&self, _req: RuntimeReq) -> miette::Result<Box<&dyn Tool>> {
        Ok(Box::new(&self.tool))
    }

    // ACTIONS

    async fn hash_run_target(
        &self,
        _project: &Project,
        _runtime: &Runtime,
        hasher: &mut ContentHasher,
        _hasher_config: &HasherConfig,
    ) -> miette::Result<()> {
        hasher.hash_content(SystemTargetHash::new())?;

        Ok(())
    }

    async fn create_run_target_command(
        &self,
        _context: &ActionContext,
        _project: &Project,
        task: &Task,
        runtime: &Runtime,
        working_dir: &Path,
    ) -> miette::Result<Command> {
        let mut command = Command::new(&task.command);

        // cmd/pwsh requires an absolute path to batch files
        if cfg!(windows) {
            use moon_process::shell::is_windows_script;

            for arg in &task.args {
                if is_windows_script(arg) {
                    command.arg(working_dir.join(arg));
                } else {
                    command.arg(arg);
                }
            }
        } else {
            command.args(&task.args);
        }

        command.envs(&task.env);

        if !runtime.requirement.is_global() {
            command.env(
                "PATH",
                prepend_path_env_var(get_proto_paths(&self.proto_env)),
            );
        }

        Ok(command)
    }
}
