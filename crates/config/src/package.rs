// package.json

use crate::errors::ConfigError;
use json_comments::StripComments;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{to_string_pretty, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    pub author: Option<Person>,
    pub bin: Option<Bin>,
    pub browser: Option<String>,
    pub bugs: Option<Bug>,
    pub bundled_dependencies: Option<Vec<String>>,
    pub config: Option<Value>,
    pub contributors: Option<Vec<Person>>,
    pub cpu: Option<Vec<String>>,
    pub dependencies: Option<DepsSet>,
    pub description: Option<String>,
    pub dev_dependencies: Option<DepsSet>,
    pub directories: Option<Directories>,
    pub engines: Option<EnginesSet>,
    pub files: Option<Vec<String>>,
    pub funding: Option<Funding>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub license: Option<License>,
    pub main: Option<String>,
    pub man: Option<StringOrArray<String>>,
    pub module: Option<String>,
    pub name: Option<String>,
    pub optional_dependencies: Option<DepsSet>,
    pub os: Option<Vec<String>>,
    pub overrides: Option<OverridesSet>,
    pub peer_dependencies: Option<DepsSet>,
    pub peer_dependencies_meta: Option<PeerDepsMetaSet>,
    pub private: Option<bool>,
    pub publish_config: Option<Value>,
    pub repository: Option<Repository>,
    pub scripts: Option<ScriptsSet>,
    #[serde(rename = "type")]
    pub type_of: Option<String>,
    pub version: Option<String>,
    pub workspaces: Option<Workspaces>,

    // Node.js specific: https://nodejs.org/api/packages.html#nodejs-packagejson-field-definitions
    pub exports: Option<Value>,
    pub imports: Option<Value>,
    pub package_manager: Option<String>,

    // Pnpm specific: https://pnpm.io/package_json
    pub pnpm: Option<Pnpm>,

    // Yarn specific: https://yarnpkg.com/configuration/manifest
    pub dependencies_meta: Option<DepsMetaSet>,
    pub language_name: Option<String>,
    pub install_config: Option<Value>,
    pub prefer_unplugged: Option<bool>,
    pub resolutions: Option<DepsSet>,

    // Unknown fields
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, Value>,

    // Non-standard
    #[serde(skip)]
    pub path: PathBuf,
}

impl PackageJson {
    pub fn load(path: &Path) -> Result<PackageJson, ConfigError> {
        let json = fs::read_to_string(path)?;

        let mut stripped = String::with_capacity(json.len());
        StripComments::new(json.as_bytes()).read_to_string(&mut stripped)?;

        // Remove trailing commas from objects.
        let pattern = Regex::new(r",(?P<valid>\s*})").unwrap();
        let stripped = pattern.replace_all(&stripped, "$valid");

        let mut cfg: PackageJson = serde_json::from_str(&stripped)?;
        cfg.path = path.to_path_buf();

        Ok(cfg)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        fs::write(&self.path, to_string_pretty(self)?)?;

        Ok(())
    }
}

pub type BinSet = BTreeMap<String, String>;
pub type DepsMetaSet = BTreeMap<String, DependencyMeta>;
pub type DepsSet = BTreeMap<String, String>;
pub type EnginesSet = BTreeMap<String, String>;
pub type OverridesSet = BTreeMap<String, StringOrObject<DepsSet>>;
pub type PeerDepsMetaSet = BTreeMap<String, PeerDependencyMeta>;
pub type ScriptsSet = BTreeMap<String, String>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum StringOrArray<T> {
    String(String),
    Object(T),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum StringOrObject<T> {
    String(String),
    Object(T),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum StringObjectAndArray<T> {
    String(String),
    Object(T),
    Array(T),
}

pub type Bin = StringOrArray<BinSet>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Bug {
    pub email: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DependencyMeta {
    pub optional: Option<bool>,

    // Yarn
    pub built: Option<bool>,
    pub unplugged: Option<bool>,

    // Pnpm
    pub injected: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Directories {
    pub bin: Option<String>,
    pub man: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FundingMetadata {
    #[serde(rename = "type")]
    pub type_of: String,
    pub url: String,
}

pub type Funding = StringObjectAndArray<FundingMetadata>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LicenseMetadata {
    #[serde(rename = "type")]
    pub type_of: String,
    pub url: String,
}

pub type License = StringObjectAndArray<LicenseMetadata>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PersonMetadata {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

pub type Person = StringOrObject<PersonMetadata>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PeerDependencyMeta {
    pub optional: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Pnpm {
    pub never_built_dependencies: Option<Vec<String>>,
    pub overrides: Option<OverridesSet>,
    pub package_extensions: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RepositoryMetadata {
    pub directory: Option<String>,
    #[serde(rename = "type")]
    pub type_of: String,
    pub url: String,
}

pub type Repository = StringOrObject<RepositoryMetadata>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorkspacesExpanded {
    pub nohoist: Option<Vec<String>>,
    pub packages: Option<Vec<String>>,
}

pub type Workspaces = StringOrObject<WorkspacesExpanded>;
