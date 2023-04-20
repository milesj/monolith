use mimalloc::MiMalloc;
use moon_cli::{run_cli, BIN_NAME};
use moon_constants::CONFIG_DIRNAME;
use moon_node_lang::{
    node::{extract_canonical_node_module_bin, BinFile},
    NODE,
};
use moon_terminal::safe_exit;
use moon_utils::is_test_env;
use starbase_utils::dirs;
use std::env;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(not(windows))]
fn get_global_lookups(home_dir: &Path) -> Vec<PathBuf> {
    vec![
        "/usr/local/bin".into(),
        home_dir.join(".moon"),
        // Node
        home_dir.join(".proto/tools"),
        home_dir.join(".nvm/versions/node"),
        home_dir.join(".nodenv/versions"),
        home_dir.join(".fnm/node-versions"),
        home_dir.join("Library/pnpm"),
        home_dir.join(".local/share/pnpm"),
        home_dir.join(".config/yarn"),
    ]
}

#[cfg(windows)]
fn get_global_lookups(home_dir: &Path) -> Vec<PathBuf> {
    vec![
        home_dir.join(".moon"),
        // Node
        home_dir.join(".proto\\tools"),
        home_dir.join(".nvm\\versions\\node"),
        home_dir.join(".nodenv\\versions"),
        home_dir.join(".fnm\\node-versions"),
        home_dir.join("AppData\\npm"),
        home_dir.join("AppData\\Roaming\\npm"),
        home_dir.join("AppData\\Local\\pnpm"),
        home_dir.join("AppData\\Yarn\\config"),
    ]
}

fn get_local_lookups(workspace_root: &Path) -> Vec<PathBuf> {
    vec![
        workspace_root
            .join(NODE.vendor_dir.unwrap())
            .join("@moonrepo/cli")
            .join(BIN_NAME),
        workspace_root
            .join(NODE.vendor_dir.unwrap())
            .join(".bin")
            .join(BIN_NAME),
    ]
}

fn set_executed_with(path: &Path) {
    // Would show up in many snapshots otherwise!
    if !is_test_env() {
        env::set_var("MOON_EXECUTED_WITH", path.to_string_lossy().to_string());
    }
}

/// Check whether this binary has been installed globally or not.
/// If we encounter an error, simply abort early instead of failing.
fn is_globally_installed() -> bool {
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(_) => return false,
    };

    set_executed_with(&exe_path);

    // Global installs happen *outside* of moon's toolchain,
    // so we simply assume they are using their environment.
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let lookups = get_global_lookups(&home_dir);

    // If our executable path starts with the global dir,
    // then we must have been installed globally!
    lookups.iter().any(|lookup| exe_path.starts_with(lookup))
}

fn find_workspace_root(dir: &Path) -> Option<PathBuf> {
    let findable = dir.join(CONFIG_DIRNAME);

    if findable.exists() {
        return Some(dir.to_path_buf());
    }

    match dir.parent() {
        Some(parent_dir) => find_workspace_root(parent_dir),
        None => None,
    }
}

async fn run_bin(bin_path: &Path, current_dir: &Path) -> Result<(), std::io::Error> {
    // Remove the binary path from the current args list
    let args = env::args()
        .enumerate()
        .filter(|(i, arg)| {
            if *i == 0 {
                !arg.ends_with(BIN_NAME)
            } else {
                true
            }
        })
        .map(|(_, arg)| arg)
        .collect::<Vec<String>>();

    // Execute the found moon binary with the current filtered args
    let result = Command::new(bin_path)
        .args(args)
        .current_dir(current_dir)
        .spawn()?
        .wait()
        .await?;

    if !result.success() {
        safe_exit(result.code().unwrap_or(1));
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // console_subscriber::init();

    // Detect if we've been installed globally
    if let Ok(current_dir) = env::current_dir() {
        if is_globally_installed() {
            // If so, find the workspace root so we can locate the
            // locally installed `moon` binary in node modules
            if let Some(workspace_root) = find_workspace_root(&current_dir) {
                for lookup in get_local_lookups(&workspace_root) {
                    // The binary exists! So let's run that one to ensure
                    // we're running the version pinned in `package.json`,
                    // instead of this global one!
                    if lookup.exists() {
                        if let Ok(BinFile::Binary(moon_bin)) =
                            extract_canonical_node_module_bin(lookup)
                        {
                            set_executed_with(&moon_bin);

                            run_bin(&moon_bin, &current_dir)
                                .await
                                .expect("Failed to run moon binary!");

                            return;
                        }
                    }
                }
            }
        }
    }

    // Otherwise just run the CLI
    run_cli().await
}
