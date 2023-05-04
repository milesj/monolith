// These configs are project-level settings that override those from the root!

use crate::validators::{is_default, validate_semver_version};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

fn validate_node_version(value: &str) -> Result<(), ValidationError> {
    validate_semver_version("toolchain.node.version", value)
}

fn validate_rust_version(value: &str) -> Result<(), ValidationError> {
    validate_semver_version("toolchain.rust.version", value)
}

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[schemars(default)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectToolchainNodeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_node_version")]
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[schemars(default)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectToolchainRustConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_rust_version")]
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[schemars(default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ProjectToolchainTypeScriptConfig {
    #[serde(skip_serializing_if = "is_default")]
    pub disabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_out_dir_to_cache: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_project_references: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_project_references_to_paths: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[schemars(default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ProjectToolchainConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate]
    pub node: Option<ProjectToolchainNodeConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate]
    pub rust: Option<ProjectToolchainRustConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub typescript: Option<ProjectToolchainTypeScriptConfig>,
}

impl ProjectToolchainConfig {
    pub fn is_typescript_enabled(&self) -> bool {
        self.typescript
            .as_ref()
            .map(|ts| !ts.disabled)
            .unwrap_or(true)
    }
}
