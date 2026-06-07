//! SCM repository types and service for the control-plane REST API.

use serde::{Deserialize, Serialize};

use crate::api::client::Client;
use crate::error::CliError;

/// Visibility of a repository.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Accessible to anyone with the URL.
    Public,
    /// Accessible only to members of the owning namespace.
    Private,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Private => write!(f, "private"),
        }
    }
}

/// Request body for creating a repository.
#[derive(Debug, Serialize)]
pub struct CreateRepoRequest {
    /// Short name of the repository.
    pub name: String,
    /// Optional human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Access visibility.
    pub visibility: Visibility,
}

/// Repository resource returned by the platform.
#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    /// Fully qualified `namespace/name` handle.
    pub full_name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Access visibility.
    pub visibility: Visibility,
    /// HTTPS clone URL.
    pub clone_url: String,
    /// SSH clone URL.
    pub ssh_url: String,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
}

/// Service for repository operations over the SCM REST API.
pub struct RepoService<'a> {
    client: &'a Client,
}

impl<'a> RepoService<'a> {
    /// Creates a service backed by `client`.
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Creates a repository.
    ///
    /// # Errors
    /// Returns [`CliError`] on a transport failure or a non-success platform response.
    pub async fn create(&self, req: &CreateRepoRequest) -> Result<Repository, CliError> {
        self.client.post_json("scm/repos", req).await
    }

    /// Returns all repositories visible to the authenticated principal.
    ///
    /// # Errors
    /// Returns [`CliError`] on a transport failure or a non-success platform response.
    pub async fn list(&self) -> Result<Vec<Repository>, CliError> {
        self.client.get_json("scm/repos").await
    }

    /// Returns the repository identified by `name` (`namespace/repo` or bare `repo`).
    ///
    /// # Errors
    /// Returns [`CliError`] on a transport failure or a non-success platform response.
    pub async fn view(&self, name: &str) -> Result<Repository, CliError> {
        self.client.get_json(&format!("scm/repos/{name}")).await
    }
}
