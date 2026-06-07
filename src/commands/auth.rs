use std::io::{self, Read, Write};

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::auth::token_store::TokenStore;
use crate::cli::GlobalArgs;
use crate::config::{Config, HostConfig};
use crate::error::CliError;
use crate::git::credential::CredentialOp;
use crate::output::{HumanRender, Printer};

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
        /// Override the git host to register for the credential helper.
        #[arg(long)]
        git_host: Option<String>,
    },
    /// Remove stored credentials for the host.
    Logout,
    /// Show the current authentication status.
    Status,
    /// Register `nub` as a git credential helper.
    SetupGit,
    /// Git credential protocol entry point (invoked by git, not by users).
    GitCredential {
        /// Protocol operation requested by git.
        operation: CredentialOp,
    },
}

/// Runs an `auth` subcommand.
///
/// # Errors
/// Returns a [`CliError`] on failure.
pub fn run(args: &AuthArgs, global: &GlobalArgs, printer: &Printer) -> Result<(), CliError> {
    match &args.command {
        AuthCommand::Login {
            with_token,
            git_host,
        } => login(*with_token, git_host.as_deref(), global, printer),
        AuthCommand::Logout => logout(global, printer),
        AuthCommand::Status => status(global, printer),
        AuthCommand::SetupGit => run_setup_git(global),
        AuthCommand::GitCredential { operation } => run_git_credential(operation, global),
    }
}

fn run_setup_git(global: &GlobalArgs) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    crate::git::credential::setup_git(&config, &host)
}

fn run_git_credential(operation: &CredentialOp, global: &GlobalArgs) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let store = TokenStore::new()?;
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    crate::git::credential::run_credential(operation, &host, &store, stdin.lock(), stdout.lock())
}

fn login(
    with_token: bool,
    git_host: Option<&str>,
    global: &GlobalArgs,
    printer: &Printer,
) -> Result<(), CliError> {
    if !with_token {
        return Err(CliError::Generic(
            "interactive login is not available yet; use `nub auth login --with-token`".to_owned(),
        ));
    }

    let mut config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let token = read_token_from_stdin()?;

    TokenStore::new()?.set(&host, &token)?;

    let stored_git_host = config.hosts.get(&host).and_then(|h| h.git_host.clone());
    let resolved_git_host =
        crate::git::resolve_git_host(git_host, stored_git_host.as_deref(), &host);
    let entry = config
        .hosts
        .entry(host.clone())
        .or_insert_with(|| HostConfig {
            api_url: format!("https://{host}"),
            git_host: None,
        });
    entry.git_host = Some(resolved_git_host);

    config.default_host = Some(host.clone());
    config.save()?;

    printer.emit(&LoginOutcome { host })
}

fn logout(global: &GlobalArgs, printer: &Printer) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    TokenStore::new()?.delete(&host)?;
    printer.emit(&LogoutOutcome { host })
}

fn status(global: &GlobalArgs, printer: &Printer) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    if TokenStore::new()?.get(&host)?.is_some() {
        printer.emit(&AuthStatus {
            host,
            logged_in: true,
        })
    } else {
        Err(CliError::NotAuthenticated)
    }
}

/// Output of `auth login`.
#[derive(Serialize)]
struct LoginOutcome {
    /// Host the token was stored for.
    host: String,
}

impl HumanRender for LoginOutcome {
    fn render(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(out, "Logged in to {}.", self.host)
    }
}

/// Output of `auth logout`.
#[derive(Serialize)]
struct LogoutOutcome {
    /// Host the token was removed for.
    host: String,
}

impl HumanRender for LogoutOutcome {
    fn render(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(out, "Logged out of {}.", self.host)
    }
}

/// Output of `auth status` when a credential is available.
#[derive(Serialize)]
struct AuthStatus {
    /// Host the status was checked against.
    host: String,
    /// Whether a credential is stored for the host.
    logged_in: bool,
}

impl HumanRender for AuthStatus {
    fn render(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(out, "Logged in to {}.", self.host)
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
