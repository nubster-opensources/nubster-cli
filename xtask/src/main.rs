//! `xtask`: workspace tooling entry point.
//!
//! Invoke through the alias declared in `.cargo/config.toml`:
//!
//! ```sh
//! cargo xtask help
//! cargo xtask doc-gen cli [--check]
//! cargo xtask doc-gen man [--check]
//! ```

use anyhow::{bail, Result};

mod doc_gen;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("doc-gen") => doc_gen::cmd(&args[1..]),
        Some("help") | None => {
            print_help();
            Ok(())
        }
        Some(other) => bail!("unknown task: {other}"),
    }
}

fn print_help() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  doc-gen <target>   Generate reference pages (targets: cli, man)");
    eprintln!("  help               Show this message");
}
