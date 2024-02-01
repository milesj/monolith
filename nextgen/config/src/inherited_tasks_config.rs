use crate::language_platform::{LanguageType, PlatformType};
use crate::project::{validate_deps, TaskConfig, TaskDependency, TaskOptionsConfig};
use crate::project_config::ProjectType;
use crate::shapes::InputPath;
use moon_common::{cacheable, Id};
use rustc_hash::FxHashMap;
use schematic::{merge, validate, Config, ConfigError};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::path::PathBuf;

#[cfg(feature = "loader")]
use std::{
    path::Path,
    sync::{Arc, RwLock},
};

fn merge_fxhashmap<K, V, C>(
    mut prev: FxHashMap<K, V>,
    next: FxHashMap<K, V>,
    _context: &C,
) -> Result<Option<FxHashMap<K, V>>, ConfigError>
where
    K: Eq + Hash,
{
    for (key, value) in next {
        prev.insert(key, value);
    }

    Ok(Some(prev))
}

cacheable!(
    /// Docs: https://moonrepo.dev/docs/config/tasks
    #[derive(Clone, Config, Debug)]
    pub struct InheritedTasksConfig {
        #[setting(
            default = "https://moonrepo.dev/schemas/tasks.json",
            rename = "$schema"
        )]
        pub schema: String,

        #[setting(extend, validate = validate::extends_string)]
        pub extends: Option<String>,

        #[setting(merge = merge_fxhashmap)]
        pub file_groups: FxHashMap<Id, Vec<InputPath>>,

        #[setting(nested, merge = merge::append_vec, validate = validate_deps)]
        pub implicit_deps: Vec<TaskDependency>,

        #[setting(merge = merge::append_vec)]
        pub implicit_inputs: Vec<InputPath>,

        #[setting(nested, merge = merge::merge_btreemap)]
        pub tasks: BTreeMap<Id, TaskConfig>,

        #[setting(nested)]
        pub task_options: Option<TaskOptionsConfig>,
    }
);

#[cfg(feature = "loader")]
impl InheritedTasksConfig {
    pub fn load<F: AsRef<Path>>(path: F) -> miette::Result<InheritedTasksConfig> {
        use crate::validate::check_yml_extension;
        use moon_common::color;
        use schematic::ConfigLoader;

        let result = ConfigLoader::<InheritedTasksConfig>::new()
            .set_help(color::muted_light("https://moonrepo.dev/docs/config/tasks"))
            .file_optional(check_yml_extension(path.as_ref()))?
            .load()?;

        Ok(result.config)
    }

    pub fn load_partial<T: AsRef<Path>, F: AsRef<Path>>(
        workspace_root: T,
        path: F,
    ) -> miette::Result<PartialInheritedTasksConfig> {
        use crate::validate::check_yml_extension;
        use moon_common::color;
        use schematic::ConfigLoader;

        Ok(ConfigLoader::<InheritedTasksConfig>::new()
            .set_help(color::muted_light("https://moonrepo.dev/docs/config/tasks"))
            .set_root(workspace_root.as_ref())
            .file_optional(check_yml_extension(path.as_ref()))?
            .load_partial(&())?)
    }
}

cacheable!(
    #[derive(Clone, Debug, Default)]
    pub struct InheritedTasksResult {
        pub order: Vec<String>,
        pub layers: BTreeMap<String, PartialInheritedTasksConfig>,
        pub config: InheritedTasksConfig,
    }
);

#[derive(Debug, Default)]
pub struct InheritedTasksEntry {
    pub input: PathBuf,
    pub config: PartialInheritedTasksConfig,
}

#[derive(Debug, Default)]
pub struct InheritedTasksManager {
    #[cfg(feature = "loader")]
    cache: Arc<RwLock<FxHashMap<String, InheritedTasksResult>>>,

    pub configs: FxHashMap<String, InheritedTasksEntry>,
}

impl InheritedTasksManager {
    pub fn get_lookup_order(
        &self,
        platform: &PlatformType,
        language: &LanguageType,
        project: &ProjectType,
        tags: &[Id],
    ) -> Vec<String> {
        let mut lookup = vec!["*".to_string()];

        if platform.is_javascript() {
            lookup.push(format!("{platform}"));
        }

        lookup.push(format!("{language}"));

        if platform.is_javascript() {
            lookup.push(format!("{platform}-{project}"));
        }

        lookup.push(format!("{language}-{project}"));

        for tag in tags {
            lookup.push(format!("tag-{tag}"));
        }

        lookup
    }
}

