use moon_codeowners::CodeownersGenerator;
use moon_config::{ProjectConfig, VcsProvider, WorkspaceConfig};
use starbase_sandbox::{assert_snapshot, create_empty_sandbox, locate_fixture, Sandbox};
use std::fs;

fn load_generator(provider: VcsProvider) -> Sandbox {
    let sandbox = create_empty_sandbox();
    let mut generator = CodeownersGenerator::new(sandbox.path(), provider).unwrap();

    let workspace_config = WorkspaceConfig::load(
        sandbox.path(),
        locate_fixture("workspace").join("workspace.yml"),
    )
    .unwrap();

    generator
        .add_workspace_entries(&workspace_config.codeowners)
        .unwrap();

    for project_fixture in ["custom-groups", "list-paths", "map-paths"] {
        let project_config = ProjectConfig::load(
            sandbox.path(),
            locate_fixture(project_fixture).join("moon.yml"),
        )
        .unwrap();

        generator
            .add_project_entry(project_fixture, project_fixture, &project_config.owners)
            .unwrap();
    }

    generator.generate().unwrap();

    sandbox
}

#[test]
fn generates_bitbucket() {
    let sandbox = load_generator(VcsProvider::Bitbucket);

    assert_snapshot!(fs::read_to_string(sandbox.path().join("CODEOWNERS")).unwrap());
}

#[test]
fn generates_github() {
    let sandbox = load_generator(VcsProvider::GitHub);

    assert_snapshot!(fs::read_to_string(sandbox.path().join(".github/CODEOWNERS")).unwrap());
}

#[test]
fn generates_gitlab() {
    let sandbox = load_generator(VcsProvider::GitLab);

    assert_snapshot!(fs::read_to_string(sandbox.path().join(".gitlab/CODEOWNERS")).unwrap());
}

#[test]
fn generates_other() {
    let sandbox = load_generator(VcsProvider::Other);

    assert_snapshot!(fs::read_to_string(sandbox.path().join("CODEOWNERS")).unwrap());
}
