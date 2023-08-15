use moon_cache_item::*;
use moon_common::consts;
use moon_hash::HashEngine;
use moon_time::parse_duration;
use serde::de::DeserializeOwned;
use serde::Serialize;
use starbase_utils::{fs, json};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tracing::debug;

pub struct CacheEngine {
    /// The `.moon/cache` directory relative to workspace root.
    /// Contains cached items pertaining to runs and processes.
    pub dir: PathBuf,

    /// An engine specifically for hashing content and generating manifests.
    pub hash: HashEngine,

    /// The `.moon/cache/states` directory. Stores state information about anything...
    /// tools, dependencies, projects, tasks, etc.
    pub states_dir: PathBuf,
}

impl CacheEngine {
    pub fn new(workspace_root: &Path) -> miette::Result<CacheEngine> {
        let dir = workspace_root.join(consts::CONFIG_DIRNAME).join("cache");
        let states_dir = dir.join("states");
        let cache_tag = dir.join("CACHEDIR.TAG");

        debug!(
            cache_dir = ?dir,
            "Creating cache engine",
        );

        // Create a cache directory tag
        if !cache_tag.exists() {
            fs::write_file(
                cache_tag,
                r#"Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag created by moon.
# For information see https://bford.info/cachedir"#,
            )?;
        }

        Ok(CacheEngine {
            hash: HashEngine::new(&dir),
            dir,
            states_dir,
        })
    }

    pub fn cache<T>(&self, path: impl AsRef<OsStr>) -> miette::Result<CacheItem<T>>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        let path = PathBuf::from(path.as_ref());

        CacheItem::<T>::load(if path.is_absolute() {
            path
        } else {
            self.dir.join(path)
        })
    }

    pub fn cache_state<T>(&self, path: impl AsRef<OsStr>) -> miette::Result<CacheItem<T>>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        self.cache(self.states_dir.join(path.as_ref()))
    }

    pub fn clean_stale_cache(&self, lifetime: &str) -> miette::Result<(usize, u64)> {
        let duration =
            parse_duration(lifetime).map_err(|e| miette::miette!("Invalid lifetime: {e}"))?;

        debug!(
            "Cleaning up and deleting stale cache older than \"{}\"",
            lifetime
        );

        let hashes_dir = fs::remove_dir_stale_contents(&self.hash.hashes_dir, duration)?;
        let outputs_dir = fs::remove_dir_stale_contents(&self.hash.outputs_dir, duration)?;
        let states_dir = fs::remove_dir_stale_contents(&self.states_dir, duration)?;

        let deleted =
            hashes_dir.files_deleted + outputs_dir.files_deleted + states_dir.files_deleted;
        let bytes = hashes_dir.bytes_saved + outputs_dir.bytes_saved + states_dir.bytes_saved;

        debug!("Deleted {} files and saved {} bytes", deleted, bytes);

        Ok((deleted, bytes))
    }

    pub fn get_mode(&self) -> CacheMode {
        get_cache_mode()
    }

    pub fn write<T>(&self, path: impl AsRef<OsStr>, data: &T) -> miette::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let path = PathBuf::from(path.as_ref());
        let path = if path.is_absolute() {
            path
        } else {
            self.dir.join(path)
        };

        debug!(cache = ?path, "Writing cache");

        // This purposefully ignores the cache mode and always writes!
        json::write_file(path, &data, false)?;

        Ok(())
    }

    pub fn write_state<T>(&self, path: impl AsRef<OsStr>, state: &T) -> miette::Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write(self.states_dir.join(path.as_ref()), state)
    }
}
