use clap::ValueEnum;
use moon_common::path::WorkspaceRelativePathBuf;
use moon_target::{Target, TargetLocator};
use rustc_hash::FxHashSet;
use scc::HashMap;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ProfileType {
    Cpu,
    Heap,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "state", content = "hash", rename_all = "lowercase")]
pub enum TargetState {
    Completed(String),
    Failed,
    Skipped,
    Passthrough,
}

impl TargetState {
    pub fn is_complete(&self) -> bool {
        matches!(self, TargetState::Completed(_) | TargetState::Passthrough)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionContext {
    /// Should only run affected targets (via `--affected`).
    pub affected_only: bool,

    /// Initial target locators passed to `moon run`, `moon ci`, etc.
    pub initial_targets: FxHashSet<TargetLocator>,

    /// Active mutexes for tasks to acquire locks against.
    /// @mutable
    #[serde(skip)]
    pub named_mutexes: HashMap<String, Arc<Mutex<()>>>,

    /// Additional arguments passed after `--` to passthrough.
    pub passthrough_args: Vec<String>,

    /// Targets to run after the initial locators have been resolved.
    pub primary_targets: FxHashSet<Target>,

    /// The type of profiler to run tasks with.
    pub profile: Option<ProfileType>,

    /// The current state of running tasks (via their target).
    /// @mutable
    pub target_states: HashMap<Target, TargetState>,

    /// Files that have currently been touched.
    pub touched_files: FxHashSet<WorkspaceRelativePathBuf>,

    /// The workspace root.
    pub workspace_root: PathBuf,
}

impl ActionContext {
    pub fn get_or_create_mutex(&self, name: &str) -> Arc<Mutex<()>> {
        if let Some(value) = self.named_mutexes.read(name, |_, v| v.clone()) {
            return value;
        }

        let mutex = Arc::new(Mutex::new(()));

        let _ = self
            .named_mutexes
            .insert(name.to_owned(), Arc::clone(&mutex));

        mutex
    }

    pub fn set_target_state<T: AsRef<Target>>(&self, target: T, state: TargetState) {
        let _ = self.target_states.insert(target.as_ref().to_owned(), state);
    }

    pub fn should_inherit_args<T: AsRef<Target>>(&self, target: T) -> bool {
        if self.passthrough_args.is_empty() {
            return false;
        }

        let target = target.as_ref();

        // scope:task == scope:task
        if self.primary_targets.contains(target) {
            return true;
        }

        // :task == scope:task
        for locator in &self.initial_targets {
            if target.is_all_task(locator.as_str()) {
                return true;
            }
        }

        false
    }
}
