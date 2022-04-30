use crate::action::ActionStatus;
use crate::errors::WorkspaceError;
use crate::workspace::Workspace;
use moon_error::map_io_to_fs_error;
use moon_logger::{color, debug, warn};
use moon_utils::{fs, is_offline};
use std::sync::Arc;
use tokio::sync::RwLock;

const TARGET: &str = "moon:action:install-node-deps";

pub async fn install_node_deps(
    workspace: Arc<RwLock<Workspace>>,
) -> Result<ActionStatus, WorkspaceError> {
    let workspace = workspace.write().await; // Mutates package.json
    let toolchain = &workspace.toolchain;
    let manager = toolchain.get_node_package_manager();
    let mut cache = workspace.cache.cache_workspace_state().await?;

    // Update artifacts based on node settings
    let node_config = &workspace.config.node;
    let mut root_package = workspace.load_package_json().await?;

    if node_config.add_engines_constraint && root_package.add_engine("node", &node_config.version) {
        root_package.save().await?;

        debug!(
            target: TARGET,
            "Adding engines version constraint to root {}",
            color::file("package.json")
        );
    }

    if let Some(version_manager) = &node_config.sync_version_manager_config {
        let rc_name = version_manager.get_config_file_name();
        let rc_path = workspace.root.join(&rc_name);

        fs::write(&rc_path, &node_config.version).await?;

        debug!(
            target: TARGET,
            "Syncing Node.js version to root {}",
            color::file(&rc_name)
        );
    }

    // Get the last modified time of the root lockfile
    let lockfile = workspace.root.join(manager.get_lockfile_name());
    let mut last_modified = 0;

    if lockfile.exists() {
        let lockfile_metadata = fs::metadata(&lockfile).await?;

        last_modified = cache.to_millis(
            lockfile_metadata
                .modified()
                .map_err(|e| map_io_to_fs_error(e, lockfile.clone()))?,
        );
    }

    // Install deps if the lockfile has been modified
    // since the last time dependencies were installed!
    if last_modified == 0 || last_modified > cache.item.last_node_install_time {
        debug!(target: TARGET, "Installing Node.js dependencies");

        if is_offline() {
            warn!(
                target: TARGET,
                "No internet connection, assuming offline and skipping install"
            );

            return Ok(ActionStatus::Skipped);
        }

        manager.install_dependencies(toolchain).await?;

        if node_config.dedupe_on_lockfile_change {
            debug!(target: TARGET, "Dedupeing dependencies");

            manager.dedupe_dependencies(toolchain).await?;
        }

        // Update the cache with the timestamp
        cache.item.last_node_install_time = cache.now_millis();
        cache.save().await?;

        return Ok(ActionStatus::Passed);
    }

    debug!(
        target: TARGET,
        "Lockfile has not changed since last install, skipping Node.js dependencies",
    );

    Ok(ActionStatus::Skipped)
}
