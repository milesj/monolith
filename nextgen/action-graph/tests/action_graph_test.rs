#![allow(clippy::disallowed_names)]

mod utils;

use moon_action_graph::*;
use moon_common::path::WorkspaceRelativePathBuf;
use moon_common::Id;
use moon_config::{TaskArgs, TaskDependencyConfig};
use moon_platform_runtime::*;
use moon_project_graph::ProjectGraph;
use moon_task::{Target, TargetLocator, Task};
use moon_test_utils2::generate_project_graph;
use rustc_hash::{FxHashMap, FxHashSet};
use starbase_sandbox::{assert_snapshot, create_sandbox};
use utils::ActionGraphContainer;

fn create_task(id: &str, project: &str) -> Task {
    Task {
        id: Id::raw(id),
        target: Target::new(project, id).unwrap(),
        ..Task::default()
    }
}

async fn create_project_graph() -> ProjectGraph {
    generate_project_graph("projects").await
}

// fn create_bun_runtime() -> Runtime {
//     Runtime::new(
//         PlatformType::Bun,
//         RuntimeReq::with_version(Version::new(1, 0, 0)),
//     )
// }

fn create_node_runtime() -> Runtime {
    Runtime::new(
        PlatformType::Node,
        RuntimeReq::with_version(Version::new(20, 0, 0)),
    )
}

fn create_rust_runtime() -> Runtime {
    Runtime::new(
        PlatformType::Rust,
        RuntimeReq::with_version(Version::new(1, 70, 0)),
    )
}

fn topo(graph: ActionGraph) -> Vec<ActionNode> {
    let mut nodes = vec![];
    let mut iter = graph.try_iter().unwrap();

    while iter.has_pending() {
        if let Some(index) = iter.next() {
            nodes.push(graph.get_node_from_index(&index).unwrap().to_owned());
            iter.mark_completed(index);
        }
    }

    nodes
}

mod action_graph {
    use super::*;

    #[tokio::test]
    #[should_panic(expected = "A dependency cycle has been detected for RunTask(deps:cycle2) →")]
    async fn errors_on_cycle() {
        let sandbox = create_sandbox("tasks");
        let container = ActionGraphContainer::new(sandbox.path()).await;
        let mut builder = container.create_builder();

        let project = container.project_graph.get("deps").unwrap();

        builder
            .run_task(
                &project,
                project.get_task("cycle1").unwrap(),
                &RunRequirements::default(),
            )
            .unwrap();
        builder
            .run_task(
                &project,
                project.get_task("cycle2").unwrap(),
                &RunRequirements::default(),
            )
            .unwrap();

        builder.build().unwrap().try_iter().unwrap();
    }

    mod install_deps {
        use super::*;

