use crate::{cli::GlobalArgs, files::get_config_file_path, AppError};
use promptly::prompt_default;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    fs,
    path::{Component, PathBuf},
};

/// Custom serde deserializer for mappings in the config file.
/// Makes sure that all paths do not contain a leading current directory by removing the leading dot:
/// `./.config => .config`.
/// This is important for comparing paths with each other, because the default compare implementation
/// of PathBuf returns `false` for `Path::from("./.config") == Path::from(".config")`.
fn into_normalized_mapping<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let input: Vec<PathBuf> = Deserialize::deserialize(deserializer)?;
    let mut output = vec![];
    for path in input {
        let normalized_path = if path.starts_with(Component::CurDir) {
            path.strip_prefix(Component::CurDir).unwrap().to_owned()
        } else {
            path
        };
        output.push(normalized_path);
    }

    Ok(output)
}

/// Describes a file which should be linked based on the dotfiles config
/// Variables of this type contain a relative path, such as `.config/some/conf`
pub type Link = PathBuf;

/// Describes the parsed configuration from the dotfiles configuration file.
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub config_version: i8, // we can increase it at anytime when necessary..
    #[serde(deserialize_with = "into_normalized_mapping")]
    pub mappings: Vec<Link>,
}

impl AppConfig {
    pub fn from_config_file(global_args: &GlobalArgs) -> Result<AppConfig, AppError> {
        let config_path = get_config_file_path(global_args)?;

        // if the config does not exist yet: ask the user to create it:
        if config_path.exists() == false {
            let should_create = prompt_default(
                format!(
                    "Could not find the dotfiles config file at {:?}. Should I create it?",
                    config_path
                ),
                true,
            )
            .unwrap_or(false);

            if should_create {
                (AppConfig {
                    config_version: 1,
                    mappings: vec![],
                })
                .to_config_file(global_args)?;
            }
        }

        let config_file_content = fs::read_to_string(&config_path)
            .map_err(|err| AppError::ConfigFileRead(config_path.clone(), err))?;

        let config: AppConfig = toml::from_str(&config_file_content)
            .map_err(|err| AppError::ConfigParse(config_path.clone(), err))?;

        config.validate_absolute_mappings()?;
        config.validate_nested_mappings()?;

        Ok(config)
    }

    /// Writes this configuration to the dotfiles configuration file by either overwriting the current content
    /// or creating the file if it does not yet exist.
    pub fn to_config_file(&self, global_args: &GlobalArgs) -> Result<(), AppError> {
        let serialized_config =
            toml::to_string_pretty(&self).map_err(|err| AppError::ConfigSerialize(err))?;
        let config_path = get_config_file_path(global_args)?;

        fs::create_dir_all(config_path.parent().unwrap())
            .map_err(|err| AppError::ConfigFileWrite(config_path.clone(), err))?;

        fs::write(&config_path, serialized_config)
            .map_err(|err| AppError::ConfigFileWrite(config_path.clone(), err))
    }

    /// makes sure all link in mappings are relative and returns an error if an absolute path was found
    fn validate_absolute_mappings(&self) -> Result<(), AppError> {
        for link in &self.mappings {
            if link.is_absolute() {
                return Err(AppError::ConfigAbsoluteLink(link.to_owned()));
            }
        }

        Ok(())
    }

    /// validates that there are not nested links, ie. a directory to link containing a file to link.
    /// Otherwise, returns an error
    fn validate_nested_mappings(&self) -> Result<(), AppError> {
        if self.mappings.len() == 0 {
            return Ok(());
        }
        let mut mappings = self.mappings.to_vec();
        mappings.sort();
        for i in 0..mappings.len() - 1 {
            let current = &mappings[i];
            let next = &mappings[i + 1];
            if next.starts_with(current) {
                return Err(AppError::ConfigNestedLinks(next.clone(), current.clone()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;
    use crate::errors::AppError;
    use std::path::PathBuf;

    #[test]
    fn validate_nested_paths_detects_nested_paths() {
        let config = AppConfig {
            config_version: 1,
            mappings: vec![
                PathBuf::from(".config/some-other-dir"),
                PathBuf::from(".config/some-dir/some-file"),
                PathBuf::from(".config/some-dir"),
            ],
        };

        let result = config.validate_nested_mappings();

        assert!(result.is_err(), "did not detect nested paths");
        if let Err(AppError::ConfigNestedLinks(nested, parent)) = result {
            assert_eq!(nested, PathBuf::from(".config/some-dir/some-file"));
            assert_eq!(parent, PathBuf::from(".config/some-dir"));
        };
    }
}
