use moon_common::path::WorkspaceRelativePathBuf;
use moon_common::Id;
use moon_config::{
    DependencyConfig, DependencyScope, DependencySource, InheritedTasksManager, LanguageType,
    PlatformType, TaskCommandArgs, TaskConfig, ToolchainConfig,
};
use moon_file_group::FileGroup;
use moon_platform_detector::detect_project_language;
use moon_project::Project;
use moon_project_builder::{DetectLanguageEvent, ProjectBuilder, ProjectBuilderContext};
use moon_task_builder::DetectPlatformEvent;
use rustc_hash::FxHashMap;
use starbase_events::{Emitter, EventState};
use starbase_sandbox::create_sandbox;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

// We need some top-level struct to hold the data used for lifetime refs.
struct Stub {
    detect_language: Emitter<DetectLanguageEvent>,
    detect_platform: Emitter<DetectPlatformEvent>,
    toolchain_config: ToolchainConfig,
    workspace_root: PathBuf,
    id: Id,
    source: WorkspaceRelativePathBuf,
}

impl Stub {
    pub fn new(id: &str, root: &Path) -> Self {
        Self {
            detect_language: Emitter::new(),
            detect_platform: Emitter::new(),
            toolchain_config: ToolchainConfig::default(),
            workspace_root: root.to_path_buf(),
            id: Id::raw(id),
            source: WorkspaceRelativePathBuf::from(id),
        }
    }

    pub async fn create_builder(&self) -> ProjectBuilder {
        self.detect_language
            .on(
                |event: Arc<DetectLanguageEvent>, data: Arc<RwLock<LanguageType>>| async move {
                    let mut data = data.write().await;
                    *data = detect_project_language(&event.project_root);

                    Ok(EventState::Stop)
                },
            )
            .await;

        ProjectBuilder::new(
            &self.id,
            &self.source,
            ProjectBuilderContext {
                detect_language: &self.detect_language,
                detect_platform: &self.detect_platform,
                toolchain_config: &self.toolchain_config,
                workspace_root: &self.workspace_root,
            },
        )
        .unwrap()
    }
}

async fn build_project(id: &str, root: &Path) -> Project {
    let stub = Stub::new(id, root);

    let manager = InheritedTasksManager::load(root, root.join("global")).unwrap();

    let mut builder = stub.create_builder().await;
    builder.load_local_config().await.unwrap();
    builder.inherit_global_config(&manager).unwrap();
    builder.build().await.unwrap()
}

async fn build_project_without_inherited(id: &str, root: &Path) -> Project {
    let stub = Stub::new(id, root);

    let mut builder = stub.create_builder().await;
    builder.load_local_config().await.unwrap();
    builder.build().await.unwrap()
}

async fn build_lang_project(id: &str) -> Project {
    let sandbox = create_sandbox("langs");
    let stub = Stub::new(id, sandbox.path());

    let mut builder = stub.create_builder().await;
    builder.load_local_config().await.unwrap();
    builder.build().await.unwrap()
}

mod project_builder {
    use super::*;

    #[tokio::test]
    async fn sets_common_fields() {
        let sandbox = create_sandbox("builder");
        let project = build_project_without_inherited("baz", sandbox.path()).await;

        assert_eq!(project.id, Id::raw("baz"));
        assert_eq!(project.source, WorkspaceRelativePathBuf::from("baz"));
        assert_eq!(project.root, sandbox.path().join("baz"));
    }

    #[tokio::test]
    async fn builds_depends_on() {
        let sandbox = create_sandbox("builder");
        let project = build_project_without_inherited("baz", sandbox.path()).await;

        assert_eq!(
            project.dependencies.into_values().collect::<Vec<_>>(),
            vec![
                DependencyConfig {
                    id: "bar".into(),
                    source: DependencySource::Explicit,
                    ..Default::default()
                },
                DependencyConfig {
                    id: "foo".into(),
                    source: DependencySource::Explicit,
                    scope: DependencyScope::Development,
                    ..Default::default()
                }
            ]
        );
    }

    // Tasks are tested heavily in the tasks-builder crate
    #[tokio::test]
    async fn builds_tasks() {
        let sandbox = create_sandbox("builder");
        let a = build_project("foo", sandbox.path()).await;
        let b = build_project("bar", sandbox.path()).await;
        let c = build_project("baz", sandbox.path()).await;

        assert_eq!(a.tasks.len(), 4);
        assert_eq!(b.tasks.len(), 3);
        assert_eq!(c.tasks.len(), 5);
    }

    mod file_groups {
        use super::*;

