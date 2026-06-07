//! Git integration: host resolution, cloning, and the credential helper.

use std::path::Path;
use std::process::Command;

use crate::error::CliError;

pub mod credential;

/// Returns whether a credential helper is registered for `git_host` in the
/// user's global git configuration.
///
/// # Errors
/// Returns [`CliError`] when git cannot be invoked.
pub fn is_helper_configured(git_host: &str) -> Result<bool, CliError> {
    let key = format!("credential.https://{git_host}.helper");
    let output = Command::new("git")
        .args(["config", "--global", "--get", &key])
        .output()
        .map_err(|e| CliError::Generic(format!("cannot run git: {e}")))?;
    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Transports git may use when cloning; anything else (such as `ext::`,
/// which spawns arbitrary commands) is refused before initialization.
const ALLOWED_CLONE_PROTOCOLS: &str = "file:https:ssh";

/// Runs `git clone -- <url> [dest]`, streaming git's own progress output to
/// the user's terminal. The `--` separator keeps a hostile URL or destination
/// from being parsed as a git flag, and the transport allow-list keeps a
/// hostile URL from selecting a command-executing scheme.
///
/// # Errors
/// Returns [`CliError::GitCommand`] when the clone fails, or a generic error
/// when git cannot be invoked.
pub fn clone_repo(url: &str, dest: Option<&Path>) -> Result<(), CliError> {
    let mut command = Command::new("git");
    command.env("GIT_ALLOW_PROTOCOL", ALLOWED_CLONE_PROTOCOLS);
    command.args(["clone", "--", url]);
    if let Some(dest) = dest {
        command.arg(dest);
    }
    let status = command
        .status()
        .map_err(|e| CliError::Generic(format!("cannot run git: {e}")))?;
    if status.success() {
        return Ok(());
    }
    Err(CliError::GitCommand {
        code: status.code(),
        context: "clone".to_owned(),
    })
}

/// Resolves the git host by precedence: an explicit override, then the value
/// stored in configuration, then the built-in default derived from the API
/// host.
#[must_use]
pub fn resolve_git_host(explicit: Option<&str>, stored: Option<&str>, api_host: &str) -> String {
    explicit
        .or(stored)
        .map_or_else(|| default_git_host(api_host), ToOwned::to_owned)
}

/// Returns the built-in default git host for an API host: `api.X` becomes
/// `git.X`, otherwise the API host is used unchanged.
fn default_git_host(api_host: &str) -> String {
    api_host
        .strip_prefix("api.")
        .map_or_else(|| api_host.to_owned(), |rest| format!("git.{rest}"))
}

#[cfg(test)]
mod tests {
    use super::{default_git_host, resolve_git_host};

    #[test]
    fn default_git_host_rewrites_api_prefix() {
        assert_eq!(default_git_host("api.nubster.com"), "git.nubster.com");
        assert_eq!(default_git_host("api.example.test"), "git.example.test");
        assert_eq!(default_git_host("nubster.com"), "nubster.com");
    }

    #[test]
    fn resolve_git_host_follows_precedence() {
        assert_eq!(
            resolve_git_host(Some("override.example"), Some("stored.example"), "api.x"),
            "override.example"
        );
        assert_eq!(
            resolve_git_host(None, Some("stored.example"), "api.x"),
            "stored.example"
        );
        assert_eq!(resolve_git_host(None, None, "api.x"), "git.x");
    }
}
