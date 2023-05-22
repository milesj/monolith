use moon_config::{
    ConfigError, DependencyConfig, DependencyScope, ProjectConfig, ProjectDependsOn,
    TaskCommandArgs, TaskConfig,
};
use moon_constants::CONFIG_PROJECT_FILENAME;
use moon_utils::string_vec;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn load_jailed_config() -> Result<ProjectConfig, figment::Error> {
    match ProjectConfig::load(PathBuf::from(CONFIG_PROJECT_FILENAME)) {
        Ok(cfg) => Ok(cfg),
        Err(error) => Err(match error {
            ConfigError::FailedValidation(errors) => errors.first().unwrap().to_owned(),
            ConfigError::Figment(f) => f,
            e => figment::Error::from(e.to_string()),
        }),
    }
}

#[test]
fn empty_file() {
    figment::Jail::expect_with(|jail| {
        // Needs a fake yaml value, otherwise the file reading panics
        jail.create_file(CONFIG_PROJECT_FILENAME, "fake: value")?;

        load_jailed_config()?;

        Ok(())
    });
}

#[test]
fn loads_defaults() {
    figment::Jail::expect_with(|jail| {
        jail.create_file(
            CONFIG_PROJECT_FILENAME,
            r#"
fileGroups:
    sources:
        - src/**/*"#,
        )?;

        let config = load_jailed_config()?;

        assert_eq!(
            config,
            ProjectConfig {
                file_groups: FxHashMap::from_iter([("sources".into(), string_vec!["src/**/*"])]),
                ..ProjectConfig::default()
            }
        );

        Ok(())
    });
}

mod depends_on {
    use super::*;

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a sequence for key \"project.dependsOn\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "dependsOn: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "expected a project name or dependency config object for key \"project.dependsOn.0\""
    )]
    fn invalid_object_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"dependsOn:
  - id: 'a'
    scope: 'invalid'"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    fn supports_list_of_strings() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "dependsOn: ['a', 'b', 'c']")?;

            let cfg: ProjectConfig = super::load_jailed_config()?;

            assert_eq!(
                cfg.depends_on,
                vec![
                    ProjectDependsOn::String("a".into()),
                    ProjectDependsOn::String("b".into()),
                    ProjectDependsOn::String("c".into())
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn supports_list_of_objects() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"dependsOn:
  - id: 'a'
    scope: 'development'
  - id: 'b'
    scope: 'production'"#,
            )?;

            let cfg: ProjectConfig = super::load_jailed_config()?;

            assert_eq!(
                cfg.depends_on,
                vec![
                    ProjectDependsOn::Object(DependencyConfig {
                        id: "a".into(),
                        scope: DependencyScope::Development,
                        via: None,
                    }),
                    ProjectDependsOn::Object(DependencyConfig {
                        id: "b".into(),
                        scope: DependencyScope::Production,
                        via: None,
                    })
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn supports_list_of_strings_and_objects() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"dependsOn:
  - 'a'
  - id: 'b'
    scope: 'production'"#,
            )?;

            let cfg: ProjectConfig = super::load_jailed_config()?;

            assert_eq!(
                cfg.depends_on,
                vec![
                    ProjectDependsOn::String("a".into()),
                    ProjectDependsOn::Object(DependencyConfig {
                        id: "b".into(),
                        scope: DependencyScope::Production,
                        via: None,
                    })
                ]
            );

            Ok(())
        });
    }
}

mod file_groups {
    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a map for key \"project.fileGroups\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "fileGroups: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a sequence for key \"project.fileGroups.sources\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
fileGroups:
    sources: 123"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod tasks {
    use moon_common::Id;

    use super::*;

    // TODO: https://github.com/SergioBenitez/Figment/issues/41
    #[test]
    fn loads_defaults() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                CONFIG_PROJECT_FILENAME,
                r#"
tasks:
    lint:
        command: eslint
        args:
            - ."#,
            )?;

            let config = load_jailed_config()?;

            assert_eq!(
                config,
                ProjectConfig {
                    tasks: BTreeMap::from([(
                        "lint".into(),
                        TaskConfig {
                            command: Some(TaskCommandArgs::String("eslint".to_owned())),
                            args: Some(TaskCommandArgs::Sequence(vec![".".to_owned()])),
                            ..TaskConfig::default()
                        }
                    )]),
                    ..ProjectConfig::default()
                }
            );

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a map for key \"project.tasks\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "tasks: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected struct TaskConfig for key \"project.tasks.test\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
tasks:
    test: 123"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "expected a string or a sequence of strings for key \"project.tasks.test.command\""
    )]
    fn invalid_value_field() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
fileGroups: {}
tasks:
    test:
        command: 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "An npm/system command is required for key \"project.tasks.test.command\""
    )]
    fn invalid_value_empty_field() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
fileGroups: {}
tasks:
    test:
        command: ''
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    fn can_use_references() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
tasks:
    build: &webpack
        command: 'webpack'
        inputs:
            - 'src/**/*'
    start:
        <<: *webpack
        args: 'serve'
