//! Nubster CLI (`nub`) binary entry point.

use clap::Parser;
use nub::cli::{self, Cli};

fn main() -> std::process::ExitCode {
    cli::run(Cli::parse())
}
