//! Nubster CLI (`nub`): unified command-line client for the Nubster platform.

use clap::Parser;

mod cli;

fn main() -> std::process::ExitCode {
    let args = cli::Cli::parse();
    cli::run(&args)
}
