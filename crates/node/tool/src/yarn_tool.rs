use crate::get_node_env_paths;
use crate::node_tool::NodeTool;
use moon_config::YarnConfig;
use moon_logger::debug;
use moon_node_lang::{yarn, LockfileDependencyVersions, YARN};
use moon_process::Command;
use moon_terminal::{print_checkpoint, Checkpoint};
use moon_tool::{
    async_trait, load_tool_plugin, prepend_path_env_var, use_global_tool_on_path,
    DependencyManager, Tool, ToolError,
};
use moon_utils::{get_workspace_root, is_ci};
use proto_core::{Id, ProtoEnvironment, Tool as ProtoTool, UnresolvedVersionSpec};
use rustc_hash::FxHashMap;
use starbase_styles::color;
use starbase_utils::fs;
use std::env;
use std::path::Path;
use std::sync::Arc;

pub struct YarnTool {
    pub config: YarnConfig,

    pub global: bool,

    pub tool: ProtoTool,

    proto_env: Arc<ProtoEnvironment>,
}

impl YarnTool {
    pub async fn new(
        proto_env: Arc<ProtoEnvironment>,
        config: &Option<YarnConfig>,
    ) -> miette::Result<YarnTool> {
        let config = config.to_owned().unwrap_or_default();

        Ok(YarnTool {
            global: use_global_tool_on_path() || config.version.is_none(),
            tool: load_tool_plugin(
                &Id::raw("yarn"),
                &proto_env,
                config.plugin.as_ref().unwrap(),
            )
            .await?,
            config,
            proto_env,
        })
    }

    pub fn is_berry(&self) -> bool {
        self.check_version(2)
    }

    pub fn is_berry_v4(&self) -> bool {
        self.check_version(4)
    }

    pub async fn set_version(&mut self, node: &NodeTool) -> miette::Result<()> {
        if !self.is_berry() {
            return Ok(());
        }

        let Some(version) = &self.config.version else {
            return Ok(());
        };

        let yarn_bin = get_workspace_root()
            .join(".yarn/releases")
            .join(format!("yarn-{version}.cjs"));

        if !yarn_bin.exists() {
            let version_string = version.to_string();

            debug!(
                "Updating yarn version with {}",
                color::shell(format!("yarn set version {version_string}"))
            );

            self.create_command(node)?
                .args(["set", "version", &version_string])
                .create_async()
                .exec_capture_output()
                .await?;

            for plugin in &self.config.plugins {
                self.create_command(node)?
                    .args(["plugin", "import", plugin])
                    .create_async()
                    .exec_capture_output()
                    .await?;
            }
        }

        Ok(())
    }

    fn check_version(&self, min_major: u64) -> bool {
        self.config
            .version
            .as_ref()
            .map(|v| match v {
                UnresolvedVersionSpec::Canary => false,
                UnresolvedVersionSpec::Alias(alias) => alias == "berry",
                UnresolvedVersionSpec::Req(req) => {
                    req.comparators.iter().any(|c| c.major >= min_major)
                }
                UnresolvedVersionSpec::Version(version) => version.major >= min_major,
                _ => false,
            })
            .unwrap_or(false)
    }
}

#[async_trait]
impl Tool for YarnTool {
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }

    async fn setup(
        &mut self,
        last_versions: &mut FxHashMap<String, UnresolvedVersionSpec>,
    ) -> miette::Result<u8> {
        let mut count = 0;
        let version = self.config.version.as_ref();

        let Some(version) = version else {
            return Ok(count);
        };

        if self.global {
            debug!("Using global binary in PATH");

            return Ok(count);
        }

        if self.tool.is_setup(version).await? {
            self.tool.locate_globals_dir().await?;

            debug!("yarn has already been setup");

            return Ok(count);
        }

        // When offline and the tool doesn't exist, fallback to the global binary
        if proto_core::is_offline() {
            debug!(
                "No internet connection and yarn has not been setup, falling back to global binary in PATH"
            );

            self.global = true;

            return Ok(count);
        }

        if let Some(last) = last_versions.get("yarn") {
            if last == version && self.tool.get_tool_dir().exists() {
                return Ok(count);
            }
        }

        print_checkpoint(format!("installing yarn {version}"), Checkpoint::Setup);

        if self.tool.setup(version, false).await? {
            last_versions.insert("yarn".into(), version.to_owned());
            count += 1;
        }

        self.tool.locate_globals_dir().await?;

        Ok(count)
    }

    async fn teardown(&mut self) -> miette::Result<()> {
        self.tool.teardown().await?;

        Ok(())
    }
}