        #[tokio::test]
        async fn graphs() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let bar = container.project_graph.get("bar").unwrap();
            builder.install_deps(&bar, None).unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    }
                ]
            );
        }

        #[tokio::test]
        async fn ignores_dupes() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let bar = container.project_graph.get("bar").unwrap();
            builder.install_deps(&bar, None).unwrap();
            builder.install_deps(&bar, None).unwrap();
            builder.install_deps(&bar, None).unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    }
                ]
            );
        }

        #[tokio::test]
        async fn installs_in_project_when_not_in_depman_workspace() {
            let sandbox = create_sandbox("dep-workspace");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let inside = container.project_graph.get("in").unwrap();
            builder.install_deps(&inside, None).unwrap();

            let outside = container.project_graph.get("out").unwrap();
            builder.install_deps(&outside, None).unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallProjectDeps {
                        project: Id::raw("out"),
                        runtime: create_node_runtime()
                    },
                ]
            );
        }

        #[tokio::test]
        async fn doesnt_install_bun_and_node() {
            let sandbox = create_sandbox("projects");
            sandbox.append_file(".moon/toolchain.yml", "bun:\n  version: '1.0.0'");

            let container = ActionGraphContainer::new(sandbox.path()).await;

            let mut bun = create_task("bun", "bar");
            bun.platform = PlatformType::Bun;

            let mut node = create_task("node", "bar");
            node.platform = PlatformType::Node;

            let mut builder = container.create_builder();
            let project = container.project_graph.get("bar").unwrap();

            builder.install_deps(&project, Some(&bun)).unwrap();
            builder.install_deps(&project, Some(&node)).unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    }
                ]
            );

            // Reverse order
            let mut builder = container.create_builder();
            let project = container.project_graph.get("bar").unwrap();

            builder.install_deps(&project, Some(&node)).unwrap();
            builder.install_deps(&project, Some(&bun)).unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    }
                ]
            );
        }
    }

    mod run_task {
        use super::*;

        #[tokio::test]
        async fn graphs() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();

            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target
                    }
                ]
            );
        }

        #[tokio::test]
        async fn ignores_dupes() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();

            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target
                    }
                ]
            );
        }

        #[tokio::test]
        async fn doesnt_graph_if_not_affected_by_touched_files() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();

            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            // Empty set works fine, just needs to be some
            let touched_files = FxHashSet::default();

            builder
                .run_task(
                    &project,
                    &task,
                    &RunRequirements {
                        touched_files: Some(&touched_files),
                        ..Default::default()
                    },
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert!(topo(graph).is_empty());
        }

        #[tokio::test]
        async fn graphs_if_affected_by_touched_files() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let file = WorkspaceRelativePathBuf::from("bar/file.js");

            let project = container.project_graph.get("bar").unwrap();

            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;
            task.input_files.insert(file.clone());

            let touched_files = FxHashSet::from_iter([file]);

            builder
                .run_task(
                    &project,
                    &task,
                    &RunRequirements {
                        touched_files: Some(&touched_files),
                        ..Default::default()
                    },
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert!(!topo(graph).is_empty());
        }

        #[tokio::test]
        async fn task_can_have_a_diff_platform_from_project() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            // node
            let project = container.project_graph.get("bar").unwrap();

            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Rust;

            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_rust_runtime()
                    },
                    ActionNode::InstallProjectDeps {
                        project: Id::raw("bar"),
                        runtime: create_rust_runtime()
                    },
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_rust_runtime(),
                        target: task.target
                    }
                ]
            );
        }

        #[tokio::test]
        async fn sets_interactive() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();

            let mut task = create_task("build", "bar");
            task.options.interactive = true;

            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph).last().unwrap(),
                &ActionNode::RunTask {
                    args: vec![],
                    env: vec![],
                    interactive: true,
                    persistent: false,
                    runtime: Runtime::system(),
                    target: task.target
                }
            );
        }

        #[tokio::test]
        async fn sets_interactive_from_requirement() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();
            let task = create_task("build", "bar");

            builder
                .run_task(
                    &project,
                    &task,
                    &RunRequirements {
                        interactive: true,
                        ..Default::default()
                    },
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph).last().unwrap(),
                &ActionNode::RunTask {
                    args: vec![],
                    env: vec![],
                    interactive: true,
                    persistent: false,
                    runtime: Runtime::system(),
                    target: task.target
                }
            );
        }

        #[tokio::test]
        async fn sets_persistent() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();

            let mut task = create_task("build", "bar");
            task.options.persistent = true;

            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph).last().unwrap(),
                &ActionNode::RunTask {
                    args: vec![],
                    env: vec![],
                    interactive: false,
                    persistent: true,
                    runtime: Runtime::system(),
                    target: task.target
                }
            );
        }

        #[tokio::test]
        async fn distinguishes_between_args() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();
            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            // Test collapsing
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            // Separate nodes
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::String("a b c".into()),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::List(vec!["x".into(), "y".into(), "z".into()]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                    ActionNode::RunTask {
                        args: vec!["a".into(), "b".into(), "c".into()],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                    ActionNode::RunTask {
                        args: vec!["x".into(), "y".into(), "z".into()],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target
                    }
                ]
            );
        }

        #[tokio::test]
        async fn flattens_same_args() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();
            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::String("a b c".into()),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::String("a b c".into()),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec!["a".into(), "b".into(), "c".into()],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                ]
            );
        }

        #[tokio::test]
        async fn flattens_same_args_with_diff_enum() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();
            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::String("a b c".into()),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::List(vec!["a".into(), "b".into(), "c".into()]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec!["a".into(), "b".into(), "c".into()],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                ]
            );
        }

        #[tokio::test]
        async fn distinguishes_between_env() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();
            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            // Test collapsing
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            // Separate nodes
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        env: FxHashMap::from_iter([("FOO".into(), "1".into())]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        env: FxHashMap::from_iter([("BAR".into(), "2".into())]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![("FOO".into(), "1".into())],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![("BAR".into(), "2".into())],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target
                    }
                ]
            );
        }

        #[tokio::test]
        async fn flattens_same_env() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();
            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        env: FxHashMap::from_iter([("FOO".into(), "1".into())]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        env: FxHashMap::from_iter([("FOO".into(), "1".into())]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![("FOO".into(), "1".into())],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                ]
            );
        }

        #[tokio::test]
        async fn distinguishes_between_args_and_env() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("bar").unwrap();
            let mut task = create_task("build", "bar");
            task.platform = PlatformType::Node;

            // Test collapsing
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();
            builder
                .run_task(&project, &task, &RunRequirements::default())
                .unwrap();

            // Separate nodes
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::String("a b c".into()),
                        env: FxHashMap::from_iter([("FOO".into(), "1".into())]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::String("a b c".into()),
                        env: FxHashMap::from_iter([("BAR".into(), "2".into())]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();
            builder
                .run_task_with_config(
                    &project,
                    &task,
                    &RunRequirements::default(),
                    Some(&TaskDependencyConfig {
                        args: TaskArgs::String("x y z".into()),
                        env: FxHashMap::from_iter([("BAR".into(), "2".into())]),
                        ..TaskDependencyConfig::default()
                    }),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::InstallDeps {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::RunTask {
                        args: vec![],
                        env: vec![],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                    ActionNode::RunTask {
                        args: vec!["a".into(), "b".into(), "c".into()],
                        env: vec![("FOO".into(), "1".into())],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                    ActionNode::RunTask {
                        args: vec!["a".into(), "b".into(), "c".into()],
                        env: vec![("BAR".into(), "2".into())],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target.clone()
                    },
                    ActionNode::RunTask {
                        args: vec!["x".into(), "y".into(), "z".into()],
                        env: vec![("BAR".into(), "2".into())],
                        interactive: false,
                        persistent: false,
                        runtime: create_node_runtime(),
                        target: task.target
                    },
                ]
            );
        }
    }

    mod run_task_dependencies {
        use super::*;

        #[tokio::test]
        async fn runs_deps_in_parallel() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("deps").unwrap();
            let task = project.get_task("parallel").unwrap();

            builder
                .run_task(&project, task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn runs_deps_in_serial() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("deps").unwrap();
            let task = project.get_task("serial").unwrap();

            builder
                .run_task(&project, task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn can_create_a_chain() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("deps").unwrap();
            let task = project.get_task("chain1").unwrap();

            builder
                .run_task(&project, task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn doesnt_include_dependents() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("deps").unwrap();
            let task = project.get_task("base").unwrap();

            builder
                .run_task(&project, task, &RunRequirements::default())
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn includes_dependents() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("deps").unwrap();
            let task = project.get_task("base").unwrap();

            builder
                .run_task(
                    &project,
                    task,
                    &RunRequirements {
                        dependents: true,
                        ..RunRequirements::default()
                    },
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn includes_dependents_for_ci() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let project = container.project_graph.get("deps").unwrap();
            let task = project.get_task("base").unwrap();

            builder
                .run_task(
                    &project,
                    task,
                    &RunRequirements {
                        ci: true,
                        dependents: true,
                        ..RunRequirements::default()
                    },
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }
    }

    mod run_task_by_target {
        use super::*;

        #[tokio::test]
        #[should_panic(expected = "Dependencies scope (^:) is not supported in run contexts.")]
        async fn errors_on_parent_scope() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("^:build").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();
        }

        #[tokio::test]
        #[should_panic(expected = "Self scope (~:) is not supported in run contexts.")]
        async fn errors_on_self_scope() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("~:build").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();
        }

        #[tokio::test]
        async fn runs_all() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse(":build").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn runs_all_with_query() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder.set_query("language=rust").unwrap();

            builder
                .run_task_by_target(
                    Target::parse(":build").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn runs_all_no_nodes() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse(":unknown").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert!(graph.is_empty());
        }

        #[tokio::test]
        async fn doesnt_run_all_internal() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse(":internal").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert!(graph.is_empty());
        }

        #[tokio::test]
        async fn runs_by_project() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("client:lint").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        #[should_panic(expected = "No project has been configured with the name or alias unknown.")]
        async fn errors_for_unknown_project() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("unknown:build").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();
        }

        #[tokio::test]
        #[should_panic(expected = "Unknown task unknown for project server.")]
        async fn errors_for_unknown_project_task() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("server:unknown").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();
        }

        #[tokio::test]
        #[should_panic(expected = "Unknown task internal for project common.")]
        async fn errors_for_internal_task_when_explicit() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let locator = TargetLocator::Qualified(Target::parse("common:internal").unwrap());

            builder
                .run_task_by_target(
                    Target::parse("common:internal").unwrap(),
                    &RunRequirements {
                        initial_locators: vec![&locator],
                        ..RunRequirements::default()
                    },
                )
                .unwrap();
        }

        #[tokio::test]
        async fn doesnt_error_for_internal_task_when_implicit() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("common:internal").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn doesnt_error_for_internal_task_when_depended_on() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let locator = TargetLocator::Qualified(Target::parse("common:internal").unwrap());

            builder
                .run_task_by_target(
                    Target::parse("misc:requiresInternal").unwrap(),
                    &RunRequirements {
                        initial_locators: vec![&locator],
                        ..RunRequirements::default()
                    },
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn runs_tag() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("#frontend:lint").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn runs_tag_no_nodes() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("#unknown:lint").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert!(graph.is_empty());
        }

        #[tokio::test]
        async fn doesnt_run_tags_internal() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target(
                    Target::parse("#frontend:internal").unwrap(),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert!(graph.is_empty());
        }
    }

    mod run_task_by_target_locator {
        use super::*;

        #[tokio::test]
        async fn runs_by_target() {
            let sandbox = create_sandbox("tasks");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            builder
                .run_task_by_target_locator(
                    TargetLocator::Qualified(Target::parse("server:build").unwrap()),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        async fn runs_by_file_path() {
            let sandbox = create_sandbox("tasks");
            let mut container = ActionGraphContainer::new(sandbox.path()).await;

            container.project_graph.working_dir = sandbox.path().join("server/nested");

            let mut builder = container.create_builder();

            builder
                .run_task_by_target_locator(
                    TargetLocator::TaskFromWorkingDir(Id::raw("lint")),
                    &RunRequirements::default(),
                )
                .unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
        }

        #[tokio::test]
        #[should_panic(expected = "No project could be located starting from path unknown/path.")]
        async fn errors_if_no_project_by_path() {
            let sandbox = create_sandbox("tasks");
            let mut container = ActionGraphContainer::new(sandbox.path()).await;

            container.project_graph.working_dir = sandbox.path().join("unknown/path");

            let mut builder = container.create_builder();

            builder
                .run_task_by_target_locator(
                    TargetLocator::TaskFromWorkingDir(Id::raw("lint")),
                    &RunRequirements::default(),
                )
                .unwrap();
        }
    }

    mod setup_tool {
        use super::*;

        #[tokio::test]
        async fn graphs() {
            let pg = ProjectGraph::default();
            let mut builder = ActionGraphBuilder::new(&pg).unwrap();
            let system = Runtime::system();
            let node = Runtime::new(
                PlatformType::Node,
                RuntimeReq::with_version(Version::new(1, 2, 3)),
            );

            builder.setup_tool(&system);
            builder.setup_tool(&node);

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool { runtime: system },
                    ActionNode::SetupTool { runtime: node },
                ]
            );
        }

        #[tokio::test]
        async fn graphs_same_platform() {
            let pg = ProjectGraph::default();
            let mut builder = ActionGraphBuilder::new(&pg).unwrap();

            let node1 = Runtime::new(
                PlatformType::Node,
                RuntimeReq::with_version(Version::new(1, 2, 3)),
            );
            let node2 = Runtime::new_override(
                PlatformType::Node,
                RuntimeReq::with_version(Version::new(4, 5, 6)),
            );
            let node3 = Runtime::new(PlatformType::Node, RuntimeReq::Global);

            builder.setup_tool(&node1);
            builder.setup_tool(&node2);
            builder.setup_tool(&node3);

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool { runtime: node1 },
                    ActionNode::SetupTool { runtime: node2 },
                    ActionNode::SetupTool { runtime: node3 },
                ]
            );
        }

        #[tokio::test]
        async fn ignores_dupes() {
            let pg = ProjectGraph::default();
            let mut builder = ActionGraphBuilder::new(&pg).unwrap();
            let system = Runtime::system();

            builder.setup_tool(&system);
            builder.setup_tool(&system);

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool { runtime: system },
                ]
            );
        }
    }

    mod sync_project {
        use super::*;

        #[tokio::test]
        async fn graphs_single() {
            let pg = create_project_graph().await;
            let mut builder = ActionGraphBuilder::new(&pg).unwrap();

            let bar = pg.get("bar").unwrap();
            builder.sync_project(&bar).unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: Runtime::system()
                    }
                ]
            );
        }

        #[tokio::test]
        async fn graphs_single_with_dep() {
            let pg = create_project_graph().await;
            let mut builder = ActionGraphBuilder::new(&pg).unwrap();

            let foo = pg.get("foo").unwrap();
            builder.sync_project(&foo).unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("foo"),
                        runtime: Runtime::system()
                    }
                ]
            );
        }

        #[tokio::test]
        async fn graphs_multiple() {
            let pg = create_project_graph().await;
            let mut builder = ActionGraphBuilder::new(&pg).unwrap();

            let foo = pg.get("foo").unwrap();
            builder.sync_project(&foo).unwrap();

            let bar = pg.get("bar").unwrap();
            builder.sync_project(&bar).unwrap();

            let qux = pg.get("qux").unwrap();
            builder.sync_project(&qux).unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("foo"),
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("qux"),
                        runtime: Runtime::system()
                    },
                ]
            );
        }

        #[tokio::test]
        async fn ignores_dupes() {
            let pg = create_project_graph().await;
            let mut builder = ActionGraphBuilder::new(&pg).unwrap();

            let foo = pg.get("foo").unwrap();

            builder.sync_project(&foo).unwrap();
            builder.sync_project(&foo).unwrap();
            builder.sync_project(&foo).unwrap();

            let graph = builder.build().unwrap();

            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: Runtime::system()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("foo"),
                        runtime: Runtime::system()
                    }
                ]
            );
        }

        #[tokio::test]
        async fn inherits_platform_tool() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let bar = container.project_graph.get("bar").unwrap();
            builder.sync_project(&bar).unwrap();

            let qux = container.project_graph.get("qux").unwrap();
            builder.sync_project(&qux).unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::SetupTool {
                        runtime: create_rust_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("qux"),
                        runtime: create_rust_runtime()
                    },
                ]
            );
        }

        #[tokio::test]
        async fn supports_platform_override() {
            let sandbox = create_sandbox("projects");
            let container = ActionGraphContainer::new(sandbox.path()).await;
            let mut builder = container.create_builder();

            let bar = container.project_graph.get("bar").unwrap();
            builder.sync_project(&bar).unwrap();

            let baz = container.project_graph.get("baz").unwrap();
            builder.sync_project(&baz).unwrap();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(
                topo(graph),
                vec![
                    ActionNode::SyncWorkspace,
                    ActionNode::SetupTool {
                        runtime: create_node_runtime()
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("bar"),
                        runtime: create_node_runtime()
                    },
                    ActionNode::SetupTool {
                        runtime: Runtime::new_override(
                            PlatformType::Node,
                            RuntimeReq::with_version(Version::new(18, 0, 0))
                        )
                    },
                    ActionNode::SyncProject {
                        project: Id::raw("baz"),
                        runtime: Runtime::new_override(
                            PlatformType::Node,
                            RuntimeReq::with_version(Version::new(18, 0, 0))
                        )
                    },
                ]
            );
        }
    }

    mod sync_workspace {
        use super::*;

        #[tokio::test]
        async fn graphs() {
            let pg = ProjectGraph::default();

            let mut builder = ActionGraphBuilder::new(&pg).unwrap();
            builder.sync_workspace();

            let graph = builder.build().unwrap();

            assert_snapshot!(graph.to_dot());
            assert_eq!(topo(graph), vec![ActionNode::SyncWorkspace]);
        }

        #[tokio::test]
        async fn ignores_dupes() {
            let pg = ProjectGraph::default();

            let mut builder = ActionGraphBuilder::new(&pg).unwrap();
            builder.sync_workspace();
            builder.sync_workspace();
            builder.sync_workspace();

            let graph = builder.build().unwrap();

            assert_eq!(topo(graph), vec![ActionNode::SyncWorkspace]);
        }
    }
}