"#,
            )?;

            let config: ProjectConfig = super::load_jailed_config()?; // jail.directory())?;

            assert_eq!(
                config.tasks.get("build").unwrap(),
                &TaskConfig {
                    command: Some(TaskCommandArgs::String("webpack".to_owned())),
                    inputs: Some(string_vec!["src/**/*"]),
                    ..TaskConfig::default()
                }
            );

            assert_eq!(
                config.tasks.get("start").unwrap(),
                &TaskConfig {
                    command: Some(TaskCommandArgs::String("webpack".to_owned())),
                    args: Some(TaskCommandArgs::String("serve".to_owned())),
                    inputs: Some(string_vec!["src/**/*"]),
                    ..TaskConfig::default()
                }
            );

            Ok(())
        });
    }

    #[test]
    fn can_use_references_from_root() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
_webpack: &webpack
    command: 'webpack'
    inputs:
        - 'src/**/*'

tasks:
    build: *webpack
    start:
        <<: *webpack
        args: 'serve'
"#,
            )?;

            let config: ProjectConfig = super::load_jailed_config()?;

            assert_eq!(
                config.tasks.get("build").unwrap(),
                &TaskConfig {
                    command: Some(TaskCommandArgs::String("webpack".to_owned())),
                    inputs: Some(string_vec!["src/**/*"]),
                    ..TaskConfig::default()
                }
            );

            assert_eq!(
                config.tasks.get("start").unwrap(),
                &TaskConfig {
                    command: Some(TaskCommandArgs::String("webpack".to_owned())),
                    args: Some(TaskCommandArgs::String("serve".to_owned())),
                    inputs: Some(string_vec!["src/**/*"]),
                    ..TaskConfig::default()
                }
            );

            Ok(())
        });
    }

    #[test]
    fn supports_name_patterns() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
tasks:
    normal:
      command: 'a'
    kebab-case:
      command: 'b'
    camelCase:
      command: 'c'
    snake_case:
      command: 'd'
    dot.case:
      command: 'e'
    slash/case:
      command: 'f'
"#,
            )?;

            let config: ProjectConfig = super::load_jailed_config()?;

            assert!(config.tasks.contains_key(&Id::raw("normal")));
            assert!(config.tasks.contains_key(&Id::raw("kebab-case")));
            assert!(config.tasks.contains_key(&Id::raw("camelCase")));
            assert!(config.tasks.contains_key(&Id::raw("snake_case")));
            assert!(config.tasks.contains_key(&Id::raw("dot.case")));
            assert!(config.tasks.contains_key(&Id::raw("slash/case")));

            Ok(())
        });
    }
}

mod project {
    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected struct ProjectMetadataConfig for key \"project.project\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "project: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"project.project.name\""
    )]
    fn invalid_name_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
project:
    name: 123
    description: ''
    owner: ''
    maintainers: []
    channel: ''"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found bool true, expected a string for key \"project.project.description\""
    )]
    fn invalid_description_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
project:
    name: ''
    description: true
    owner: ''
    maintainers: []
    channel: ''"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found map, expected a string for key \"project.project.owner\""
    )]
    fn invalid_owner_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
project:
    name: ''
    description: ''
    owner: {}
    maintainers: []
    channel: ''"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found string \"abc\", expected a sequence for key \"project.project.maintainers\""
    )]
    fn invalid_maintainers_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
project:
    name: ''
    description: ''
    owner: ''
    maintainers: 'abc'
    channel: ''"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"project.project.channel\""
    )]
    fn invalid_channel_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
project:
    name: ''
    description: ''
    owner: ''
    maintainers: []
    channel: 123"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(expected = "Must start with a `#` for key \"project.project.channel\"")]
    fn channel_leading_hash() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
project:
    name: ''
    description: ''
    owner: ''
    maintainers: []
    channel: name"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod workspace {
    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected struct ProjectWorkspaceConfig for key \"project.workspace\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "workspace: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected struct ProjectWorkspaceInheritedTasksConfig for key \"project.workspace.inheritedTasks\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
workspace:
    inheritedTasks: 123"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found string \"abc\", expected a sequence for key \"project.workspace.inheritedTasks.include\""
    )]
    fn invalid_nested_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
workspace:
    inheritedTasks:
        include: abc"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod language {
    use moon_config::ProjectLanguage;

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"project.language\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "language: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    fn supported_lang() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "language: javascript")?;

            let config = super::load_jailed_config()?;

            assert_eq!(config.language, ProjectLanguage::JavaScript);

            Ok(())
        });
    }

    #[test]
    fn other_lang() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "language: dotnet")?;

            let config = super::load_jailed_config()?;

            assert_eq!(config.language, ProjectLanguage::Other("dotnet".into()));

            Ok(())
        });
    }

    #[test]
    fn formats_other_lang() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "language: 'Dot Net'")?;

            let config = super::load_jailed_config()?;

            assert_eq!(config.language, ProjectLanguage::Other("dot-net".into()));

            Ok(())
        });
    }
}

mod tags {
    use moon_utils::string_vec;

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a sequence for key \"project.tags\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "tags: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "Invalid identifier foo bar. May only contain alpha-numeric characters, dashes (-), slashes (/), underscores (_), and dots (.)"
    )]
    fn invalid_format() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_PROJECT_FILENAME, "tags: ['foo bar']")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    fn valid_tags() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_PROJECT_FILENAME,
                r#"
tags:
    - normal
    - camelCase
    - kebab-case
    - snake_case
    - dot.case
    - slash/case
"#,
            )?;

            let config = super::load_jailed_config()?;

            assert_eq!(
                config.tags,
                string_vec![
                    "normal",
                    "camelCase",
                    "kebab-case",
                    "snake_case",
                    "dot.case",
                    "slash/case"
                ]
            );

            Ok(())
        });
    }
}
