//! Per-host token storage.
//!
//! Wired into commands starting with issue #5; until then the API is plumbing,
//! hence the module-wide `dead_code` allowance.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::config_dir;
use crate::error::CliError;

/// Service name used for OS keychain entries.
const KEYRING_SERVICE: &str = "com.nubster.nub";
/// Environment variable forcing the file backend (no keychain).
const NO_KEYCHAIN_ENV: &str = "NUB_NO_KEYCHAIN";
/// File name of the fallback credentials store.
const CREDENTIALS_FILE: &str = "credentials.toml";

/// Stores and retrieves per-host access tokens.
pub struct TokenStore {
    backend: Backend,
}

enum Backend {
    Keyring,
    File(PathBuf),
}

/// On-disk fallback layout (host to token).
#[derive(Default, Serialize, Deserialize)]
struct CredentialsFile {
    #[serde(default)]
    tokens: BTreeMap<String, String>,
}

impl TokenStore {
    /// Builds a store using the OS keychain, or the file backend when the
    /// keychain is opted out via `NUB_NO_KEYCHAIN`.
    ///
    /// # Errors
    /// Returns [`CliError`] if the fallback path cannot be resolved.
    pub fn new() -> Result<Self, CliError> {
        if std::env::var_os(NO_KEYCHAIN_ENV).is_some() {
            return Ok(Self::file_at(Self::credentials_path()?));
        }
        Ok(Self {
            backend: Backend::Keyring,
        })
    }

    fn file_at(path: PathBuf) -> Self {
        Self {
            backend: Backend::File(path),
        }
    }

    fn credentials_path() -> Result<PathBuf, CliError> {
        Ok(config_dir()?.join(CREDENTIALS_FILE))
    }

    /// Returns the stored token for `host`, if any.
    ///
    /// # Errors
    /// Returns [`CliError`] on a keychain or file backend failure.
    pub fn get(&self, host: &str) -> Result<Option<String>, CliError> {
        match &self.backend {
            Backend::Keyring => match keyring_entry(host)?.get_password() {
                Ok(token) => Ok(Some(token)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(keychain_error(&e)),
            },
            Backend::File(path) => Ok(read_credentials(path)?.tokens.get(host).cloned()),
        }
    }

    /// Stores (or replaces) the token for `host`.
    ///
    /// # Errors
    /// Returns [`CliError`] on a keychain or file backend failure.
    pub fn set(&self, host: &str, token: &str) -> Result<(), CliError> {
        match &self.backend {
            Backend::Keyring => keyring_entry(host)?
                .set_password(token)
                .map_err(|e| keychain_error(&e)),
            Backend::File(path) => {
                let mut file = read_credentials(path)?;
                file.tokens.insert(host.to_owned(), token.to_owned());
                write_credentials(path, &file)
            }
        }
    }

    /// Removes the token for `host`. Absent hosts are not an error.
    ///
    /// # Errors
    /// Returns [`CliError`] on a keychain or file backend failure.
    pub fn delete(&self, host: &str) -> Result<(), CliError> {
        match &self.backend {
            Backend::Keyring => match keyring_entry(host)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(keychain_error(&e)),
            },
            Backend::File(path) => {
                let mut file = read_credentials(path)?;
                if file.tokens.remove(host).is_some() {
                    write_credentials(path, &file)?;
                }
                Ok(())
            }
        }
    }
}

fn keyring_entry(host: &str) -> Result<keyring::Entry, CliError> {
    keyring::Entry::new(KEYRING_SERVICE, host).map_err(|e| keychain_error(&e))
}

fn keychain_error(error: &keyring::Error) -> CliError {
    CliError::Generic(format!("keychain error: {error}"))
}

fn read_credentials(path: &Path) -> Result<CredentialsFile, CliError> {
    match std::fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).map_err(|e| {
            CliError::Generic(format!("invalid credentials at {}: {e}", path.display()))
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(CredentialsFile::default()),
        Err(e) => Err(CliError::Generic(format!(
            "cannot read credentials at {}: {e}",
            path.display()
        ))),
    }
}

fn write_credentials(path: &Path, file: &CredentialsFile) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CliError::Generic(format!("cannot create {}: {e}", parent.display())))?;
    }
    let text = toml::to_string_pretty(file)
        .map_err(|e| CliError::Generic(format!("cannot serialize credentials: {e}")))?;
    std::fs::write(path, &text).map_err(|e| {
        CliError::Generic(format!(
            "cannot write credentials at {}: {e}",
            path.display()
        ))
    })?;
    set_owner_only_permissions(path)
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> Result<(), CliError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| CliError::Generic(format!("cannot secure {}: {e}", path.display())))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)] // Signature must mirror the Unix variant, which can fail.
fn set_owner_only_permissions(_path: &Path) -> Result<(), CliError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TokenStore;

    #[test]
    fn set_get_delete_roundtrip_on_file_backend() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = TokenStore::file_at(dir.path().join("credentials.toml"));

        assert_eq!(store.get("api.nubster.com").expect("get"), None);

        store.set("api.nubster.com", "tok_123").expect("set");
        assert_eq!(
            store.get("api.nubster.com").expect("get").as_deref(),
            Some("tok_123")
        );

        store.set("api.nubster.com", "tok_456").expect("overwrite");
        assert_eq!(
            store.get("api.nubster.com").expect("get").as_deref(),
            Some("tok_456")
        );

        store.delete("api.nubster.com").expect("delete");
        assert_eq!(store.get("api.nubster.com").expect("get"), None);
        store
            .delete("absent.example")
            .expect("delete absent is a no-op");
    }

    #[cfg(unix)]
    #[test]
    fn credentials_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("credentials.toml");

        TokenStore::file_at(path.clone())
            .set("h", "t")
            .expect("set");

        let mode = std::fs::metadata(&path).expect("meta").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
