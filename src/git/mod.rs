//! Git integration: host resolution and the credential helper.

pub mod credential;

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
