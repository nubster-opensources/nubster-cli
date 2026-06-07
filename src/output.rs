//! Output formatting: human-readable rendering and stable JSON emission.

use std::io::{self, Write};

use serde::Serialize;

use crate::cli::GlobalArgs;
use crate::error::CliError;

/// How command results are written to stdout.
#[derive(Clone, Copy, Debug)]
enum OutputMode {
    /// Human-readable text, optionally styled.
    Human,
    /// Machine-readable JSON, never styled.
    Json,
}

/// Renders a value as human-readable text.
///
/// Implementations may emit ANSI styling through [`anstyle`]; the printer
/// writes through a stream that strips styling when colors are disabled.
pub trait HumanRender {
    /// Writes the human representation of `self` to `out`.
    ///
    /// # Errors
    /// Returns an error when writing to `out` fails.
    fn render(&self, out: &mut dyn Write) -> io::Result<()>;
}

/// Writes command results to stdout in the mode selected by global flags.
pub struct Printer {
    mode: OutputMode,
    colors: anstream::ColorChoice,
}

impl Printer {
    /// Builds a printer from the global CLI flags.
    #[must_use]
    pub fn new(global: &GlobalArgs) -> Self {
        Self {
            mode: if global.json {
                OutputMode::Json
            } else {
                OutputMode::Human
            },
            colors: if global.no_color {
                anstream::ColorChoice::Never
            } else {
                anstream::ColorChoice::Auto
            },
        }
    }

    /// Emits `value` on stdout: a single line of JSON in `--json` mode,
    /// human-readable text otherwise.
    ///
    /// # Errors
    /// Returns [`CliError::Generic`] when serialization or writing fails.
    pub fn emit<T: HumanRender + Serialize>(&self, value: &T) -> Result<(), CliError> {
        match self.mode {
            OutputMode::Json => {
                let json = serde_json::to_string(value)
                    .map_err(|e| CliError::Generic(format!("cannot serialize output: {e}")))?;
                writeln!(io::stdout(), "{json}")
            }
            OutputMode::Human => {
                let stdout = io::stdout();
                let mut stream = anstream::AutoStream::new(stdout.lock(), self.colors);
                value.render(&mut stream)
            }
        }
        .map_err(|e| CliError::Generic(format!("cannot write output: {e}")))
    }
}

/// Writes `rows` as two-space separated columns aligned under a bold header
/// row. The last column is left unpadded to avoid trailing whitespace.
///
/// # Errors
/// Returns an error when writing to `out` fails.
pub fn write_table(out: &mut dyn Write, headers: &[&str], rows: &[Vec<String>]) -> io::Result<()> {
    const HEADER: anstyle::Style = anstyle::Style::new().bold();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.len());
        }
    }
    write!(out, "{HEADER}")?;
    write_cells(out, headers, &widths)?;
    writeln!(out, "{HEADER:#}")?;
    for row in rows {
        let cells: Vec<&str> = row.iter().map(String::as_str).collect();
        write_cells(out, &cells, &widths)?;
        writeln!(out)?;
    }
    Ok(())
}

/// Writes a `key: value` line with the key rendered bold and the value
/// aligned at `width` characters.
///
/// # Errors
/// Returns an error when writing to `out` fails.
pub fn write_field(out: &mut dyn Write, key: &str, width: usize, value: &str) -> io::Result<()> {
    const KEY: anstyle::Style = anstyle::Style::new().bold();
    let pad = width.saturating_sub(key.len() + 1);
    writeln!(out, "{KEY}{key}:{KEY:#}{:pad$} {value}", "")
}

fn write_cells(out: &mut dyn Write, cells: &[&str], widths: &[usize]) -> io::Result<()> {
    for (i, (cell, &width)) in cells.iter().zip(widths).enumerate() {
        if i > 0 {
            write!(out, "  ")?;
        }
        if i + 1 == cells.len() {
            write!(out, "{cell}")?;
        } else {
            write!(out, "{cell:<width$}")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{write_field, write_table};

    #[test]
    fn write_table_aligns_columns_and_skips_last_padding() {
        let mut buf = Vec::new();
        write_table(
            &mut buf,
            &["NAME", "VISIBILITY"],
            &[
                vec!["ns/a".to_owned(), "private".to_owned()],
                vec!["ns/longer-name".to_owned(), "public".to_owned()],
            ],
        )
        .expect("write table");
        let text = String::from_utf8(buf).expect("utf8 output");
        assert!(text.contains("NAME            VISIBILITY"), "{text:?}");
        assert!(text.contains("ns/a            private\n"), "{text:?}");
        assert!(text.contains("ns/longer-name  public\n"), "{text:?}");
    }

    #[test]
    fn write_table_styles_the_header_row() {
        let mut buf = Vec::new();
        write_table(&mut buf, &["NAME"], &[]).expect("write table");
        let text = String::from_utf8(buf).expect("utf8 output");
        assert!(text.starts_with("\u{1b}["), "{text:?}");
    }

    #[test]
    fn write_field_pads_the_key_column() {
        let mut buf = Vec::new();
        write_field(&mut buf, "name", 14, "ns/a").expect("write field");
        let text = String::from_utf8(buf).expect("utf8 output");
        assert!(text.contains("name:"), "{text:?}");
        assert!(text.ends_with("          ns/a\n"), "{text:?}");
    }
}
