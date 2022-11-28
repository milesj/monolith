use moon_test_utils::{assert_snapshot, create_sandbox, get_assert_output};
use predicates::prelude::*;
use std::fs;

fn get_path_safe_output(assert: &assert_cmd::assert::Assert) -> String {
    get_assert_output(assert).replace('\\', "/")
}

#[test]
fn creates_a_new_template() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate").arg("new-name").arg("--template");
    });

    let output = get_path_safe_output(&assert);

    assert!(predicate::str::contains("Created a new template new-name at").eval(&output));
    assert!(sandbox.path().join("templates/new-name").exists());

    assert.success();
}

#[test]
fn generates_files_from_template() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate").arg("standard").arg("./test");
    });

    assert_snapshot!(get_path_safe_output(&assert));

    assert!(sandbox.path().join("test").exists());
    assert!(sandbox.path().join("test/file.ts").exists());
    assert!(sandbox.path().join("test/folder/nested-file.ts").exists());
    assert!(!sandbox.path().join("test/template.yml").exists());
}

#[test]
fn doesnt_generate_files_when_dryrun() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("standard")
            .arg("./test")
            .arg("--dryRun");
    });

    assert_snapshot!(get_path_safe_output(&assert));

    assert!(!sandbox.path().join("test").exists());
    assert!(!sandbox.path().join("test/file.ts").exists());
    assert!(!sandbox.path().join("test/folder/nested-file.ts").exists());
    assert!(!sandbox.path().join("test/template.yml").exists());
}

#[test]
fn overwrites_existing_files_when_forced() {
    let sandbox = create_sandbox("generator");

    sandbox.run_moon(|cmd| {
        cmd.arg("generate").arg("standard").arg("./test");
    });

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("standard")
            .arg("./test")
            .arg("--force");
    });

    assert_snapshot!(get_path_safe_output(&assert));

    assert!(sandbox.path().join("test").exists());
    assert!(sandbox.path().join("test/file.ts").exists());
    assert!(sandbox.path().join("test/folder/nested-file.ts").exists());
    assert!(!sandbox.path().join("test/template.yml").exists());
}

#[test]
fn overwrites_existing_files_when_interpolated_path() {
    let sandbox = create_sandbox("generator");

    sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("vars")
            .arg("./test")
            .arg("--defaults");
    });

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("vars")
            .arg("./test")
            .arg("--defaults")
            .arg("--force");
    });

    assert_snapshot!(get_path_safe_output(&assert));

    // file-[stringNotEmpty]-[number].txt
    assert!(sandbox.path().join("./test/file-default-0.txt").exists());
}

#[test]
fn renders_and_interpolates_templates() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("vars")
            .arg("./test")
            .arg("--defaults");
    });

    assert.success();

    assert_snapshot!(fs::read_to_string(sandbox.path().join("./test/expressions.txt")).unwrap());
    assert_snapshot!(fs::read_to_string(sandbox.path().join("./test/control.txt")).unwrap());
}

#[test]
fn renders_with_custom_vars_via_args() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("vars")
            .arg("./test")
            .arg("--defaults")
            .arg("--")
            .args([
                "--no-boolTrue",
                "--boolFalse",
                "--string=abc",
                "--stringNotEmpty",
                "xyz",
                "--number=123",
                "--numberNotEmpty",
                "456",
                "--enum=c",
                "--multenumNotEmpty",
                "a",
            ]);
    });

    assert.success();

    assert_snapshot!(fs::read_to_string(sandbox.path().join("./test/expressions.txt")).unwrap());
    assert_snapshot!(fs::read_to_string(sandbox.path().join("./test/control.txt")).unwrap());
}

#[test]
fn interpolates_destination_path() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("vars")
            .arg("./test")
            .arg("--defaults");
    });

    // Verify output paths are correct
    assert_snapshot!(get_path_safe_output(&assert));

    // file-[stringNotEmpty]-[number].txt
    assert!(sandbox.path().join("./test/file-default-0.txt").exists());
}

#[test]
fn errors_when_parsing_custom_var_types() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("vars")
            .arg("./test")
            .arg("--defaults")
            .arg("--")
            .arg("--number=abc");
    });

    assert_snapshot!(get_path_safe_output(&assert));
}

#[test]
fn supports_custom_filters() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("vars")
            .arg("./test")
            .arg("--defaults");
    });

    assert.success();

    assert_snapshot!(fs::read_to_string(sandbox.path().join("./test/filters.txt")).unwrap());
}

#[test]
fn supports_tera_twig_exts() {
    let sandbox = create_sandbox("generator");

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("generate")
            .arg("extensions")
            .arg("./test")
            .arg("--defaults");
    });

    assert.success();

    let tera = sandbox.path().join("./test/file.ts");
    let twig = sandbox.path().join("./test/file.tsx");

    assert!(tera.exists());
    assert!(twig.exists());

    assert_eq!(
        fs::read_to_string(tera).unwrap(),
        "export type FooBar = true;\n"
    );
    assert_eq!(
        fs::read_to_string(twig).unwrap(),
        "export type FooBar = true;\n"
    );
}

mod frontmatter {
    use super::*;

    #[test]
    fn changes_dest_path() {
        let sandbox = create_sandbox("generator");

        let assert = sandbox.run_moon(|cmd| {
            cmd.arg("generate")
                .arg("frontmatter")
                .arg("./test")
                .arg("--defaults");
        });

        assert.success();

        assert!(!sandbox.path().join("./test/to.txt").exists());
        assert!(sandbox.path().join("./test/to-NEW.txt").exists());
        assert_snapshot!(fs::read_to_string(sandbox.path().join("./test/to-NEW.txt")).unwrap());
    }

    #[test]
    fn force_writes_file() {
        let sandbox = create_sandbox("generator");

        fs::create_dir_all(sandbox.path().join("test")).unwrap();
        fs::write(sandbox.path().join("test/forced.txt"), "Original content").unwrap();

        let assert = sandbox.run_moon(|cmd| {
            cmd.arg("generate")
                .arg("frontmatter")
                .arg("./test")
                .arg("--defaults");
        });

        assert.success();

        assert_snapshot!(fs::read_to_string(sandbox.path().join("./test/forced.txt")).unwrap());
    }

    #[test]
    fn skips_over_file() {
        let sandbox = create_sandbox("generator");

        let assert = sandbox.run_moon(|cmd| {
            cmd.arg("generate")
                .arg("frontmatter")
                .arg("./test")
                .arg("--defaults");
        });

        assert.success();

        assert!(!sandbox.path().join("./test/skipped.txt").exists());
    }

    #[test]
    fn supports_component_vars() {
        let sandbox = create_sandbox("generator");

        let assert = sandbox.run_moon(|cmd| {
            cmd.arg("generate")
                .arg("frontmatter")
                .arg("./test")
                .arg("--defaults");
        });

        assert.success();

        assert!(sandbox
            .path()
            .join("./test/components/SmallButton.tsx")
            .exists());
        assert_snapshot!(fs::read_to_string(
            sandbox.path().join("./test/components/SmallButton.tsx")
        )
        .unwrap());
    }
}
