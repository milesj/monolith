mod actions;
mod bun_platform;
mod target_hash;

pub use bun_platform::*;

use moon_common::Id;
use moon_config::{NodePackageManager, PartialTaskConfig, PlatformType};
use moon_javascript_platform::ScriptParser;
use moon_node_lang::PackageJson;
use std::collections::BTreeMap;

pub fn infer_tasks_from_scripts(
    project_id: &str,
    package_json: &PackageJson,
) -> miette::Result<BTreeMap<Id, PartialTaskConfig>> {
    let mut parser = ScriptParser::new(project_id, PlatformType::Bun, NodePackageManager::Bun);

    parser.infer_scripts(package_json)?;

    Ok(parser.tasks)
}
