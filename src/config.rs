use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::CliError;

/// Built-in default API host used when nothing is configured.
pub const DEFAULT_API_HOST: &str = "api.nubster.com";

/// Environment variable overriding the configuration directory.
const CONFIG_DIR_ENV: &str = "NUB_CONFIG_DIR";

/// Per-host settings. Never contains secrets, which live in the OS keychain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    /// Base URL of the control-plane API for this host.
    pub api_url: String,
    /// Git host used for clone and push URLs, when it differs from the API host.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_host: Option<String>,
}

/// Non-secret user configuration, persisted as TOML.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Host selected when `--host` is not provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_host: Option<String>,
    /// Known hosts and their settings.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hosts: BTreeMap<String, HostConfig>,
}

impl Config {
    /// Returns the configuration file path.
    ///
    /// Honors `NUB_CONFIG_DIR` first, then the OS-specific config directory.
    ///
    /// # Errors
    /// Returns [`CliError`] if no configuration directory can be determined.
    pub fn path() -> Result<PathBuf, CliError> {
        Ok(config_dir()?.join("config.toml"))
    }

    /// Loads the configuration, returning defaults when the file is absent.
    ///
    /// # Errors
    /// Returns [`CliError`] if the file exists but cannot be read or parsed.
    pub fn load() -> Result<Self, CliError> {
        Self::load_from(&Self::path()?)
    }

    /// Persists the configuration to disk, creating parent directories.
    ///
    /// # Errors
    /// Returns [`CliError`] if the file cannot be serialized or written.
    #[allow(dead_code)] // Exercised by tests; first runtime caller lands with `auth login`.
    pub fn save(&self) -> Result<(), CliError> {
        self.save_to(&Self::path()?)
    }

    /// Returns the effective host: `--host` override, else the default, else the built-in.
    #[must_use]
    pub fn resolve_host(&self, override_host: Option<&str>) -> String {
        override_host
            .map(ToOwned::to_owned)
            .or_else(|| self.default_host.clone())
            .unwrap_or_else(|| DEFAULT_API_HOST.to_owned())
    }

    fn load_from(path: &Path) -> Result<Self, CliError> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| {
                CliError::Generic(format!("invalid config at {}: {e}", path.display()))
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(CliError::Generic(format!(
                "cannot read config at {}: {e}",
                path.display()
            ))),
        }
    }

    fn save_to(&self, path: &Path) -> Result<(), CliError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CliError::Generic(format!("cannot create {}: {e}", parent.display()))
            })?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|e| CliError::Generic(format!("cannot serialize config: {e}")))?;
        std::fs::write(path, text).map_err(|e| {
            CliError::Generic(format!("cannot write config at {}: {e}", path.display()))
        })
    }
}

/// Resolves the configuration directory (env override first, then OS default).
pub(crate) fn config_dir() -> Result<PathBuf, CliError> {
    if let Some(dir) = std::env::var_os(CONFIG_DIR_ENV) {
        return Ok(PathBuf::from(dir));
    }
    directories::ProjectDirs::from("com", "Nubster", "nub")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| CliError::Generic("cannot determine a configuration directory".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{Config, HostConfig, DEFAULT_API_HOST};
    use std::collections::BTreeMap;

    #[test]
    fn resolve_host_prefers_override_then_default_then_builtin() {
        let mut config = Config::default();
        assert_eq!(config.resolve_host(None), DEFAULT_API_HOST);
        assert_eq!(config.resolve_host(Some("cli.example")), "cli.example");

        config.default_host = Some("default.example".to_owned());
        assert_eq!(config.resolve_host(None), "default.example");
        assert_eq!(config.resolve_host(Some("cli.example")), "cli.example");
    }

    #[test]
    fn save_then_load_roundtrips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");

        let mut hosts = BTreeMap::new();
        hosts.insert(
            "api.nubster.com".to_owned(),
            HostConfig {
                api_url: "https://api.nubster.com".to_owned(),
                git_host: Some("git.nubster.com".to_owned()),
            },
        );
        let original = Config {
            default_host: Some("api.nubster.com".to_owned()),
            hosts,
        };

        original.save_to(&path).expect("save");
        let loaded = Config::load_from(&path).expect("load");

        assert_eq!(loaded.default_host.as_deref(), Some("api.nubster.com"));
        assert_eq!(
            loaded
                .hosts
                .get("api.nubster.com")
                .map(|h| h.api_url.as_str()),
            Some("https://api.nubster.com")
        );
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("does-not-exist.toml");
        let loaded = Config::load_from(&path).expect("load");
        assert!(loaded.default_host.is_none());
        assert!(loaded.hosts.is_empty());
    }
}