#[async_trait]
impl DependencyManager<NodeTool> for YarnTool {
    fn create_command(&self, node: &NodeTool) -> miette::Result<Command> {
        let mut cmd = Command::new("yarn");

        cmd.env(
            "PATH",
            prepend_path_env_var(get_node_env_paths(&self.proto_env)),
        );

        if !self.global {
            cmd.env(
                "PROTO_YARN_VERSION",
                self.tool.get_resolved_version().to_string(),
            );
            cmd.env(
                "PROTO_NODE_VERSION",
                node.tool.get_resolved_version().to_string(),
            );
        }

        Ok(cmd)
    }

    async fn dedupe_dependencies(
        &self,
        node: &NodeTool,
        working_dir: &Path,
        log: bool,
    ) -> miette::Result<()> {
        if self.config.version.is_none() {
            return Ok(());
        }

        // Yarn v1 doesnt dedupe natively, so use:
        // npx yarn-deduplicate yarn.lock
        if self.is_berry() {
            self.create_command(node)?
                .arg("dedupe")
                .cwd(working_dir)
                .set_print_command(log)
                .create_async()
                .exec_capture_output()
                .await?;
        } else {
            // Will error if the lockfile does not exist!
            if working_dir.join(self.get_lock_filename()).exists() {
                node.exec_package(
                    "yarn-deduplicate",
                    &["yarn-deduplicate", YARN.lockfile],
                    working_dir,
                )
                .await?;
            }
        }

        Ok(())
    }

    fn get_lock_filename(&self) -> String {
        String::from(YARN.lockfile)
    }

    fn get_manifest_filename(&self) -> String {
        String::from(YARN.manifest)
    }

    async fn get_resolved_dependencies(
        &self,
        project_root: &Path,
    ) -> miette::Result<LockfileDependencyVersions> {
        let Some(lockfile_path) =
            fs::find_upwards_until(YARN.lockfile, project_root, get_workspace_root())
        else {
            return Ok(FxHashMap::default());
        };

        Ok(yarn::load_lockfile_dependencies(lockfile_path)?)
    }

    async fn install_dependencies(
        &self,
        node: &NodeTool,
        working_dir: &Path,
        log: bool,
    ) -> miette::Result<()> {
        let mut args = vec!["install"];

        if !self.global {
            if !self.is_berry() {
                args.push("--ignore-engines");
            }

            if is_ci() {
                if self.is_berry() {
                    args.push("--immutable");
                } else {
                    args.push("--check-files");
                    args.push("--frozen-lockfile");
                    args.push("--non-interactive");
                }
            }
        }

        let mut cmd = self.create_command(node)?;

        cmd.args(args).cwd(working_dir).set_print_command(log);

        let mut cmd = cmd.create_async();

        if env::var("MOON_TEST_HIDE_INSTALL_OUTPUT").is_ok() {
            cmd.exec_capture_output().await?;
        } else {
            cmd.exec_stream_output().await?;
        }

        Ok(())
    }

    async fn install_focused_dependencies(
        &self,
        node: &NodeTool,
        packages: &[String],
        production_only: bool,
    ) -> miette::Result<()> {
        let mut cmd = self.create_command(node)?;

        if self.is_berry() {
            cmd.args(["workspaces", "focus"]);
            cmd.args(packages);

            if !self.is_berry_v4() {
                let workspace_plugin =
                    get_workspace_root().join(".yarn/plugins/@yarnpkg/plugin-workspace-tools.cjs");

                if !workspace_plugin.exists() {
                    return Err(ToolError::RequiresPlugin(
                        "yarn plugin import workspace-tools".into(),
                    )
                    .into());
                }
            }
        } else {
            cmd.arg("install");
        };

        if production_only {
            cmd.arg("--production");
        }

        cmd.create_async().exec_stream_output().await?;

        Ok(())
    }
}
