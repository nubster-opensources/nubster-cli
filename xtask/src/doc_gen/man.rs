//! Generation of man pages from the `clap` command tree.
//!
//! `doc-gen man` reads the `nub` command definitions through
//! [`clap::CommandFactory`], then emits one roff man page per command and
//! subcommand under `man/`. The output is checked in and verified fresh by CI,
//! so the man pages can never drift from the parser.

use anyhow::{Context, Result};
use clap::CommandFactory;

use super::{page, write_or_check, GeneratedPage};

/// Directory holding the generated man pages.
const MAN_DIR: &str = "man";

/// Generate (or verify with `--check`) the man pages.
pub(super) fn man(check: bool) -> Result<()> {
    let root = nub::cli::Cli::command();
    let mut pages: Vec<GeneratedPage> = Vec::new();
    collect_man_pages(&root, &[], &mut pages)?;
    write_or_check("man", MAN_DIR, &pages, check)
}

/// Recursively collect man pages for `cmd` and all its subcommands.
///
/// `ancestors` holds the chain of parent command names (empty for the root).
fn collect_man_pages(
    cmd: &clap::Command,
    ancestors: &[&str],
    pages: &mut Vec<GeneratedPage>,
) -> Result<()> {
    let name = cmd.get_name();

    // Build the full hyphenated name: e.g. "nub-auth-login".
    let full_name = if ancestors.is_empty() {
        name.to_owned()
    } else {
        format!("{}-{name}", ancestors.join("-"))
    };

    // Render this command into a roff buffer.
    let mut buf: Vec<u8> = Vec::new();
    clap_mangen::Man::new(cmd.clone())
        .render(&mut buf)
        .with_context(|| format!("failed to render man page for `{full_name}`"))?;
    let body = String::from_utf8(buf)
        .with_context(|| format!("man page for `{full_name}` is not valid UTF-8"))?;

    let file_name = format!("{full_name}.1");
    pages.push(page(MAN_DIR, &file_name, body));

    // Build the ancestor chain for children.
    let mut child_ancestors: Vec<&str> = ancestors.to_vec();
    child_ancestors.push(name);

    for sub in cmd
        .get_subcommands()
        .filter(|s| !s.is_hide_set() && s.get_name() != "help")
    {
        collect_man_pages(sub, &child_ancestors, pages)?;
    }

    Ok(())
}
