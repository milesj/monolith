use proto_core::{PluginLocator, UnresolvedVersionSpec};
use schematic::Config;

/// Docs: https://moonrepo.dev/docs/config/toolchain#bun
#[derive(Clone, Config, Debug)]
pub struct BunConfig {
    pub plugin: Option<PluginLocator>,

    #[setting(env = "MOON_BUN_VERSION")]
    pub version: Option<UnresolvedVersionSpec>,
}