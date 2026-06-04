//! Git credential helper and host setup.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::process::Command;

use crate::auth::token_store::TokenStore;
use crate::config::Config;
use crate::error::CliError;

/// Username advertised to git; the personal access token travels as the
/// password and is validated by the platform through introspection.
const GIT_USERNAME: &str = "x-access-token";

/// Operation requested by git on the credential helper, passed as the single
/// command argument.
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CredentialOp {
    /// Provide credentials for a host.
    Get,
    /// Persist credentials that were used successfully.
    Store,
    /// Discard credentials that were rejected.
    Erase,
}

/// Registers `nub` as the git credential helper for the host's git endpoint,
/// writing to the user's global git configuration.
///
/// # Errors
/// Returns [`CliError`] when the executable path cannot be resolved or the
/// `git config` invocation fails.
pub fn setup_git(config: &Config, host: &str) -> Result<(), CliError> {
    let stored = config.hosts.get(host).and_then(|h| h.git_host.as_deref());
    let git_host = super::resolve_git_host(None, stored, host);
    let value = helper_command(host)?;
    let key = format!("credential.https://{git_host}.helper");

    let status = Command::new("git")
        .args(["config", "--global", &key, &value])
        .status()
        .map_err(|e| CliError::Generic(format!("cannot run git: {e}")))?;
    if !status.success() {
        return Err(CliError::Generic(format!(
            "git config failed with status {status}"
        )));
    }

    println!("Configured git credential helper for {git_host}.");
    Ok(())
}

/// Runs the git-credential protocol for `operation`, reading the request from
/// `input` and writing any response to `output`. `Get` answers with the stored
/// token; `Store` and `Erase` consume the request and succeed without side
/// effects.
///
/// # Errors
/// Returns [`CliError`] on a malformed request, a token backend failure, or an
/// output failure.
pub fn run_credential<R: BufRead, W: Write>(
    operation: &CredentialOp,
    host: &str,
    store: &TokenStore,
    input: R,
    mut output: W,
) -> Result<(), CliError> {
    parse_request(input)?;
    match operation {
        CredentialOp::Get => {
            if let Some(token) = store.get(host)? {
                writeln!(output, "username={GIT_USERNAME}")
                    .and_then(|()| writeln!(output, "password={token}"))
                    .map_err(|e| CliError::Generic(format!("cannot write credentials: {e}")))?;
            }
            Ok(())
        }
        CredentialOp::Store | CredentialOp::Erase => Ok(()),
    }
}

/// Builds the credential helper command value stored in git configuration.
fn helper_command(host: &str) -> Result<String, CliError> {
    let exe = std::env::current_exe()
        .map_err(|e| CliError::Generic(format!("cannot resolve executable path: {e}")))?;
    let exe = exe.to_string_lossy().replace('\\', "/");
    let exe = if exe.contains(' ') {
        format!("\"{exe}\"")
    } else {
        exe
    };
    Ok(format!("{exe} auth git-credential --host {host}"))
}

/// Parses a git-credential request: `key=value` lines until a blank line or
/// end of input.
fn parse_request<R: BufRead>(input: R) -> Result<BTreeMap<String, String>, CliError> {
    let mut fields = BTreeMap::new();
    for line in input.lines() {
        let line = line.map_err(|e| CliError::Generic(format!("cannot read request: {e}")))?;
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once('=') {
            fields.insert(key.to_owned(), value.to_owned());
        }
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::parse_request;

    #[test]
    fn parse_request_reads_until_blank_line() {
        let input = b"protocol=https\nhost=git.nubster.com\n\nignored=after-blank\n";
        let fields = parse_request(&input[..]).expect("parse");
        assert_eq!(fields.get("protocol").map(String::as_str), Some("https"));
        assert_eq!(
            fields.get("host").map(String::as_str),
            Some("git.nubster.com")
        );
        assert!(!fields.contains_key("ignored"));
    }

    #[test]
    fn parse_request_tolerates_values_with_equals() {
        let input = b"path=a=b=c\n\n";
        let fields = parse_request(&input[..]).expect("parse");
        assert_eq!(fields.get("path").map(String::as_str), Some("a=b=c"));
    }
}
