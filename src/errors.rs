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
    /// an invlid CLI argument value was provided.
    /// Consists of the name of the argument and the reason why the value is invalid.
    CliInvalidArgValue(String, String),
    /// Failed to open the config file
    /// Consists of the requested path and the underlying IO error.
    ConfigOpen(PathBuf, std::io::Error),
    /// Failed to parse the config file
    /// Consists of the config file path and the underlying toml parse error
    ConfigParse(PathBuf, toml::de::Error),
    /// File system error: Could not find a user file system location, such as home or config directory
    /// Consists of the name of the location, such as `home directory` or `config directory`
    FsUserLocation(String),
    /// Failed to resolve the relative location of the user config directlry.
    /// TODO: remove this error and replace it with a user-friendlier version
    FsResolveConfig(StripPrefixError),
    /// An unspecified file system related error. Consists of a custom error message.
    FsOther(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::CliMissingCommand => {
                write!(f, "no command provided. Use dotfiles --help for more info")
            }
            AppError::CliInvalidCommand(cmd) => write!(f, "Invalid CLI command: {}", cmd),
            AppError::CliInvalidArgValue(arg, reason) => {
                write!(f, "the provided value for <{}> is invalid: {}", arg, reason)
            }
            AppError::ConfigOpen(path, err) => {
                write!(
                    f,
                    "Could not find dotfiles config file at {:?}: {}",
                    path, err
                )
            }
            AppError::ConfigParse(path, err) => {
                write!(f, "Failed to parse config file at {:?}: {}", path, err)
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
        }
    }
}
