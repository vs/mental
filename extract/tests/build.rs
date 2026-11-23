//! The build model of a project directory: a fixture for each gap in
//! `tests/data/err/<code>/`, one with no gap in `tests/data/ok/`, and
//! report-jobs against its hand-written reference.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;

use mental_extract::{BuildModel, Member, ProjectFiles, Root, build_model};
use mental_model::{Anchor, Diagnostic, jsonl};

/// The build files of a project directory, as `mentalc` reads them.
struct Tree {
    name: String,
    pyproject: Option<String>,
    lock: Option<String>,
    python_version: Option<String>,
    pyvenv: Option<String>,
    has_src: bool,
    /// For a member built on its own: the root's folder, name and
    /// `pyproject.toml`.
    root: Option<(String, String, String)>,
    /// For a workspace root: each member's folder, `pyproject.toml` and
    /// whether it has `src/`.
    members: Vec<(String, Option<String>, bool)>,
}

impl Tree {
    fn read(dir: &Path) -> Tree {
        Tree::read_in(dir, ".", &[])
    }

    /// `dir`, with its build files in `shared`, and the members `members`.
    fn read_in(dir: &Path, shared: &str, members: &[&str]) -> Tree {
        let file = |name: &str| fs::read_to_string(dir.join(shared).join(name)).ok();
        let root = (shared != ".").then(|| {
            let root = dir.join(shared).canonicalize().expect("the root is there");
            let name = root
                .file_name()
                .and_then(|name| name.to_str())
                .expect("a fixture name");
            let pyproject =
                fs::read_to_string(root.join("pyproject.toml")).expect("a root project");
            let name = name.to_owned();
            (shared.to_owned(), name, pyproject)
        });
        Tree {
            name: dir
                .file_name()
                .and_then(|name| name.to_str())
                .expect("a fixture name")
                .to_owned(),
            pyproject: fs::read_to_string(dir.join("pyproject.toml")).ok(),
            lock: file("uv.lock"),
            python_version: file(".python-version"),
            pyvenv: file(".venv/pyvenv.cfg"),
            has_src: dir.join("src").is_dir(),
            root,
            members: members
                .iter()
                .map(|member| {
                    let pyproject =
                        fs::read_to_string(dir.join(member).join("pyproject.toml")).ok();
                    (
                        (*member).to_owned(),
                        pyproject,
                        dir.join(member).join("src").is_dir(),
                    )
                })
                .collect(),
        }
    }

    /// The text of the file at `path`, relative to the build directory.
    fn text(&self, path: &str) -> Option<&str> {
        let shared = self.root.as_ref().map_or(".", |(dir, _, _)| dir.as_str());
        let lock = if shared == "." {
            "uv.lock".to_owned()
        } else {
            format!("{shared}/uv.lock")
        };
        if path == "pyproject.toml" {
            return self.pyproject.as_deref();
        }
        if path == lock {
            return self.lock.as_deref();
        }
        let member = path.strip_suffix("/pyproject.toml")?;
        let (_, pyproject, _) = self.members.iter().find(|(dir, _, _)| dir == member)?;
        pyproject.as_deref()
    }

