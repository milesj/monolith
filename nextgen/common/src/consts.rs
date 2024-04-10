#[cfg(windows)]
pub const BIN_NAME: &str = "moon.exe";

#[cfg(not(windows))]
pub const BIN_NAME: &str = "moon";

pub const CONFIG_DIRNAME: &str = ".moon";

pub const CONFIG_TOOLCHAIN_FILENAME: &str = "toolchain.yml";

pub const CONFIG_WORKSPACE_FILENAME: &str = "workspace.yml";

pub const CONFIG_TASKS_FILENAME: &str = "tasks.yml";

pub const CONFIG_PROJECT_FILENAME: &str = "moon.yml";

pub const CONFIG_TEMPLATE_FILENAME: &str = "template.yml";

pub const PROTO_CLI_VERSION: &str = "0.34.3";
