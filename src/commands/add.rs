use super::CommandResult;
use crate::{
    cli::GlobalArgs,
    config::AppConfig,
    errors::AppError,
    files::{create_symlink_for, get_cwd, get_home_dir, normalize_paths},
};
use clap::{App, Arg, ArgMatches, SubCommand};
use fs_extra::{dir, file};
use std::{fmt::Display, fs, path::PathBuf};

pub const CMD_IDENTIFIER: &str = "add";
const CMD_ABOUT: &str = r#"
Adds the given path to the dotfiles mappings.
If the file or folder is in your dotfiles directory, this command will:
1) add the path to the mappings in the dotfiles configuration file.
2) create a symlink to this path at the appropriate location in your home directory if it does not exist yet.

If the file or folder is located in in your home directory, it will:
1) add the path to the mappings in the dotfiles configuration file.
2) move the file or folder from your home directory to your dotfiles directory.
3) create a symlink to this path at the appropriate location in your home directory.
"#;

/// Describes a single required IO change to be done. Used to display a list of changes
/// to the user to sign of.
enum RequiredChanges {
    AddMapping(PathBuf),
    CreateSymlink(PathBuf, PathBuf),
    MoveFile(PathBuf, PathBuf),
}
/// Describes a list of steps which can be skipped
type SkippingChanges = Vec<&'static str>;

/// Describes an `add` sub-command specific error.
#[derive(Debug)]
pub enum Error {
    /// the given path is outside the home *and* dotfiles directory.
    OutsideValidDir(PathBuf),
    /// The given path already exists in both the home and dotfiles dir, but the do not point to each other.
    /// Consists of the absolute path into the dotfiles and home directory.
    BothPathsExist(PathBuf, PathBuf),
    /// Another mapping exists which is a parent of the given path.
    /// Consists of the given path and existing parent path.
    ExistingParent(PathBuf, PathBuf),
    /// Another mapping exists which is a child of the given path.
    /// Consists of the given path and existing nested path.
    ExistingChild(PathBuf, PathBuf),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OutsideValidDir(_) => write!(
                f,
                "The given path must be either inside your home or dotfiles directory."
            ),
            Error::BothPathsExist(dotfiles, home) => write!(
                f,
                "Both {} and {} already exist. Remove one of them and run this command again.",
                dotfiles.display(),
                home.display()
            ),
            Error::ExistingParent(path, parent) => write!(
                f,
                "Cannot add this path: The given path {} is a parent of the existing mapping {}. Nested mappings are not supported.",
                path.display(),
                parent.display(),
            ),
            Error::ExistingChild(path, child) => write!(
                f,
                "Cannot add this path: The existing mapping {} is a child of the given path {}. Nested mappings are not supported.",
                child.display(),
                path.display()
            ),
        }
    }
}

/// returns the clap definition for the status sub-command
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_IDENTIFIER).about(CMD_ABOUT).arg(
        Arg::with_name("path")
            .help("the path to the directory or file to add.")
            .required(true),
    )
}

struct AddCommandArgs {
    /// The path to add to the dotfiles. If accessed outside of this struct,
    /// it is guaranteed to be absolute and existing.
    path: PathBuf,
}
impl AddCommandArgs {
    fn from_args(args: &ArgMatches) -> Result<AddCommandArgs, AppError> {
        let path = PathBuf::from(args.value_of("path").unwrap());
        let cwd = get_cwd()?;
        // we cannot use canonicalize because we do not want to resolve symlinks here:
        let abs_path = normalize_paths(&cwd, &path)?;
        if abs_path.exists() == false {
            return Err(AppError::CliInvalidArgValue(
                "path".to_string(),
                format!("The given path {} does not exist", abs_path.display()),
            ));
        };

        return Ok(AddCommandArgs { path: abs_path });
    }
}

/// command handler for the `add` sub-command
/// see `dotfiles add -h` for an overview.
pub fn run(args: &ArgMatches, global_args: &GlobalArgs) -> CommandResult {
    let AddCommandArgs { path } = AddCommandArgs::from_args(args)?;
    let mut config = AppConfig::from_config_file(global_args)?;
    let home_dir = get_home_dir()?;

    let (changes, skipped) =
        get_required_changes(&config, &global_args.dotfiles_root, &home_dir, &path)
            .map_err(|err| AppError::CmdAddError(err))?;

    if !skipped.is_empty() {
        println!("Following steps can be skipped:");
        for skip in skipped {
            println!("- {}", skip);
        }
    }
    if !changes.is_empty() {
        println!("Following things will be done:");
        for change in &changes {
            let line = match change {
                RequiredChanges::AddMapping(path) => {
                    format!("adding {} to mappings in config file", path.display())
                }
                RequiredChanges::CreateSymlink(from, to) => {
                    format!("creating symlink {} -> {}", from.display(), to.display())
                }
                RequiredChanges::MoveFile(from, to) => {
                    format!("moving {} -> {}", from.display(), to.display())
                }
            };
            println!("- {}", line);
        }

        if promptly::prompt_default("Continue?", true).unwrap_or(false) {
            apply_changes(&changes, &mut config, global_args)?;
        }
    } else {
        println!("Nothing left to be done. Have a good time!");
    }

    Ok(())
}

