use clap::ArgEnum;
use serde::Serialize;
use strum_macros::Display;

#[derive(ArgEnum, Clone, Debug, Default, Display)]
pub enum CacheMode {
    Off,
    Read,
    #[default]
    Write,
}

#[derive(ArgEnum, Clone, Debug, Default, Display)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(ArgEnum, Clone, Copy, Debug, Display, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TouchedStatus {
    Added,
    #[default]
    All,
    Deleted,
    Modified,
    Staged,
    Unstaged,
    Untracked,
}
