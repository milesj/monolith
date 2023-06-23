use crate::DENO_DEPS;
use cached::proc_macro::cached;
use moon_lang::{config_cache, LockfileDependencyVersions};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use starbase_utils::json::read_file as read_json;
use std::path::{Path, PathBuf};

config_cache!(DenoLock, DENO_DEPS.lockfile, read_json);

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DenoLock {
    remote: FxHashMap<String, String>,

    #[serde(skip)]
    pub path: PathBuf,
}

#[cached(result)]
pub fn load_lockfile_dependencies(path: PathBuf) -> miette::Result<LockfileDependencyVersions> {
    let mut deps: LockfileDependencyVersions = FxHashMap::default();

    if let Some(lockfile) = DenoLock::read(path)? {
        for (key, value) in lockfile.remote {
            deps.insert(key, vec![value]);
        }
    }

    Ok(deps)
}