#[cfg(feature = "loader")]
impl InheritedTasksManager {
    pub fn load<T: AsRef<Path>, D: AsRef<Path>>(
        workspace_root: T,
        moon_dir: D,
    ) -> miette::Result<InheritedTasksManager> {
        use moon_common::consts;

        let mut manager = InheritedTasksManager::default();
        let workspace_root = workspace_root.as_ref();
        let moon_dir = moon_dir.as_ref();

        // tasks.yml
        let tasks_file = moon_dir.join(consts::CONFIG_TASKS_FILENAME);

        if tasks_file.exists() {
            manager.add_config(
                workspace_root,
                &tasks_file,
                InheritedTasksConfig::load_partial(workspace_root, &tasks_file)?,
            );
        }

        // tasks/**/*.yml
        let tasks_dir = moon_dir.join("tasks");

        if tasks_dir.exists() {
            load_dir(&mut manager, workspace_root, &tasks_dir)?;
        }

        Ok(manager)
    }

    pub fn load_from<T: AsRef<Path>>(workspace_root: T) -> miette::Result<InheritedTasksManager> {
        use moon_common::consts;

        let workspace_root = workspace_root.as_ref();

        Self::load(workspace_root, workspace_root.join(consts::CONFIG_DIRNAME))
    }

    pub fn add_config(
        &mut self,
        workspace_root: &Path,
        path: &Path,
        config: PartialInheritedTasksConfig,
    ) {
        use moon_common::consts;

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        let name = if name == consts::CONFIG_TASKS_FILENAME {
            "*"
        } else if let Some(stripped_name) = name.strip_suffix(".yml") {
            stripped_name
        } else {
            return;
        };

        self.configs.insert(
            name.to_owned(),
            InheritedTasksEntry {
                input: path.strip_prefix(workspace_root).unwrap().to_path_buf(),
                config,
            },
        );
    }

    pub fn get_inherited_config(
        &self,
        platform: &PlatformType,
        language: &LanguageType,
        project: &ProjectType,
        tags: &[Id],
    ) -> miette::Result<InheritedTasksResult> {
        use moon_common::color;
        use moon_common::path::standardize_separators;
        use schematic::PartialConfig;

        let lookup_order = self.get_lookup_order(platform, language, project, tags);
        let lookup_key = lookup_order.join(":");

        // Check the cache first in read only mode!
        {
            if let Some(cache) = self.cache.read().unwrap().get(&lookup_key) {
                return Ok(cache.to_owned());
            }
        }

        // Cache the result as this lookup may be the same for a large number of projects,
        // and since this clones constantly, we can avoid a lot of allocations and overhead.
        let mut partial_config = PartialInheritedTasksConfig::default();
        let mut layers = BTreeMap::default();

        #[allow(clippy::let_unit_value)]
        let context = ();

        for lookup in &lookup_order {
            if let Some(config_entry) = self.configs.get(lookup) {
                let source_path =
                    standardize_separators(format!("{}", config_entry.input.display()));
                let mut managed_config = config_entry.config.clone();

                // Only modify tasks for `tasks/*.yml` files instead of `tasks.yml`,
                // as the latter will be globbed alongside toolchain/workspace configs.
                // We also don't know what platform each of the tasks should be yet.
                if lookup != "*" {
                    if let Some(tasks) = &mut managed_config.tasks {
                        for task in tasks.values_mut() {
                            // Automatically set this source as an input
                            task.global_inputs
                                .get_or_insert(vec![])
                                .push(InputPath::WorkspaceFile(source_path.clone()));

                            // Automatically set the platform
                            if task.platform.unwrap_or_default().is_unknown() {
                                task.platform = Some(platform.to_owned());
                            }
                        }
                    }
                }

                layers.insert(source_path, managed_config.clone());
                partial_config.merge(&context, managed_config)?;
            }
        }

        let config = partial_config.finalize(&context)?;

        config
            .validate(&context, true)
            .map_err(|error| ConfigError::Validator {
                config: format!(
                    "inherited tasks {}",
                    if platform.is_javascript() {
                        format!("({}, {}, {})", platform, language, project)
                    } else {
                        format!("({}, {})", language, project)
                    }
                ),
                error,
                help: Some(color::muted_light("https://moonrepo.dev/docs/config/tasks")),
            })?;

        let result = InheritedTasksResult {
            config: InheritedTasksConfig::from_partial(config),
            layers,
            order: lookup_order,
        };

        self.cache
            .write()
            .unwrap()
            .insert(lookup_key, result.clone());

        Ok(result)
    }
}

#[cfg(feature = "loader")]
fn load_dir(
    manager: &mut InheritedTasksManager,
    workspace_root: &Path,
    dir: &Path,
) -> miette::Result<()> {
    use std::fs;

    for entry in fs::read_dir(dir)
        .map_err(|error| ConfigError::ReadFileFailed {
            path: dir.to_path_buf(),
            error,
        })?
        .flatten()
    {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| ConfigError::ReadFileFailed {
                path: path.to_path_buf(),
                error,
            })?;

        if file_type.is_file() {
            // Non-yaml files may be located in these folders,
            // so avoid failing when trying to parse it as a config
            if path
                .extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
            {
                manager.add_config(
                    workspace_root,
                    &path,
                    InheritedTasksConfig::load_partial(workspace_root, &path)?,
                );
            }
        } else if file_type.is_dir() {
            load_dir(manager, workspace_root, &path)?;
        }
    }

    Ok(())
}
