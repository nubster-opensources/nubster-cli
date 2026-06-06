use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use crate::api::client::Client;
use crate::api::repo::{CreateRepoRequest, RepoService, Repository, Visibility};
use crate::auth::token_store::TokenStore;
use crate::cli::GlobalArgs;
use crate::config::Config;
use crate::error::CliError;
use crate::git;

/// Repository commands.
#[derive(Args, Debug)]
pub struct RepoArgs {
    #[command(subcommand)]
    pub command: RepoCommand,
}

/// Subcommands under `nub repo`.
#[derive(Subcommand, Debug)]
pub enum RepoCommand {
    /// Create a new repository on the platform.
    Create {
        /// Short name for the new repository.
        #[arg(long)]
        name: String,
        /// Optional description shown in listings.
        #[arg(long)]
        description: Option<String>,
        /// Make the repository publicly accessible (default: private).
        #[arg(long)]
        public: bool,
    },
    /// Clone a repository to the local filesystem.
    Clone {
        /// Repository name (`namespace/repo` or bare `repo`).
        name: String,
        /// Destination directory (defaults to the repository name).
        dest: Option<PathBuf>,
        /// Destination directory, flag form.
        #[arg(long = "dest", value_name = "PATH", conflicts_with = "dest")]
        dest_flag: Option<PathBuf>,
        /// Clone over SSH instead of HTTPS.
        #[arg(long)]
        ssh: bool,
    },
    /// List repositories accessible to the authenticated principal.
    List,
    /// Show details of a single repository.
    View {
        /// Repository name (`namespace/repo` or bare `repo`).
        name: String,
    },
}

/// Runs a `repo` subcommand.
///
/// # Errors
/// Returns a [`CliError`] on failure.
pub fn run(args: &RepoArgs, global: &GlobalArgs) -> Result<(), CliError> {
    match &args.command {
        RepoCommand::Create {
            name,
            description,
            public,
        } => run_create(name, description.as_deref(), *public, global),
        RepoCommand::Clone {
            name,
            dest,
            dest_flag,
            ssh,
        } => run_clone(name, dest.as_deref().or(dest_flag.as_deref()), *ssh, global),
        RepoCommand::List => run_list(global),
        RepoCommand::View { name } => run_view(name, global),
    }
}

fn run_create(
    name: &str,
    description: Option<&str>,
    public: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let client = Client::for_host(&config, &host, &TokenStore::new()?)?;
    let req = CreateRepoRequest {
        name: name.to_owned(),
        description: description.map(str::to_owned),
        visibility: if public {
            Visibility::Public
        } else {
            Visibility::Private
        },
    };
    let repo = runtime()?.block_on(RepoService::new(&client).create(&req))?;
    print_repo(&repo);
    Ok(())
}

fn run_list(global: &GlobalArgs) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let client = Client::for_host(&config, &host, &TokenStore::new()?)?;
    let repos = runtime()?.block_on(RepoService::new(&client).list())?;
    print_repo_list(&repos);
    Ok(())
}

fn run_clone(
    name: &str,
    dest: Option<&Path>,
    ssh: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let client = Client::for_host(&config, &host, &TokenStore::new()?)?;
    let repo = runtime()?.block_on(RepoService::new(&client).view(name))?;
    let url = if ssh { &repo.ssh_url } else { &repo.clone_url };
    ensure_helper_for(url)?;
    git::clone_repo(url, dest)?;
    println!("Cloned {}.", repo.full_name);
    Ok(())
}

/// Verifies that a credential helper is configured before an HTTPS clone,
/// guiding the user towards `nub auth setup-git` otherwise. Non-HTTPS URLs
/// (SSH, local) authenticate through other means and pass unchecked.
fn ensure_helper_for(url: &str) -> Result<(), CliError> {
    let Some(git_host) = https_host(url) else {
        return Ok(());
    };
    if git::is_helper_configured(git_host)? {
        return Ok(());
    }
    Err(CliError::Generic(format!(
        "git credential helper is not configured for {git_host}; run `nub auth setup-git` first"
    )))
}

/// Extracts the host (with optional port) from an `https://` URL.
fn https_host(url: &str) -> Option<&str> {
    url.strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
}

fn run_view(name: &str, global: &GlobalArgs) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let client = Client::for_host(&config, &host, &TokenStore::new()?)?;
    let repo = runtime()?.block_on(RepoService::new(&client).view(name))?;
    print_repo_detail(&repo);
    Ok(())
}

fn runtime() -> Result<tokio::runtime::Runtime, CliError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Generic(format!("cannot create async runtime: {e}")))
}

fn print_repo(repo: &Repository) {
    println!("Created {}.", repo.full_name);
}

fn print_repo_list(repos: &[Repository]) {
    for repo in repos {
        println!("{:<40}  [{}]", repo.full_name, repo.visibility);
    }
}

fn print_repo_detail(repo: &Repository) {
    println!("name:          {}", repo.full_name);
    println!("visibility:    {}", repo.visibility);
    println!("clone (https): {}", repo.clone_url);
    println!("clone (ssh):   {}", repo.ssh_url);
    if let Some(desc) = &repo.description {
        println!("description:   {desc}");
    }
    println!("created:       {}", repo.created_at);
}
