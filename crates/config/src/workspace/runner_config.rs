use moon_target::Target;
use schematic::Config;

/// Configures aspects of the task runner (also known as the action pipeline).
#[derive(Clone, Config, Debug, PartialEq)]
pub struct RunnerConfig {
    /// List of target's for tasks without outputs, that should be
    /// cached and persisted.
    pub archivable_targets: Vec<Target>,

    /// Automatically clean the cache after every task run.
    #[setting(default = true)]
    pub auto_clean_cache: bool,

    /// The lifetime in which task outputs will be cached.
    #[setting(default = "7 days")]
    pub cache_lifetime: String,

    /// Automatically inherit color settings for all tasks being ran.
    #[setting(default = true)]
    pub inherit_colors_for_piped_tasks: bool,

    /// Logs the task's command and arguments when running the task.
    pub log_running_command: bool,
}
