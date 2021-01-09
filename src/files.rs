use crate::{cli::GlobalArgs, AppError};
use dirs::{config_dir, home_dir};
use std::path::PathBuf;

/// returns the home directory of the current user
pub fn get_home_dir() -> Result<PathBuf, AppError> {
    home_dir().ok_or(AppError::FsUserLocation(String::from("home directory")))
}

/// Returns the PathBuf of the dotfiles configuration file in the dotfiles repository.
/// This means that the dotfiles config itself does not have to be linked, but is fetched from the dotfiles repo itself.
/// The path is resolved the following way:
/// DOTFILES: path of dotfiles repository
/// CONFIG:   relative path to user config from home directory, in most cases: `.config`
/// config file path is resolved as: DOTFILES/CONFIG/dotfiles/config.toml
pub fn get_config_file_path(global_args: &GlobalArgs) -> Result<PathBuf, AppError> {
    let home = get_home_dir()?;
    let config = config_dir().ok_or(AppError::FsUserLocation(String::from("config directory")))?;

    // the relative path of the user config dir (~/.config) from the home directory (=> '.config')
    let rel_config = config
        .strip_prefix(home)
        .map_err(|err| AppError::FsResolveConfig(err))?;

    let config_file_path = global_args
        .dotfiles_root
        .join(rel_config)
        .join("dotfiles/config.toml");

    Ok(config_file_path)
}