        #[tokio::test]
        async fn inherits_from_global_when_no_local() {
            let sandbox = create_sandbox("builder");
            let project = build_project("foo", sandbox.path()).await;

            assert_eq!(
                project.file_groups,
                FxHashMap::from_iter([
                    (
                        "sources".into(),
                        FileGroup::new_with_source(
                            "sources",
                            [WorkspaceRelativePathBuf::from("foo/node")]
                        )
                        .unwrap()
                    ),
                    (
                        "tests".into(),
                        FileGroup::new_with_source(
                            "tests",
                            [WorkspaceRelativePathBuf::from("foo/global")]
                        )
                        .unwrap()
                    ),
                    (
                        "other".into(),
                        FileGroup::new_with_source(
                            "other",
                            [WorkspaceRelativePathBuf::from("foo/global")]
                        )
                        .unwrap()
                    )
                ])
            );
        }

        #[tokio::test]
        async fn inherits_from_global_but_local_overrides() {
            let sandbox = create_sandbox("builder");
            let project = build_project("bar", sandbox.path()).await;

            assert_eq!(
                project.file_groups,
                FxHashMap::from_iter([
                    (
                        "sources".into(),
                        FileGroup::new_with_source(
                            "sources",
                            // Not node since the language is rust
                            [WorkspaceRelativePathBuf::from("bar/global")]
                        )
                        .unwrap()
                    ),
                    (
                        "tests".into(),
                        FileGroup::new_with_source(
                            "tests",
                            [WorkspaceRelativePathBuf::from("bar/global")]
                        )
                        .unwrap()
                    ),
                    (
                        "other".into(),
                        FileGroup::new_with_source(
                            "other",
                            [WorkspaceRelativePathBuf::from("bar/bar")]
                        )
                        .unwrap()
                    )
                ])
            );
        }
    }

    mod language_detect {
        use super::*;

        #[tokio::test]
        async fn inherits_from_config() {
            let sandbox = create_sandbox("builder");
            let project = build_project_without_inherited("bar", sandbox.path()).await;

            assert_eq!(project.language, LanguageType::Rust);
        }

        #[tokio::test]
        async fn detects_from_env() {
            let sandbox = create_sandbox("builder");
            let project = build_project_without_inherited("foo", sandbox.path()).await;

            assert_eq!(project.language, LanguageType::TypeScript);
        }

        #[tokio::test]
        async fn detects_bash() {
            let project = build_lang_project("bash").await;

            assert_eq!(project.language, LanguageType::Bash);
            assert_eq!(project.platform, PlatformType::System);
        }

        #[tokio::test]
        async fn detects_batch() {
            let project = build_lang_project("batch").await;

            assert_eq!(project.language, LanguageType::Batch);
            assert_eq!(project.platform, PlatformType::System);
        }

        #[tokio::test]
        async fn detects_deno() {
            let project = build_lang_project("deno").await;

            assert_eq!(project.language, LanguageType::JavaScript);
            assert_eq!(project.platform, PlatformType::Deno);

            let project = build_lang_project("deno-config").await;

            assert_eq!(project.language, LanguageType::TypeScript);
            // assert_eq!(project.platform, PlatformType::Deno);
        }

        #[tokio::test]
        async fn detects_go() {
            let project = build_lang_project("go").await;

            assert_eq!(project.language, LanguageType::Go);
            assert_eq!(project.platform, PlatformType::System);

            let project = build_lang_project("go-config").await;

            assert_eq!(project.language, LanguageType::Go);
            assert_eq!(project.platform, PlatformType::System);
        }

        #[tokio::test]
        async fn detects_js() {
            let project = build_lang_project("js").await;

            assert_eq!(project.language, LanguageType::JavaScript);
            assert_eq!(project.platform, PlatformType::Node);

            let project = build_lang_project("js-config").await;

            assert_eq!(project.language, LanguageType::JavaScript);
            assert_eq!(project.platform, PlatformType::Node);
        }

        #[tokio::test]
        async fn detects_other() {
            let project = build_lang_project("other").await;

            assert_eq!(project.language, LanguageType::Other("kotlin".into()));
            assert_eq!(project.platform, PlatformType::System);
        }

        #[tokio::test]
        async fn detects_php() {
            let project = build_lang_project("php").await;

            assert_eq!(project.language, LanguageType::Php);
            assert_eq!(project.platform, PlatformType::System);

            let project = build_lang_project("php-config").await;

            assert_eq!(project.language, LanguageType::Php);
            assert_eq!(project.platform, PlatformType::System);
        }

