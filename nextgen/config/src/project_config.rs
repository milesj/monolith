// moon.yml

use crate::language_platform::{LanguageType, PlatformType};
use crate::project::*;
use crate::shapes::InputPath;
use moon_common::{cacheable, Id};
use rustc_hash::FxHashMap;
use schematic::{derive_enum, validate, Config, ConfigEnum, ValidateError};
use std::collections::BTreeMap;

#[cfg(feature = "loader")]
use std::path::Path;

fn validate_channel<D, C>(value: &str, _data: &D, _ctx: &C) -> Result<(), ValidateError> {
    if !value.is_empty() && !value.starts_with('#') {
        return Err(ValidateError::new("must start with a `#`"));
    }

    Ok(())
}

derive_enum!(
    #[derive(ConfigEnum, Copy, Default)]
    pub enum ProjectType {
        Application,
        Automation,
        Library,
        Tool,
        #[default]
        Unknown,
    }
);

cacheable!(
    #[derive(Clone, Config, Debug, Eq, PartialEq)]
    pub struct ProjectMetadataConfig {
        pub name: Option<String>,

        #[setting(validate = validate::not_empty)]
        pub description: String,

        pub owner: Option<String>,

        pub maintainers: Vec<String>,

        #[setting(validate = validate_channel)]
        pub channel: Option<String>,
    }
);

cacheable!(
    #[derive(Clone, Config, Debug, Eq, PartialEq)]
    #[serde(
        untagged,
        expecting = "expected a project name or dependency config object"
    )]
    pub enum ProjectDependsOn {
        String(Id),
        #[setting(nested)]
        Object(DependencyConfig),
    }
);

cacheable!(
    /// Docs: https://moonrepo.dev/docs/config/project
    #[derive(Clone, Config, Debug, PartialEq)]
    pub struct ProjectConfig {
        #[setting(
            default = "https://moonrepo.dev/schemas/project.json",
            rename = "$schema"
        )]
        pub schema: String,

        #[setting(nested)]
        pub depends_on: Vec<ProjectDependsOn>,

        pub env: FxHashMap<String, String>,

        pub file_groups: FxHashMap<Id, Vec<InputPath>>,

        pub id: Option<Id>,

        pub language: LanguageType,

        #[setting(nested)]
        pub owners: OwnersConfig,

        pub platform: Option<PlatformType>,

        #[setting(nested)]
        pub project: Option<ProjectMetadataConfig>,

        pub tags: Vec<Id>,

        #[setting(nested)]
        pub tasks: BTreeMap<Id, TaskConfig>,

        #[setting(nested)]
        pub toolchain: ProjectToolchainConfig,

        #[serde(rename = "type")]
        pub type_of: ProjectType,

        #[setting(nested)]
        pub workspace: ProjectWorkspaceConfig,
    }
);

#[cfg(feature = "loader")]
impl ProjectConfig {
    pub fn load<R: AsRef<Path>, P: AsRef<Path>>(
        workspace_root: R,
        path: P,
    ) -> miette::Result<ProjectConfig> {
        use crate::validate::check_yml_extension;
        use moon_common::color;
        use schematic::ConfigLoader;

        let result = ConfigLoader::<ProjectConfig>::new()
            .set_help(color::muted_light(
                "https://moonrepo.dev/docs/config/project",
            ))
            .set_root(workspace_root.as_ref())
            .file_optional(check_yml_extension(path.as_ref()))?
            .load()?;

        Ok(result.config)
    }

    pub fn load_from<R: AsRef<Path>, P: AsRef<str>>(
        workspace_root: R,
        project_source: P,
    ) -> miette::Result<ProjectConfig> {
        use moon_common::consts;

        let workspace_root = workspace_root.as_ref();

        Self::load(
            workspace_root,
            workspace_root
                .join(project_source.as_ref())
                .join(consts::CONFIG_PROJECT_FILENAME),
        )
    }

    pub fn load_partial<P: AsRef<Path>>(project_root: P) -> miette::Result<PartialProjectConfig> {
        use moon_common::{color, consts};
        use schematic::ConfigLoader;

        let path = project_root.as_ref().join(consts::CONFIG_PROJECT_FILENAME);

        Ok(ConfigLoader::<ProjectConfig>::new()
            .set_help(color::muted_light(
                "https://moonrepo.dev/docs/config/project",
            ))
            .file_optional(path)?
            .load_partial(&())?)
    }
}
