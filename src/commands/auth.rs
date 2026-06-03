use std::io::Read;

use clap::{Args, Subcommand};

use crate::auth::token_store::TokenStore;
use crate::cli::GlobalArgs;
use crate::config::{Config, HostConfig};
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
    /// Log in to a host.
    Login {
        /// Read a personal access token from stdin instead of the browser flow.
        #[arg(long)]
        with_token: bool,
    },
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
/// Returns a [`CliError`] on failure.
pub fn run(args: &AuthArgs, global: &GlobalArgs) -> Result<(), CliError> {
    match &args.command {
        AuthCommand::Login { with_token } => login(*with_token, global),
        AuthCommand::Logout => logout(global),
        AuthCommand::Status => status(global),
        AuthCommand::SetupGit => Err(CliError::NotImplemented("nub auth setup-git")),
        AuthCommand::GitCredential => Err(CliError::NotImplemented("nub auth git-credential")),
    }
}

fn login(with_token: bool, global: &GlobalArgs) -> Result<(), CliError> {
    if !with_token {
        return Err(CliError::Generic(
            "interactive login is not available yet; use `nub auth login --with-token`".to_owned(),
        ));
    }

    let mut config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let token = read_token_from_stdin()?;

    TokenStore::new()?.set(&host, &token)?;
    config
        .hosts
        .entry(host.clone())
        .or_insert_with(|| HostConfig {
            api_url: format!("https://{host}"),
            git_host: None,
        });
    config.default_host = Some(host.clone());
    config.save()?;

    println!("Logged in to {host}.");
    Ok(())
}

fn logout(global: &GlobalArgs) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    TokenStore::new()?.delete(&host)?;
    println!("Logged out of {host}.");
    Ok(())
}

fn status(global: &GlobalArgs) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    if TokenStore::new()?.get(&host)?.is_some() {
        println!("Logged in to {host}.");
        Ok(())
    } else {
        Err(CliError::NotAuthenticated)
    }
}

fn read_token_from_stdin() -> Result<String, CliError> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| CliError::Generic(format!("cannot read token from stdin: {e}")))?;
    let token = buffer.trim().to_owned();
    if token.is_empty() {
        return Err(CliError::Generic("no token provided on stdin".to_owned()));
    }
    Ok(token)
}
