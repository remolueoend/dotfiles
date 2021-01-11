/// This module folder contains a sub-module for each sub-command callable via CLI.
/// Each command-module should provide the following objects:
/// 1. A clap sub-command definition in form of a `clap::App` instance which is used by the cli-module to register the command.
/// 2. The name of the command (eg. `CMD_IDENTIFIER`) which is used to register the command and match on the CLI arguments.
/// 3. Some kind of `run` function which accepts the sub-command arguments and global arguments passed via CLI
///    and executes the program of the command.
use crate::{cli::GlobalArgs, AppError};
use clap::ArgMatches;

pub mod add;
pub mod status;

pub type CommandResult = Result<(), AppError>;

/// runs the appropriate command based on the provided process arguments
pub fn run_command(cli_args: &ArgMatches) -> CommandResult {
    let global_args = GlobalArgs::from_cli_args(&cli_args)?;

    match cli_args.subcommand() {
        (status::CMD_IDENTIFIER, Some(cmd_args)) => status::run(cmd_args, &global_args),
        (add::CMD_IDENTIFIER, Some(cmd_args)) => add::run(cmd_args, &global_args),
        ("", _) => Err(AppError::CliMissingCommand),
        // should never be called thanks to `clap`s own validation:
        (cmd, _) => Err(AppError::CliInvalidCommand(cmd.to_string())),
    }
}
