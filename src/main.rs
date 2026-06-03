//! Nubster CLI (`nub`): unified command-line client for the Nubster platform.

use clap::Parser;

mod cli;
mod commands;
mod error;

fn main() -> std::process::ExitCode {
    cli::run(cli::Cli::parse())
}
