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

/// A write that fails on a later file leaves every file of the earlier
/// build as it was, so the model is never a mix of two builds.
#[test]
fn a_failed_write_keeps_the_earlier_model() {
    let dir = project(|_| ());
    assert_eq!(check(&dir, &mut io::sink()), "Failure");
    let unit = dir.path().join(".mental/model/build_unit.jsonl");
    let earlier = fs::read_to_string(&unit).unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"store\"\n",
    )
    .unwrap();
    fs::create_dir(dir.path().join(".mental/model/pin.partial")).unwrap();
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot write `.mental/model/pin.jsonl`; check that the project directory is writable"
    );
    assert_eq!(fs::read_to_string(&unit).unwrap(), earlier);
}

#[test]
fn an_earlier_model_that_cannot_be_removed_fails_the_tool() {
    let dir = project(|dir| {
        fs::remove_file(dir.path().join("pyproject.toml")).unwrap();
        fs::create_dir_all(dir.path().join(".mental/model/format.json")).unwrap();
    });
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot remove `.mental/model/format.json`; check that the project directory is writable"
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

/// `project` with an environment as `uv sync` makes one, nothing installed,
/// and `prepare` applied after.
fn synced(prepare: impl Fn(&TempDir)) -> TempDir {
    project(|dir| {
        fs::create_dir_all(dir.path().join(".venv/lib/python3.12/site-packages"))
            .expect("the directory is writable");
        fs::write(
            dir.path().join(".venv/pyvenv.cfg"),
            "home = /usr/local/bin\nversion_info = 3.12.8\n",
        )
        .expect("the directory is writable");
        fs::create_dir(dir.path().join("src")).expect("the directory is writable");
        prepare(dir);
    })
}

const NOT_UTF8: &[u8] = &[0xff, 0xfe];

#[test]
fn hidden_folders_caches_and_other_files_are_not_read() {
    let dir = synced(|dir| {
        for path in ["src/.hidden", "src/__pycache__"] {
            fs::create_dir(dir.path().join(path)).unwrap();
            fs::write(dir.path().join(path).join("x.py"), NOT_UTF8).unwrap();
        }
        fs::write(dir.path().join("src/notes.txt"), NOT_UTF8).unwrap();
        fs::write(dir.path().join("src/run.py"), "print('ok')\n").unwrap();
        fs::create_dir(dir.path().join("src/pkg")).unwrap();
        fs::write(dir.path().join("src/pkg/mod.py"), "print('nested')\n").unwrap();
        let installed = dir.path().join(".venv/lib/python3.12/site-packages");
        fs::create_dir(installed.join("bare-1.0.dist-info")).unwrap();
        fs::create_dir(installed.join("greet-1.0.dist-info")).unwrap();
        fs::write(
            installed.join("greet-1.0.dist-info/RECORD"),
            "greet/__init__.py,,\n",
        )
        .unwrap();
        fs::create_dir(dir.path().join(".venv/lib/other")).unwrap();
        fs::write(installed.join("README"), "").unwrap();
    });
    assert_eq!(check(&dir, &mut io::sink()), "Success");
    let sites = fs::read_to_string(dir.path().join(".mental/model/site.jsonl")).unwrap();
    assert!(sites.contains("site:pkg-mod-callee-print"), "{sites}");
}

/// A socket or other special file named `.py` is not read: reading a pipe
/// would wait forever.
#[cfg(unix)]
#[test]
fn a_special_file_is_not_read() {
    let dir = synced(|_| ());
    let _socket = std::os::unix::net::UnixListener::bind(dir.path().join("src/sock.py")).unwrap();
    assert_eq!(check(&dir, &mut io::sink()), "Success");
}

#[test]
fn a_python_file_that_is_not_utf8_fails_the_tool() {
    let dir = synced(|dir| fs::write(dir.path().join("src/bad.py"), NOT_UTF8).unwrap());
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot read `src/bad.py`; make it readable, as a UTF-8 file if it is one, or remove it"
    );
}

#[test]
fn a_record_that_cannot_be_read_fails_the_tool() {
    let dir = synced(|dir| {
        let info = dir
            .path()
            .join(".venv/lib/python3.12/site-packages/x-1.0.dist-info");
        fs::create_dir_all(info.join("RECORD")).unwrap();
    });
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot read `.venv/lib/python3.12/site-packages/x-1.0.dist-info/RECORD`; make it \
         readable, as a UTF-8 file if it is one, or remove it"
    );
}

#[test]
fn an_environment_lib_that_is_no_folder_fails_the_tool() {
    let dir = synced(|dir| {
        fs::remove_dir_all(dir.path().join(".venv/lib")).unwrap();
        fs::write(dir.path().join(".venv/lib"), "").unwrap();
    });
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot read `.venv/lib`; make it readable, as a UTF-8 file if it is one, or remove it"
    );
}

