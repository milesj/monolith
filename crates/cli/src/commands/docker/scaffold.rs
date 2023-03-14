use super::MANIFEST_NAME;
use crate::helpers::AnyError;
use moon::{generate_project_graph, load_workspace};
use moon_config::{ProjectID, ProjectLanguage};
use moon_constants::CONFIG_DIRNAME;
use moon_error::MoonError;
use moon_platform_detector::detect_language_files;
use moon_project_graph::{ProjectGraph, ProjectGraphError};
use moon_utils::{fs, glob, json, path};
use moon_workspace::Workspace;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::path::Path;
use strum::IntoEnumIterator;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerManifest {
    pub focused_projects: FxHashSet<ProjectID>,
    pub unfocused_projects: FxHashSet<ProjectID>,
}

fn copy_files<T: AsRef<str>>(list: &[T], source: &Path, dest: &Path) -> Result<(), MoonError> {
    for file in list {
        let file = file.as_ref();
        let source_file = source.join(file);

        if source_file.exists() {
            if source_file.is_dir() {
                fs::copy_dir_all(&source_file, &source_file, &dest.join(file))?;
            } else {
                fs::copy_file(source_file, dest.join(file))?;
            }
        }
    }

    Ok(())
}

fn scaffold_workspace(
    workspace: &Workspace,
    project_graph: &ProjectGraph,
    docker_root: &Path,
) -> Result<(), ProjectGraphError> {
    let docker_workspace_root = docker_root.join("workspace");

    fs::create_dir_all(&docker_workspace_root)?;

    // Copy manifest and config files for every type of language,
    // not just the one the project is configured as!
    let mut files: Vec<String> = vec![];

    for lang in ProjectLanguage::iter() {
        files.extend(detect_language_files(&lang));

        // This is a special case as TS file names are configured
        if matches!(lang, ProjectLanguage::TypeScript) {
            if let Some(typescript_config) = &workspace.toolchain_config.typescript {
                files.push(typescript_config.project_config_file_name.to_owned());
                files.push(typescript_config.root_config_file_name.to_owned());
                files.push(typescript_config.root_options_config_file_name.to_owned());
            }
        }
    }

    // Copy each project and mimic the folder structure
    for project_source in project_graph.sources.values() {
        let docker_project_root = docker_workspace_root.join(project_source);

        fs::create_dir_all(&docker_project_root)?;

        copy_files(
            &files,
            &workspace.root.join(project_source),
            &docker_project_root,
        )?;
    }

    // Copy root lockfiles and configurations
    copy_files(&files, &workspace.root, &docker_workspace_root)?;

    // Copy moon configuration
    let moon_configs = glob::walk(workspace.root.join(CONFIG_DIRNAME), ["*.yml"])?;
    let moon_configs = moon_configs
        .iter()
        .map(|f| path::to_string(f.strip_prefix(&workspace.root).unwrap()))
        .collect::<Result<Vec<String>, MoonError>>()?;

    copy_files(&moon_configs, &workspace.root, &docker_workspace_root)?;

    Ok(())
}

fn scaffold_sources_project(
    workspace: &Workspace,
    project_graph: &ProjectGraph,
    docker_sources_root: &Path,
    project_id: &str,
    manifest: &mut DockerManifest,
) -> Result<(), ProjectGraphError> {
    let project = project_graph.get(project_id)?;

    manifest.focused_projects.insert(project_id.to_owned());

    copy_files(&[&project.source], &workspace.root, docker_sources_root)?;

    for dep_id in project.get_dependency_ids() {
        scaffold_sources_project(
            workspace,
            project_graph,
            docker_sources_root,
            dep_id,
            manifest,
        )?;
    }

    Ok(())
}

fn scaffold_sources(
    workspace: &Workspace,
    project_graph: &ProjectGraph,
    docker_root: &Path,
    project_ids: &[String],
    include: &[String],
) -> Result<(), ProjectGraphError> {
    let docker_sources_root = docker_root.join("sources");
    let mut manifest = DockerManifest {
        focused_projects: FxHashSet::default(),
        unfocused_projects: FxHashSet::default(),
    };

    // Copy all projects
    for project_id in project_ids {
        scaffold_sources_project(
            workspace,
            project_graph,
            &docker_sources_root,
            project_id,
            &mut manifest,
        )?;
    }

    // Include non-focused projects in the manifest
    for project_id in project_graph.ids() {
        if !manifest.focused_projects.contains(&project_id) {
            manifest.unfocused_projects.insert(project_id);
        }
    }

    // Include via globs
    if !include.is_empty() {
        let files = glob::walk_files(&workspace.root, include)?;
        let files = files
            .iter()
            .map(|f| path::to_string(f.strip_prefix(&workspace.root).unwrap()))
            .collect::<Result<Vec<String>, MoonError>>()?;

        copy_files(&files, &workspace.root, &docker_sources_root)?;
    }

    json::write(docker_sources_root.join(MANIFEST_NAME), &manifest, true)?;

    // Sync to the workspace scaffold for staged builds
    json::write(
        docker_root.join("workspace").join(MANIFEST_NAME),
        &manifest,
        true,
    )?;

    Ok(())
}

pub async fn scaffold(project_ids: &[String], include: &[String]) -> Result<(), AnyError> {
    let mut workspace = load_workspace().await?;
    let docker_root = workspace.root.join(CONFIG_DIRNAME).join("docker");

    // Delete the docker skeleton to remove any stale files
    fs::remove_dir_all(&docker_root)?;
    fs::create_dir_all(&docker_root)?;

    // Create the workspace skeleton
    let project_graph = generate_project_graph(&mut workspace).await?;

    scaffold_workspace(&workspace, &project_graph, &docker_root)?;

    scaffold_sources(
        &workspace,
        &project_graph,
        &docker_root,
        project_ids,
        include,
    )?;

    Ok(())
}
