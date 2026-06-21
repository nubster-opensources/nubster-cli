//! CLI entry point: argument parser and top-level dispatch for `nub`.

use clap::{Args, Parser, Subcommand};
use std::process::ExitCode;

use crate::commands;
use crate::output::Printer;

/// Options accepted at any level of the command tree.
#[derive(Args, Debug)]
pub(crate) struct GlobalArgs {
    /// Target platform host (overrides the configured default).
    #[arg(long, global = true)]
    pub(crate) host: Option<String>,
    /// Emit machine-readable JSON instead of human output.
    #[arg(long, global = true)]
    pub(crate) json: bool,
    /// Disable colored output.
    #[arg(long, global = true)]
    pub(crate) no_color: bool,
}

/// Top-level command groups.
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Authentication and credentials.
    Auth(commands::auth::AuthArgs),
    /// Repository operations.
    Repo(commands::scm::repo::RepoArgs),
    /// Inspect the CLI configuration.
    Config(commands::config::ConfigArgs),
}

/// Root command for `nub`.
#[derive(Parser, Debug)]
#[command(
    name = "nub",
    version,
    about = "Unified command-line client for the Nubster platform.",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(flatten)]
    pub(crate) global: GlobalArgs,
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Runs the parsed CLI and returns a process exit code.
#[must_use]
pub fn run(cli: Cli) -> ExitCode {
    let Cli { global, command } = cli;
    let printer = Printer::new(&global);
    let result = match &command {
        Command::Auth(args) => commands::auth::run(args, &global, &printer),
        Command::Repo(args) => commands::scm::repo::run(args, &global, &printer),
        Command::Config(args) => commands::config::run(args, &global),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}
