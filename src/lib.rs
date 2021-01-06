use cli::build_cli;
use errors::AppError;

pub mod cli;
pub mod commands;
pub mod config;
pub mod errors;
pub mod files;

/// runs the application. Reads all process arguments and calls the appropriate command handler
pub fn run() -> Result<(), AppError> {
    let cli_args = build_cli().get_matches();
    commands::run_command(&cli_args)
}
