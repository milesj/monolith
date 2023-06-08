use crate::errors::VcsError;
use crate::git::Git;
use crate::svn::Svn;
use crate::BoxedVcs;
use moon_config::{VcsManager, WorkspaceConfig};
use std::path::Path;

pub struct VcsLoader {}

impl VcsLoader {
    pub fn load(
        workspace_root: &Path,
        workspace_config: &WorkspaceConfig,
    ) -> Result<BoxedVcs, VcsError> {
        let vcs_config = &workspace_config.vcs;

        Ok(match vcs_config.manager {
            VcsManager::Svn => Box::new(Svn::load(vcs_config, workspace_root)),
            _ => Box::new(Git::load(vcs_config, workspace_root)?),
        })
    }
}