    /// Builds the tree and checks what holds for every build: one record per
    /// ID, one anchor per record, and every quote at its lines.
    fn build(&self) -> (BuildModel, Vec<Diagnostic>) {
        let members: Vec<Member> = self
            .members
            .iter()
            .map(|(dir, pyproject, has_src)| Member {
                dir,
                pyproject: pyproject.as_deref(),
                has_src: *has_src,
            })
            .collect();
        let (model, diagnostics) = build_model(&ProjectFiles {
            name: &self.name,
            pyproject: self.pyproject.as_deref(),
            lock: self.lock.as_deref(),
            python_version: self.python_version.as_deref(),
            pyvenv: self.pyvenv.as_deref(),
            has_src: self.has_src,
            root: self.root.as_ref().map(|(dir, name, pyproject)| Root {
                dir,
                name,
                pyproject,
            }),
            members: &members,
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
            let text = self.text(&anchor.path).unwrap_or_else(|| {
                panic!("{} is not a file a build record stands on", anchor.path)
            });
            let [first, last]: [u32; 2] = anchor.lines.into();
            let lines: Vec<&str> = text
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
{"code":"project-invalid","message":"`pyproject.toml` has no `[project]` table and no `[tool.uv.workspace]`, so it is neither a project nor a workspace; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
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
        name: "flat".to_owned(),
        pyproject: Some(pyproject.to_owned()),
        lock: Some(lock.to_owned()),
        python_version: python_version.map(str::to_owned),
        pyvenv: Some(pyvenv.to_owned()),
        has_src: false,
        root: None,
        members: Vec::new(),
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
        let tree = Tree {
            name: "flat".to_owned(),
            pyproject,
            lock,
            python_version,
            pyvenv,
            has_src,
            root: None,
            members: Vec::new(),
        };
        prop_assert_eq!(tree.build(), tree.build());
    }
}

/// The build units, pins and diagnostics of `tree`.
fn show(tree: &Tree) -> String {
    let (model, diagnostics) = tree.build();
    format!(
        "{}{}{}",
        jsonl::write(&model.units),
        jsonl::write(&model.pins),
        jsonl::write(&diagnostics)
    )
}

const API: &str = r#"{"id":"workspace/unit:api/pin:core","package":"core","version":"0.2.0","origin":{"path":"packages/core"},"source":"extractor"}
{"id":"workspace/unit:api/pin:requests","package":"requests","version":"2.32.3","origin":{"registry":"https://pypi.org/simple"},"source":"extractor"}
"#;

#[test]
fn a_workspace_has_a_unit_per_member_with_the_pins_it_reaches() {
    let tree = Tree::read_in(
        &fixture("ok/workspace"),
        ".",
        &["packages/api", "packages/core"],
    );
    assert_eq!(
        show(&tree),
        r#"{"id":"workspace/unit:api","dir":"packages/api","roots":["packages/api/src"],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","lock":{"path":"uv.lock","version":1},"environment":".venv","source":"extractor"}
{"id":"workspace/unit:core","dir":"packages/core","roots":["packages/core"],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","lock":{"path":"uv.lock","version":1},"environment":".venv","source":"extractor"}
"#
        .to_owned()
            + API
    );
}

#[test]
fn a_member_built_alone_takes_the_roots_files_and_keeps_its_ids() {
    let tree = Tree::read_in(&fixture("ok/workspace/packages/api"), "../..", &[]);
    assert_eq!(
        show(&tree),
        r#"{"id":"workspace/unit:api","dir":".","roots":["src"],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","lock":{"path":"../../uv.lock","version":1},"environment":"../../.venv","source":"extractor"}
"#
        .to_owned()
            + API
    );
    let mut bare = Tree::read_in(&fixture("ok/workspace/packages/core"), "../..", &[]);
    (bare.lock, bare.pyvenv, bare.python_version) = (None, None, None);
    assert_eq!(
        show(&bare),
        r#"{"id":"workspace/unit:core","dir":".","roots":["."],"requires_python":">=3.12","source":"extractor"}
{"id":"workspace/unit:core","code":"environment-missing","message":"the project has no environment (`../../.venv/pyvenv.cfg`); run `uv sync` to make it"}
{"id":"workspace/unit:core","code":"lock-missing","message":"the project has no `../../uv.lock`; run `uv lock` to write it"}
{"id":"workspace/unit:core","code":"python-version-unknown","message":"there is no `../../.python-version` and no environment to take the Python version from; write one in `../../.python-version`, for example with `uv python pin 3.12`"}
"#
    );
}

#[test]
fn a_root_that_is_a_project_is_a_unit_too_and_names_the_system() {
    let mut tree = Tree::read_in(&fixture("ok/workspace"), ".", &["packages/core"]);
    tree.pyproject = Some(format!(
        "[project]\nname = \"Shop\"\n\n{}",
        tree.pyproject.unwrap()
    ));
    let (model, diagnostics) = tree.build();
    let units: Vec<&str> = model.units.iter().map(|unit| unit.id.as_str()).collect();
    assert_eq!(units, ["shop/unit:shop", "shop/unit:core"]);
    assert_eq!(
        jsonl::write(&diagnostics),
        r#"{"id":"shop/unit:shop","code":"lock-invalid","message":"`uv.lock` does not lock the project `shop`, so it says nothing of what the project depends on; run `uv lock` to lock it"}
"#
    );
    let mut alone = Tree::read_in(&fixture("ok/workspace/packages/core"), "../..", &[]);
    let root = alone.root.as_mut().unwrap();
    root.2 = format!("[project]\nname = \"Shop\"\n\n{}", root.2);
    assert_eq!(alone.build().0.units[0].id.as_str(), "shop/unit:core");
}

#[test]
fn a_member_with_no_project_is_a_gap() {
    let mut tree = Tree::read_in(
        &fixture("ok/workspace"),
        ".",
        &["packages/core", "packages/none", "packages/api"],
    );
    tree.members[2].1 = Some("[tool.uv.workspace]\n".to_owned());
    let mut broken = Tree::read_in(&fixture("ok/workspace"), ".", &["packages/core"]);
    broken.members[0].1 = Some("[project\n".to_owned());
    assert_eq!(
        format!(
            "{}{}",
            jsonl::write(&tree.build().1),
            jsonl::write(&broken.build().1)
        ),
        r#"{"code":"project-invalid","message":"the workspace member `packages/api` declares no `[project]`, so it is no project; give `packages/api/pyproject.toml` a `[project]` table with a `name`"}
{"code":"project-missing","message":"the workspace member `packages/none` has no `pyproject.toml`; give it one with `uv init`, or leave it out of the workspace's `members`"}
{"code":"project-invalid","at":{"path":"packages/core/pyproject.toml","lines":[1,1]},"message":"`packages/core/pyproject.toml` does not read: unclosed table, expected `]`; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
"#
    );
}

#[test]
fn a_root_with_no_project_is_named_after_its_directory() {
    let mut tree = Tree::read_in(&fixture("ok/workspace"), ".", &["packages/core"]);
    tree.name = "2048".to_owned();
    let mut alone = Tree::read_in(&fixture("ok/workspace/packages/core"), "../..", &[]);
    alone.root.as_mut().unwrap().1 = "2048".to_owned();
    let mut unreadable = Tree::read_in(&fixture("ok/workspace/packages/core"), "../..", &[]);
    unreadable.root.as_mut().unwrap().2 = "[tool\n".to_owned();
    let problems = [tree, alone, unreadable].map(|tree| jsonl::write(&tree.build().1));
    assert_eq!(
        problems.join(""),
        r#"{"code":"project-invalid","message":"the workspace root has no `[project]`, so the system is named after its directory, and `2048` has no ASCII letter; rename the directory, or give the root a `[project]` with a name"}
{"code":"project-invalid","message":"the workspace root has no `[project]`, so the system is named after its directory, and `2048` has no ASCII letter; rename the directory, or give the root a `[project]` with a name"}
{"code":"project-invalid","at":{"path":"../../pyproject.toml","lines":[1,1]},"message":"`../../pyproject.toml` does not read: unclosed table, expected `]`; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
"#
    );
}
