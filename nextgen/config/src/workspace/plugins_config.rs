use proto_core::PluginLocator;
use schematic::Config;
use std::collections::BTreeMap;

#[derive(Clone, Config, Debug, PartialEq)]
#[config(allow_unknown_fields)]
pub struct ExtensionConfig {
    #[setting(required)]
    pub plugin: Option<PluginLocator>,

    #[setting(flatten)]
    pub config: BTreeMap<String, serde_json::Value>,
}

impl ExtensionConfig {
    pub fn get_plugin_locator(&self) -> &PluginLocator {
        self.plugin.as_ref().unwrap()
    }
}
