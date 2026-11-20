//! `mentalc build` as a user runs it: the real binary on a copy of a
//! project, with its exit code, its output and the files it writes.

use std::fs;
use std::path::Path;
use std::process::Command;

use insta_cmd::assert_cmd_snapshot;
use mental_model::{Anchor, BuildUnit, jsonl};
use tempfile::TempDir;

const EXAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/report-jobs");

/// A copy of report-jobs' build files and its `src/`, with no `.venv`.
fn report_jobs() -> TempDir {
    let dir = TempDir::new().expect("a temporary directory can be made");
    for name in ["pyproject.toml", "uv.lock", ".python-version"] {
        fs::copy(Path::new(EXAMPLE).join(name), dir.path().join(name)).expect("report-jobs has it");
    }
    fs::create_dir(dir.path().join("src")).expect("the copy has no `src/` yet");
    dir
}

/// Gives the copy the environment `uv sync` would make.
fn sync(dir: &TempDir) {
    fs::create_dir(dir.path().join(".venv")).expect("the copy has no `.venv/` yet");
    fs::write(
        dir.path().join(".venv/pyvenv.cfg"),
        "home = /usr/local/bin\nuv = 0.9.2\nversion_info = 3.12.8\n",
    )
    .expect("`.venv/` is writable");
}

fn mentalc(dir: &TempDir, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mentalc"));
    command.args(args).current_dir(dir.path());
    command
}

fn written(dir: &TempDir, path: &str) -> String {
    fs::read_to_string(dir.path().join(".mental").join(path)).expect("the build wrote it")
}

fn reference(name: &str) -> String {
    fs::read_to_string(Path::new(EXAMPLE).join("model").join(name)).expect("the reference has it")
}

/// Goal 1: without `.venv`, the reference without its `environment`, and
/// the one gap with its fix.
#[test]
fn build_without_an_environment() {
    let dir = report_jobs();
    assert_cmd_snapshot!(mentalc(&dir, &["build"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    error[environment-missing] reports/unit:reports: the project has no environment (`.venv/pyvenv.cfg`); run `uv sync` to make it
    wrote .mental/ (build units: 1, pins: 40)

    ----- stderr -----
    ");
    let mut units: Vec<BuildUnit> = jsonl::read(&reference("build_unit.jsonl")).unwrap();
    units[0].environment = None;
    assert_eq!(
        written(&dir, "model/build_unit.jsonl"),
        jsonl::write(&units)
    );
    assert_eq!(written(&dir, "model/pin.jsonl"), reference("pin.jsonl"));
    let build_anchors: Vec<Anchor> = jsonl::read::<Anchor>(&reference("anchors.jsonl"))
        .unwrap()
        .into_iter()
        .filter(|anchor| !anchor.id.as_str().contains("/site:"))
        .collect();
    assert_eq!(written(&dir, "anchors.jsonl"), jsonl::write(&build_anchors));
    assert_eq!(written(&dir, "model/format.json"), "{\"version\":1}\n");
    assert_eq!(
        written(&dir, "diagnostics.jsonl"),
        "{\"id\":\"reports/unit:reports\",\"code\":\"environment-missing\",\"message\":\"the project has no environment (`.venv/pyvenv.cfg`); run `uv sync` to make it\"}\n"
    );
}

#[test]
fn build_json_prints_the_diagnostics_lines() {
    let dir = report_jobs();
    assert_cmd_snapshot!(mentalc(&dir, &["build", "--json"]), @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    {"id":"reports/unit:reports","code":"environment-missing","message":"the project has no environment (`.venv/pyvenv.cfg`); run `uv sync` to make it"}

    ----- stderr -----
    "#);
}

/// After `uv sync`: the reference, and the same bytes when built again.
#[test]
fn build_with_an_environment_writes_the_reference() {
    let dir = report_jobs();
    sync(&dir);
    assert_cmd_snapshot!(mentalc(&dir, &["build"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    wrote .mental/ (build units: 1, pins: 40)

    ----- stderr -----
    ");
    let files = [
        "model/format.json",
        "model/build_unit.jsonl",
        "model/pin.jsonl",
        "anchors.jsonl",
        "diagnostics.jsonl",
    ];
    let first = files.map(|path| written(&dir, path));
    assert_eq!(first[1], reference("build_unit.jsonl"));
    assert_eq!(first[4], "");
    assert!(mentalc(&dir, &["build"]).status().unwrap().success());
    assert_eq!(files.map(|path| written(&dir, path)), first);
}

#[test]
fn a_directory_that_is_no_project_gets_no_model() {
    let dir = TempDir::new().unwrap();
    assert_cmd_snapshot!(mentalc(&dir, &["build"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    error[project-missing]: there is no `pyproject.toml` here, so there is no project to build; run mentalc in a project's directory, or make one with `uv init`
    wrote nothing: there is no build unit

    ----- stderr -----
    ");
    assert!(!dir.path().join(".mental").exists());
}

/// A model of an earlier build would read as the project's current one.
#[test]
fn a_build_with_no_unit_removes_the_earlier_model() {
    let dir = report_jobs();
    sync(&dir);
    assert!(mentalc(&dir, &["build"]).status().unwrap().success());
    fs::remove_file(dir.path().join("pyproject.toml")).unwrap();
    assert_eq!(mentalc(&dir, &["build"]).status().unwrap().code(), Some(1));
    let files = [
        "model/format.json",
        "model/build_unit.jsonl",
        "model/pin.jsonl",
        "anchors.jsonl",
        "diagnostics.jsonl",
    ];
    let left: Vec<&str> = files
        .into_iter()
        .filter(|path| dir.path().join(".mental").join(path).exists())
        .collect();
    assert_eq!(left, Vec::<&str>::new());
}

#[test]
fn a_file_that_cannot_be_read_fails_the_tool() {
    let dir = report_jobs();
    fs::remove_file(dir.path().join("uv.lock")).unwrap();
    fs::create_dir(dir.path().join("uv.lock")).unwrap();
    assert_cmd_snapshot!(mentalc(&dir, &["build"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: cannot read `uv.lock`; make it a readable UTF-8 file, or remove it: Is a directory (os error 21)
    ");
}

#[test]
fn an_unknown_command_is_bad_usage() {
    let dir = TempDir::new().unwrap();
    assert_cmd_snapshot!(mentalc(&dir, &["bild"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: unrecognized subcommand 'bild'

      tip: a similar subcommand exists: 'build'

    Usage: mentalc <COMMAND>

    For more information, try '--help'.
    ");
}
