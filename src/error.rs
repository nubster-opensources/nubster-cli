use std::fmt;

/// Errors surfaced by the CLI, each mapped to a process exit code.
#[derive(Debug)]
pub enum CliError {
    /// The requested command exists but is not implemented yet.
    NotImplemented(&'static str),
    /// No valid credentials are available for the target host.
    NotAuthenticated,
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
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "`{what}` is not implemented yet"),
            Self::NotAuthenticated => write!(f, "not authenticated; run `nub auth login`"),
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
    }
}
