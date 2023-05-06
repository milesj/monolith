use moon_action_context::ActionContext;
use moon_config::{PlatformType, RustConfig};
use moon_platform::{Platform, Runtime, Version};
use moon_project::Project;
use moon_rust_platform::RustPlatform;
use moon_task::Task;
use moon_test_utils::create_sandbox;
use moon_utils::{process::Command, string_vec};
use rustc_hash::FxHashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn create_platform() -> RustPlatform {
    RustPlatform::new(&RustConfig::default(), &PathBuf::new())
}

fn create_task() -> Task {
    Task {
        platform: PlatformType::Rust,
        ..Task::default()
    }
}

async fn create_target_command(task: Task) -> Command {
    let platform = create_platform();

    platform
        .create_run_target_command(
            &ActionContext::default(),
            &Project::default(),
            &task,
            &Runtime::Rust(Version::new_global()),
            &PathBuf::from("cwd"),
        )
        .await
        .unwrap()
}

mod sync_project {
    use super::*;

    const TOOLCHAIN: &str = "[toolchain]\nchannel = \"1.69.0\"\n";

    #[tokio::test]
    async fn converts_legacy_file() {
        let sandbox = create_sandbox("rust/project");
        sandbox.create_file("rust-toolchain", "1.69.0");

        let project = Project {
            root: sandbox.path().to_path_buf(),
            ..Project::default()
        };

        let result = create_platform()
            .sync_project(&ActionContext::default(), &project, &FxHashMap::default())
            .await
            .unwrap();

        assert!(result);
        assert!(!sandbox.path().join("rust-toolchain").exists());
        assert!(sandbox.path().join("rust-toolchain.toml").exists());

        assert_eq!(
            fs::read_to_string(sandbox.path().join("rust-toolchain.toml")).unwrap(),
            TOOLCHAIN,
        );
    }

    #[tokio::test]
    async fn renames_legacy_file() {
        let sandbox = create_sandbox("rust/project");
        sandbox.create_file("rust-toolchain", TOOLCHAIN);

        let project = Project {
            root: sandbox.path().to_path_buf(),
            ..Project::default()
        };

        let result = create_platform()
            .sync_project(&ActionContext::default(), &project, &FxHashMap::default())
            .await
            .unwrap();

        assert!(result);
        assert!(!sandbox.path().join("rust-toolchain").exists());
        assert!(sandbox.path().join("rust-toolchain.toml").exists());

        assert_eq!(
            fs::read_to_string(sandbox.path().join("rust-toolchain.toml")).unwrap(),
            TOOLCHAIN,
        );
    }

    mod sync_toolchain_version {
        use super::*;

        #[tokio::test]
        async fn does_nothing_if_not_enabled() {
            let sandbox = create_sandbox("rust/project");
            sandbox.create_file("rust-toolchain.toml", TOOLCHAIN);

            let mut platform = create_platform();
            platform.config = RustConfig {
                sync_toolchain_config: false,
                version: Some("1.70.0".into()),
                ..RustConfig::default()
            };

            let project = Project {
                root: sandbox.path().to_path_buf(),
                ..Project::default()
            };

            let result = platform
                .sync_project(&ActionContext::default(), &project, &FxHashMap::default())
                .await
                .unwrap();

            assert!(!result);
            assert_eq!(
                fs::read_to_string(sandbox.path().join("rust-toolchain.toml")).unwrap(),
                TOOLCHAIN,
            );
        }

        #[tokio::test]
        async fn does_nothing_if_version_not_set() {
            let sandbox = create_sandbox("rust/project");
            sandbox.create_file("rust-toolchain.toml", TOOLCHAIN);

            let mut platform = create_platform();
            platform.config = RustConfig {
                sync_toolchain_config: true,
                version: None,
                ..RustConfig::default()
            };

            let project = Project {
                root: sandbox.path().to_path_buf(),
                ..Project::default()
            };

            let result = platform
                .sync_project(&ActionContext::default(), &project, &FxHashMap::default())
                .await
                .unwrap();

            assert!(!result);
            assert_eq!(
                fs::read_to_string(sandbox.path().join("rust-toolchain.toml")).unwrap(),
                TOOLCHAIN,
            );
        }

        #[tokio::test]
        async fn syncs_file() {
            let sandbox = create_sandbox("rust/project");
            sandbox.create_file("rust-toolchain.toml", TOOLCHAIN);

            let mut platform = create_platform();
            platform.config = RustConfig {
                sync_toolchain_config: true,
                version: Some("1.70.0".into()),
                ..RustConfig::default()
            };

            let project = Project {
                root: sandbox.path().to_path_buf(),
                ..Project::default()
            };

            let result = platform
                .sync_project(&ActionContext::default(), &project, &FxHashMap::default())
                .await
                .unwrap();

            assert!(result);
            assert_eq!(
                fs::read_to_string(sandbox.path().join("rust-toolchain.toml")).unwrap(),
                "[toolchain]\nchannel = \"1.70.0\"\n",
            );
        }

