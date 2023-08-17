use crate::hasher::ContentHasher;
use serde::Serialize;
use starbase_utils::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

pub struct HashEngine {
    /// The `.moon/cache/hashes` directory. Stores hash manifests.
    pub hashes_dir: PathBuf,

    /// The `.moon/cache/outputs` directory. Stores task outputs as hashed archives.
    pub outputs_dir: PathBuf,
}

impl HashEngine {
    pub fn new(cache_dir: &Path) -> HashEngine {
        let hashes_dir = cache_dir.join("hashes");
        let outputs_dir = cache_dir.join("outputs");

        debug!(
            hashes_dir = ?hashes_dir,
            outputs_dir = ?outputs_dir,
            "Creating hash engine",
        );

        HashEngine {
            hashes_dir,
            outputs_dir,
        }
    }

    pub fn create_hasher(&self, label: &str) -> ContentHasher {
        ContentHasher::new(label)
    }

    pub fn get_archive_path(&self, hash: &str) -> PathBuf {
        self.outputs_dir.join(format!("{hash}.tar.gz"))
    }

    pub fn get_manifest_path(&self, hash: &str) -> PathBuf {
        self.hashes_dir.join(format!("{hash}.json"))
    }

    pub fn save_manifest(&self, mut hasher: ContentHasher) -> miette::Result<String> {
        let hash = hasher.generate_hash()?;
        let path = self.get_manifest_path(&hash);

        debug!(label = hasher.label, manifest = ?path, "Saving hash manifest");

        fs::write_file(&path, hasher.serialize()?)?;

        Ok(hash)
    }

    pub fn save_manifest_without_hasher<T: Serialize>(
        &self,
        label: &str,
        content: T,
    ) -> miette::Result<String> {
        let mut hasher = ContentHasher::new(label);
        hasher.hash_content(content)?;

        self.save_manifest(hasher)
    }
}
