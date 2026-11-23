//! Compiles a repository's working tree into a model of the system: the
//! build model of the project in one directory and, when it has no gap, its
//! reference sites with their outcomes, written to `.mental/`.
#![deny(clippy::wildcard_enum_match_arm)]

mod build;
mod cli;
mod repo;

use std::io::{self, Write};
use std::path::Path;

use anyhow::Context;
use mental_extract::{DistRecord, Environment, PythonFile, Sites, build_model, extract_sites};

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
/// When the tool itself fails: a file it cannot read or write, a name that
/// is not UTF-8 or a symbolic link to source it does not follow, a project
/// directory whose path is not UTF-8, or a type checker that cannot open
/// the project or its environment.
pub fn run(cli: &Cli, dir: &Path, out: &mut dyn Write) -> anyhow::Result<Status> {
    let Command::Build { json } = cli.command;
    let tree = repo::read(dir)?;
    let members = tree.members();
    let (model, diagnostics) = build_model(&tree.files(&members));
    let stage_two = if build::ready(&diagnostics) {
        let project_dir = tree.path.as_str();
        let mut found = (
            Sites {
                sites: Vec::new(),
                anchors: Vec::new(),
            },
            Vec::new(),
        );
        for unit in &model.units {
            let others: Vec<&str> = model
                .units
                .iter()
                .filter(|other| other.id != unit.id)
                .map(|other| other.dir.as_str())
                .collect();
            let sources = repo::read_sources(dir, unit, &others)?;
            let files: Vec<PythonFile> = sources
                .files
                .iter()
                .map(|(path, text)| PythonFile { path, text })
                .collect();
            let records: Vec<DistRecord> = sources
                .records
                .iter()
                .map(|(dist_info, text)| DistRecord { dist_info, text })
                .collect();
            let environment = Environment {
                project_dir,
                records: &records,
            };
            let (sites, problems) = extract_sites(unit, &model.pins, &environment, &files)
                .with_context(|| format!("cannot resolve the reference sites of `{}`", unit.id))?;
            found.0.sites.extend(sites.sites);
            found.0.anchors.extend(sites.anchors);
            found.1.extend(problems);
        }
        Some(found)
    } else {
        None
    };
    let built = build::build(model, diagnostics, stage_two);
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
