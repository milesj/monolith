use crate::errors::RunnerError;
use moon_hasher::{hash_btree, hash_vec, Digest, Hasher, Sha256};
use moon_target::Target;
use moon_task::Task;
use moon_utils::path;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetHasher {
    // Task `command`
    command: String,

    // Task `args`
    args: Vec<String>,

    // Task `deps` mapped to their hash
    deps: BTreeMap<String, String>,

    // Environment variables
    env_vars: BTreeMap<String, String>,

    // Input files and globs mapped to a unique hash
    inputs: BTreeMap<String, String>,

    // Relative output paths
    outputs: Vec<String>,

    // `moon.yml` `dependsOn`
    project_deps: Vec<String>,

    // Task `target`
    target: String,

    // Bump this to invalidate all caches
    version: String,
}

impl TargetHasher {
    pub fn new() -> Self {
        TargetHasher {
            version: "1".into(),
            ..TargetHasher::default()
        }
    }

    /// Hash additional args outside of the provided task.
    pub fn hash_args(&mut self, passthrough_args: &[String]) {
        if !passthrough_args.is_empty() {
            for arg in passthrough_args {
                self.args.push(arg.clone());
            }

            // Sort vectors to be deterministic
            self.args.sort();
        }
    }

    /// Hash a mapping of input file paths to unique file hashes.
    /// File paths *must* be relative from the workspace root.
    pub fn hash_inputs(&mut self, inputs: BTreeMap<String, String>) {
        for (file, hash) in inputs {
            // Standardize on `/` separators so that the hash is
            // the same between windows and nix machines.
            self.inputs.insert(path::standardize_separators(file), hash);
        }
    }

    /// Hash `dependsOn` from the owning project.
    pub fn hash_project_deps(&mut self, deps: Vec<&String>) {
        self.project_deps = deps.into_iter().map(|d| d.to_owned()).collect();
        self.project_deps.sort();
    }

    /// Hash `args`, `inputs`, `deps`, and `env` vars from a task.
    pub fn hash_task(&mut self, task: &Task) {
        self.command = task.command.clone();
        self.args = task.args.clone();
        self.env_vars.extend(task.env.clone());
        self.outputs = task.outputs.clone();
        self.target = task.target.id.clone();

        // Sort vectors to be deterministic
        self.args.sort();
        self.outputs.sort();

        // Inherits vars from inputs
        for var_name in &task.input_vars {
            self.env_vars
                .entry(var_name.to_owned())
                .or_insert_with(|| env::var(var_name).unwrap_or_default());
        }
    }

    /// Hash `deps` from a task and associate it with their current hash.
    pub fn hash_task_deps(
        &mut self,
        task: &Task,
        hashes: &FxHashMap<Target, String>,
    ) -> Result<(), RunnerError> {
        for dep in &task.deps {
            self.deps.insert(
                dep.id.to_owned(),
                match hashes.get(dep) {
                    Some(hash) => hash.to_owned(),
                    None => {
                        return Err(RunnerError::MissingDependencyHash(
                            dep.id.to_owned(),
                            task.target.id.to_owned(),
                        ));
                    }
                },
            );
        }

        Ok(())
    }
}

impl Hasher for TargetHasher {
    fn hash(&self, sha: &mut Sha256) {
        sha.update(self.version.as_bytes());
        sha.update(self.command.as_bytes());
        sha.update(self.target.as_bytes());

        hash_vec(&self.args, sha);
        hash_btree(&self.deps, sha);
        hash_btree(&self.env_vars, sha);
        hash_btree(&self.inputs, sha);
        hash_vec(&self.outputs, sha);
        hash_vec(&self.project_deps, sha);
    }

    fn serialize(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
