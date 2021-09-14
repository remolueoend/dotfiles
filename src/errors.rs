use crate::commands;
use std::{
    fmt::{Debug, Display},
    path::{PathBuf, StripPrefixError},
    write,
};

/// All `Result`s of functions in this binary should return this error type.
#[derive(Debug)]
pub enum AppError {
    /// no sub-command was provided via CLI arguments
    CliMissingCommand,
    /// an invalid sub-command was provided via CLI arguments
    /// Consists of the name of the invalid command
    CliInvalidCommand(String),
    /// an invalid CLI argument value was provided.
    /// Consists of the name of the argument and the reason why the value is invalid.
    CliInvalidArgValue(String, String),
    /// Failed to read the config file
    /// Consists of the requested path and the underlying IO error.
    ConfigFileRead(PathBuf, std::io::Error),
    /// Failed to write the config file
    /// Consists of the requested path and the underlying IO error.
    ConfigFileWrite(PathBuf, std::io::Error),
    /// Failed to parse the config file
    /// Consists of the config file path and the underlying toml parse error
    ConfigParse(PathBuf, toml::de::Error),
    /// Failed to serialize the config
    /// Consists of the underlying toml parse error
    ConfigSerialize(toml::ser::Error),
    /// The configuration contains nested link entries, which is not supported
    /// Consists of the nested and parent paths
    ConfigNestedLinks(PathBuf, PathBuf),
    /// Found an absolute path in the mappings, which is not valid.
    /// Consists of the found absolute path.
    ConfigAbsoluteLink(PathBuf),
    /// File system error: Could not find a user file system location, such as home or config directory
    /// Consists of the name of the location, such as `home directory` or `config directory`
    FsUserLocation(String),
    /// Failed to resolve the relative location of the user config directory.
    /// TODO: remove this error and replace it with a user-friendlier version
    FsResolveConfig(StripPrefixError),
    /// An unspecified file system related error. Consists of a custom error message.
    FsOther(String),
    /// An error specific to the `add` sub-command occurred.
    /// Consists of the error itself.
    CmdAddError(commands::add::Error),
    NotImplemented,
}

impl Display for AppError {
    // TODO: most of these error messages are missing a clear action for the user
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::CliMissingCommand => {
                write!(f, "no command provided. Use dotfiles --help for more info")
            }
            AppError::CliInvalidCommand(cmd) => write!(f, "Invalid CLI command: {}", cmd),
            AppError::CliInvalidArgValue(arg, reason) => {
                write!(f, "the provided value for <{}> is invalid: {}", arg, reason)
            }
            AppError::ConfigFileRead(path, err) => {
                write!(
                    f,
                    "Could not read dotfiles config file at {:?}: {}",
                    path, err
                )
            }
            AppError::ConfigFileWrite(path, err) => {
                write!(
                    f,
                    "Could not write dotfiles config file at {:?}: {}",
                    path, err
                )
            }
            AppError::ConfigParse(path, err) => {
                write!(f, "Failed to parse config file at {:?}: {}", path, err)
            }
            AppError::ConfigSerialize(err) => {
                write!(f, "Failed to serialize config : {}", err)
            }
            AppError::ConfigNestedLinks(nested, parent) => {
                write!(
                    f,
                    "Invalid mappings in config: The mappings entry {:?} is nested in the entry {:?}",
                    nested, parent
                )
            }
            AppError::ConfigAbsoluteLink(link) => {
                write!(f, "found an absolute path in the configured mappings: {:?}. This is not allowed. Mappings should be relative to the root of your dotfiles repository.", link)
            }
            AppError::FsUserLocation(location) => {
                write!(f, "Could not find location: {}", location)
            }
            AppError::FsResolveConfig(err) => {
                write!(f, "Could not resolve user config directory: {}", err)
            }
            AppError::FsOther(message) => {
                write!(f, "A file system error occurred: {}", message)
            }
            AppError::CmdAddError(err) => {
                write!(f, "{}", err)
            }
            AppError::NotImplemented => {
                write!(f, "Not implemented")
            }
        }
    }
}
