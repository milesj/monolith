use crate::errors::ActionError;
use moon_platform_system::SystemTargetHasher;
use moon_project::Project;
use moon_task::Task;
use moon_utils::process::Command;
use moon_workspace::Workspace;
use std::path::Path;

#[cfg(not(windows))]
pub fn create_target_command(task: &Task, _cwd: &Path) -> Command {
    let mut cmd = Command::new(&task.command);
    cmd.args(&task.args).envs(&task.env);
    cmd
}

#[cfg(windows)]
pub fn create_target_command(task: &Task, cwd: &Path) -> Command {
    use moon_utils::process::is_windows_script;

    let mut cmd = Command::new(&task.command);

    for arg in &task.args {
        // cmd.exe requires an absolute path to batch files
        if is_windows_script(arg) {
            cmd.arg(cwd.join(arg));
        } else {
            cmd.arg(arg);
        }
    }

    cmd.envs(&task.env);
    cmd
}

pub fn create_target_hasher(
    _workspace: &Workspace,
    _project: &Project,
) -> Result<SystemTargetHasher, ActionError> {
    Ok(SystemTargetHasher::new())
}