        #[tokio::test]
        async fn detects_python() {
            let project = build_lang_project("python").await;

            assert_eq!(project.language, LanguageType::Python);
            assert_eq!(project.platform, PlatformType::System);

            let project = build_lang_project("python-config").await;

            assert_eq!(project.language, LanguageType::Python);
            assert_eq!(project.platform, PlatformType::System);
        }

        #[tokio::test]
        async fn detects_ruby() {
            let project = build_lang_project("ruby").await;

            assert_eq!(project.language, LanguageType::Ruby);
            assert_eq!(project.platform, PlatformType::System);

            let project = build_lang_project("ruby-config").await;

            assert_eq!(project.language, LanguageType::Ruby);
            assert_eq!(project.platform, PlatformType::System);
        }

        #[tokio::test]
        async fn detects_rust() {
            let project = build_lang_project("rust").await;

            assert_eq!(project.language, LanguageType::Rust);
            assert_eq!(project.platform, PlatformType::Rust);

            let project = build_lang_project("rust-config").await;

            assert_eq!(project.language, LanguageType::Rust);
            assert_eq!(project.platform, PlatformType::Rust);
        }

        #[tokio::test]
        async fn detects_ts() {
            let project = build_lang_project("ts").await;

            assert_eq!(project.language, LanguageType::TypeScript);
            assert_eq!(project.platform, PlatformType::Node);

            let project = build_lang_project("ts-config").await;

            assert_eq!(project.language, LanguageType::TypeScript);
            assert_eq!(project.platform, PlatformType::Node);
        }
    }

    mod platform_detect {
        use super::*;

        #[tokio::test]
        async fn inherits_from_config() {
            let sandbox = create_sandbox("builder");
            let project = build_project_without_inherited("baz", sandbox.path()).await;

            assert_eq!(project.platform, PlatformType::Node);
        }

        #[tokio::test]
        async fn infers_from_config_lang() {
            let sandbox = create_sandbox("builder");
            let project = build_project_without_inherited("bar", sandbox.path()).await;

            assert_eq!(project.platform, PlatformType::Rust);
        }

        #[tokio::test]
        async fn infers_from_detected_lang() {
            let sandbox = create_sandbox("builder");
            let project = build_project_without_inherited("foo", sandbox.path()).await;

            assert_eq!(project.platform, PlatformType::Node);
        }

        #[tokio::test]
        async fn fallsback_to_project() {
            let project = build_lang_project("project-platform").await;

            assert_eq!(
                project.tasks.get("node-a").unwrap().platform,
                PlatformType::Node
            );

            assert_eq!(
                project.tasks.get("node-b").unwrap().platform,
                PlatformType::Node
            );

            assert_eq!(
                project.tasks.get("system").unwrap().platform,
                PlatformType::System
            );
        }
    }

    mod graph_extending {
        use super::*;

        #[tokio::test]
        async fn inherits_dep() {
            let sandbox = create_sandbox("builder");
            let stub = Stub::new("bar", sandbox.path());

            let mut builder = stub.create_builder().await;
            builder.load_local_config().await.unwrap();

            builder.extend_with_dependency(DependencyConfig {
                id: "foo".into(),
                scope: DependencyScope::Development,
                ..DependencyConfig::default()
            });

            let project = builder.build().await.unwrap();

            assert_eq!(
                project.dependencies.into_values().collect::<Vec<_>>(),
                vec![DependencyConfig {
                    id: "foo".into(),
                    scope: DependencyScope::Development,
                    source: DependencySource::Implicit,
                    ..DependencyConfig::default()
                }]
            );
        }

        #[tokio::test]
        async fn inherits_task() {
            let sandbox = create_sandbox("builder");
            let stub = Stub::new("bar", sandbox.path());

            let mut builder = stub.create_builder().await;
            builder.load_local_config().await.unwrap();

            builder.extend_with_task(
                Id::raw("task"),
                TaskConfig {
                    ..TaskConfig::default()
                },
            );

            let project = builder.build().await.unwrap();

            assert!(project.tasks.contains_key("task"));
        }

        #[tokio::test]
        async fn doesnt_override_task_of_same_id() {
            let sandbox = create_sandbox("builder");
            let stub = Stub::new("baz", sandbox.path());

            let mut builder = stub.create_builder().await;
            builder.load_local_config().await.unwrap();

            builder.extend_with_task(
                Id::raw("baz"),
                TaskConfig {
                    command: TaskCommandArgs::String("new-command-name".into()),
                    ..TaskConfig::default()
                },
            );

            let project = builder.build().await.unwrap();

            assert!(project.tasks.contains_key("baz"));
            assert_eq!(project.tasks.get("baz").unwrap().command, "baz");
        }
    }
}
