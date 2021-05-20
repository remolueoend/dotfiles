use super::CommandResult;
use crate::{cli::GlobalArgs, config, errors::AppError, files::get_home_dir};
use clap::{App, ArgMatches, SubCommand};
use colored::*;
use config::{AppConfig, Mapping};
use std::{collections::VecDeque, fs, io, iter::FromIterator, path::PathBuf};

pub const CMD_IDENTIFIER: &str = "status";
const CMD_ABOUT: &str = r#"
Shows the current status for all files and directories in your dotfiles repository.
For each file, a status and a possible explanation is shown:
LINKED  : The file is linked from the home directory to the dotfiles directory.
INVALID : This path is listed in the configuration, but does not exist in the dotfiles repository.
CONFLICT: The path exists in the home directory, but is either not a symlink
          or does not point to its counterpart in the dotfiles directory.
UNLINKED: The file is currently not linked to the home directory.
UNMAPPED: This file or directory in the dotfiles repository is nowhere mentioned under mappings
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

    let dotfile_entries = get_dotfiles_entries(global_args, &config).map_err(|err| {
        AppError::FsOther(format!(
            "Failed to read your dotfile directory at {}: {}",
            global_args.dotfiles_root.display(),
            err
        ))
    })?;
    for entry in &dotfile_entries {
        let status =
            get_dotfiles_entry_state(global_args, entry, &get_home_dir()?).map_err(|err| {
                AppError::FsOther(format!("Failed to read your linked dotfiles: {}", err))
            })?;
        let text_status = match status {
            LinkState::Unlinked => "UNLINKED".yellow(),
            LinkState::Linked => "LINKED  ".green(),
            LinkState::Invalid(_) => "INVALID ".purple(),
            LinkState::ConflictNoLink(_) => "CONFLICT".red(),
            LinkState::ConflictWrongTarget(_) => "CONFLICT".red(),
            LinkState::Unmapped => "UNMAPPED".white(),
        };

        let description = match status {
            LinkState::ConflictNoLink(target) => format!("{:?} is not a symlink", target),
            LinkState::ConflictWrongTarget(target) => format!("points to {:?} instead", target),
            LinkState::Invalid(target) => format!("{:?} does not exist", target),
            _ => String::new(),
        };

        println!(
            "{} {} {}",
            text_status,
            entry.0.display(),
            description.red()
        );
    }

    Ok(())
}

pub enum MappingSourceStatus {
    Existing,
    Missing,
}
pub enum MappingTargetStatus<'a> {
    Linked(&'a PathBuf),
    Missing,
    WrongLink(&'a PathBuf),
    Conflict,
}

pub struct MappingStatus<'a> {
    pub path: &'a PathBuf,
    pub src_state: MappingSourceStatus,
    pub target_state: MappingTargetStatus<'a>,
}

impl<'a> MappingStatus<'a> {
    pub fn from_mapping(mapping: &'a Mapping) -> Result<MappingStatus<'a>, AppError> {
        Ok(MappingStatus {
            path: mapping,
            src_state: MappingSourceStatus::Existing,
            target_state: MappingTargetStatus::Missing,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DotfilesEntryState {
    Mapped,
    Unmapped,
    Invalid,
}
pub type DotfilesEntry = (PathBuf, DotfilesEntryState);

/// Returns a list of relative dotfiles repo paths, which are filtered the following way:
/// If a directory is in the configured mappings, all its children are excluded.
/// If a directory or file is nested in a parent which is not part of any configured mapping, it is also excluded.
/// Each returned path additionally contains the information, if it is linked or unlinked based on the configured mappings.
/// All entries of config.mappings which could not be found in the dotfiles directory are also attached with the state `Invalid`.
fn get_dotfiles_entries(
    global_args: &GlobalArgs,
    config: &AppConfig,
) -> io::Result<Vec<DotfilesEntry>> {
    let mut dotfiles: Vec<DotfilesEntry> = Vec::new();
    let dotfile_root = &global_args.dotfiles_root;
    let mut queue = VecDeque::from_iter(fs::read_dir(dotfile_root)?);
    let mappings = &config.mappings;

    while let Some(next) = queue.pop_front() {
        let path = next?.path();
        // this is safe, because we are only iterating items contained in the dotfiles root directory:
        let rel_path = path.strip_prefix(dotfile_root).unwrap().to_owned();
        // if the entry itself is mapped: add it to the output but don't traverse it further:
        if mappings.contains(&rel_path) {
            dotfiles.push((rel_path, DotfilesEntryState::Mapped));
        // there is no mapping on or into the current path: stop traversing it,
        // but add the current path itself to output (as "unmapped")
        } else if mappings.into_iter().any(|m| m.starts_with(&rel_path)) == false {
            dotfiles.push((rel_path, DotfilesEntryState::Unmapped));
        // make sure we only traverse into directories and do not follow symlinks:
        } else if path.symlink_metadata()?.is_dir() {
            // there exist one or more mappings into the current directory:
            // we do not add the current path to the output, but traverse it instead.
            queue.extend(fs::read_dir(&path)?);
        }
    }

    // every other entry in config.mapping which is not part of dotfiles yet, has to be invalid,
    // because it was not found during the traversal of the dotfiles directory.
    // insert them with state `Invalid`:
    dotfiles.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for mapping in mappings {
        match dotfiles.binary_search_by(|(path, _)| path.cmp(mapping)) {
            Ok(_) => (),
            Err(pos) => dotfiles.insert(pos, (mapping.to_owned(), DotfilesEntryState::Invalid)),
        }
    }

    Ok(dotfiles)
}

/// Describes the status of a link configured in mappings
pub enum LinkState {
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
    /// File in dotfiles repo is not listen in mappings
    Unmapped,
}

/// Returns the status for a given dotfiles entry.
pub fn get_dotfiles_entry_state(
    global_args: &GlobalArgs,
    entry: &DotfilesEntry,
    target_dir: &PathBuf,
) -> io::Result<LinkState> {
    let (path, state) = entry;

    // path to the symlink at the target location
    let actual_file_path = target_dir.join(path);
    // path to the file in the dotfiles repository
    let expected_target = global_args.dotfiles_root.join(path);

    // invalid and unmapped entries can be translated directly:
    match state {
        DotfilesEntryState::Invalid => return Ok(LinkState::Invalid(expected_target)),
        DotfilesEntryState::Unmapped => return Ok(LinkState::Unmapped),
        _ => (),
    };

    // the entry in the dotfiles exists, but the corresponding file in the home directory does not:
    if actual_file_path.exists() == false {
        return Ok(LinkState::Unlinked);
    };

    let actual_file_meta = actual_file_path.symlink_metadata()?;
    if actual_file_meta.file_type().is_symlink() == false {
        return Ok(LinkState::ConflictNoLink(actual_file_path));
    };

    let actual_target = fs::read_link(&actual_file_path)?;
    if actual_target != expected_target {
        Ok(LinkState::ConflictWrongTarget(actual_target))
    } else {
        Ok(LinkState::Linked)
    }
}
