pub mod actions;
mod platform;
mod target_hasher;
pub mod task;

pub use platform::NodePlatform;
pub use target_hasher::NodeTargetHasher;

use moon_common::Id;
use moon_config2::PartialTaskConfig;
use moon_node_lang::PackageJson;
use moon_task::TaskError;
use std::collections::BTreeMap;
use task::ScriptParser;

pub fn create_tasks_from_scripts(
    project_id: &str,
    package_json: &mut PackageJson,
) -> Result<BTreeMap<Id, PartialTaskConfig>, TaskError> {
    let mut parser = ScriptParser::new(project_id);

    parser.parse_scripts(package_json)?;
    parser.update_package(package_json)?;

    Ok(parser.tasks)
}

pub fn infer_tasks_from_scripts(
    project_id: &str,
    package_json: &PackageJson,
) -> Result<BTreeMap<Id, PartialTaskConfig>, TaskError> {
    let mut parser = ScriptParser::new(project_id);

    parser.infer_scripts(package_json)?;

    Ok(parser.tasks)
}