fn get_required_changes(
    config: &AppConfig,
    dotfiles_root: &PathBuf,
    home_dir: &PathBuf,
    path: &PathBuf,
) -> Result<(Vec<RequiredChanges>, SkippingChanges), Error> {
    let is_in_dotfiles = path.starts_with(&dotfiles_root);
    // this variable is true if the path points exclusively into home dir, but not dotfiles dir.
    // Often though, the dotfiles dir is a subdirectory of the home dir:
    let is_in_home_dir = path.starts_with(&home_dir) && !is_in_dotfiles;

    // the relative path which will be stored in config.mappings:
    let mappings_path = if is_in_dotfiles {
        Ok(path.strip_prefix(&dotfiles_root).unwrap())
    } else if is_in_home_dir {
        Ok(path.strip_prefix(&home_dir).unwrap())
    } else {
        Err(Error::OutsideValidDir(path.clone()))
    }?
    .to_owned();

    // the absolute paths into the home dir and dotfiles dir:
    let homedir_path = home_dir.join(&mappings_path);
    let dotfiles_path = dotfiles_root.join(&mappings_path);

    let mut changes: Vec<RequiredChanges> = Vec::new();
    let mut skipped: SkippingChanges = Vec::new();

    if config.mappings.contains(&mappings_path) {
        skipped.push("This path is already mapped, no need to update config.");
    } else {
        // make sure we do not end up with nested mappings:
        for mapping in &config.mappings {
            if mapping.starts_with(&mappings_path) {
                return Err(Error::ExistingParent(
                    mappings_path.to_owned(),
                    mapping.to_owned(),
                ));
            } else if mappings_path.starts_with(&mapping) {
                return Err(Error::ExistingChild(
                    mappings_path.to_owned(),
                    mapping.to_owned(),
                ));
            }
        }
        changes.push(RequiredChanges::AddMapping(mappings_path.to_owned()));
    };

    // special case: file exists in both home and dotfiles dir:
    // either they are already correctly linked or this operation is invalid:
    if homedir_path.exists() && dotfiles_path.exists() {
        let meta = fs::symlink_metadata(&homedir_path).unwrap();
        if meta.file_type().is_symlink() && fs::read_link(&homedir_path).unwrap() == dotfiles_path {
            skipped.push("no symlink will be created, paths are already linked.");
        } else {
            return Err(Error::BothPathsExist(
                dotfiles_path.clone(),
                homedir_path.clone(),
            ));
        }
    } else {
        // exists in home dir, but not in dotfiles dir => move files to dotfiles dir:
        if homedir_path.exists() {
            changes.push(RequiredChanges::MoveFile(
                homedir_path.clone(),
                dotfiles_path.clone(),
            ))
        }
        // has to be done either way, but make sure to add it after moving files if necessary:
        changes.push(RequiredChanges::CreateSymlink(
            homedir_path.clone(),
            dotfiles_path.clone(),
        ));
    }

    Ok((changes, skipped))
}

fn apply_changes(
    changes: &Vec<RequiredChanges>,
    config: &mut AppConfig,
    global_args: &GlobalArgs,
) -> Result<(), AppError> {
    for change in changes {
        match change {
            RequiredChanges::AddMapping(path) => {
                config.add_mapping(path.to_owned());
                config.to_config_file(global_args)?;
            }
            RequiredChanges::CreateSymlink(from, to) => create_symlink_for(&from, &to)?,
            RequiredChanges::MoveFile(from, to) => {
                if from.is_dir() {
                    let mut options = dir::CopyOptions::new();
                    options.copy_inside = true;
                    dir::move_dir(&from, &to, &options).map_err(|err| {
                        AppError::FsOther(format!(
                            "failed to move directory {} -> {}: {}",
                            from.display(),
                            to.display(),
                            err
                        ))
                    })?;
                } else {
                    let options = file::CopyOptions::new();
                    file::move_file(&from, &to, &options).map_err(|err| {
                        AppError::FsOther(format!(
                            "failed to move file {} -> {}: {}",
                            from.display(),
                            to.display(),
                            err
                        ))
                    })?;
                }
            }
        }
    }

    Ok(())
}
