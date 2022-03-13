use crate::errors::WorkspaceError;
use crate::workspace::Workspace;
use moon_cache::RunTargetState;
use moon_config::TaskType;
use moon_logger::{color, debug};
use moon_project::{Project, Target, Task};
use moon_terminal::output::label_run_target;
use moon_toolchain::tools::node::NodeTool;
use moon_toolchain::{get_path_env_var, Tool};
use moon_utils::process::{create_command, exec_command, output_to_string, spawn_command};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;

const TARGET: &str = "moon:task-runner:run-target";

async fn create_env_vars(
    workspace: &Workspace,
    project: &Project,
    task: &Task,
) -> Result<HashMap<String, String>, WorkspaceError> {
    let map_path_buf = |path: &Path| String::from(path.to_str().unwrap());
    let mut env_vars = HashMap::new();

    env_vars.insert(
        "MOON_CACHE_DIR".to_owned(),
        map_path_buf(&workspace.cache.dir),
    );
    env_vars.insert(
        "MOON_OUT_DIR".to_owned(),
        map_path_buf(&workspace.cache.out),
    );
    env_vars.insert("MOON_PROJECT_ID".to_owned(), project.id.clone());
    env_vars.insert("MOON_PROJECT_ROOT".to_owned(), map_path_buf(&project.root));
    env_vars.insert("MOON_PROJECT_SOURCE".to_owned(), project.source.clone());
    env_vars.insert("MOON_RUN_TARGET".to_owned(), task.target.clone());
    env_vars.insert(
        "MOON_TOOLCHAIN_DIR".to_owned(),
        map_path_buf(&workspace.toolchain.dir),
    );
    env_vars.insert(
        "MOON_WORKSPACE_ROOT".to_owned(),
        map_path_buf(&workspace.root),
    );
    env_vars.insert(
        "MOON_WORKING_DIR".to_owned(),
        map_path_buf(&workspace.working_dir),
    );

    // Store runtime data on the file system so that downstream commands can utilize it
    let runfile = workspace.cache.create_runfile(&project.id, project).await?;

    env_vars.insert(
        "MOON_PROJECT_RUNFILE".to_owned(),
        map_path_buf(&runfile.path),
    );

    Ok(env_vars)
}

fn create_node_options(task: &Task) -> Vec<&str> {
    vec![
        // "--inspect", // Enable node inspector
        "--preserve-symlinks",
        "--title",
        &task.target,
        "--unhandled-rejections",
        "throw",
    ]
}

/// Runs a task command through our toolchain's installed Node.js instance.
/// We accomplish this by executing the Node.js binary as a child process,
/// while passing a file path to a package's node module binary (this is the file
/// being executed). We then also pass arguments defined in the task.
/// This would look something like the following:
///
/// ~/.moon/tools/node/1.2.3/bin/node --inspect /path/to/node_modules/.bin/eslint
///     --cache --color --fix --ext .ts,.tsx,.js,.jsx
#[cfg(not(windows))]
fn create_node_target_command(
    project: &Project,
    task: &Task,
    node: &NodeTool,
) -> Result<Command, WorkspaceError> {
    // Node binary args
    let mut args = create_node_options(task);

    // Package binary args
    let package_bin_path = node.find_package_bin_path(&task.command, &project.root)?;

    args.push(package_bin_path.to_str().unwrap());
    args.extend(task.args.iter().map(|a| a.as_str()));

    // Create the command
    let mut cmd = create_command(node.get_bin_path());

    cmd.args(&args)
        .envs(&task.env)
        .env("PATH", get_path_env_var(node.get_bin_dir()));

    Ok(cmd)
}

/// Windows works quite differently than other systems, so we cannot do the above.
/// On Windows, the package binary is a ".cmd" file, which means it needs to run
/// through "cmd.exe" and not "node.exe". Because of this, the order of operations
/// is switched, and "node.exe" is detected through the `PATH` env var.
#[cfg(windows)]
fn create_node_target_command(
    project: &Project,
    task: &Task,
    node: &NodeTool,
) -> Result<Command, WorkspaceError> {
    let package_bin_path = node.find_package_bin_path(&task.command, &project.root)?;

    // Create the command
    let mut cmd = create_command(package_bin_path);

    cmd.args(&task.args)
        .envs(&task.env)
        .env("PATH", get_path_env_var(node.get_bin_dir()))
        .env("NODE_OPTIONS", create_node_options(task).join(" "));

    Ok(cmd)
}

