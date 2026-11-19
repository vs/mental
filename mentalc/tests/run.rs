//! The failure paths of `run`, each forced on a real temporary directory or
//! a failing output, as the CLI cannot reach them reliably.

use std::fs;
use std::io::{self, Write};

use mentalc::{Cli, Command, Status, run};
use tempfile::TempDir;

/// A project with every build file, and `prepare` applied to its directory.
fn project(prepare: impl Fn(&TempDir)) -> TempDir {
    let dir = TempDir::new().expect("a temporary directory can be made");
    let files = [
        ("pyproject.toml", "[project]\nname = \"shop\"\n"),
        ("uv.lock", "version = 1\n"),
        (".python-version", "3.12\n"),
    ];
    for (name, text) in files {
        fs::write(dir.path().join(name), text).expect("the directory is writable");
    }
    prepare(&dir);
    dir
}

/// `mentalc build` on `dir`, printing to `out`: the status, or what failed
/// without the system's own words for it, which differ by platform.
#[track_caller]
fn check(dir: &TempDir, out: &mut dyn Write) -> String {
    let cli = Cli {
        command: Command::Build { json: false },
    };
    match run(&cli, dir.path(), out) {
        Ok(status) => format!("{status:?}"),
        Err(error) => error.to_string(),
    }
}

/// Fails every write with `kind`.
struct Closed(io::ErrorKind);

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(self.0.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_reader_that_quits_is_no_failure() {
    let dir = project(|_| ());
    assert_eq!(
        check(&dir, &mut Closed(io::ErrorKind::BrokenPipe)),
        "Failure"
    );
}

#[test]
fn output_that_cannot_be_written_fails_the_tool() {
    let dir = project(|_| ());
    assert_eq!(
        check(&dir, &mut Closed(io::ErrorKind::StorageFull)),
        "cannot write the output"
    );
}

#[test]
fn a_status_is_its_exit_code() {
    assert_eq!([Status::Success, Status::Failure].map(Status::code), [0, 1]);
}

#[test]
fn an_environment_that_cannot_be_read_fails_the_tool() {
    let dir = project(|dir| fs::create_dir_all(dir.path().join(".venv/pyvenv.cfg")).unwrap());
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot read `.venv/pyvenv.cfg`; make it a readable UTF-8 file, or remove it"
    );
}

#[test]
fn a_model_folder_that_cannot_be_made_fails_the_tool() {
    let dir = project(|dir| fs::write(dir.path().join(".mental"), "").unwrap());
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot make the folder of `.mental/model/format.json`; check that the project directory is writable"
    );
}

#[test]
fn a_file_that_cannot_be_written_fails_the_tool() {
    let dir =
        project(|dir| fs::create_dir_all(dir.path().join(".mental/model/format.partial")).unwrap());
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot write `.mental/model/format.json`; check that the project directory is writable"
    );
}

#[test]
fn a_file_that_cannot_be_replaced_fails_the_tool() {
    let dir = project(|dir| {
        fs::create_dir_all(dir.path().join(".mental/model/format.json/kept")).unwrap();
    });
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot replace `.mental/model/format.json`; check that the project directory is writable"
    );
}
