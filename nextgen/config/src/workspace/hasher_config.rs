use schematic::{derive_enum, Config, ConfigEnum};

derive_enum!(
    #[derive(ConfigEnum, Default)]
    pub enum HasherOptimization {
        #[default]
        Accuracy,
        Performance,
    }
);

derive_enum!(
    #[derive(ConfigEnum, Default)]
    pub enum HasherWalkStrategy {
        Glob,
        #[default]
        Vcs,
    }
);

#[derive(Config)]
pub struct HasherConfig {
    #[setting(default = 2500)]
    pub batch_size: u16,

    pub optimization: HasherOptimization,

    pub walk_strategy: HasherWalkStrategy,

    #[setting(default = true)]
    pub warn_on_missing_inputs: bool,
}
