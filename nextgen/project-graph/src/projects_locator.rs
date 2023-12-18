use moon_common::path::{to_virtual_string, WorkspaceRelativePathBuf};
use moon_common::{color, consts, Id};
use moon_vcs::BoxedVcs;
use rustc_hash::FxHashMap;
use starbase_utils::{fs, glob};
use std::path::Path;
use tracing::warn;

/// Infer a project name from a source path, by using the name of
/// the project folder.
pub fn infer_project_id_and_source(
    path: &str,
    workspace_root: &Path,
) -> miette::Result<(Id, WorkspaceRelativePathBuf)> {
    if path.is_empty() {
        return Ok((
            Id::clean(fs::file_name(workspace_root))?,
            WorkspaceRelativePathBuf::from("."),
        ));
    }

    let (id, source) = if path.contains('/') {
        (path.split('/').last().unwrap().to_owned(), path)
    } else {
        (path.to_owned(), path)
    };

    Ok((Id::clean(id)?, WorkspaceRelativePathBuf::from(source)))
}

/// For each pattern in the globs list, glob the file system
/// for potential projects, and infer their name and source.
pub fn locate_projects_with_globs<'glob, I, V>(
    workspace_root: &Path,
    globs: I,
    sources: &mut FxHashMap<Id, WorkspaceRelativePathBuf>,
    vcs: Option<&BoxedVcs>,
) -> miette::Result<()>
where
    I: IntoIterator<Item = &'glob V>,
    V: AsRef<str> + 'glob,
{
    let mut locate_globs = vec![];
    let mut has_root_level = sources.values().any(|source| source == ".");

    // Root-level project has special handling
    for glob in globs.into_iter() {
        let glob = glob.as_ref();

        if glob == "." {
            if has_root_level {
                continue;
            }

            let (id, source) = infer_project_id_and_source("", workspace_root)?;

            has_root_level = true;
            sources.insert(id, source);
        } else {
            locate_globs.push(glob);
        }
    }

    // Glob for all other projects
    let mut potential_projects = glob::walk(workspace_root, locate_globs)?;
    potential_projects.sort();

    for mut project_root in potential_projects {
        // Remove trailing moon.yml
        if project_root.is_file() {
            if project_root.ends_with(consts::CONFIG_PROJECT_FILENAME) {
                project_root = project_root.parent().unwrap().to_owned();

                // Avoid overwriting an existing root project
                if project_root == workspace_root && has_root_level {
                    continue;
                }
            } else {
                // Don't warn on dotfiles
                if project_root
                    .file_name()
                    .map(|name| !name.to_string_lossy().starts_with('.'))
                    .unwrap_or_default()
                {
                    warn!(
                        source = ?project_root,
                        "Received a file path for a project root, must be a directory",
                    );
                }

                continue;
            }
        }

        if project_root.is_dir() {
            let project_source =
                to_virtual_string(project_root.strip_prefix(workspace_root).unwrap())?;

            if project_source == consts::CONFIG_DIRNAME
                || project_source.starts_with(consts::CONFIG_DIRNAME)
            {
                continue;
            }

            if let Some(vcs) = vcs {
                if vcs.is_ignored(&project_root) {
                    warn!(
                        source = project_source,
                        "Found a project with source {}, but this path has been ignored by your VCS, skipping",
                        color::file(&project_source)
                    );

                    continue;
                }
            }

            let (id, source) = infer_project_id_and_source(&project_source, workspace_root)?;

            if let Some(existing_source) = sources.get(&id) {
                warn!(
                    source = project_source,
                    existing_source = existing_source.as_str(),
                    "A project already exists at source {}, skipping conflicting source {}. Try renaming the project folder to make it unique.",
                    color::file(existing_source),
                    color::file(&source)
                );
            } else {
                sources.insert(id, source);
            }
        }
    }

    Ok(())
}
