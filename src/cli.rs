use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, SubCommand};

use crate::AppError;

const ARG_DOTFILES_ROOT: &str = "dotfiles-root";
pub const CMD_STATUS: &str = "status";

/// returns a new clap APP CLI interface used for this app
pub fn build_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("dotfiles")
        .version("0.1")
        .author("remolueoend")
        .about("Simple dotfiles manager keeping track of file links")
        .arg(
            Arg::with_name(ARG_DOTFILES_ROOT)
                .short("r")
                .required(true)
                .takes_value(true)
                .help("the absolute path of the dotfiles repository root directory")
                .env("DOTFILES_ROOT"),
        )
        .subcommand(
            SubCommand::with_name(CMD_STATUS).about("shows the current status of all dotfiles"),
        )
}

/// Contains all global cli options which are independent of the chosen sub-command
pub struct GlobalArgs {
    pub dotfiles_root: PathBuf,
}
impl<'a> GlobalArgs {
    /// returns a new global options struct based on the parsed CLI arguments
    pub fn from_cli_args(arg_matches: &'a ArgMatches) -> Result<GlobalArgs, AppError> {
        // unwrap is OK here, this attributes are marked as required:
        let dotfiles_root = arg_matches.value_of(ARG_DOTFILES_ROOT).unwrap();

        let dotfiles_root_path = PathBuf::from(dotfiles_root);
        if !dotfiles_root_path.is_dir() {
            return Err(AppError::CliInvalidArgValue(
                String::from(ARG_DOTFILES_ROOT),
                format!("{} is not a valid directory", dotfiles_root),
            ));
        }

        Ok(GlobalArgs {
            dotfiles_root: PathBuf::from(dotfiles_root),
        })
    }
}