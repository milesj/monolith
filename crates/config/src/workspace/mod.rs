// .moon/workspace.yml

mod action_runner;
pub mod node;
mod typescript;
mod vcs;

use crate::errors::map_validation_errors_to_figment_errors;
use crate::helpers::gather_extended_sources;
use crate::providers::url::Url;
use crate::types::{FileGlob, FilePath};
use crate::validators::{validate_child_relative_path, validate_extends, validate_id};
use crate::ConfigError;
pub use action_runner::ActionRunnerConfig;
use figment::{
    providers::{Format, Serialized, Yaml},
    Figment,
};
use node::NodeConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
pub use typescript::TypeScriptConfig;
use validator::{Validate, ValidationError};
pub use vcs::{VcsConfig, VcsManager};

type ProjectsMap = HashMap<String, FilePath>;

// Validate the `projects` field is a map of valid file system paths
// that are relative from the workspace root. Will fail on absolute
// paths ("/"), and parent relative paths ("../").
fn validate_projects(projects: &WorkspaceProjects) -> Result<(), ValidationError> {
    if let WorkspaceProjects::Map(map) = projects {
        for (key, value) in map {
            validate_id(&format!("projects.{}", key), key)?;

            match validate_child_relative_path("projects", value) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(
    untagged,
    expecting = "expected a sequence of globs or a map of projects"
)]
pub enum WorkspaceProjects {
    List(Vec<FileGlob>),
    Map(ProjectsMap),
}

impl Default for WorkspaceProjects {
    fn default() -> Self {
        WorkspaceProjects::Map(HashMap::new())
    }
}

/// Docs: https://moonrepo.dev/docs/config/workspace
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[schemars(default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
    #[validate]
    pub action_runner: ActionRunnerConfig,

    #[validate(custom = "validate_extends")]
    pub extends: Option<String>,

    #[validate]
    pub node: NodeConfig,

    #[validate(custom = "validate_projects")]
    pub projects: WorkspaceProjects,

    #[validate]
    pub typescript: Option<TypeScriptConfig>,

    #[validate]
    pub vcs: VcsConfig,

    /// JSON schema URI.
    #[serde(skip, rename = "$schema")]
    pub schema: String,
}

impl WorkspaceConfig {
    pub fn load(path: PathBuf) -> Result<WorkspaceConfig, ConfigError> {
        let profile_name = "workspace";
        let mut figment =
            Figment::from(Serialized::defaults(WorkspaceConfig::default()).profile(&profile_name));

        for source in gather_extended_sources(&path)? {
            if source.starts_with("http") {
                figment = figment.merge(Url::from(source).profile(&profile_name));
            } else {
                figment = figment.merge(Yaml::file(source).profile(&profile_name));
            };
        }

        let mut config = WorkspaceConfig::load_config(figment.select(&profile_name))?;

        // let mut config = WorkspaceConfig::load_config(
        //     Figment::from(Serialized::defaults(WorkspaceConfig::default()).profile(&profile_name))
        //         .merge(Yaml::file(&path).profile(&profile_name))
        //         .select(&profile_name),
        // )?;

        // This is janky, but figment does not support any kind of extends mechanism,
        // and figment providers do not have access to the current config dataset,
        // so we need to double-load this config and extract in the correct order!
        // if let Some(extends) = config.extends {
        //     let mut figment = Figment::from(
        //         Serialized::defaults(WorkspaceConfig::default()).profile(&profile_name),
        //     );

        // if extends.starts_with("http") {
        //     figment = figment.merge(Url::from(extends).profile(&profile_name));
        // } else {
        //     figment = figment
        //         .merge(Yaml::file(path.parent().unwrap().join(extends)).profile(&profile_name));
        // };

        //     figment = figment.merge(Yaml::file(&path).profile(&profile_name));

        //     config = WorkspaceConfig::load_config(figment.select(&profile_name))?;
        // }

        // Versions from env vars should take precedence
        if let Ok(node_version) = env::var("MOON_NODE_VERSION") {
            config.node.version = node_version;
        }

        if let Ok(npm_version) = env::var("MOON_NPM_VERSION") {
            config.node.npm.version = npm_version;
        }

        if let Ok(pnpm_version) = env::var("MOON_PNPM_VERSION") {
            if let Some(pnpm_config) = &mut config.node.pnpm {
                pnpm_config.version = pnpm_version;
            }
        }

        if let Ok(yarn_version) = env::var("MOON_YARN_VERSION") {
            if let Some(yarn_config) = &mut config.node.yarn {
                yarn_config.version = yarn_version;
            }
        }

        Ok(config)
    }

    fn load_config(figment: Figment) -> Result<WorkspaceConfig, ConfigError> {
        let config: WorkspaceConfig = figment.extract()?;

        if let Err(errors) = config.validate() {
            return Err(ConfigError::FailedValidation(
                map_validation_errors_to_figment_errors(&figment, &errors),
            ));
        }

        Ok(config)
    }
}
