// This test is testing the project crate in the context of the project graph,
// as we need to test task inheritance, task expansion, etc...

use moon::{generate_project_graph, load_workspace_from};
use moon_common::path::WorkspaceRelativePathBuf;
use moon_config::{
    InputPath, LanguageType, OutputPath, PartialInheritedTasksConfig, PartialNodeConfig,
    PartialRustConfig, PartialTaskConfig, PartialTaskOptionsConfig, PartialToolchainConfig,
    PartialWorkspaceConfig, PlatformType, TaskCommandArgs, WorkspaceProjects,
};
use moon_project::Project;
use moon_project_graph::ProjectGraph;
use moon_target::Target;
use moon_test_utils::{
    create_sandbox, create_sandbox_with_config, get_tasks_fixture_configs, Sandbox,
};
use rustc_hash::{FxHashMap, FxHashSet};
use starbase_utils::string_vec;
use std::collections::BTreeMap;
use std::env;
use std::fs;

async fn tasks_sandbox() -> (Sandbox, ProjectGraph) {
    tasks_sandbox_with_config(|_, _| {}).await
}

async fn tasks_sandbox_with_config<C>(callback: C) -> (Sandbox, ProjectGraph)
where
    C: FnOnce(&mut PartialWorkspaceConfig, &mut PartialInheritedTasksConfig),
{
    tasks_sandbox_internal(callback, |_| {}).await
}

async fn tasks_sandbox_with_setup<C>(callback: C) -> (Sandbox, ProjectGraph)
where
    C: FnOnce(&Sandbox),
{
    tasks_sandbox_internal(|_, _| {}, callback).await
}

async fn tasks_sandbox_internal<C, S>(cfg_callback: C, box_callback: S) -> (Sandbox, ProjectGraph)
where
    C: FnOnce(&mut PartialWorkspaceConfig, &mut PartialInheritedTasksConfig),
    S: FnOnce(&Sandbox),
{
    let (mut workspace_config, toolchain_config, mut tasks_config) = get_tasks_fixture_configs();

    cfg_callback(&mut workspace_config, &mut tasks_config);

    let sandbox = create_sandbox_with_config(
        "tasks",
        Some(workspace_config),
        Some(toolchain_config),
        Some(tasks_config),
    );

    box_callback(&sandbox);

    let mut workspace = load_workspace_from(sandbox.path()).await.unwrap();
    let graph = generate_project_graph(&mut workspace).await.unwrap();

    (sandbox, graph)
}

mod task_inheritance {
    use super::*;

    #[tokio::test]
    async fn inherits_global_tasks() {
        let (_sandbox, project_graph) = tasks_sandbox().await;

        assert_eq!(
            project_graph
                .get("noTasks")
                .unwrap()
                .get_task("standard")
                .unwrap()
                .command,
            "cmd".to_string()
        );

        assert_eq!(
            project_graph
                .get("basic")
                .unwrap()
                .get_task("withArgs")
                .unwrap()
                .args,
            string_vec!["--foo", "--bar", "baz"]
        );
    }

    #[tokio::test]
    async fn inherits_global_file_groups() {
        let (_sandbox, project_graph) = tasks_sandbox().await;

        assert_eq!(
            *project_graph
                .get("noTasks")
                .unwrap()
                .file_groups
                .get("files_glob")
                .unwrap()
                .globs,
            string_vec!["no-tasks/**/*.{ts,tsx}"]
        );

        assert_eq!(
            *project_graph
                .get("noTasks")
                .unwrap()
                .file_groups
                .get("static")
                .unwrap()
                .files,
            string_vec![
                "no-tasks/file.ts",
                "no-tasks/dir",
                "no-tasks/dir/other.tsx",
                "no-tasks/dir/subdir",
                "no-tasks/dir/subdir/another.ts"
            ]
        );
    }

    #[tokio::test]
    async fn can_override_global_file_groups() {
        let (_sandbox, project_graph) = tasks_sandbox().await;

        assert_eq!(
            *project_graph
                .get("fileGroups")
                .unwrap()
                .file_groups
                .get("files_glob")
                .unwrap()
                .globs,
            string_vec!["file-groups/**/*.{ts,tsx}"]
        );

        assert_eq!(
            *project_graph
                .get("fileGroups")
                .unwrap()
                .file_groups
                .get("static")
                .unwrap()
                .files,
            string_vec!["file-groups/file.js"]
        );
    }

    #[tokio::test]
    async fn inherits_tag_based_tasks() {
        let (_sandbox, project_graph) = tasks_sandbox_with_setup(|sandbox| {
            fs::create_dir_all(sandbox.path().join(".moon/tasks")).unwrap();

            fs::write(
                sandbox.path().join(".moon/tasks/tag-will-inherit.yml"),
                r#"
tasks:
    fromTagCommand:
        command: 'from-tag'
"#,
            )
            .unwrap();

            fs::write(
                sandbox.path().join(".moon/tasks/tag-wont-inherit.yml"),
                r#"
tasks:
    otherTagCommand:
        command: 'other-tag'
"#,
            )
            .unwrap();
        })
        .await;

        let project = project_graph.get("inheritTags").unwrap();

        assert_eq!(
            project.get_task("nonTagCommand").unwrap().command,
            "non-tag".to_string()
        );
        assert_eq!(
            project.get_task("fromTagCommand").unwrap().command,
            "from-tag".to_string()
        );
        assert_eq!(
            project.tasks.keys().cloned().collect::<Vec<_>>(),
            string_vec![
                "fromTagCommand",
                "nonTagCommand",
                "standard",
                "withArgs",
                "withInputs",
                "withOutputs"
            ]
        );
    }

