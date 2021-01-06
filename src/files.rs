use crate::{cli::GlobalArgs, config::Link, AppError};
use dirs::{config_dir, home_dir};
use std::{fs::read_link, path::PathBuf};

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

/// Describes the status of a link configured in mappings
pub enum LinkStatus {
    /// file does not exist in the dotfiles repository
    Invalid(PathBuf),
    /// symlink found and pointing to correct target in dotfiles repository
    Linked,
    /// file is currently not linked
    Unlinked,
    /// symlink found but pointing to another target
    ConflictWrongTarget(PathBuf),
    /// file/directory found, but not a symlink
    ConflictNoLink(PathBuf),
}

/// Returns the link status for the given target.
/// The given target should be a resolved path from the configured mappings
pub fn get_file_status(global_args: &GlobalArgs, link: &Link) -> Result<LinkStatus, AppError> {
    // path to the symlink at the target location
    let actual_file_path = get_home_dir()?.join(link);
    // path to the file in the dotfiles repository
    let expected_target = global_args.dotfiles_root.join(link);

    if expected_target.exists() == false {
        return Ok(LinkStatus::Invalid(expected_target));
    }

    if actual_file_path.exists() == false {
        return Ok(LinkStatus::Unlinked);
    };

    let actual_file_meta = actual_file_path.symlink_metadata().map_err(|err| {
        AppError::FsOther(format!(
            "failed to read file metadata at {:?}: {}",
            actual_file_path.clone(),
            err
        ))
    })?;

    if actual_file_meta.file_type().is_symlink() == false {
        return Ok(LinkStatus::ConflictNoLink(actual_file_path));
    };

    let actual_target = read_link(&actual_file_path).map_err(|err| {
        AppError::FsOther(format!(
            "failed to read symlink at {:?}: {}",
            actual_file_path.clone(),
            err
        ))
    })?;

    if actual_target != expected_target {
        Ok(LinkStatus::ConflictWrongTarget(actual_target))
    } else {
        Ok(LinkStatus::Linked)
    }
}
