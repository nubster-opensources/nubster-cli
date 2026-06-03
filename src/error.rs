use std::fmt;

/// Errors surfaced by the CLI, each mapped to a process exit code.
#[derive(Debug)]
pub enum CliError {
    /// The requested command exists but is not implemented yet.
    NotImplemented(&'static str),
    /// No valid credentials are available for the target host.
    NotAuthenticated,
    /// A transport-level failure reaching the platform (connection, timeout).
    Network(String),
    /// The platform returned an unsuccessful HTTP status.
    Api {
        /// HTTP status code returned by the platform.
        status: u16,
        /// Message extracted from the response body.
        message: String,
    },
    /// A generic, message-carrying failure.
    Generic(String),
}

impl CliError {
    /// Returns the process exit code associated with this error.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Generic(_) => 1,
            Self::NotImplemented(_) => 3,
            Self::NotAuthenticated => 4,
            Self::Network(_) => 5,
            Self::Api { .. } => 6,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "`{what}` is not implemented yet"),
            Self::NotAuthenticated => write!(f, "not authenticated; run `nub auth login`"),
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::Api { status, message } => write!(f, "API error ({status}): {message}"),
            Self::Generic(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::CliError;

    #[test]
    fn error_exit_codes_are_stable() {
        assert_eq!(CliError::Generic(String::new()).exit_code(), 1);
        assert_eq!(CliError::NotImplemented("x").exit_code(), 3);
        assert_eq!(CliError::NotAuthenticated.exit_code(), 4);
        assert_eq!(CliError::Network(String::new()).exit_code(), 5);
        assert_eq!(
            CliError::Api {
                status: 500,
                message: String::new()
            }
            .exit_code(),
            6
        );
    }
}
