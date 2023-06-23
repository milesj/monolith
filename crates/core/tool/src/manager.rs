use crate::errors::ToolError;
use crate::tool::Tool;
use moon_platform_runtime::{Runtime, Version};
use rustc_hash::FxHashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub struct ToolManager<T: Tool> {
    cache: FxHashMap<String, T>,
    default_version: Version,
    runtime: Runtime,
}

impl<T: Tool> ToolManager<T> {
    pub fn new(runtime: Runtime) -> Self {
        ToolManager {
            cache: FxHashMap::default(),
            default_version: runtime.version(),
            runtime,
        }
    }

    pub fn get(&self) -> miette::Result<&T> {
        self.get_for_version(&self.default_version)
    }

    pub fn get_for_version<V: AsRef<Version>>(&self, version: V) -> miette::Result<&T> {
        let version = version.as_ref();

        if !self.has(version) {
            return Err(
                ToolError::UnknownTool(format!("{} {}", self.runtime, version.number)).into(),
            );
        }

        Ok(self.cache.get(&version.number).unwrap())
    }

    pub fn has(&self, version: &Version) -> bool {
        self.cache.contains_key(&version.number)
    }

    pub fn register(&mut self, version: &Version, tool: T) {
        // Nothing exists in the cache yet, so this tool must be the top-level
        // workspace tool. If so, update the default version within the platform.
        if self.default_version.is_global() && !version.is_global() {
            self.default_version = version.to_owned();
        }

        self.cache.insert(version.number.to_owned(), tool);
    }

    pub async fn setup(
        &mut self,
        version: &Version,
        last_versions: &mut FxHashMap<String, String>,
    ) -> miette::Result<u8> {
        match self.cache.get_mut(&version.number) {
            Some(cache) => Ok(cache.setup(last_versions).await?),
            None => Err(ToolError::UnknownTool(self.runtime.to_string()).into()),
        }
    }

    pub async fn teardown(&mut self, version: &Version) -> miette::Result<()> {
        if let Some(mut tool) = self.cache.remove(&version.number) {
            tool.teardown().await?;
        }

        Ok(())
    }

    pub async fn teardown_all(&mut self) -> miette::Result<()> {
        for (_, mut tool) in self.cache.drain() {
            tool.teardown().await?;
        }

        Ok(())
    }
}
