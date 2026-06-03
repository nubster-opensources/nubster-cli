use clap::{CommandFactory, Parser};
use std::process::ExitCode;

/// Root command for `nub`.
#[derive(Parser, Debug)]
#[command(
    name = "nub",
    version,
    about = "Unified command-line client for the Nubster platform."
)]
pub struct Cli {
    // Subcommands and global flags are introduced in a later iteration.
}

/// Runs the parsed CLI and returns a process exit code.
#[must_use]
pub fn run(_cli: &Cli) -> ExitCode {
    let mut command = Cli::command();
    if command.print_help().is_err() {
        return ExitCode::FAILURE;
    }
    println!();
    ExitCode::SUCCESS
}
