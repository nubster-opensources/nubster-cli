use clap::{Args, Subcommand};

use crate::cli::GlobalArgs;
use crate::error::CliError;

/// Authentication and credential commands.
#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

/// Subcommands under `nub auth`.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Log in by storing a personal access token.
    Login,
    /// Remove stored credentials for the host.
    Logout,
    /// Show the current authentication status.
    Status,
    /// Register `nub` as a git credential helper.
    SetupGit,
    /// Git credential protocol entry point (invoked by git, not by users).
    GitCredential,
}

/// Runs an `auth` subcommand.
///
/// # Errors
/// Returns a [`CliError`] on failure. Leaf commands currently return
/// [`CliError::NotImplemented`] until their dedicated issues land.
pub fn run(args: &AuthArgs, _global: &GlobalArgs) -> Result<(), CliError> {
    match args.command {
        AuthCommand::Login => Err(CliError::NotImplemented("nub auth login")),
        AuthCommand::Logout => Err(CliError::NotImplemented("nub auth logout")),
        AuthCommand::Status => Err(CliError::NotImplemented("nub auth status")),
        AuthCommand::SetupGit => Err(CliError::NotImplemented("nub auth setup-git")),
        AuthCommand::GitCredential => Err(CliError::NotImplemented("nub auth git-credential")),
    }
}
