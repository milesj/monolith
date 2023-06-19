use moon_common::path::WorkspaceRelativePathBuf;
use moon_common::Id;
use moon_config::{ProjectsAliasesMap, ProjectsSourcesMap};
use moon_hasher::{hash_btree, Digest, Hasher, Sha256};
use moon_utils::is_docker_container;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphHasher {
    aliases: BTreeMap<String, Id>,

    configs: BTreeMap<String, String>,

    sources: BTreeMap<Id, String>,

    // The project graph stores absolute file paths, which breaks moon when
    // running tasks inside and outside of a container at the same time.
    // This flag helps to continuously bust the cache.
    in_container: bool,

    // Version of the moon CLI. We need to include this so that the graph
    // cache is invalidated between each release, otherwise internal Rust
    // changes (in project or task crates) are not reflected until the cache
    // is invalidated, which puts the program in a weird state.
    version: String,
}

impl GraphHasher {
    pub fn new() -> Self {
        GraphHasher {
            aliases: BTreeMap::default(),
            configs: BTreeMap::default(),
            in_container: is_docker_container(),
            sources: BTreeMap::default(),
            version: env::var("MOON_VERSION").unwrap_or_default(),
        }
    }

    pub fn hash_aliases(&mut self, aliases: &ProjectsAliasesMap) {
        self.aliases.extend(aliases.to_owned());
    }

    pub fn hash_configs(&mut self, configs: &BTreeMap<WorkspaceRelativePathBuf, String>) {
        for (config, hash) in configs {
            self.configs.insert(config.to_string(), hash.to_owned());
        }
    }

    pub fn hash_sources(&mut self, sources: &ProjectsSourcesMap) {
        self.sources.extend(sources.to_owned());
    }
}

impl Hasher for GraphHasher {
    fn hash(&self, sha: &mut Sha256) {
        hash_btree(&self.aliases, sha);
        hash_btree(&self.configs, sha);
        hash_btree(&self.sources, sha);
        sha.update(self.version.as_bytes());
        sha.update(self.in_container.to_string().as_bytes());
    }

    fn serialize(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
