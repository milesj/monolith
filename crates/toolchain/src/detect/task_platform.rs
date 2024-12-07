use moon_config::PlatformType;
use regex::Regex;
use std::sync::OnceLock;

pub static BUN_COMMANDS: OnceLock<Regex> = OnceLock::new();
pub static DENO_COMMANDS: OnceLock<Regex> = OnceLock::new();
pub static PYTHON_COMMANDS: OnceLock<Regex> = OnceLock::new();
pub static RUST_COMMANDS: OnceLock<Regex> = OnceLock::new();
pub static NODE_COMMANDS: OnceLock<Regex> = OnceLock::new();
pub static UNIX_SYSTEM_COMMANDS: OnceLock<Regex> = OnceLock::new();
pub static WINDOWS_SYSTEM_COMMANDS: OnceLock<Regex> = OnceLock::new();

fn use_platform_if_enabled(
    platform: PlatformType,
    enabled_platforms: &[PlatformType],
) -> PlatformType {
    match platform {
        PlatformType::Bun if enabled_platforms.contains(&PlatformType::Bun) => return platform,
        PlatformType::Deno if enabled_platforms.contains(&PlatformType::Deno) => return platform,
        PlatformType::Node if enabled_platforms.contains(&PlatformType::Node) => return platform,
        PlatformType::Python if enabled_platforms.contains(&PlatformType::Python) => {
            return platform
        }
        PlatformType::Rust if enabled_platforms.contains(&PlatformType::Rust) => return platform,
        _ => {}
    };

    PlatformType::System
}

pub fn is_system_command(command: &str) -> bool {
    let unix = UNIX_SYSTEM_COMMANDS.get_or_init(|| {
        Regex::new(
            "^(bash|cat|cd|chmod|cp|docker|echo|find|git|grep|make|mkdir|mv|pwd|rm|rsync|svn)$",
        )
        .unwrap()
    });

    let windows = WINDOWS_SYSTEM_COMMANDS.get_or_init(|| Regex::new(
        "^(cd|cmd|cmd.exe|copy|del|dir|echo|erase|find|git|mkdir|move|rd|rename|replace|rmdir|svn|xcopy|pwsh|pwsh.exe|powershell|powershell.exe)$",
    )
    .unwrap());

    unix.is_match(command) || windows.is_match(command)
}

pub fn detect_task_platform(command: &str, enabled_platforms: &[PlatformType]) -> PlatformType {
    if BUN_COMMANDS
        .get_or_init(|| Regex::new("^(bun|bunx)$").unwrap())
        .is_match(command)
    {
        return use_platform_if_enabled(PlatformType::Bun, enabled_platforms);
    }

    if DENO_COMMANDS
        .get_or_init(|| Regex::new("^(deno)$").unwrap())
        .is_match(command)
    {
        return use_platform_if_enabled(PlatformType::Deno, enabled_platforms);
    }

    if PYTHON_COMMANDS
        .get_or_init(|| Regex::new("^(python|python3|python-3|pip|pip3|pip-3)$").unwrap())
        .is_match(command)
    {
        return use_platform_if_enabled(PlatformType::Python, enabled_platforms);
    }

    if RUST_COMMANDS
        .get_or_init(|| Regex::new("^(rust-|rustc|rustdoc|rustfmt|rustup|cargo)").unwrap())
        .is_match(command)
    {
        return use_platform_if_enabled(PlatformType::Rust, enabled_platforms);
    }

    if NODE_COMMANDS
        .get_or_init(|| {
            Regex::new("^(node|nodejs|npm|npx|yarn|yarnpkg|pnpm|pnpx|corepack)$").unwrap()
        })
        .is_match(command)
    {
        return use_platform_if_enabled(PlatformType::Node, enabled_platforms);
    }

    if is_system_command(command) {
        return PlatformType::System;
    }

    PlatformType::Unknown
}
