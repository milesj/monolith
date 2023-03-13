use moon_test_utils::{
    create_sandbox_with_config, get_cases_fixture_configs, predicates::prelude::*,
};

#[test]
fn valid_tool() {
    let (workspace_config, toolchain_config, tasks_config) = get_cases_fixture_configs();
    let sandbox = create_sandbox_with_config(
        "cases",
        Some(&workspace_config),
        Some(&toolchain_config),
        Some(&tasks_config),
    );

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("bin").arg("node");
    });

    assert
        .success()
        .code(0)
        .stdout(predicate::str::contains("18.0.0"));
}

#[test]
fn invalid_tool() {
    let (workspace_config, toolchain_config, tasks_config) = get_cases_fixture_configs();
    let sandbox = create_sandbox_with_config(
        "cases",
        Some(&workspace_config),
        Some(&toolchain_config),
        Some(&tasks_config),
    );

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("bin").arg("unknown");
    });

    assert
        .failure()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains(
            "invalid value 'unknown' for '<TOOL>'",
        ));
}

// We use a different Node.js version as to not conflict with other tests!

#[test]
fn not_configured() {
    let (workspace_config, toolchain_config, tasks_config) = get_cases_fixture_configs();
    let sandbox = create_sandbox_with_config(
        "cases",
        Some(&workspace_config),
        Some(&toolchain_config),
        Some(&tasks_config),
    );

    let assert = sandbox.run_moon(|cmd| {
        cmd.arg("bin")
            .arg("yarn")
            .env("MOON_NODE_VERSION", "17.1.0");
    });

    assert.failure().code(1).stdout("");
}

// #[test]
// fn not_installed() {
//     let fixture = create_sandbox("cases");

//     let assert = create_moon_command(fixture.path())
//         .arg("bin")
//         .arg("node")
//         .env("MOON_NODE_VERSION", "17.1.0")
//         .assert();

//     assert.failure().code(2).stdout("");
// }
