mod config;
mod constants;
mod errors;

use config::workspace::WorkspaceConfig;
use std::env;
use std::path::PathBuf;

/// Recursively attempt to find the workspace root by locating the ".monolith"
/// configuration folder, starting from the current working directory.
fn find_workspace_root(current_dir: PathBuf) -> Option<PathBuf> {
    let mut config_dir = current_dir.clone();
    config_dir.push(constants::CONFIG_DIRNAME);

    if config_dir.exists() {
        return Some(config_dir);
    }

    let parent_dir = current_dir.parent();

    match parent_dir {
        Some(dir) => find_workspace_root(PathBuf::from(dir)),
        None => None,
    }
}

pub struct Workspace {
    pub config: WorkspaceConfig,

    /// The root of the workspace that contains the ".monolith" config folder.
    pub root_dir: PathBuf,

    /// The current working directory.
    pub working_dir: PathBuf,
}

impl Workspace {
    /// Create a new workspace instance starting from the current working directory.
    /// Will locate the workspace root and load available configuration files.
    pub fn new() -> Result<Workspace, errors::WorkspaceError> {
        let working_dir = env::current_dir().unwrap();

        // Find root dir
        let root_dir = find_workspace_root(working_dir.clone()).ok_or(
            errors::WorkspaceError::MissingConfigDir(String::from(constants::CONFIG_DIRNAME)),
        )?;

        // Load "workspace.yml"
        let config_path = root_dir.clone().join(constants::CONFIG_WORKSPACE_FILENAME);
        let config = WorkspaceConfig::load(config_path)?;

        Ok(Workspace {
            config,
            root_dir,
            working_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
