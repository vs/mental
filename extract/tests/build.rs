//! The build model of a project directory: a fixture for each gap in
//! `tests/data/err/<code>/`, one with no gap in `tests/data/ok/`, and
//! report-jobs against its hand-written reference.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;

use mental_extract::{BuildModel, ProjectFiles, build_model};
use mental_model::{Anchor, Diagnostic, jsonl};

/// The build files of a project directory, as `mentalc` reads them.
struct Tree {
    pyproject: Option<String>,
    lock: Option<String>,
    python_version: Option<String>,
    pyvenv: Option<String>,
    has_src: bool,
}

impl Tree {
    fn read(dir: &Path) -> Tree {
        let file = |name: &str| fs::read_to_string(dir.join(name)).ok();
        Tree {
            pyproject: file("pyproject.toml"),
            lock: file("uv.lock"),
            python_version: file(".python-version"),
            pyvenv: file(".venv/pyvenv.cfg"),
            has_src: dir.join("src").is_dir(),
        }
    }

    /// Builds the tree and checks what holds for every build: one record per
    /// ID, one anchor per record, and every quote at its lines.
    fn build(&self) -> (BuildModel, Vec<Diagnostic>) {
        let (model, diagnostics) = build_model(&ProjectFiles {
            pyproject: self.pyproject.as_deref(),
            lock: self.lock.as_deref(),
            python_version: self.python_version.as_deref(),
            pyvenv: self.pyvenv.as_deref(),
            has_src: self.has_src,
        });
        let units = model.units.iter().map(|unit| unit.id.as_str());
        let pins = model.pins.iter().map(|pin| pin.id.as_str());
        let mut records: Vec<&str> = units.chain(pins).collect();
        records.sort_unstable();
        let mut anchored: Vec<&str> = model
            .anchors
            .iter()
            .map(|anchor| anchor.id.as_str())
            .collect();
        anchored.sort_unstable();
        anchored.dedup();
        assert_eq!(
            anchored, records,
            "one record per ID and one anchor per record"
        );
        for anchor in &model.anchors {
            let text = match anchor.path.as_str() {
                "pyproject.toml" => &self.pyproject,
                "uv.lock" => &self.lock,
                other => panic!("{other} is not a file a build record stands on"),
            };
            let [first, last]: [u32; 2] = anchor.lines.into();
            let lines: Vec<&str> = text
                .as_deref()
                .expect("a record stands on a file that is there")
                .lines()
                .skip(first as usize - 1)
                .take((last - first + 1) as usize)
                .collect();
            assert_eq!(lines.join("\n"), anchor.quote, "{}", anchor.id);
        }
        (model, diagnostics)
    }
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name)
}

/// The build unit, the number of pins and the diagnostics of a fixture.
#[track_caller]
fn check(name: &str) -> String {
    let (model, diagnostics) = Tree::read(&fixture(name)).build();
    format!(
        "{}{} pins\n{}",
        jsonl::write(&model.units),
        model.pins.len(),
        jsonl::write(&diagnostics)
    )
}

#[test]
fn a_project_with_every_file_has_no_gap() {
    assert_eq!(
        check("ok/flat"),
        r#"{"id":"shop/unit:shop","dir":".","roots":["."],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","lock":{"path":"uv.lock","version":1},"environment":".venv","source":"extractor"}
1 pins
"#
    );
}

