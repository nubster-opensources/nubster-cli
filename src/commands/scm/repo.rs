use clap::{Args, Subcommand};

use crate::api::client::Client;
use crate::api::repo::{CreateRepoRequest, RepoService, Repository, Visibility};
use crate::auth::token_store::TokenStore;
use crate::cli::GlobalArgs;
use crate::config::Config;
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
    Clone,
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
        RepoCommand::Clone => Err(CliError::NotImplemented("nub repo clone")),
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
