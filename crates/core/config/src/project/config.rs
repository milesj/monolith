// <project path>/moon.yml

use crate::errors::{
    create_validation_error, map_validation_errors_to_figment_errors, ConfigError,
};
use crate::helpers::warn_for_unknown_fields;
use crate::project::dep::DependencyConfig;
use crate::project::language_platform::{PlatformType, ProjectLanguage};
use crate::project::task::TaskConfig;
use crate::project::toolchain::ProjectToolchainConfig;
use crate::project::workspace::ProjectWorkspaceConfig;
use crate::types::{FileGroups, ProjectID};
use crate::validators::{is_default, validate_id};
use figment::{
    providers::{Format, Serialized, YamlExtended},
    Figment,
};
use moon_utils::get_workspace_root;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;
use strum::Display;
use validator::{Validate, ValidationError};

fn deserialize_language<'de, D>(deserializer: D) -> Result<ProjectLanguage, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer) {
        Ok(buffer) => ProjectLanguage::from_str(&buffer).map_err(serde::de::Error::custom),
        Err(error) => {
            // Not aware of another way to handle nulls/undefined
            if error.to_string().contains("invalid type: null") {
                return Ok(ProjectLanguage::default());
            }

            Err(error)
        }
    }
}

fn validate_file_groups(map: &FileGroups) -> Result<(), ValidationError> {
    for key in map.keys() {
        validate_id(format!("fileGroups.{key}"), key)?;
    }

    Ok(())
}

fn validate_tasks(map: &BTreeMap<String, TaskConfig>) -> Result<(), ValidationError> {
    for (name, task) in map {
        validate_id(format!("tasks.{name}"), name)?;

        // Only fail for empty strings and not `None`
        if task.command.is_some() && task.get_command().is_empty() {
            return Err(create_validation_error(
                "required_command",
                format!("tasks.{name}.command"),
                "An npm/system command is required",
            ));
        }
    }

    Ok(())
}

fn validate_channel(value: &str) -> Result<(), ValidationError> {
    if !value.is_empty() && !value.starts_with('#') {
        return Err(create_validation_error(
            "invalid_channel",
            "project.channel",
            "Must start with a `#`",
        ));
    }

    Ok(())
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Display, Eq, JsonSchema, PartialEq, Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    #[strum(serialize = "application")]
    Application,

    #[strum(serialize = "library")]
    Library,

    #[strum(serialize = "tool")]
    Tool,

    #[default]
    #[strum(serialize = "unknown")]
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
pub struct ProjectMetadataConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_channel")]
    pub channel: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(
    untagged,
    expecting = "expected a project name or dependency config object"
)]
pub enum ProjectDependsOn {
    String(ProjectID),
    Object(DependencyConfig),
}

/// Docs: https://moonrepo.dev/docs/config/project
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[schemars(default)]
#[serde(default, rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(skip_serializing_if = "is_default")]
    pub depends_on: Vec<ProjectDependsOn>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<FxHashMap<String, String>>,

    #[serde(skip_serializing_if = "is_default")]
    #[validate(custom = "validate_file_groups")]
    pub file_groups: FileGroups,

    #[serde(
        deserialize_with = "deserialize_language",
        skip_serializing_if = "is_default"
    )]
    pub language: ProjectLanguage,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate]
    pub project: Option<ProjectMetadataConfig>,

    #[serde(skip_serializing_if = "is_default")]
    #[validate(custom = "validate_tasks")]
    #[validate]
    pub tasks: BTreeMap<String, TaskConfig>,

    #[serde(skip_serializing_if = "is_default")]
    #[validate]
    pub toolchain: ProjectToolchainConfig,

    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "type")]
    pub type_of: ProjectType,

    #[serde(skip_serializing_if = "is_default")]
    #[validate]
    pub workspace: ProjectWorkspaceConfig,

    /// JSON schema URI
    #[serde(rename = "$schema", skip_serializing_if = "is_default")]
    pub schema: String,

    /// Unknown fields
    #[serde(flatten)]
    #[schemars(skip)]
    pub unknown: BTreeMap<String, serde_yaml::Value>,
}

impl ProjectConfig {
    pub fn load<T: AsRef<Path>>(path: T) -> Result<ProjectConfig, ConfigError> {
        let path = path.as_ref();
        let profile_name = "project";
        let figment =
            Figment::from(Serialized::defaults(ProjectConfig::default()).profile(profile_name))
                .merge(YamlExtended::file(path).profile(profile_name))
                .select(profile_name);

        let config: ProjectConfig = figment.extract()?;

        warn_for_unknown_fields(
            path.strip_prefix(get_workspace_root()).unwrap(),
            &config.unknown,
        );

        if let Err(errors) = config.validate() {
            return Err(ConfigError::FailedValidation(
                map_validation_errors_to_figment_errors(&figment, &errors),
            ));
        }

        Ok(config)
    }
}
