use crate::{
    cli::GlobalArgs,
    config,
    files::{get_file_status, LinkStatus},
};
use clap::ArgMatches;
use colored::*;
use config::AppConfig;

use super::CommandResult;

struct StatusCommandArgs {}
impl StatusCommandArgs {
    fn from_args(_: &ArgMatches) -> StatusCommandArgs {
        StatusCommandArgs {}
    }
}

/// Handler of the `status` sub-command.
/// Interates over all files configured under mappings in the dotfiles config file and
/// prints the status of each file, which is either:
/// 1. LINKED: The file is correctly linked to its counterpart in the dotfiles repo
/// 2. UNLINKED: The file is correctly not linked
/// 3. MISSING: The file does not exist in the dotfiles repository
/// 4. CONFLICT: One of several possible conflicts, such as a symlink to another file or no symlink at all.
pub fn run(args: &ArgMatches, global_args: &GlobalArgs) -> CommandResult {
    let _args = StatusCommandArgs::from_args(args);
    let config = AppConfig::from_config_file(global_args)?;

    for link in config.mappings {
        let status = get_file_status(global_args, &link)?;
        let text_status = match status {
            LinkStatus::Unlinked => "UNLINKED".yellow(),
            LinkStatus::Linked => "LINKED  ".green(),
            LinkStatus::Missing => "MISSING".red(),
            LinkStatus::ConflictNoLink(_) => "CONFLICT".red(),
            LinkStatus::ConfictWrongTarget(_) => "CONFLICT".red(),
        };

        let description = match status {
            LinkStatus::ConflictNoLink(target) => format!("{:?} is not a symlink", target),
            LinkStatus::ConfictWrongTarget(target) => format!("points to {:?} instead", target),
            _ => String::new(),
        };

        println!("{} {} {}", text_status, &link, description.red());
    }

    Ok(())
}
