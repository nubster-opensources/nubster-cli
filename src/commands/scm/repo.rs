use clap::{Args, Subcommand};

use crate::cli::GlobalArgs;
use crate::error::CliError;

/// Repository commands.
#[derive(Args, Debug)]
pub struct RepoArgs {
    #[command(subcommand)]
    pub command: RepoCommand,
}

/// Subcommands under `nub repo`.
#[derive(Subcommand, Debug)]
pub enum RepoCommand {
    /// Create a repository.
    Create,
    /// Clone a repository by handle.
    Clone,
    /// List repositories.
    List,
    /// Show a repository.
    View,
}

/// Runs a `repo` subcommand.
///
/// # Errors
/// Returns a [`CliError`] on failure. Leaf commands currently return
/// [`CliError::NotImplemented`] until their dedicated issues land.
pub fn run(args: &RepoArgs, _global: &GlobalArgs) -> Result<(), CliError> {
    match args.command {
        RepoCommand::Create => Err(CliError::NotImplemented("nub repo create")),
        RepoCommand::Clone => Err(CliError::NotImplemented("nub repo clone")),
        RepoCommand::List => Err(CliError::NotImplemented("nub repo list")),
        RepoCommand::View => Err(CliError::NotImplemented("nub repo view")),
    }
}