    mod merge_strategies {
        use super::*;

        fn stub_global_env_vars() -> FxHashMap<String, String> {
            FxHashMap::from_iter([
                ("GLOBAL".to_owned(), "1".to_owned()),
                ("KEY".to_owned(), "a".to_owned()),
            ])
        }

        fn stub_global_task_config() -> PartialTaskConfig {
            PartialTaskConfig {
                args: Some(TaskCommandArgs::List(string_vec!["--a"])),
                command: Some(TaskCommandArgs::String("standard".to_owned())),
                deps: Some(vec![Target::parse("a:standard").unwrap()]),
                env: Some(stub_global_env_vars()),
                inputs: Some(vec![InputPath::ProjectGlob("a.*".into())]),
                outputs: Some(vec![OutputPath::ProjectFile("a.ts".into())]),
                options: Some(PartialTaskOptionsConfig {
                    cache: Some(true),
                    retry_count: Some(1),
                    run_deps_in_parallel: Some(true),
                    run_in_ci: Some(true),
                    ..PartialTaskOptionsConfig::default()
                }),
                platform: Some(PlatformType::Node),
                ..PartialTaskConfig::default()
            }
        }

        #[tokio::test]
        async fn replace() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config
                    .tasks
                    .as_mut()
                    .unwrap()
                    .insert("standard".into(), stub_global_task_config());
            })
            .await;

            let project = project_graph.get("mergeReplace").unwrap();
            let task = project.get_task("standard").unwrap();

            assert_eq!(task.command, "newcmd".to_string());
            assert_eq!(task.args, string_vec!["--b"]);
            assert_eq!(task.env, FxHashMap::from_iter([("KEY".into(), "b".into())]));

            assert_eq!(
                task.inputs,
                vec![
                    InputPath::ProjectGlob("b.*".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into())
                ]
            );
            assert_eq!(task.outputs, vec![OutputPath::ProjectFile("b.ts".into())]);
        }

        #[tokio::test]
        async fn append() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config
                    .tasks
                    .as_mut()
                    .unwrap()
                    .insert("standard".into(), stub_global_task_config());
            })
            .await;

            let project = project_graph.get("mergeAppend").unwrap();
            let task = project.get_task("standard").unwrap();

            assert_eq!(task.command, "standard".to_string());
            assert_eq!(task.args, string_vec!["--a", "--b"]);
            assert_eq!(
                task.env,
                FxHashMap::from_iter([
                    ("GLOBAL".to_owned(), "1".to_owned()),
                    ("KEY".to_owned(), "b".to_owned()),
                ])
            );
            assert_eq!(
                task.inputs,
                vec![
                    InputPath::ProjectGlob("a.*".into()),
                    InputPath::ProjectGlob("b.*".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
            assert_eq!(
                task.outputs,
                vec![
                    OutputPath::ProjectFile("a.ts".into()),
                    OutputPath::ProjectFile("b.ts".into())
                ]
            );
        }

        #[tokio::test]
        async fn prepend() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config
                    .tasks
                    .as_mut()
                    .unwrap()
                    .insert("standard".into(), stub_global_task_config());
            })
            .await;

            let project = project_graph.get("mergePrepend").unwrap();
            let task = project.get_task("standard").unwrap();

            assert_eq!(task.command, "newcmd".to_string());
            assert_eq!(task.args, string_vec!["--b", "--a"]);
            assert_eq!(
                task.env,
                FxHashMap::from_iter([
                    ("GLOBAL".to_owned(), "1".to_owned()),
                    ("KEY".to_owned(), "a".to_owned()),
                ])
            );
            assert_eq!(
                task.inputs,
                vec![
                    InputPath::ProjectGlob("b.*".into()),
                    InputPath::ProjectGlob("a.*".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
            assert_eq!(
                task.outputs,
                vec![
                    OutputPath::ProjectFile("b.ts".into()),
                    OutputPath::ProjectFile("a.ts".into())
                ]
            );
        }

        #[tokio::test]
        async fn all() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config
                    .tasks
                    .as_mut()
                    .unwrap()
                    .insert("standard".into(), stub_global_task_config());
            })
            .await;

            let project = project_graph.get("mergeAllStrategies").unwrap();
            let task = project.get_task("standard").unwrap();

            assert_eq!(task.command, "standard".to_string());
            assert_eq!(task.args, string_vec!["--a", "--b"]);
            assert_eq!(
                task.env,
                FxHashMap::from_iter([("KEY".to_owned(), "b".to_owned()),])
            );
            assert_eq!(
                task.inputs,
                vec![
                    InputPath::ProjectGlob("b.*".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
            assert_eq!(
                task.outputs,
                vec![
                    OutputPath::ProjectFile("a.ts".into()),
                    OutputPath::ProjectFile("b.ts".into())
                ]
            );
        }
    }

    mod workspace_override {
        use super::*;
        use std::collections::BTreeMap;

        async fn tasks_inheritance_sandbox() -> (Sandbox, ProjectGraph) {
            let workspace_config = PartialWorkspaceConfig {
                projects: Some(WorkspaceProjects::Globs(string_vec!["*"])),
                ..PartialWorkspaceConfig::default()
            };

            let toolchain_config = PartialToolchainConfig {
                node: Some(PartialNodeConfig::default()),
                ..PartialToolchainConfig::default()
            };

            let tasks_config = PartialInheritedTasksConfig {
                tasks: Some(BTreeMap::from_iter([
                    (
                        "a".into(),
                        PartialTaskConfig {
                            command: Some(TaskCommandArgs::String("a".into())),
                            inputs: Some(vec![InputPath::ProjectFile("a".into())]),
                            platform: Some(PlatformType::Unknown),
                            ..PartialTaskConfig::default()
                        },
                    ),
                    (
                        "b".into(),
                        PartialTaskConfig {
                            command: Some(TaskCommandArgs::String("b".into())),
                            inputs: Some(vec![InputPath::ProjectFile("b".into())]),
                            platform: Some(PlatformType::Node),
                            ..PartialTaskConfig::default()
                        },
                    ),
                    (
                        "c".into(),
                        PartialTaskConfig {
                            command: Some(TaskCommandArgs::String("c".into())),
                            inputs: Some(vec![InputPath::ProjectFile("c".into())]),
                            platform: Some(PlatformType::System),
                            ..PartialTaskConfig::default()
                        },
                    ),
                ])),
                ..PartialInheritedTasksConfig::default()
            };

            let sandbox = create_sandbox_with_config(
                "task-inheritance",
                Some(workspace_config),
                Some(toolchain_config),
                Some(tasks_config),
            );

            let mut workspace = load_workspace_from(sandbox.path()).await.unwrap();
            let graph = generate_project_graph(&mut workspace).await.unwrap();

            (sandbox, graph)
        }

        fn get_project_task_ids(project: &Project) -> Vec<String> {
            let mut ids = project
                .tasks
                .keys()
                .map(|k| k.to_string())
                .collect::<Vec<String>>();
            ids.sort();
            ids
        }

        #[tokio::test]
        async fn include() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("include").unwrap()),
                string_vec!["a", "c"]
            );
        }

        #[tokio::test]
        async fn include_none() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("include-none").unwrap()),
                string_vec![]
            );
        }

        #[tokio::test]
        async fn exclude() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("exclude").unwrap()),
                string_vec!["b"]
            );
        }

        #[tokio::test]
        async fn exclude_all() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("exclude-all").unwrap()),
                string_vec![]
            );
        }

        #[tokio::test]
        async fn exclude_none() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("exclude-none").unwrap()),
                string_vec!["a", "b", "c"]
            );
        }

        #[tokio::test]
        async fn exclude_scoped_inheritance() {
            let sandbox = create_sandbox("config-inheritance/override");
            let mut workspace = load_workspace_from(sandbox.path()).await.unwrap();
            let project_graph = generate_project_graph(&mut workspace).await.unwrap();

            assert_eq!(
                get_project_task_ids(project_graph.get("excluded").unwrap()),
                string_vec![]
            );
        }

        #[tokio::test]
        async fn rename() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            let ids = string_vec!["bar", "baz", "foo"];

            assert_eq!(
                get_project_task_ids(project_graph.get("rename").unwrap()),
                ids
            );

            for id in &ids {
                let task = project_graph.get("rename").unwrap().get_task(id).unwrap();

                assert_eq!(task.id, id.to_owned());
                assert_eq!(task.target.id, format!("rename:{id}"));
            }
        }

        #[tokio::test]
        async fn rename_scoped_inheritance() {
            let sandbox = create_sandbox("config-inheritance/override");
            let mut workspace = load_workspace_from(sandbox.path()).await.unwrap();
            let project_graph = generate_project_graph(&mut workspace).await.unwrap();

            assert_eq!(
                get_project_task_ids(project_graph.get("renamed").unwrap()),
                string_vec!["cmd"]
            );

            let task = project_graph
                .get("renamed")
                .unwrap()
                .get_task("cmd")
                .unwrap();

            assert_eq!(task.id, "cmd");
            assert_eq!(task.target.id, "renamed:cmd");
        }

        #[tokio::test]
        async fn rename_merge() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("rename-merge").unwrap()),
                string_vec!["b", "c", "foo"]
            );

            let task = project_graph
                .get("rename-merge")
                .unwrap()
                .get_task("foo")
                .unwrap();

            assert_eq!(task.id, "foo");
            assert_eq!(task.target.id, "rename-merge:foo");
            assert_eq!(task.args, string_vec!["renamed-and-merge-foo"]);
        }

        #[tokio::test]
        async fn include_exclude() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("include-exclude").unwrap()),
                string_vec!["a"]
            );
        }

        #[tokio::test]
        async fn include_exclude_rename() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            assert_eq!(
                get_project_task_ids(project_graph.get("include-exclude-rename").unwrap()),
                string_vec!["only"]
            );

            let task = project_graph
                .get("include-exclude-rename")
                .unwrap()
                .get_task("only")
                .unwrap();

            assert_eq!(task.id, "only");
            assert_eq!(task.target.id, "include-exclude-rename:only");
        }

        #[tokio::test]
        async fn handles_platforms() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            let project = project_graph.get("platform-detect").unwrap();

            assert_eq!(
                project.get_task("a").unwrap().platform,
                PlatformType::System
            );
            assert_eq!(
                project.get_task("b").unwrap().platform,
                PlatformType::System
            );
            assert_eq!(
                project.get_task("c").unwrap().platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn handles_platforms_with_language() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            let project = project_graph.get("platform-detect-lang").unwrap();

            assert_eq!(project.get_task("a").unwrap().platform, PlatformType::Node);
            assert_eq!(
                project.get_task("b").unwrap().platform,
                PlatformType::System
            );
            assert_eq!(
                project.get_task("c").unwrap().platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn resets_inputs_to_empty() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            let project = project_graph.get("inputs").unwrap();

            assert_eq!(
                project.get_task("a").unwrap().inputs,
                vec![
                    InputPath::ProjectFile("a".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
        }

        #[tokio::test]
        async fn replaces_inputs() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            let project = project_graph.get("inputs").unwrap();

            assert_eq!(
                project.get_task("b").unwrap().inputs,
                vec![
                    InputPath::ProjectFile("other".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
        }

        #[tokio::test]
        async fn appends_inputs() {
            let (_sandbox, project_graph) = tasks_inheritance_sandbox().await;

            let project = project_graph.get("inputs").unwrap();

            assert_eq!(
                project.get_task("c").unwrap().inputs,
                vec![
                    InputPath::ProjectFile("c".into()),
                    InputPath::ProjectFile("other".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
        }
    }
}

mod task_expansion {
    use super::*;

    mod expand_command {
        use super::*;

        #[tokio::test]
        async fn resolves_var_tokens() {
            let (sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();

            assert_eq!(
                *project.get_task("commandVars").unwrap().command,
                format!(
                    "{}/{}-script.sh",
                    sandbox.path().to_str().unwrap(),
                    "commandVars"
                )
            );
        }
    }

    mod expand_args {
        use super::*;

        #[tokio::test]
        async fn resolves_file_group_tokens() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            assert_eq!(
                *project_graph
                    .get("tokens")
                    .unwrap()
                    .get_task("argsFileGroups")
                    .unwrap()
                    .args,
                vec![
                    "--dirs",
                    "./dir",
                    "./dir/subdir",
                    "--files",
                    "./file.ts",
                    "./dir/other.tsx",
                    "./dir/subdir/another.ts",
                    "--globs",
                    "./**/*.{ts,tsx}",
                    "./*.js",
                    "--root",
                    "./dir",
                ]
            );
        }

        #[tokio::test]
        async fn resolves_file_group_tokens_from_workspace() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();

            assert_eq!(
                *project.get_task("argsFileGroupsWorkspace").unwrap().args,
                vec![
                    "--dirs",
                    "./tokens/dir",
                    "./tokens/dir/subdir",
                    "--files",
                    "./tokens/file.ts",
                    "./tokens/dir/other.tsx",
                    "./tokens/dir/subdir/another.ts",
                    "--globs",
                    "./tokens/**/*.{ts,tsx}",
                    "./tokens/*.js",
                    "--root",
                    "./tokens/dir",
                ]
            );
        }

        #[tokio::test]
        async fn resolves_var_tokens() {
            let (sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();

            assert_eq!(
                *project.get_task("argsVars").unwrap().args,
                vec![
                    "some/$unknown/var",
                    "--pid",
                    "tokens/foo",
                    "--proot",
                    project.root.to_str().unwrap(),
                    "--psource",
                    "foo/tokens",
                    "--target",
                    "foo/tokens:argsVars/bar",
                    "--tid=argsVars",
                    "--wsroot",
                    sandbox.path().to_str().unwrap(),
                    "--last",
                    "unknown-javascript"
                ]
            );
        }
    }

    mod expand_deps {
        use super::*;

        #[tokio::test]
        async fn inherits_implicit_deps() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config.implicit_deps = Some(vec![
                    Target::parse("build").unwrap(),
                    Target::parse("~:build").unwrap(),
                    Target::parse("project:task").unwrap(),
                ]);
            })
            .await;

            assert_eq!(
                project_graph
                    .get("basic")
                    .unwrap()
                    .get_task("build")
                    .unwrap()
                    .deps,
                // No circular!
                vec![Target::new("project", "task").unwrap()]
            );

            assert_eq!(
                project_graph
                    .get("basic")
                    .unwrap()
                    .get_task("lint")
                    .unwrap()
                    .deps,
                vec![
                    Target::new("basic", "build").unwrap(),
                    Target::new("project", "task").unwrap()
                ]
            );

            assert_eq!(
                project_graph
                    .get("basic")
                    .unwrap()
                    .get_task("test")
                    .unwrap()
                    .deps,
                vec![
                    Target::new("basic", "build").unwrap(),
                    Target::new("project", "task").unwrap()
                ]
            );
        }

        #[tokio::test]
        async fn resolves_implicit_deps_parent_depends_on() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config.implicit_deps = Some(vec![Target::parse("^:build").unwrap()]);
            })
            .await;

            assert_eq!(
                project_graph
                    .get("buildA")
                    .unwrap()
                    .get_task("build")
                    .unwrap()
                    .deps,
                vec![
                    Target::new("basic", "build").unwrap(),
                    Target::new("buildC", "build").unwrap()
                ]
            );
        }

        #[tokio::test]
        async fn avoids_implicit_deps_matching_target() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config.implicit_deps = Some(vec![Target::parse("basic:build").unwrap()]);
            })
            .await;

            assert_eq!(
                project_graph
                    .get("basic")
                    .unwrap()
                    .get_task("build")
                    .unwrap()
                    .deps,
                vec![]
            );

            assert_eq!(
                project_graph
                    .get("basic")
                    .unwrap()
                    .get_task("lint")
                    .unwrap()
                    .deps,
                vec![Target::new("basic", "build").unwrap()]
            );
        }

        #[tokio::test]
        async fn resolves_self_scope() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            assert_eq!(
                project_graph
                    .get("scopeSelf")
                    .unwrap()
                    .get_task("lint")
                    .unwrap()
                    .deps,
                vec![
                    Target::new("scopeSelf", "clean").unwrap(),
                    Target::new("scopeSelf", "build").unwrap()
                ]
            );

            // Dedupes
            assert_eq!(
                project_graph
                    .get("scopeSelf")
                    .unwrap()
                    .get_task("lintNoDupes")
                    .unwrap()
                    .deps,
                vec![Target::new("scopeSelf", "build").unwrap()]
            );

            // Ignores self
            assert_eq!(
                project_graph
                    .get("scopeSelf")
                    .unwrap()
                    .get_task("filtersSelf")
                    .unwrap()
                    .deps,
                vec![]
            );
        }

        #[tokio::test]
        async fn resolves_deps_scope() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            assert_eq!(
                project_graph
                    .get("scopeDeps")
                    .unwrap()
                    .get_task("build")
                    .unwrap()
                    .deps,
                vec![
                    Target::new("buildC", "build").unwrap(),
                    Target::new("buildB", "build").unwrap(),
                    Target::new("buildA", "build").unwrap(),
                ]
            );

            // Dedupes
            assert_eq!(
                project_graph
                    .get("scopeDeps")
                    .unwrap()
                    .get_task("buildNoDupes")
                    .unwrap()
                    .deps,
                vec![
                    Target::new("buildA", "build").unwrap(),
                    Target::new("buildC", "build").unwrap(),
                    Target::new("buildB", "build").unwrap(),
                ]
            );
        }

        #[tokio::test]
        #[should_panic(expected = "target scope not supported as a task dependency")]
        async fn errors_for_all_scope() {
            tasks_sandbox_with_setup(|sandbox| {
                sandbox.create_file(
                    "scope-all/moon.yml",
                    r#"tasks:
                build:
                  command: webpack
                  deps:
                    - :build"#,
                );
            })
            .await;
        }

        #[tokio::test]
        #[should_panic(expected = "target scope not supported as a task dependency")]
        async fn errors_for_tag_scope() {
            tasks_sandbox_with_setup(|sandbox| {
                sandbox.create_file(
                    "scope-all/moon.yml",
                    r#"tasks:
                build:
                  command: webpack
                  deps:
                    - '#tag:build'"#,
                );
            })
            .await;
        }
    }

    mod expand_env {
        use super::*;

        #[tokio::test]
        async fn loads_using_bool() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("expandEnv").unwrap();
            let task = project.get_task("envFile").unwrap();

            assert_eq!(
                task.env,
                FxHashMap::from_iter([
                    ("FOO".to_owned(), "abc".to_owned()),
                    ("BAR".to_owned(), "123".to_owned())
                ])
            );

            assert!(task.inputs.contains(&InputPath::ProjectFile(".env".into())));
            assert!(task
                .input_paths
                .contains(&WorkspaceRelativePathBuf::from(&project.source).join(".env")));
        }

        #[tokio::test]
        async fn loads_using_custom_name() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("expandEnv").unwrap();
            let task = project.get_task("envFileNamed").unwrap();

            assert_eq!(
                task.env,
                FxHashMap::from_iter([
                    ("FOO".to_owned(), "xyz".to_owned()),
                    ("BAR".to_owned(), "456".to_owned())
                ])
            );

            assert!(task
                .inputs
                .contains(&InputPath::ProjectFile(".env.production".into())));
            assert!(task.input_paths.contains(
                &WorkspaceRelativePathBuf::from(&project.source).join(".env.production")
            ));
        }

        #[tokio::test]
        async fn loads_from_workspace_root() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("expandEnv").unwrap();
            let task = project.get_task("envFileWorkspace").unwrap();

            assert_eq!(
                task.env,
                FxHashMap::from_iter([("SOURCE".to_owned(), "workspace-level".to_owned()),])
            );

            assert!(task
                .inputs
                .contains(&InputPath::WorkspaceFile(".env".into())));
            assert!(task
                .input_paths
                .contains(&WorkspaceRelativePathBuf::from(".env")));
        }

        #[tokio::test]
        async fn doesnt_override_other_env() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("expandEnv").unwrap();
            let task = project.get_task("mergeWithEnv").unwrap();

            assert_eq!(
                task.env,
                FxHashMap::from_iter([
                    ("FOO".to_owned(), "original".to_owned()),
                    ("BAR".to_owned(), "123".to_owned())
                ])
            );
        }

        #[tokio::test]
        async fn substitutes_values() {
            env::set_var("VALID", "valid-value");
            env::set_var("FOO", "foo");
            env::set_var("BAR", "bar");

            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("expandEnv").unwrap();
            let task = project.get_task("substitute").unwrap();

            assert_eq!(
                task.env,
                FxHashMap::from_iter([
                    ("BASE".to_owned(), "base".to_owned()),
                    ("SUB".to_owned(), "valid-value".to_owned()),
                    ("SUB_MISSING".to_owned(), "".to_owned()),
                    ("SUB_MULTI".to_owned(), "foo-bar".to_owned()),
                    ("SUB_MULTI_SAME".to_owned(), "foo-foo".to_owned()),
                    ("SUB_REF_SELF".to_owned(), "".to_owned())
                ])
            );

            env::remove_var("VALID");
            env::remove_var("FOO");
            env::remove_var("BAR");
        }

        #[tokio::test]
        async fn substitutes_values_in_env_file() {
            env::set_var("VALID", "valid-value");
            env::set_var("FOO", "foo");
            env::set_var("BAR", "bar");

            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("expandEnv").unwrap();
            let task = project.get_task("substituteEnvFile").unwrap();

            assert_eq!(
                task.env,
                FxHashMap::from_iter([
                    ("BASE".to_owned(), "base".to_owned()),
                    ("SUB".to_owned(), "valid-value".to_owned()),
                    ("SUB_MISSING".to_owned(), "".to_owned()),
                    ("SUB_MULTI".to_owned(), "foo-bar".to_owned()),
                    ("SUB_MULTI_SAME".to_owned(), "foo-foo".to_owned()),
                    // This is different than the `env` setting
                    ("SUB_REF_SELF".to_owned(), "base".to_owned())
                ])
            );

            env::remove_var("VALID");
            env::remove_var("FOO");
            env::remove_var("BAR");
        }

        mod project_level {
            use super::*;

            #[tokio::test]
            async fn inherits_by_default() {
                env::set_var("VALID", "valid-value");

                let (_sandbox, project_graph) = tasks_sandbox().await;

                let project = project_graph.get("expandEnvProject").unwrap();
                let task = project.get_task("inherit").unwrap();

                assert_eq!(
                    task.env,
                    FxHashMap::from_iter([
                        ("SOURCE".to_owned(), "project-level".to_owned()),
                        ("PROJECT".to_owned(), "true".to_owned()),
                        ("SUB".to_owned(), "valid-value".to_owned())
                    ])
                );

                env::remove_var("VALID");
            }

            #[tokio::test]
            async fn doesnt_override_task_level() {
                let (_sandbox, project_graph) = tasks_sandbox().await;

                let project = project_graph.get("expandEnvProject").unwrap();
                let task = project.get_task("env").unwrap();

                assert_eq!(
                    task.env,
                    FxHashMap::from_iter([
                        ("SOURCE".to_owned(), "task-level".to_owned()),
                        ("PROJECT".to_owned(), "true".to_owned()),
                        ("TASK".to_owned(), "true".to_owned()),
                        ("SUB".to_owned(), "".to_owned()),
                    ])
                );
            }

            #[tokio::test]
            async fn doesnt_override_env_file() {
                let (_sandbox, project_graph) = tasks_sandbox().await;

                let project = project_graph.get("expandEnvProject").unwrap();
                let task = project.get_task("envFile").unwrap();

                assert_eq!(
                    task.env,
                    FxHashMap::from_iter([
                        ("SOURCE".to_owned(), "env-file".to_owned()),
                        ("PROJECT".to_owned(), "true".to_owned()),
                        ("FILE".to_owned(), "true".to_owned()),
                        ("SUB".to_owned(), "".to_owned()),
                    ])
                );
            }

            #[tokio::test]
            async fn supports_all_patterns_in_parallel() {
                let (_sandbox, project_graph) = tasks_sandbox().await;

                let project = project_graph.get("expandEnvProject").unwrap();
                let task = project.get_task("all").unwrap();

                assert_eq!(
                    task.env,
                    FxHashMap::from_iter([
                        ("SOURCE".to_owned(), "task-level".to_owned()),
                        ("PROJECT".to_owned(), "true".to_owned()),
                        ("FILE".to_owned(), "true".to_owned()),
                        ("TASK".to_owned(), "true".to_owned()),
                        ("SUB".to_owned(), "".to_owned()),
                    ])
                );
            }
        }
    }

    mod expand_inputs {
        use super::*;
        use moon_test_utils::pretty_assertions::assert_eq;

        #[tokio::test]
        async fn sets_empty_inputs() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let task = project_graph
                .get("inputs")
                .unwrap()
                .get_task("noInputs")
                .unwrap();

            assert_eq!(
                task.inputs,
                vec![InputPath::WorkspaceGlob(".moon/*.yml".into()),]
            );
        }

        #[tokio::test]
        async fn sets_explicit_inputs() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let task = project_graph
                .get("inputs")
                .unwrap()
                .get_task("explicitInputs")
                .unwrap();

            assert_eq!(
                task.inputs,
                vec![
                    InputPath::ProjectFile("a".into()),
                    InputPath::ProjectFile("b".into()),
                    InputPath::ProjectFile("c".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
        }

        #[tokio::test]
        async fn defaults_to_all_glob_when_no_inputs() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let task = project_graph
                .get("inputs")
                .unwrap()
                .get_task("allInputs")
                .unwrap();

            assert_eq!(
                task.inputs,
                vec![
                    InputPath::ProjectGlob("**/*".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );
        }

        #[tokio::test]
        async fn inherits_implicit_inputs() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config.implicit_inputs =
                    Some(vec![InputPath::ProjectFile("package.json".into())]);
            })
            .await;

            let a = project_graph.get("inputA").unwrap().get_task("a").unwrap();

            assert_eq!(
                a.inputs,
                vec![
                    InputPath::ProjectFile("a.ts".into()),
                    InputPath::ProjectFile("package.json".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into()),
                ]
            );

            let c = project_graph.get("inputC").unwrap().get_task("c").unwrap();

            assert_eq!(
                c.inputs,
                vec![
                    InputPath::ProjectGlob("**/*".into()),
                    InputPath::ProjectFile("package.json".into()),
                    InputPath::WorkspaceGlob(".moon/*.yml".into())
                ]
            );
        }

        #[tokio::test]
        async fn inherits_implicit_inputs_env_vars() {
            let (_sandbox, project_graph) = tasks_sandbox_with_config(|_, tasks_config| {
                tasks_config.implicit_inputs = Some(vec![
                    InputPath::EnvVar("FOO".into()),
                    InputPath::EnvVar("BAR".into()),
                ]);
            })
            .await;

            assert_eq!(
                project_graph
                    .get("inputA")
                    .unwrap()
                    .get_task("a")
                    .unwrap()
                    .input_vars,
                FxHashSet::from_iter(string_vec!["FOO", "BAR"])
            );

            assert_eq!(
                project_graph
                    .get("inputC")
                    .unwrap()
                    .get_task("c")
                    .unwrap()
                    .input_vars,
                FxHashSet::from_iter(string_vec!["FOO", "BAR"])
            );
        }

        #[tokio::test]
        async fn resolves_file_group_tokens() {
            let (_, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();
            let project_source = WorkspaceRelativePathBuf::from(&project.source);
            let task = project.get_task("inputsFileGroups").unwrap();

            assert_eq!(
                task.input_globs,
                FxHashSet::from_iter([
                    WorkspaceRelativePathBuf::from(".moon/*.yml"),
                    project_source.join("**/*.{ts,tsx}"),
                    project_source.join("*.js"),
                ]),
            );

            let a: FxHashSet<WorkspaceRelativePathBuf> =
                FxHashSet::from_iter(task.input_paths.iter().map(WorkspaceRelativePathBuf::from));
            let b: FxHashSet<WorkspaceRelativePathBuf> = FxHashSet::from_iter(
                vec![
                    WorkspaceRelativePathBuf::from("package.json"),
                    project_source.join("file.ts"),
                    project_source.join("dir"),
                    project_source.join("dir/subdir"),
                    project_source.join("file.ts"),
                    project_source.join("dir/other.tsx"),
                    project_source.join("dir/subdir/another.ts"),
                ]
                .iter()
                .map(WorkspaceRelativePathBuf::from),
            );

            assert_eq!(a, b);
        }

        #[tokio::test]
        async fn resolves_var_tokens() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();
            let task = project.get_task("inputsVars").unwrap();

            assert!(task
                .input_globs
                .contains(&WorkspaceRelativePathBuf::from(&project.source).join("$unknown.*")));

            assert!(task.input_paths.contains(
                &WorkspaceRelativePathBuf::from(&project.source).join("dir/javascript/file")
            ));

            assert!(task
                .input_paths
                .contains(&WorkspaceRelativePathBuf::from(&project.source).join("file.unknown")));
        }

        #[tokio::test]
        async fn expands_into_correct_containers() {
            let (_, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();
            let task = project.get_task("inputs").unwrap();

            assert!(task
                .input_globs
                .contains(&WorkspaceRelativePathBuf::from(&project.source).join("glob/*")));
            assert!(task
                .input_globs
                .contains(&WorkspaceRelativePathBuf::from("glob.*")));

            assert!(task
                .input_paths
                .contains(&WorkspaceRelativePathBuf::from(&project.source).join("path.ts")));
            assert!(task
                .input_paths
                .contains(&WorkspaceRelativePathBuf::from("path/dir")));

            assert!(task.input_vars.contains("VAR"));
            assert!(task.input_vars.contains("FOO_BAR"));
            assert!(!task.input_vars.contains("UNKNOWN"));
        }
    }

    mod expand_outputs {
        use super::*;
        use moon_test_utils::pretty_assertions::assert_eq;

        #[tokio::test]
        async fn expands_into_correct_containers() {
            let (_sandbox, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();
            let task = project.get_task("outputs").unwrap();

            assert!(task
                .output_paths
                .contains(&WorkspaceRelativePathBuf::from(&project.source).join("dir")));

            let task = project.get_task("outputsGlobs").unwrap();

            assert!(task
                .output_globs
                .contains(&WorkspaceRelativePathBuf::from(&project.source).join("dir/**/*.js")));
        }

        #[tokio::test]
        async fn resolves_file_group_tokens() {
            let (_, project_graph) = tasks_sandbox().await;

            let project = project_graph.get("tokens").unwrap();
            let task = project.get_task("outputsFileGroups").unwrap();

            assert_eq!(
                task.output_globs,
                FxHashSet::from_iter([
                    WorkspaceRelativePathBuf::from(&project.source).join("**/*.{ts,tsx}"),
                    WorkspaceRelativePathBuf::from(&project.source).join("*.js")
                ]),
            );

            let a: FxHashSet<WorkspaceRelativePathBuf> =
                FxHashSet::from_iter(task.output_paths.iter().map(WorkspaceRelativePathBuf::from));
            let b: FxHashSet<WorkspaceRelativePathBuf> = FxHashSet::from_iter(vec![
                WorkspaceRelativePathBuf::from("package.json"),
                WorkspaceRelativePathBuf::from(&project.source).join("file.ts"),
                WorkspaceRelativePathBuf::from(&project.source).join("dir"),
                WorkspaceRelativePathBuf::from(&project.source).join("dir/subdir"),
                WorkspaceRelativePathBuf::from(&project.source).join("file.ts"),
                WorkspaceRelativePathBuf::from(&project.source).join("dir/other.tsx"),
                WorkspaceRelativePathBuf::from(&project.source).join("dir/subdir/another.ts"),
            ]);

            assert_eq!(a, b);
        }
    }
}

