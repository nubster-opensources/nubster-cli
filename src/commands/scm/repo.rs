use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::api::client::Client;
use crate::api::repo::{CreateRepoRequest, RepoService, Repository, Visibility};
use crate::auth::token_store::TokenStore;
use crate::cli::GlobalArgs;
use crate::config::Config;
use crate::error::CliError;
use crate::git;
use crate::output::{self, HumanRender, Printer};

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
pub fn run(args: &RepoArgs, global: &GlobalArgs, printer: &Printer) -> Result<(), CliError> {
    match &args.command {
        RepoCommand::Create {
            name,
            description,
            public,
        } => run_create(name, description.as_deref(), *public, global, printer),
        RepoCommand::Clone {
            name,
            dest,
            dest_flag,
            ssh,
        } => run_clone(
            name,
            dest.as_deref().or(dest_flag.as_deref()),
            *ssh,
            global,
            printer,
        ),
        RepoCommand::List => run_list(global, printer),
        RepoCommand::View { name } => run_view(name, global, printer),
    }
}

fn run_create(
    name: &str,
    description: Option<&str>,
    public: bool,
    global: &GlobalArgs,
    printer: &Printer,
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
    printer.emit(&RepoCreated(repo))
}

fn run_list(global: &GlobalArgs, printer: &Printer) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let client = Client::for_host(&config, &host, &TokenStore::new()?)?;
    let repos = runtime()?.block_on(RepoService::new(&client).list())?;
    printer.emit(&RepoList(repos))
}

fn run_clone(
    name: &str,
    dest: Option<&Path>,
    ssh: bool,
    global: &GlobalArgs,
    printer: &Printer,
) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let client = Client::for_host(&config, &host, &TokenStore::new()?)?;
    let repo = runtime()?.block_on(RepoService::new(&client).view(name))?;
    let url = if ssh { &repo.ssh_url } else { &repo.clone_url };
    ensure_helper_for(url)?;
    git::clone_repo(url, dest)?;
    let path = resolve_clone_path(url, dest)?;
    printer.emit(&CloneOutcome {
        full_name: repo.full_name,
        path,
    })
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

fn run_view(name: &str, global: &GlobalArgs, printer: &Printer) -> Result<(), CliError> {
    let config = Config::load()?;
    let host = config.resolve_host(global.host.as_deref());
    let client = Client::for_host(&config, &host, &TokenStore::new()?)?;
    let repo = runtime()?.block_on(RepoService::new(&client).view(name))?;
    printer.emit(&RepoDetail(repo))
}

fn runtime() -> Result<tokio::runtime::Runtime, CliError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Generic(format!("cannot create async runtime: {e}")))
}

/// Output of `repo create`: serializes as the created repository resource.
#[derive(Serialize)]
#[serde(transparent)]
struct RepoCreated(Repository);

impl HumanRender for RepoCreated {
    fn render(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(out, "Created {}.", self.0.full_name)
    }
}

/// Output of `repo list`: serializes as an array of repository resources.
#[derive(Serialize)]
#[serde(transparent)]
struct RepoList(Vec<Repository>);

impl HumanRender for RepoList {
    fn render(&self, out: &mut dyn Write) -> io::Result<()> {
        let rows: Vec<Vec<String>> = self
            .0
            .iter()
            .map(|repo| {
                vec![
                    repo.full_name.clone(),
                    repo.visibility.to_string(),
                    repo.description.clone().unwrap_or_default(),
                ]
            })
            .collect();
        output::write_table(out, &["NAME", "VISIBILITY", "DESCRIPTION"], &rows)
    }
}

/// Output of `repo view`: serializes as the repository resource.
#[derive(Serialize)]
#[serde(transparent)]
struct RepoDetail(Repository);

/// Width of the key column in the detail view, sized to its longest label.
const DETAIL_KEY_WIDTH: usize = 14;

impl HumanRender for RepoDetail {
    fn render(&self, out: &mut dyn Write) -> io::Result<()> {
        let repo = &self.0;
        output::write_field(out, "name", DETAIL_KEY_WIDTH, &repo.full_name)?;
        output::write_field(
            out,
            "visibility",
            DETAIL_KEY_WIDTH,
            &repo.visibility.to_string(),
        )?;
        output::write_field(out, "clone (https)", DETAIL_KEY_WIDTH, &repo.clone_url)?;
        output::write_field(out, "clone (ssh)", DETAIL_KEY_WIDTH, &repo.ssh_url)?;
        if let Some(desc) = &repo.description {
            output::write_field(out, "description", DETAIL_KEY_WIDTH, desc)?;
        }
        output::write_field(out, "created", DETAIL_KEY_WIDTH, &repo.created_at)
    }
}

/// Output of `repo clone`: the cloned repository and its local path.
#[derive(Serialize)]
struct CloneOutcome {
    /// Fully qualified `namespace/name` handle.
    full_name: String,
    /// Absolute path of the local clone.
    path: String,
}

impl HumanRender for CloneOutcome {
    fn render(&self, out: &mut dyn Write) -> io::Result<()> {
        writeln!(out, "Cloned {}.", self.full_name)
    }
}

/// Resolves the absolute local path a clone lands in: the explicit
/// destination when given, otherwise the directory git derives from the
/// clone URL.
fn resolve_clone_path(url: &str, dest: Option<&Path>) -> Result<String, CliError> {
    let dest = dest.map_or_else(|| PathBuf::from(humanish_name(url)), Path::to_path_buf);
    let absolute = if dest.is_absolute() {
        dest
    } else {
        std::env::current_dir()
            .map_err(|e| CliError::Generic(format!("cannot resolve working directory: {e}")))?
            .join(dest)
    };
    Ok(absolute.to_string_lossy().into_owned())
}

/// Returns the directory name git derives from a clone URL: the last path
/// segment with a trailing `.git` suffix removed.
fn humanish_name(url: &str) -> &str {
    let trimmed = url.trim_end_matches('/');
    let last = trimmed.rsplit('/').next().unwrap_or(trimmed);
    last.strip_suffix(".git").unwrap_or(last)
}

#[cfg(test)]
mod tests {
    use super::humanish_name;

    #[test]
    fn humanish_name_strips_path_and_git_suffix() {
        assert_eq!(
            humanish_name("https://git.nubster.com/ns/test-repo.git"),
            "test-repo"
        );
        assert_eq!(
            humanish_name("ssh://git@git.nubster.com/ns/test-repo.git"),
            "test-repo"
        );
        assert_eq!(humanish_name("https://host/ns/repo/"), "repo");
        assert_eq!(humanish_name("repo.git"), "repo");
    }
}
