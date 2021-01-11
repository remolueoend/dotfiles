use crate::{cli::GlobalArgs, AppError};
use dirs::{config_dir, home_dir};

use std::{env::current_dir, os::unix::fs, path::PathBuf};

/// returns the home directory of the current user
pub fn get_home_dir() -> Result<PathBuf, AppError> {
    home_dir().ok_or(AppError::FsUserLocation("home directory".to_string()))
}

/// returns the current working directory or an AppError if something went wrong.
pub fn get_cwd() -> Result<PathBuf, AppError> {
    current_dir().map_err(|_| AppError::FsUserLocation("current directory".to_string()))
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

pub fn create_symlink_for(from: &PathBuf, to: &PathBuf) -> Result<(), AppError> {
    fs::symlink(to, from).map_err(|err| {
        AppError::FsOther(format!(
            "Could not create a symlink {} -> {}: {}",
            from.clone().display(),
            to.clone().display(),
            err
        ))
    })
}

/// returns a canonicalized paths of the two given paths joined together.
/// The joined path must exists.
/// This method does *not* resolve symlinks.
pub fn normalize_paths(p1: &PathBuf, p2: &PathBuf) -> Result<PathBuf, AppError> {
    match p2.parent() {
        None => Ok(p1.join(p2)),
        Some(parent) => {
            let first_part = p1.join(parent).canonicalize().map_err(|err| {
                AppError::FsOther(format!(
                    "Could not canonicalize path {}: {}",
                    p1.join(parent).display(),
                    err
                ))
            })?;
            // should be safe
            let file_name = p2.file_name().unwrap();
            Ok(first_part.join(file_name))
        }
    }
}
