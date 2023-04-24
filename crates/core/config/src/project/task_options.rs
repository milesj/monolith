use crate::{errors::create_validation_error, validators::validate_child_or_root_path};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

fn validate_affected_files(file: &TaskOptionAffectedFilesConfig) -> Result<(), ValidationError> {
    if let TaskOptionAffectedFilesConfig::Value(value) = file {
        if value != "args" && value != "env" {
            return Err(create_validation_error(
                "invalid_value",
                "options.affectedFiles",
                "expected `args`, `env`, or a boolean",
            ));
        }
    }

    Ok(())
}

fn validate_env_file(file: &TaskOptionEnvFileConfig) -> Result<(), ValidationError> {
    if let TaskOptionEnvFileConfig::File(path) = file {
        validate_child_or_root_path("options.envFile", path)?;
    }

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged, expecting = "expected `args`, `env`, or a boolean")]
pub enum TaskOptionAffectedFilesConfig {
    Enabled(bool),
    Value(String),
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged, expecting = "expected a boolean or a file system path")]
pub enum TaskOptionEnvFileConfig {
    Enabled(bool),
    File(String),
}

impl TaskOptionEnvFileConfig {
    pub fn to_option(&self) -> Option<String> {
        match self {
            TaskOptionEnvFileConfig::Enabled(true) => Some(".env".to_owned()),
            TaskOptionEnvFileConfig::Enabled(false) => None,
            TaskOptionEnvFileConfig::File(path) => Some(path.to_owned()),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskMergeStrategy {
    #[default]
    Append,
    Prepend,
    Replace,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskOutputStyle {
    Buffer,
    BufferOnlyFailure,
    Hash,
    None,
    Stream,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[schemars(default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct TaskOptionsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_affected_files")]
    pub affected_files: Option<TaskOptionAffectedFilesConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_env_file")]
    pub env_file: Option<TaskOptionEnvFileConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_args: Option<TaskMergeStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_deps: Option<TaskMergeStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_env: Option<TaskMergeStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_inputs: Option<TaskMergeStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_outputs: Option<TaskMergeStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<TaskOutputStyle>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_deps_in_parallel: Option<bool>,

    #[serde(rename = "runInCI", skip_serializing_if = "Option::is_none")]
    pub run_in_ci: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_from_workspace_root: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<bool>,
}
