use std::fs;

use crate::{cli::GlobalArgs, files::get_config_file_path, AppError};
use serde::{Deserialize, Serialize};

/// Describes a file which should be linked based on the dotfiles config
/// Variables of this type contain a relative path, susch as `.config/someapp/conf`
pub type Link = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub mappings: Vec<Link>,
}

impl AppConfig {
    pub fn from_config_file(global_args: &GlobalArgs) -> Result<AppConfig, AppError> {
        let config_path = get_config_file_path(global_args)?;

        let config_file_content = fs::read_to_string(&config_path)
            .map_err(|err| AppError::ConfigOpen(config_path.clone(), err))?;

        let config: AppConfig = toml::from_str(&config_file_content)
            .map_err(|err| AppError::ConfigParse(config_path.clone(), err))?;

        Ok(config)
    }
}
