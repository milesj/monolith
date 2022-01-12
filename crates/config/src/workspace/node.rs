use crate::validators::validate_semver_version;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

const NODE_VERSION: &str = "16.13.1";
const NPM_VERSION: &str = "8.1.0";
const PNPM_VERSION: &str = "6.23.6";
const YARN_VERSION: &str = "3.1.0";

fn validate_node_version(value: &str) -> Result<(), ValidationError> {
    validate_semver_version("node.version", value)
}

fn validate_npm_version(value: &str) -> Result<(), ValidationError> {
    validate_semver_version("node.npm.version", value)
}

fn validate_pnpm_version(value: &str) -> Result<(), ValidationError> {
    validate_semver_version("node.pnpm.version", value)
}

fn validate_yarn_version(value: &str) -> Result<(), ValidationError> {
    validate_semver_version("node.yarn.version", value)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Validate)]
pub struct NpmConfig {
    #[validate(custom = "validate_npm_version")]
    pub version: String,
}

impl Default for NpmConfig {
    fn default() -> Self {
        NpmConfig {
            version: String::from(NPM_VERSION),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Validate)]
pub struct PnpmConfig {
    #[validate(custom = "validate_pnpm_version")]
    pub version: String,
}

impl Default for PnpmConfig {
    fn default() -> Self {
        PnpmConfig {
            version: String::from(PNPM_VERSION),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Validate)]
pub struct YarnConfig {
    #[validate(custom = "validate_yarn_version")]
    pub version: String,
}

impl Default for YarnConfig {
    fn default() -> Self {
        YarnConfig {
            version: String::from(YARN_VERSION),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig {
    pub dedupe_on_install: Option<bool>,

    #[validate]
    pub npm: Option<NpmConfig>,

    pub package_manager: Option<PackageManager>,

    #[validate]
    pub pnpm: Option<PnpmConfig>,

    pub sync_project_workspace_dependencies: Option<bool>,

    #[serde(rename = "syncTypeScriptProjectReferences")]
    pub sync_typescript_project_references: Option<bool>,

    #[validate(custom = "validate_node_version")]
    pub version: String,

    #[validate]
    pub yarn: Option<YarnConfig>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            dedupe_on_install: Some(true),
            npm: Some(NpmConfig::default()),
            package_manager: Some(PackageManager::Npm),
            pnpm: None,
            sync_project_workspace_dependencies: Some(true),
            sync_typescript_project_references: Some(true),
            version: String::from(NODE_VERSION),
            yarn: None,
        }
    }
}