#[test]
fn every_gap_of_the_build_has_a_fixture() {
    let gaps: Vec<String> = fs::read_dir(fixture("err"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(
        gaps.join(" "),
        "environment-missing lock-ambiguous lock-invalid lock-missing lock-version-unknown \
         project-invalid project-missing python-version-unknown"
    );
}

#[test]
fn project_missing() {
    assert_eq!(
        check("err/project-missing"),
        r#"0 pins
{"code":"project-missing","message":"there is no `pyproject.toml` here, so there is no project to build; run mentalc in a project's directory, or make one with `uv init`"}
"#
    );
}

#[test]
fn project_invalid() {
    assert_eq!(
        check("err/project-invalid"),
        r#"0 pins
{"code":"project-invalid","at":{"path":"pyproject.toml","lines":[1,1]},"message":"`pyproject.toml` does not read: missing field `project`; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
"#
    );
}

#[test]
fn lock_missing() {
    assert_eq!(
        check("err/lock-missing"),
        r#"{"id":"shop/unit:shop","dir":".","roots":["."],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","environment":".venv","source":"extractor"}
0 pins
{"id":"shop/unit:shop","code":"lock-missing","message":"the project has no `uv.lock`; run `uv lock` to write it"}
"#
    );
}

#[test]
fn lock_invalid() {
    assert_eq!(
        check("err/lock-invalid"),
        r#"{"id":"shop/unit:shop","dir":".","roots":["."],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","environment":".venv","source":"extractor"}
0 pins
{"code":"lock-invalid","at":{"path":"uv.lock","lines":[5,5]},"message":"`uv.lock` does not read: missing field `source`; run `uv lock` to write it again"}
"#
    );
}

#[test]
fn lock_version_unknown() {
    assert_eq!(
        check("err/lock-version-unknown"),
        r#"{"id":"shop/unit:shop","dir":".","roots":["."],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","environment":".venv","source":"extractor"}
0 pins
{"code":"lock-version-unknown","at":{"path":"uv.lock","lines":[1,1]},"message":"`uv.lock` has schema version 2, and this build reads only version 1; lock with a uv that writes version 1"}
"#
    );
}

#[test]
fn lock_ambiguous() {
    assert_eq!(
        check("err/lock-ambiguous"),
        r#"{"id":"shop/unit:shop","dir":".","roots":["."],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","lock":{"path":"uv.lock","version":1},"environment":".venv","source":"extractor"}
0 pins
{"code":"lock-ambiguous","at":{"path":"uv.lock","lines":[6,11]},"message":"`uv.lock` locks `requests` 2 times, such as once per platform, and a pin has one version; let uv lock one version for every platform, for example with a tighter bound on it or with `environments` under `[tool.uv]`"}
"#
    );
}

#[test]
fn python_version_unknown() {
    assert_eq!(
        check("err/python-version-unknown"),
        r#"{"id":"shop/unit:shop","dir":".","roots":["."],"requires_python":">=3.12","lock":{"path":"uv.lock","version":1},"environment":".venv","source":"extractor"}
1 pins
{"id":"shop/unit:shop","code":"python-version-unknown","at":{"path":".python-version","lines":[1,1]},"message":"`.python-version` says the Python version is `pypy@3.10`, which is not a version such as 3.12 or 3.12.1; write one in `.python-version`, for example with `uv python pin 3.12`"}
"#
    );
}

#[test]
fn environment_missing() {
    assert_eq!(
        check("err/environment-missing"),
        r#"{"id":"shop/unit:shop","dir":".","roots":["."],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","lock":{"path":"uv.lock","version":1},"source":"extractor"}
1 pins
{"id":"shop/unit:shop","code":"environment-missing","message":"the project has no environment (`.venv/pyvenv.cfg`); run `uv sync` to make it"}
"#
    );
}

/// report-jobs after `uv sync` equals its reference. Its own `.venv` is not
/// read, so the test does not depend on whether `uv sync` was run here.
#[test]
fn report_jobs_equals_its_reference() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/report-jobs");
    let mut tree = Tree::read(&example);
    tree.pyvenv = Some(PYVENV.to_owned());
    let (model, diagnostics) = tree.build();
    let reference = |name: &str| fs::read_to_string(example.join("model").join(name)).unwrap();
    assert_eq!(jsonl::write(&model.units), reference("build_unit.jsonl"));
    assert_eq!(jsonl::write(&model.pins), reference("pin.jsonl"));
    let build_anchors: Vec<Anchor> = jsonl::read::<Anchor>(&reference("anchors.jsonl"))
        .unwrap()
        .into_iter()
        .filter(|anchor| !anchor.id.as_str().contains("/site:"))
        .collect();
    assert_eq!(jsonl::write(&model.anchors), jsonl::write(&build_anchors));
    assert_eq!(diagnostics, []);
}

const PYPROJECT: &str = include_str!("data/ok/flat/pyproject.toml");
const LOCK: &str = include_str!("data/ok/flat/uv.lock");
const PYVENV: &str = include_str!("data/ok/flat/.venv/pyvenv.cfg");

/// `ok/flat` with these files in place of its own.
fn flat(pyproject: &str, lock: &str, python_version: Option<&str>, pyvenv: &str) -> Tree {
    Tree {
        pyproject: Some(pyproject.to_owned()),
        lock: Some(lock.to_owned()),
        python_version: python_version.map(str::to_owned),
        pyvenv: Some(pyvenv.to_owned()),
        has_src: false,
    }
}

/// A file cut short at any line still builds.
#[test]
fn every_line_prefix_of_a_build_file_builds() {
    let prefixes = |text: &'static str| {
        let lines = text
            .split_inclusive('\n')
            .scan(String::new(), |prefix, line| {
                prefix.push_str(line);
                Some(prefix.clone())
            });
        std::iter::once(String::new()).chain(lines)
    };
    for pyproject in prefixes(PYPROJECT) {
        flat(&pyproject, LOCK, Some("3.12\n"), PYVENV).build();
    }
    for lock in prefixes(LOCK) {
        flat(PYPROJECT, &lock, Some("3.12\n"), PYVENV).build();
    }
    // With no `.python-version`, the version comes from `pyvenv.cfg`.
    let versions: BTreeSet<Option<String>> = prefixes(PYVENV)
        .map(|pyvenv| {
            let (model, _) = flat(PYPROJECT, LOCK, None, &pyvenv).build();
            model.units[0]
                .python
                .as_ref()
                .map(|python| python.version.clone())
        })
        .collect();
    assert_eq!(versions, BTreeSet::from([None, Some("3.12.8".to_owned())]));
}

/// A real file with a stretch of it replaced by arbitrary text.
fn damaged(text: &'static str) -> impl Strategy<Value = String> {
    assert!(text.is_ascii(), "so every byte offset is a char boundary");
    (0..=text.len(), 0..=text.len(), any::<String>()).prop_map(move |(a, b, insert)| {
        format!("{}{insert}{}", &text[..a.min(b)], &text[a.max(b)..])
    })
}

fn file(text: &'static str) -> impl Strategy<Value = Option<String>> {
    prop::option::of(prop_oneof![damaged(text), any::<String>()])
}

proptest! {
    /// Damaged build files never panic the build, which gives the same
    /// output twice and anchors every record it writes.
    #[test]
    fn damaged_files_build_the_same_twice(
        pyproject in file(PYPROJECT),
        lock in file(LOCK),
        python_version in file("3.12\n"),
        pyvenv in file(PYVENV),
        has_src: bool,
    ) {
        let tree = Tree { pyproject, lock, python_version, pyvenv, has_src };
        prop_assert_eq!(tree.build(), tree.build());
    }
}
