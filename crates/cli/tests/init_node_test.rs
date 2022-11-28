use moon_test_utils::create_sandbox;
use predicates::prelude::*;
use std::fs;

mod init_node {
    use super::*;

    #[test]
    fn infers_version_from_nvm() {
        let sandbox = create_sandbox("init-sandbox");
        let root = sandbox.path().to_path_buf();
        let config = root.join(".moon").join("toolchain.yml");

        fs::write(&root.join(".nvmrc"), "1.2.3").unwrap();

        sandbox.run_moon(|cmd| {
            cmd.arg("init").arg("--yes").arg(root);
        });

        let content = fs::read_to_string(config).unwrap();

        assert!(predicate::str::contains("version: '1.2.3'").eval(&content));
    }

    #[test]
    fn infers_version_from_nodenv() {
        let sandbox = create_sandbox("init-sandbox");
        let root = sandbox.path().to_path_buf();
        let config = root.join(".moon").join("toolchain.yml");

        fs::write(&root.join(".node-version"), "1.2.3").unwrap();

        sandbox.run_moon(|cmd| {
            cmd.arg("init").arg("--yes").arg(root);
        });

        let content = fs::read_to_string(config).unwrap();

        assert!(predicate::str::contains("version: '1.2.3'").eval(&content));
    }

    #[test]
    fn infers_globs_from_workspaces() {
        let sandbox = create_sandbox("init-sandbox");
        let root = sandbox.path().to_path_buf();
        let config = root.join(".moon").join("workspace.yml");

        fs::create_dir_all(root.join("packages").join("foo")).unwrap();
        fs::write(&root.join("packages").join("foo").join("README"), "Hello").unwrap();

        fs::create_dir_all(root.join("app")).unwrap();
        fs::write(&root.join("app").join("README"), "World").unwrap();

        fs::write(
            &root.join("package.json"),
            r#"{"workspaces": ["packages/*", "app"] }"#,
        )
        .unwrap();

        sandbox.run_moon(|cmd| {
            cmd.arg("init").arg("--yes").arg(root);
        });

        let content = fs::read_to_string(config).unwrap();

        assert!(predicate::str::contains("projects:\n  - 'app'").eval(&content));
    }

    #[test]
    fn infers_globs_from_workspaces_expanded() {
        let sandbox = create_sandbox("init-sandbox");
        let root = sandbox.path().to_path_buf();
        let config = root.join(".moon").join("workspace.yml");

        fs::create_dir_all(root.join("packages").join("bar")).unwrap();
        fs::write(&root.join("packages").join("bar").join("README"), "Hello").unwrap();

        fs::create_dir_all(root.join("app")).unwrap();
        fs::write(&root.join("app").join("README"), "World").unwrap();

        fs::write(
            &root.join("package.json"),
            r#"{"workspaces": { "packages": ["packages/*", "app"] }}"#,
        )
        .unwrap();

        sandbox.run_moon(|cmd| {
            cmd.arg("init").arg("--yes").arg(root);
        });

        let content = fs::read_to_string(config).unwrap();

        assert!(predicate::str::contains("projects:\n  - 'app'").eval(&content));
    }

    mod package_manager {
        use super::*;

        #[test]
        fn infers_npm() {
            let sandbox = create_sandbox("init-sandbox");
            let root = sandbox.path().to_path_buf();
            let config = root.join(".moon").join("toolchain.yml");

            fs::write(&root.join("package-lock.json"), "").unwrap();

            sandbox.run_moon(|cmd| {
                cmd.arg("init").arg("--yes").arg(root);
            });

            let content = fs::read_to_string(config).unwrap();

            assert!(predicate::str::contains("packageManager: 'npm'").eval(&content));
        }

        #[test]
        fn infers_npm_from_package() {
            let sandbox = create_sandbox("init-sandbox");
            let root = sandbox.path().to_path_buf();
            let config = root.join(".moon").join("toolchain.yml");

            fs::write(
                &root.join("package.json"),
                r#"{"packageManager":"npm@4.5.6"}"#,
            )
            .unwrap();

            sandbox.run_moon(|cmd| {
                cmd.arg("init").arg("--yes").arg(root);
            });

            let content = fs::read_to_string(config).unwrap();

            assert!(predicate::str::contains("packageManager: 'npm'").eval(&content));
            assert!(predicate::str::contains("npm:\n    version: '4.5.6'").eval(&content));
        }

        #[test]
        fn infers_pnpm() {
            let sandbox = create_sandbox("init-sandbox");
            let root = sandbox.path().to_path_buf();
            let config = root.join(".moon").join("toolchain.yml");

            fs::write(&root.join("pnpm-lock.yaml"), "").unwrap();

            sandbox.run_moon(|cmd| {
                cmd.arg("init").arg("--yes").arg(root);
            });

            let content = fs::read_to_string(config).unwrap();

            assert!(predicate::str::contains("packageManager: 'pnpm'").eval(&content));
        }

        #[test]
        fn infers_pnpm_from_package() {
            let sandbox = create_sandbox("init-sandbox");
            let root = sandbox.path().to_path_buf();
            let config = root.join(".moon").join("toolchain.yml");

            fs::write(
                &root.join("package.json"),
                r#"{"packageManager":"pnpm@4.5.6"}"#,
            )
            .unwrap();

            sandbox.run_moon(|cmd| {
                cmd.arg("init").arg("--yes").arg(root);
            });

            let content = fs::read_to_string(config).unwrap();

            assert!(predicate::str::contains("packageManager: 'pnpm'").eval(&content));
            assert!(predicate::str::contains("pnpm:\n    version: '4.5.6'").eval(&content));
        }

        #[test]
        fn infers_yarn() {
            let sandbox = create_sandbox("init-sandbox");
            let root = sandbox.path().to_path_buf();
            let config = root.join(".moon").join("toolchain.yml");

            fs::write(&root.join("yarn.lock"), "").unwrap();

            sandbox.run_moon(|cmd| {
                cmd.arg("init").arg("--yes").arg(root);
            });

            let content = fs::read_to_string(config).unwrap();

            assert!(predicate::str::contains("packageManager: 'yarn'").eval(&content));
        }

        #[test]
        fn infers_yarn_from_package() {
            let sandbox = create_sandbox("init-sandbox");
            let root = sandbox.path().to_path_buf();
            let config = root.join(".moon").join("toolchain.yml");

            fs::write(
                &root.join("package.json"),
                r#"{"packageManager":"yarn@4.5.6"}"#,
            )
            .unwrap();

            sandbox.run_moon(|cmd| {
                cmd.arg("init").arg("--yes").arg(root);
            });

            let content = fs::read_to_string(config).unwrap();

            assert!(predicate::str::contains("packageManager: 'yarn'").eval(&content));
            assert!(predicate::str::contains("yarn:\n    version: '4.5.6'").eval(&content));
        }
    }
}
