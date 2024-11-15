// use super::bin_config::BinEntry;
use schematic::Config;
use serde::Serialize;
use version_spec::UnresolvedVersionSpec;
use warpgate_api::PluginLocator;

#[derive(Clone, Config, Debug, PartialEq, Serialize)]
pub struct PipConfig {
    /// List of arguments to append to `pip install` commands.
    pub install_args: Option<Vec<String>>,
}

#[derive(Clone, Config, Debug, PartialEq)]
pub struct PythonConfig {
    /// Location of the WASM plugin to use for Python support.
    pub plugin: Option<PluginLocator>,

    /// Options for pip, when used as a package manager.
    #[setting(nested)]
    pub pip: Option<PipConfig>,

    /// Defines the virtual environment name which will be created on workspace root.
    /// Project dependencies will be installed into this. Defaults to `.venv`
    #[setting(default = ".venv")]
    pub venv_name: String,

    /// Assumes only the root `requirements.txt` is used for dependencies.
    /// Can be used to support the "one version policy" pattern.
    #[setting(default = true)]
    pub root_requirements_only: bool,

    /// The version of Python to download, install, and run `python` tasks with.
    #[setting(env = "MOON_PYTHON_VERSION")]
    pub version: Option<UnresolvedVersionSpec>,
}
