//! Compiles a repository's working tree into a model of the system: the
//! build model of the project in one directory, written to `.mental/`.
#![deny(clippy::wildcard_enum_match_arm)]

mod build;
mod cli;
mod repo;

use std::io::{self, Write};
use std::path::Path;

use anyhow::Context;

pub use cli::{Cli, Command};

/// How a command that ran ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// No error-level diagnostic: exit code 0.
    Success,
    /// At least one error-level diagnostic: exit code 1.
    Failure,
}

impl Status {
    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            Status::Success => 0,
            Status::Failure => 1,
        }
    }
}

/// Runs `cli` on the project in `dir` and prints to `out`. A closed `out`,
/// as when a pipe's reader quits, is not an error.
///
/// # Errors
///
/// When the tool itself fails: a file it cannot read or write.
pub fn run(cli: &Cli, dir: &Path, out: &mut dyn Write) -> anyhow::Result<Status> {
    let Command::Build { json } = cli.command;
    let tree = repo::read(dir)?;
    let built = build::build(&tree.files());
    repo::write(dir, &built.files)?;
    let printed = if json {
        cli::print_json(out, &built.diagnostics)
    } else {
        cli::print_text(out, &built)
    };
    match printed {
        Err(error) if error.kind() != io::ErrorKind::BrokenPipe => {
            Err(error).context("cannot write the output")
        }
        Ok(()) | Err(_) => Ok(built.status()),
    }
}