mod detection {
    use super::*;

    async fn langs_sandbox() -> (Sandbox, ProjectGraph) {
        let workspace_config = PartialWorkspaceConfig {
            projects: Some(WorkspaceProjects::Globs(string_vec!["*"])),
            ..PartialWorkspaceConfig::default()
        };

        let toolchain_config = PartialToolchainConfig {
            node: Some(PartialNodeConfig::default()),
            rust: Some(PartialRustConfig::default()),
            ..PartialToolchainConfig::default()
        };

        let tasks_config = PartialInheritedTasksConfig {
            tasks: Some(BTreeMap::from_iter([(
                "command".into(),
                PartialTaskConfig {
                    command: Some(TaskCommandArgs::String("command".into())),
                    ..PartialTaskConfig::default()
                },
            )])),
            ..PartialInheritedTasksConfig::default()
        };

        let sandbox = create_sandbox_with_config(
            "project-graph/langs",
            Some(workspace_config),
            Some(toolchain_config),
            Some(tasks_config),
        );

        let mut workspace = load_workspace_from(sandbox.path()).await.unwrap();
        let graph = generate_project_graph(&mut workspace).await.unwrap();

        (sandbox, graph)
    }

    #[tokio::test]
    async fn detects_bash() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("bash").unwrap().language,
            LanguageType::Bash
        );
    }

    #[tokio::test]
    async fn detects_batch() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("batch").unwrap().language,
            LanguageType::Batch
        );
    }

    #[tokio::test]
    async fn detects_deno() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("deno").unwrap().language,
            LanguageType::JavaScript
        );
        assert_eq!(
            project_graph.get("deno").unwrap().config.platform.unwrap(),
            PlatformType::Deno
        );

        assert_eq!(
            project_graph.get("deno-config").unwrap().language,
            LanguageType::TypeScript
        );
    }

    #[tokio::test]
    async fn detects_go() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(project_graph.get("go").unwrap().language, LanguageType::Go);
        assert_eq!(
            project_graph.get("go-config").unwrap().language,
            LanguageType::Go
        );
    }

    #[tokio::test]
    async fn detects_js() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("js").unwrap().language,
            LanguageType::JavaScript
        );
        assert_eq!(
            project_graph.get("js-config").unwrap().language,
            LanguageType::JavaScript
        );
    }

    #[tokio::test]
    async fn detects_php() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("php").unwrap().language,
            LanguageType::Php
        );
        assert_eq!(
            project_graph.get("php-config").unwrap().language,
            LanguageType::Php
        );
    }

    #[tokio::test]
    async fn detects_python() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("python").unwrap().language,
            LanguageType::Python
        );
        assert_eq!(
            project_graph.get("python-config").unwrap().language,
            LanguageType::Python
        );
    }

    #[tokio::test]
    async fn detects_ruby() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("ruby").unwrap().language,
            LanguageType::Ruby
        );
        assert_eq!(
            project_graph.get("ruby-config").unwrap().language,
            LanguageType::Ruby
        );
    }

    #[tokio::test]
    async fn detects_rust() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("rust").unwrap().language,
            LanguageType::Rust
        );
        assert_eq!(
            project_graph.get("rust-config").unwrap().language,
            LanguageType::Rust
        );
    }

    #[tokio::test]
    async fn detects_ts() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("ts").unwrap().language,
            LanguageType::TypeScript
        );
        assert_eq!(
            project_graph.get("ts-config").unwrap().language,
            LanguageType::TypeScript
        );
    }

    #[tokio::test]
    async fn detects_other() {
        let (_sandbox, project_graph) = langs_sandbox().await;

        assert_eq!(
            project_graph.get("other").unwrap().language,
            LanguageType::Other("kotlin".into())
        );
    }

    mod task_platform {
        use super::*;

        #[tokio::test]
        async fn detects_bash() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("bash")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn detects_batch() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("batch")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn detects_deno() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("deno")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Deno
            );
            assert_eq!(
                project_graph
                    .get("deno-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Deno
            );
        }

        #[tokio::test]
        async fn detects_go() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("go")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
            assert_eq!(
                project_graph
                    .get("go-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn detects_js() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("js")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Node
            );
            assert_eq!(
                project_graph
                    .get("js-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Node
            );
        }

        #[tokio::test]
        async fn detects_php() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("php")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
            assert_eq!(
                project_graph
                    .get("php-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn detects_python() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("python")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
            assert_eq!(
                project_graph
                    .get("python-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn detects_ruby() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("ruby")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
            assert_eq!(
                project_graph
                    .get("ruby-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn detects_rust() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("rust")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Rust
            );
            assert_eq!(
                project_graph
                    .get("rust-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Rust
            );
        }

        #[tokio::test]
        async fn detects_ts() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("ts")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Node
            );
            assert_eq!(
                project_graph
                    .get("ts-config")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::Node
            );
        }

        #[tokio::test]
        async fn fallsback_to_project_platform() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("project-platform")
                    .unwrap()
                    .get_task("node-a")
                    .unwrap()
                    .platform,
                PlatformType::Node
            );

            assert_eq!(
                project_graph
                    .get("project-platform")
                    .unwrap()
                    .get_task("node-b")
                    .unwrap()
                    .platform,
                PlatformType::Node
            );

            assert_eq!(
                project_graph
                    .get("project-platform")
                    .unwrap()
                    .get_task("system")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }

        #[tokio::test]
        async fn detects_other() {
            let (_sandbox, project_graph) = langs_sandbox().await;

            assert_eq!(
                project_graph
                    .get("other")
                    .unwrap()
                    .get_task("command")
                    .unwrap()
                    .platform,
                PlatformType::System
            );
        }
    }
}
