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

fn validate_channel<D, C>(
    value: &str,
    _data: &D,
    _ctx: &C,
    _finalize: bool,
) -> Result<(), ValidateError> {
    if !value.is_empty() && !value.starts_with('#') {
        return Err(ValidateError::new("must start with a `#`"));
    }

    Ok(())
}

derive_enum!(
    /// The type of project, for categorizing.
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
    /// Expanded information about the project.
    #[derive(Clone, Config, Debug, PartialEq)]
    pub struct ProjectMetadataConfig {
        /// A human-readable name of the project.
        pub name: Option<String>,

        /// A description on what the project does, and why it exists.
        #[setting(validate = validate::not_empty)]
        pub description: String,

        /// The owner of the project. Can be an individual, team, or
        /// organization. The format is unspecified.
        pub owner: Option<String>,

        /// The individual maintainers of the project. The format is unspecified.
        pub maintainers: Vec<String>,

        /// The Slack, Discord, etc, channel to discuss the project.
        /// Must start with a `#`.
        #[setting(validate = validate_channel)]
        pub channel: Option<String>,
    }
);

cacheable!(
    /// Expanded information about a project dependency.
    #[derive(Clone, Config, Debug, PartialEq)]
    #[serde(
        untagged,
        expecting = "expected a project name or dependency config object"
    )]
    pub enum ProjectDependsOn {
        /// A project referenced by ID.
        String(Id),

        /// A project referenced by ID, with additional parameters to pass through.
        #[setting(nested)]
        Object(DependencyConfig),
    }
);

cacheable!(
    /// Configures information and tasks for a project.
    /// Docs: https://moonrepo.dev/docs/config/project
    #[derive(Clone, Config, Debug, PartialEq)]
    pub struct ProjectConfig {
        #[setting(
            default = "https://moonrepo.dev/schemas/project.json",
            rename = "$schema"
        )]
        pub schema: String,

        /// Other projects that this project depends on.
        #[setting(nested)]
        pub depends_on: Vec<ProjectDependsOn>,

        /// A mapping of environment variables that will be set for
        /// all tasks within the project.
        pub env: FxHashMap<String, String>,

        /// A mapping of group IDs to a list of file paths, globs, and
        /// environment variables, that can be referenced from tasks.
        pub file_groups: FxHashMap<Id, Vec<InputPath>>,

        /// Overrides the ID within the project graph, as defined in
        /// the workspace `projects` setting.
        pub id: Option<Id>,

        /// The primary programming language of the project.
        pub language: LanguageType,

        /// Defines ownership of source code within the current project, by mapping
        /// file paths and globs to owners. An owner is either a user, team, or group.
        #[setting(nested)]
        pub owners: OwnersConfig,

        /// The default platform for all tasks within the project,
        /// if their platform is unknown.
        pub platform: Option<PlatformType>,

        /// Expanded information about the project.
        #[setting(nested)]
        pub project: Option<ProjectMetadataConfig>,

        /// A list of tags that this project blongs to, for categorizing,
        /// boundary enforcement, and task inheritance.
        pub tags: Vec<Id>,

        /// A mapping of tasks by ID to parameters required for running the task.
        #[setting(nested)]
        pub tasks: BTreeMap<Id, TaskConfig>,

        /// Overrides top-level toolchain settings, scoped to this project.
        #[setting(nested)]
        pub toolchain: ProjectToolchainConfig,

        /// The type of project.
        #[serde(rename = "type")]
        pub type_of: ProjectType,

        /// Overrides top-level workspace settings, scoped to this project.
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
