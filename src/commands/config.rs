use clap::{Args, Subcommand};

use crate::cli::GlobalArgs;
use crate::config::Config;
use crate::error::CliError;

/// Inspect the CLI configuration.
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

/// Subcommands under `nub config`.
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Print the configuration file path.
    Path,
    /// Print the current configuration and the effective host.
    Show,
}

/// Runs a `config` subcommand.
///
/// # Errors
/// Returns a [`CliError`] if the configuration cannot be located, read, or rendered.
pub fn run(args: &ConfigArgs, global: &GlobalArgs) -> Result<(), CliError> {
    match args.command {
        ConfigCommand::Path => {
            println!("{}", Config::path()?.display());
            Ok(())
        }
        ConfigCommand::Show => {
            let config = Config::load()?;
            let host = config.resolve_host(global.host.as_deref());
            println!("# effective host: {host}");
            let text = toml::to_string_pretty(&config)
                .map_err(|e| CliError::Generic(format!("cannot render config: {e}")))?;
            print!("{text}");
            Ok(())
        }
    }
}
