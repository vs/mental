//! The command line and the two ways `mentalc` prints: text for people, and
//! the model's own JSON lines for agents and scripts.

use std::io::{self, Write};

use clap::{Parser, Subcommand};
use mental_model::{Diagnostic, Severity, jsonl};

use crate::build::Built;

/// Compiles a repository into a model of the system that you can run.
#[derive(Debug, Parser)]
#[command(name = "mentalc", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, Subcommand)]
pub enum Command {
    /// Builds the model of the project in the current directory into `.mental/`.
    Build {
        /// Print the diagnostics as JSON lines, one `diagnostics.jsonl` line each.
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn print_json(out: &mut dyn Write, diagnostics: &[Diagnostic]) -> io::Result<()> {
    out.write_all(jsonl::write(diagnostics).as_bytes())
}

/// One line per diagnostic, then what was written.
pub(crate) fn print_text(out: &mut dyn Write, built: &Built) -> io::Result<()> {
    for diagnostic in &built.diagnostics {
        writeln!(out, "{}", text(diagnostic))?;
    }
    if built.units == 0 {
        return writeln!(out, "wrote nothing: there is no build unit");
    }
    let (units, pins) = (built.units, built.pins);
    match &built.sites {
        None => writeln!(out, "wrote .mental/ (build units: {units}, pins: {pins})"),
        Some(sites) => {
            let (resolved, external, unresolved) =
                (sites.resolved, sites.external, sites.unresolved_total());
            let all = resolved + external + unresolved;
            writeln!(
                out,
                "wrote .mental/ (build units: {units}, pins: {pins}, sites: {all}: resolved \
                 {resolved}, external {external}, unresolved {unresolved})"
            )
        }
    }
}

/// `error[lock-missing] reports/unit:reports: the project has no ...`: the
/// severity, the code, the place or else the record, and the message.
fn text(diagnostic: &Diagnostic) -> String {
    let severity = match diagnostic.code.severity() {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    let code = serde_json::to_value(diagnostic.code).expect("a code is written as a string");
    let code = code.as_str().expect("a code is written as its name");
    let place = match (&diagnostic.at, &diagnostic.id) {
        (Some(at), _) => {
            let [first, last]: [u32; 2] = at.lines.into();
            let lines = if first == last {
                first.to_string()
            } else {
                format!("{first}-{last}")
            };
            format!(" {}:{lines}", at.path)
        }
        (None, Some(id)) => format!(" {id}"),
        (None, None) => String::new(),
    };
    format!("{severity}[{code}]{place}: {}", diagnostic.message)
}

#[cfg(test)]
mod tests;
