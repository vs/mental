//! `mentalc`: parses the arguments, runs the command in the current
//! directory, and reports a failure of the tool itself with exit code 2.

use std::io;
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;
use mentalc::{Cli, Status};

fn main() -> ExitCode {
    match run(&Cli::parse()) {
        Ok(status) => ExitCode::from(status.code()),
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli) -> anyhow::Result<Status> {
    let dir = std::env::current_dir().context("cannot read the current directory")?;
    mentalc::run(cli, &dir, &mut io::stdout().lock())
}
