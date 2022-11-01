use moon_config::{ConfigError, TemplateConfig};
use moon_constants::CONFIG_TEMPLATE_FILENAME;
use std::path::PathBuf;

fn load_jailed_config() -> Result<TemplateConfig, figment::Error> {
    match TemplateConfig::load(&PathBuf::from(CONFIG_TEMPLATE_FILENAME)) {
        Ok(cfg) => Ok(cfg),
        Err(error) => Err(match error {
            ConfigError::FailedValidation(errors) => errors.first().unwrap().to_owned(),
            ConfigError::Figment(f) => f,
            e => figment::Error::from(e.to_string()),
        }),
    }
}

mod title {
    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"template.title\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_TEMPLATE_FILENAME, "title: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(expected = "Must be a non-empty string for key \"template.title\"")]
    fn min_length() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_TEMPLATE_FILENAME,
                "title: ''\ndescription: 'asd'",
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod description {
    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"template.description\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_TEMPLATE_FILENAME, "description: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(expected = "Must be a non-empty string for key \"template.description\"")]
    fn min_length() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_TEMPLATE_FILENAME,
                "title: 'asd'\ndescription: ''",
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}