fn create_shell_target_command(task: &Task) -> Command {
    let mut cmd = create_command(&task.command);
    cmd.args(&task.args);
    cmd
}

async fn create_target_command(
    workspace: &Workspace,
    project: &Project,
    task: &Task,
) -> Result<Command, WorkspaceError> {
    let toolchain = &workspace.toolchain;

    let exec_dir = if task.options.run_from_workspace_root {
        &workspace.root
    } else {
        &project.root
    };

    let env_vars = create_env_vars(workspace, project, task).await?;

    let mut command = match task.type_of {
        TaskType::Node => create_node_target_command(project, task, toolchain.get_node())?,
        _ => create_shell_target_command(task),
    };

    command.current_dir(&exec_dir).envs(env_vars);

    Ok(command)
}

pub async fn run_target(
    workspace: Arc<RwLock<Workspace>>,
    target: &str,
    primary_target: &str,
    passthrough_args: &[String],
) -> Result<(), WorkspaceError> {
    debug!(target: TARGET, "Running target {}", color::id(target));

    let workspace = workspace.read().await;
    let mut cache = workspace.cache.cache_run_target_state(target).await?;

    // TODO abort early for a cache hit

    // Gather the project and task
    let is_primary = primary_target == target;
    let (project_id, task_id) = Target::parse(target)?;
    let project = workspace.projects.load(&project_id)?;
    let task = project.get_task(&task_id)?;

    // Build the command to run based on the task
    let mut command = create_target_command(&workspace, &project, task).await?;

    if is_primary && !passthrough_args.is_empty() {
        command.args(passthrough_args);
    }

    // Run the command as a child process and capture its output.
    // If the process fails and `retry_count` is greater than 0,
    // attempt the process again in case it passes.
    let attempt_count = task.options.retry_count + 1;
    let mut attempt = 1;
    let output;

    loop {
        if attempt == 1 {
            println!("{}", label_run_target(target));
        } else {
            println!(
                "{} {}",
                label_run_target(target),
                color::muted(&format!("(attempt {} of {})", attempt, attempt_count))
            );
        }

        let possible_output = if is_primary {
            // If this target matches the primary target (the last task to run),
            // then we want to stream the output directly to the parent (inherit mode).
            spawn_command(&mut command).await
        } else {
            // Otherwise we run the process in the background and write the output
            // once it has completed.
            exec_command(&mut command).await
        };

        match possible_output {
            Ok(o) => {
                output = o;
                break;
            }
            Err(_) => {
                if attempt >= attempt_count {
                    return Err(WorkspaceError::TaskRunnerFailedTarget(target.to_owned()));
                } else {
                    attempt += 1;

                    debug!(
                        target: TARGET,
                        "Target {} failed, running again with attempt {}",
                        color::target(target),
                        attempt
                    );
                }
            }
        }
    }

    // Hard link outputs to the `.moon/out` folder and to the cloud,
    // so that subsequent builds are faster, and any local outputs
    // can be rehydrated easily.
    for output_path in &task.output_paths {
        workspace
            .cache
            // TODO hash
            .link_task_output_to_out(&project_id, "hash", &project.root, output_path)
            .await?;
    }

    // Update the cache with the result
    cache.item.exit_code = output.status.code().unwrap_or(0);
    cache.item.last_run_time = cache.now_millis();
    cache.item.stderr = output_to_string(&output.stderr);
    cache.item.stdout = output_to_string(&output.stdout);
    cache.save().await?;

    handle_cache_item(&cache.item, !is_primary);

    Ok(())
}

fn handle_cache_item(item: &RunTargetState, log: bool) {
    // Only log when *not* the primary target, or a cache hit
    if log {
        if !item.stderr.is_empty() {
            eprint!("{}", item.stderr);
        }

        if !item.stdout.is_empty() {
            print!("{}", item.stdout);
        }
    }
}