        #[tokio::test]
        async fn creates_file() {
            let sandbox = create_sandbox("rust/project");

            let mut platform = create_platform();
            platform.config = RustConfig {
                sync_toolchain_config: true,
                version: Some("1.70.0".into()),
                ..RustConfig::default()
            };

            let project = Project {
                root: sandbox.path().to_path_buf(),
                ..Project::default()
            };

            let result = platform
                .sync_project(&ActionContext::default(), &project, &FxHashMap::default())
                .await
                .unwrap();

            assert!(result);
            assert_eq!(
                fs::read_to_string(sandbox.path().join("rust-toolchain.toml")).unwrap(),
                "[toolchain]\nchannel = \"1.70.0\"\n",
            );
        }
    }
}

mod target_command {
    use super::*;

    #[tokio::test]
    async fn uses_rustc() {
        let mut task = create_task();
        task.command = "rustc".into();
        task.args = string_vec!["-o", "test.out", "test.rs"];

        let command = create_target_command(task).await;

        assert_eq!(command.bin, "rustc");
        assert_eq!(command.args, &["-o", "test.out", "test.rs"]);
    }

    #[tokio::test]
    async fn uses_rust_others() {
        let mut task = create_task();
        task.command = "rust-analyzer".into();
        task.args = string_vec!["-o", "test.out", "test.rs"];

        let command = create_target_command(task).await;

        assert_eq!(command.bin, "rust-analyzer");
        assert_eq!(command.args, &["-o", "test.out", "test.rs"]);
    }

    #[tokio::test]
    async fn uses_cargo() {
        let mut task = create_task();
        task.command = "cargo".into();
        task.args = string_vec!["build", "-w"];

        let command = create_target_command(task).await;

        assert_eq!(command.bin, "cargo");
        assert_eq!(command.args, &["build", "-w"]);
    }

    #[tokio::test]
    async fn uses_cargo_with_version_override() {
        let mut task = create_task();
        task.command = "cargo".into();
        task.args = string_vec!["build", "-w"];

        let platform = create_platform();

        let command = platform
            .create_run_target_command(
                &ActionContext::default(),
                &Project::default(),
                &task,
                &Runtime::Rust(Version::new_override("1.60.0")),
                &PathBuf::from("cwd"),
            )
            .await
            .unwrap();

        assert_eq!(command.bin, "cargo");
        assert_eq!(command.args, &["+1.60.0", "build", "-w"]);
    }

    #[tokio::test]
    async fn uses_cargo_bin() {
        let sandbox = create_sandbox("rust/project");
        sandbox.create_file("bin/cargo-nextest", "");

        let mut task = create_task();
        task.command = "nextest".into();
        task.args = string_vec!["run", "-w"];

        env::set_var("CARGO_HOME", sandbox.path());

        let command = create_target_command(task).await;

        env::remove_var("CARGO_HOME");

        assert_eq!(command.bin, "cargo");
        assert_eq!(command.args, &["nextest", "run", "-w"]);
    }

    #[tokio::test]
    async fn uses_cargo_bin_with_prefix() {
        let sandbox = create_sandbox("rust/project");
        sandbox.create_file("bin/cargo-nextest", "");

        let mut task = create_task();
        task.command = "cargo-nextest".into();
        task.args = string_vec!["run", "-w"];

        env::set_var("CARGO_HOME", sandbox.path());

        let command = create_target_command(task).await;

        env::remove_var("CARGO_HOME");

        assert_eq!(command.bin, "cargo");
        assert_eq!(command.args, &["nextest", "run", "-w"]);
    }

    #[tokio::test]
    async fn uses_global_bin() {
        let sandbox = create_sandbox("rust/project");
        sandbox.create_file("bin/sea-orm", "");

        let mut task = create_task();
        task.command = "sea-orm".into();
        task.args = string_vec!["migrate", "-u"];

        env::set_var("CARGO_HOME", sandbox.path());

        let command = create_target_command(task).await;

        env::remove_var("CARGO_HOME");

        assert_eq!(
            command.bin,
            sandbox.path().join("bin").join("sea-orm").to_str().unwrap()
        );
        assert_eq!(command.args, &["migrate", "-u"]);
    }

    #[tokio::test]
    #[should_panic(expected = "MissingBinary(\"Cargo binary\", \"nextest\")")]
    async fn errors_for_missing_cargo_bin() {
        let sandbox = create_sandbox("rust/project");

        let mut task = create_task();
        task.command = "nextest".into();
        task.args = string_vec!["run", "-w"];

        env::set_var("CARGO_HOME", sandbox.path());

        create_target_command(task).await;

        env::remove_var("CARGO_HOME");
    }
}