#[cfg(unix)]
#[test]
fn a_source_folder_that_cannot_be_read_fails_the_tool() {
    use std::os::unix::fs::PermissionsExt;
    let dir = synced(|dir| {
        fs::create_dir(dir.path().join("src/app")).unwrap();
        fs::set_permissions(
            dir.path().join("src/app"),
            fs::Permissions::from_mode(0o000),
        )
        .unwrap();
    });
    let result = check(&dir, &mut io::sink());
    fs::set_permissions(
        dir.path().join("src/app"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    assert_eq!(
        result,
        "cannot read `src/app`; make it readable, as a UTF-8 file if it is one, or remove it"
    );
}

/// Only Linux lets a file or folder have a name that is not UTF-8.
#[cfg(target_os = "linux")]
#[test]
fn names_that_are_not_utf8_fail_the_tool() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let name = OsStr::from_bytes(b"\xff.py");
    let dir = synced(|dir| fs::write(dir.path().join("src").join(name), "").unwrap());
    assert_eq!(
        check(&dir, &mut io::sink()),
        "`src` holds `\u{fffd}.py`, whose name is not UTF-8; rename it"
    );
    let outer = TempDir::new().unwrap();
    let inner = outer.path().join(OsStr::from_bytes(b"\xff"));
    fs::create_dir(&inner).unwrap();
    let project = synced(|_| ());
    for name in ["pyproject.toml", "uv.lock", ".python-version"] {
        fs::copy(project.path().join(name), inner.join(name)).unwrap();
    }
    fs::create_dir_all(inner.join(".venv/lib/python3.12/site-packages")).unwrap();
    fs::copy(
        project.path().join(".venv/pyvenv.cfg"),
        inner.join(".venv/pyvenv.cfg"),
    )
    .unwrap();
    let cli = Cli {
        command: Command::Build { json: false },
    };
    let error = run(&cli, &inner, &mut io::sink()).unwrap_err().to_string();
    assert!(
        error.ends_with("is not UTF-8; build from a directory whose path is"),
        "{error}"
    );
}

/// A project with no `src/` has its own directory as its root: its Python
/// files are read there, and its environment is not.
#[test]
fn a_project_with_no_src_is_read_from_its_directory() {
    let dir = synced(|dir| {
        fs::remove_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("run.py"), "print('ok')\n").unwrap();
        fs::write(dir.path().join(".venv/stray.py"), NOT_UTF8).unwrap();
    });
    assert_eq!(check(&dir, &mut io::sink()), "Success");
    let sites = fs::read_to_string(dir.path().join(".mental/model/site.jsonl")).unwrap();
    assert!(sites.contains("site:run-callee-print"), "{sites}");
}

/// A build that stops at stage 1 removes stage 2's files, but only once
/// every other file is in place: a failed write keeps the earlier sites.
#[test]
fn a_failed_write_keeps_the_earlier_sites() {
    let dir = synced(|dir| fs::write(dir.path().join("src/run.py"), "print('ok')\n").unwrap());
    assert_eq!(check(&dir, &mut io::sink()), "Success");
    let sites = dir.path().join(".mental/model/site.jsonl");
    let earlier = fs::read_to_string(&sites).unwrap();
    fs::remove_dir_all(dir.path().join(".venv")).unwrap();
    fs::create_dir(dir.path().join(".mental/anchors.partial")).unwrap();
    assert_eq!(
        check(&dir, &mut io::sink()),
        "cannot write `.mental/anchors.jsonl`; check that the project directory is writable"
    );
    assert_eq!(fs::read_to_string(&sites).unwrap(), earlier);
    fs::remove_dir(dir.path().join(".mental/anchors.partial")).unwrap();
    assert_eq!(check(&dir, &mut io::sink()), "Failure");
    assert!(!sites.exists());
}

#[cfg(unix)]
#[test]
fn a_link_to_source_fails_the_tool_and_a_link_to_another_file_is_ignored() {
    use std::os::unix::fs::symlink;
    let dir = synced(|dir| {
        fs::write(dir.path().join("notes.txt"), "").unwrap();
        symlink(
            dir.path().join("notes.txt"),
            dir.path().join("src/notes.txt"),
        )
        .unwrap();
    });
    assert_eq!(check(&dir, &mut io::sink()), "Success");
    let message = |path: &str| {
        format!(
            "`{path}` is a symbolic link, which mentalc does not follow, so the files it leads \
             to would go unread; replace it with what it links to, or remove it"
        )
    };
    fs::create_dir(dir.path().join("shared")).unwrap();
    symlink(dir.path().join("shared"), dir.path().join("src/shared")).unwrap();
    assert_eq!(check(&dir, &mut io::sink()), message("src/shared"));
    fs::remove_file(dir.path().join("src/shared")).unwrap();
    fs::write(dir.path().join("shared.py"), "").unwrap();
    symlink(
        dir.path().join("shared.py"),
        dir.path().join("src/linked.py"),
    )
    .unwrap();
    assert_eq!(check(&dir, &mut io::sink()), message("src/linked.py"));
}

/// Folders that are skipped are skipped before any link is looked at: a
/// project whose root is its directory may link its `.venv`, and an
/// editor's lock file is a broken link.
#[cfg(unix)]
#[test]
fn skipped_and_broken_links_are_ignored() {
    use std::os::unix::fs::symlink;
    let elsewhere = synced(|_| ());
    let dir = synced(|dir| {
        fs::remove_dir(dir.path().join("src")).unwrap();
        fs::remove_dir_all(dir.path().join(".venv")).unwrap();
        symlink(elsewhere.path().join(".venv"), dir.path().join(".venv")).unwrap();
        fs::write(dir.path().join("run.py"), "print('ok')\n").unwrap();
        symlink(dir.path().join("gone.py"), dir.path().join(".#run.py")).unwrap();
        symlink(dir.path().join("gone"), dir.path().join("ghost")).unwrap();
    });
    assert_eq!(check(&dir, &mut io::sink()), "Success");
}
