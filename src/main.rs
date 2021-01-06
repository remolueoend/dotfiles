use colored::*;

/// Entry point of this binary. Use `dotfiles --help` to get an overview of how to use it.
///
/// This project consists of following modules:
/// lib.rs     : root lib module, re-publishes all other modules
/// errors.rs  : provides objects for error handling
/// commands   : directory containing a module for each CLI sub-command
/// cli.rs     : CLI interface definitions
/// config.rs  : everything related to reading and writing configurations
/// files.rs   : file system abstractions commonly used in this binary
///
/// Error Handling:
/// This binary declares its own error enum `AppError` in `lib.rs`. All functions which return a `Result`
/// use `AppError` as the error type. Errors which cannot be handled by the app itself (almost all of them)
/// are passed up to the `main` entrypoint where they are printed to the user. There should be no usage of
/// `panic!`, unhandled `unwrap` call and similar constructs in this app. This should improve the readability
/// and usefulness of errors to the user in front of the screen.
///
/// Further notes (mostly to myself):
/// 1. Avoid over-abstraction: remember KISS and don't be scared of repeating yourself here and there.
/// 2. This binary is human-first: All output including errors should be human-readable and helpful.
///    It makes use of special flags such as `--json` to provide a machine-readable output.
fn main() {
    if let Err(msg) = dotfiles::run() {
        // TODO: should we print errors differently when `--json` has been provided?
        eprintln!("{}: {}", "Error".red().bold(), msg);
    }
}
