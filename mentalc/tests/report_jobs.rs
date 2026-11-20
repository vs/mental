//! The acceptance of milestone 1 on report-jobs with its environment
//! (goals 2 to 4): the real `mentalc` on a copy of the project whose `.venv`
//! links to report-jobs' own. `uv sync` makes that environment; these tests
//! run when `RUN_SLOW_TESTS` is set. CI cannot make it yet (CONTRIBUTING.md,
//! Exceptions).

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use insta_cmd::assert_cmd_snapshot;
use mental_model::{Outcome, Site, SiteKind, jsonl};
use tempfile::TempDir;

const SITE_PACKAGES: &str = "lib/python3.12/site-packages";

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/report-jobs")
        .canonicalize()
        .expect("report-jobs is in examples/")
}

/// report-jobs' environment, when the slow tests run; `None` otherwise.
fn environment() -> Option<PathBuf> {
    std::env::var_os("RUN_SLOW_TESTS")?;
    let venv = example().join(".venv");
    assert!(
        venv.join(SITE_PACKAGES).is_dir(),
        "report-jobs has no environment; run `uv sync --locked` in examples/report-jobs"
    );
    Some(venv)
}

fn copy_folder(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("the copy's folder can be made");
    for entry in fs::read_dir(from).expect("report-jobs' folder reads") {
        let entry = entry.expect("a folder entry reads");
        let name = entry.file_name();
        if entry.file_type().expect("an entry has a type").is_dir() {
            if name != "__pycache__" {
                copy_folder(&entry.path(), &to.join(&name));
            }
        } else {
            fs::copy(entry.path(), to.join(&name)).expect("a file copies");
        }
    }
}

/// A copy of report-jobs' build files and source, with `.venv` made by `venv`.
fn report_jobs(venv: impl FnOnce(&Path)) -> TempDir {
    let dir = TempDir::new().expect("a temporary directory can be made");
    for name in ["pyproject.toml", "uv.lock", ".python-version"] {
        fs::copy(example().join(name), dir.path().join(name)).expect("report-jobs has it");
    }
    copy_folder(&example().join("src"), &dir.path().join("src"));
    venv(&dir.path().join(".venv"));
    dir
}

fn mentalc(dir: &TempDir) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mentalc"));
    command.arg("build").current_dir(dir.path());
    command
}

fn written(dir: &TempDir, path: &str) -> String {
    fs::read_to_string(dir.path().join(".mental").join(path)).expect("the build wrote it")
}

/// Goals 2 and 4: every site has the outcome the reference gives it, every
/// `report.status = …` resolves to `Report`, and a second build writes the
/// same bytes.
#[test]
fn r1_report_jobs_builds_to_its_reference_twice_alike() {
    let Some(venv) = environment() else {
        return;
    };
    let dir = report_jobs(|link| symlink(&venv, link).unwrap());
    assert_cmd_snapshot!(mentalc(&dir), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    wrote .mental/ (build units: 1, pins: 40, sites: 140: resolved 28, external 105, unresolved 7)

    ----- stderr -----
    ");
    let reference = |name: &str| fs::read_to_string(example().join("model").join(name)).unwrap();
    for name in ["build_unit.jsonl", "pin.jsonl", "site.jsonl"] {
        assert_eq!(
            written(&dir, &format!("model/{name}")),
            reference(name),
            "{name}"
        );
    }
    assert_eq!(written(&dir, "anchors.jsonl"), reference("anchors.jsonl"));
    assert_eq!(written(&dir, "diagnostics.jsonl"), "");
    let sites: Vec<Site> = jsonl::read(&written(&dir, "model/site.jsonl")).unwrap();
    let status_writes: Vec<&Outcome> = sites
        .iter()
        .filter(|site| {
            site.kind == SiteKind::WriteReceiver && site.id.as_str().contains("report-status")
        })
        .map(|site| &site.outcome)
        .collect();
    assert_eq!(status_writes.len(), 5);
    for outcome in status_writes {
        assert_eq!(
            *outcome,
            Outcome::Resolved {
                module: "app.models".to_owned(),
                name: "Report".to_owned(),
            }
        );
    }
    let files = [
        "model/site.jsonl",
        "anchors.jsonl",
        "diagnostics.jsonl",
        "resolution.json",
    ];
    let first = files.map(|path| written(&dir, path));
    assert!(mentalc(&dir).status().unwrap().success());
    assert_eq!(files.map(|path| written(&dir, path)), first);
}

/// Goal 3: with one package taken out of the environment, its import is a
/// `module-missing` that names the fix, and the build fails.
#[test]
fn r1_a_package_missing_from_the_environment_fails_the_build() {
    let Some(venv) = environment() else {
        return;
    };
    let dir = report_jobs(|copy| {
        let installed = copy.join(SITE_PACKAGES);
        fs::create_dir_all(&installed).unwrap();
        fs::copy(venv.join("pyvenv.cfg"), copy.join("pyvenv.cfg")).unwrap();
        for entry in fs::read_dir(venv.join(SITE_PACKAGES)).unwrap() {
            let name = entry.unwrap().file_name();
            let name = name.to_str().unwrap();
            if name != "requests" && !name.starts_with("requests-") {
                symlink(venv.join(SITE_PACKAGES).join(name), installed.join(name)).unwrap();
            }
        }
    });
    assert_cmd_snapshot!(mentalc(&dir), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    error[module-missing] reports/unit:reports/site:app-tasks-import-requests: the import `requests` names a module that is neither under a root nor in the environment; run `uv sync`, or add the package that provides it with `uv add`
    wrote .mental/ (build units: 1, pins: 40, sites: 140: resolved 28, external 103, unresolved 9)

    ----- stderr -----
    ");
}
