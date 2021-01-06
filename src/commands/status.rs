use super::CommandResult;
use crate::{
    cli::GlobalArgs,
    config,
    files::{get_file_status, LinkStatus},
};
use clap::{App, ArgMatches, SubCommand};
use colored::*;
use config::AppConfig;

pub const CMD_IDENTIFIER: &str = "status";
const CMD_ABOUT: &str = r#"
Shows the current status for all files and directories in your dotfiles repository.
For each file, a status and a possible explanation is shown:
LINKED  : The file is linked from the home directory to the dotfiles directory.
INVALID : This path is listed in the configuration, but does not exist in the dotfiles repository.
CONFLICT: The path exists in the home directory, but is either not a symlink
          or does not point to its counterpart in the dotfiles directory.
UNLINKED: The file is currently not linked to the home directory.
MISSING : This file or directory in the dotfiles repository is nowhere mentioned under mappings
          and will therefore never be linked.
"#;

/// returns the clap definition for the status sub-command
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_IDENTIFIER).about(CMD_ABOUT)
}

struct StatusCommandArgs {}
impl StatusCommandArgs {
    fn from_args(_: &ArgMatches) -> StatusCommandArgs {
        StatusCommandArgs {}
    }
}

/// Handler of the `status` sub-command.
/// Iterates over all files configured under mappings in the dotfiles config file and
pub fn run(args: &ArgMatches, global_args: &GlobalArgs) -> CommandResult {
    let _args = StatusCommandArgs::from_args(args);
    let config = AppConfig::from_config_file(global_args)?;

    if config.mappings.len() == 0 {
        println!("There are no links configured")
    }

    for link in config.mappings {
        let status = get_file_status(global_args, &link)?;
        let text_status = match status {
            LinkStatus::Unlinked => "UNLINKED".yellow(),
            LinkStatus::Linked => "LINKED  ".green(),
            LinkStatus::Invalid(_) => "INVALID ".purple(),
            LinkStatus::ConflictNoLink(_) => "CONFLICT".red(),
            LinkStatus::ConflictWrongTarget(_) => "CONFLICT".red(),
        };

        let description = match status {
            LinkStatus::ConflictNoLink(target) => format!("{:?} is not a symlink", target),
            LinkStatus::ConflictWrongTarget(target) => format!("points to {:?} instead", target),
            LinkStatus::Invalid(target) => format!("{:?} does not exist", target),
            _ => String::new(),
        };

        println!("{} {:?} {}", text_status, &link, description.red());
    }

    Ok(())
}
