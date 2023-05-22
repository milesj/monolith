use crate::commands::run::{run_target, RunOptions};
use crate::helpers::AnyError;
use moon::{generate_project_graph, load_workspace};
use moon_common::Id;
use moon_logger::trace;
use moon_project::Project;
use std::env;

pub struct CheckOptions {
    pub all: bool,
    pub concurrency: Option<usize>,
    pub update_cache: bool,
}

const LOG_TARGET: &str = "moon:check";

pub async fn check(project_ids: &[Id], options: CheckOptions) -> Result<(), AnyError> {
    let mut workspace = load_workspace().await?;
    let project_graph = generate_project_graph(&mut workspace).await?;
    let mut projects: Vec<&Project> = vec![];

    // Load projects
    if options.all {
        trace!(target: LOG_TARGET, "Running check on all projects");

        projects.extend(project_graph.get_all()?);
    } else if project_ids.is_empty() {
        trace!(target: LOG_TARGET, "Loading from path");

        projects.push(project_graph.get_from_path(env::current_dir()?)?);
    } else {
        trace!(
            target: LOG_TARGET,
            "Running for specific projects: {}",
            project_ids
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        for id in project_ids {
            projects.push(project_graph.get(id)?);
        }
    };

    // Find all applicable targets
    let mut targets = vec![];

    for project in projects {
        for task in project.tasks.values() {
            if task.is_build_type() || task.is_test_type() {
                targets.push(task.target.id.clone());
            }
        }
    }

    // Run targets using our run command
    run_target(
        &targets,
        RunOptions {
            concurrency: options.concurrency,
            update_cache: options.update_cache,
            ..RunOptions::default()
        },
        workspace,
        project_graph,
    )
    .await?;

    Ok(())
}
