use crate::app::{
    Commands, DockerCommands, MigrateCommands, NodeCommands, QueryCommands, SyncCommands,
};
use crate::commands::bin::bin;
use crate::commands::check::check;
use crate::commands::ci::ci;
use crate::commands::clean::clean;
use crate::commands::completions;
use crate::commands::docker;
use crate::commands::generate::generate;
use crate::commands::graph::{dep::dep_graph, project::project_graph};
use crate::commands::init::init;
use crate::commands::migrate;
use crate::commands::node;
use crate::commands::project::project;
use crate::commands::query;
use crate::commands::run::run;
use crate::commands::setup::setup;
use crate::commands::sync::sync;
use crate::commands::syncs;
use crate::commands::task::task;
use crate::commands::teardown::teardown;
use crate::commands::upgrade::upgrade;
use crate::states::{CurrentCommand, WorkspaceInstance};
use moon_api::Launchpad;
use moon_common::{color, is_test_env, is_unformatted_stdout};
use moon_terminal::{get_checkpoint_prefix, Checkpoint};
use starbase::system;
use tracing::debug;

#[system]
pub async fn load_workspace(resources: ResourcesMut, global_args: StateRef<CurrentCommand>) {
    let workspace = match &global_args.command {
        // Must not load the workspace!
        Commands::Init(_) | Commands::Completions { .. } | Commands::Setup | Commands::Upgrade => {
            return Ok(());
        }

        // Requires the toolchain
        Commands::Bin { .. }
        | Commands::Docker {
            command: DockerCommands::Prune | DockerCommands::Setup,
        }
        | Commands::Node {
            command: NodeCommands::RunScript(_),
        }
        | Commands::Teardown => moon::load_workspace_with_toolchain().await?,

        // Does not require the toolchain
        _ => moon::load_workspace().await?,
    };

    resources.set(WorkspaceInstance(workspace));
}

#[system]
pub async fn check_for_new_version(
    global_args: StateRef<CurrentCommand>,
    workspace: ResourceRef<WorkspaceInstance>,
) {
    if is_test_env() || !is_unformatted_stdout() || !moon::is_telemetry_enabled() {
        return Ok(());
    }

    if matches!(
        &global_args.command,
        Commands::Check { .. } | Commands::Ci { .. } | Commands::Run { .. } | Commands::Sync { .. }
    ) {
        let current_version = env!("CARGO_PKG_VERSION");
        let prefix = get_checkpoint_prefix(Checkpoint::Announcement);

        match Launchpad::check_version(&workspace.0.cache_engine, current_version, false).await {
            Ok(Some(latest)) => {
                println!(
                    "{} There's a new version of moon available, {} (currently on {})!",
                    prefix,
                    color::success(latest.current_version),
                    current_version,
                );

                if let Some(newer_message) = latest.message {
                    println!("{} {}", prefix, newer_message);
                }

                println!(
                    "{} Run {} or install from {}",
                    prefix,
                    color::success("moon upgrade"),
                    color::url("https://moonrepo.dev/docs/install"),
                );
            }
            Err(error) => {
                debug!("Failed to check for current version: {}", error);
            }
            _ => {}
        }
    }
}

#[system(instrument = false)]
pub async fn run_command(
    global_args: StateRef<CurrentCommand>,
    workspace: ResourceRef<WorkspaceInstance>,
) {
    // Take ownership so that we can freely pass it to commands
    let workspace = workspace.0.to_owned();

    let result = match global_args.command.clone() {
        Commands::Bin { tool } => bin(tool).await,
        Commands::Ci(args) => ci(args, global_args.concurrency, workspace).await,
        Commands::Check(args) => check(args, global_args.concurrency, workspace).await,
        Commands::Clean(args) => clean(args, workspace).await,
        Commands::Completions { shell } => completions::completions(shell).await,
        Commands::DepGraph(args) => dep_graph(args, workspace).await,
        Commands::Docker { command } => match command {
            DockerCommands::Prune => docker::prune(workspace).await,
            DockerCommands::Scaffold(args) => docker::scaffold(args, workspace).await,
            DockerCommands::Setup => docker::setup(workspace).await,
        },
        Commands::Generate(args) => generate(args, workspace).await,
        Commands::Init(args) => init(args).await,
        Commands::Migrate {
            command,
            skip_touched_files_check,
        } => match command {
            MigrateCommands::FromPackageJson(args) => {
                migrate::from_package_json(args, skip_touched_files_check, workspace).await
            }
            MigrateCommands::FromTurborepo => {
                migrate::from_turborepo(skip_touched_files_check, workspace).await
            }
        },
        Commands::Node { command } => match command {
            NodeCommands::RunScript(args) => node::run_script(args, workspace).await,
        },
        Commands::Project(args) => project(args, workspace).await,
        Commands::ProjectGraph(args) => project_graph(args, workspace).await,
        Commands::Query { command } => match command {
            QueryCommands::Hash(args) => query::hash(args, workspace).await,
            QueryCommands::HashDiff(args) => query::hash_diff(args, workspace).await,
            QueryCommands::Projects(args) => query::projects(args, workspace).await,
            QueryCommands::Tasks(args) => query::tasks(args, workspace).await,
            QueryCommands::TouchedFiles(args) => query::touched_files(args, workspace).await,
        },
        Commands::Run(args) => run(args, global_args.concurrency, workspace).await,
        Commands::Setup => setup().await,
        Commands::Sync { command } => match command {
            Some(SyncCommands::Codeowners(args)) => syncs::codeowners::sync(args, workspace).await,
            Some(SyncCommands::Hooks(args)) => syncs::hooks::sync(args, workspace).await,
            Some(SyncCommands::Projects) => syncs::projects::sync(workspace).await,
            None => sync(workspace).await,
        },
        Commands::Task(args) => task(args, workspace).await,
        Commands::Teardown => teardown().await,
        Commands::Upgrade => upgrade().await,
    };

    if let Err(error) = result {
        // Rust crashes with a broken pipe error by default,
        // so we unfortunately need to work around it with this hack!
        // https://github.com/rust-lang/rust/issues/46016
        if error.to_string().to_lowercase().contains("broken pipe") {
            std::process::exit(0);
        } else {
            return Err(error);
        }
    }
}