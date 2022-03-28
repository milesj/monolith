use crate::{Workspace, WorkspaceError};
use moon_cache::Hasher;
use moon_project::{ExpandedFiles, Project, Task};
use moon_utils::fs;
use std::path::Path;

fn convert_paths_to_strings(
    paths: &ExpandedFiles,
    workspace_root: &Path,
) -> Result<Vec<String>, WorkspaceError> {
    let mut files: Vec<String> = vec![];

    for path in paths {
        // Inputs may not exist and `git hash-object` will fail if you pass an unknown file
        if path.exists() {
            // We also need to use relative paths from the workspace root,
            // so that it works across machines
            let rel_path = if path.starts_with(workspace_root) {
                path.strip_prefix(workspace_root).unwrap()
            } else {
                path
            };

            files.push(fs::path_to_string(rel_path)?);
        }
    }

    Ok(files)
}

pub async fn create_target_hasher(
    workspace: &Workspace,
    project: &Project,
    task: &Task,
) -> Result<Hasher, WorkspaceError> {
    let vcs = workspace.detect_vcs();
    let mut hasher = Hasher::new(workspace.config.node.version.clone());

    hasher.hash_project(project);
    hasher.hash_task(task);

    // Hash root configs first
    hasher.hash_package_json(&workspace.load_package_json().await?);

    if let Some(root_tsconfig) = workspace
        .load_tsconfig_json(&workspace.config.typescript.root_config_file_name)
        .await?
    {
        hasher.hash_tsconfig_json(&root_tsconfig);
    }

    // Hash project configs second so they can override
    if let Some(package) = project.load_package_json().await? {
        hasher.hash_package_json(&package);
    }

    if let Some(tsconfig) = project
        .load_tsconfig_json(&workspace.config.typescript.project_config_file_name)
        .await?
    {
        hasher.hash_tsconfig_json(&tsconfig);
    }

    // For input files, hash them with the vcs layer first
    if !task.input_paths.is_empty() {
        let files = convert_paths_to_strings(&task.input_paths, &workspace.root)?;
        let hashed_files = vcs.get_file_hashes(&files).await?;

        hasher.hash_inputs(hashed_files);
    }

    // TODO input globs

    Ok(hasher)
}
