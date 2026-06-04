//! Nubster CLI (`nub`): unified command-line client for the Nubster platform.

use clap::Parser;

mod api;
mod auth;
mod cli;
mod commands;
mod config;
mod error;
mod git;

fn main() -> std::process::ExitCode {
    cli::run(cli::Cli::parse())
}
