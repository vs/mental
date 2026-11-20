//! The reference sites of Python files and their outcomes: every fixture in
//! `tests/fixtures/` against the `# $ kind=slug@outcome` expectations on
//! its lines, an independent count of every site, and report-jobs against
//! its hand-written reference.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mental_extract::{DistRecord, Environment, PythonFile, ResolveError, Sites, extract_sites};
use mental_model::{
    Anchor, BuildUnit, Code, Diagnostic, Id, Origin, Outcome, Package, Pin, Python, PythonFrom,
    Reason, Site, SiteKind, Source, jsonl,
};
use ruff_python_ast::visitor::{self, Visitor};
use ruff_python_ast::{Decorator, ExceptHandler, Expr, PySourceType, Stmt};
use ruff_python_parser::parse_unchecked_source;

const MARK: &str = "# $ ";
const SITE_PACKAGES: &str = "lib/python3.12/site-packages";

/// A project on disk, read the way `mentalc` reads it.
#[derive(Clone)]
struct Project {
    dir: PathBuf,
    unit: BuildUnit,
    pins: Vec<Pin>,
    records: Vec<(String, String)>,
    files: Vec<(String, String)>,
}

/// A unit whose environment is `environment`, relative to its directory.
fn unit(name: &str, root: &str, environment: &str) -> BuildUnit {
    BuildUnit {
        id: Id::parse(&format!("{name}/unit:{name}")).expect("the test names units with slugs"),
        dir: ".".to_owned(),
        roots: BTreeSet::from([root.to_owned()]),
        python: Some(Python {
            version: "3.12".to_owned(),
            from: PythonFrom::PythonVersionFile,
        }),
        requires_python: None,
        lock: None,
        environment: Some(environment.to_owned()),
        source: Source::Extractor,
    }
}

/// The `RECORD` of each distribution installed in `environment`.
fn records(environment: &Path) -> Vec<(String, String)> {
    let site_packages = environment.join(SITE_PACKAGES);
    let mut records: Vec<(String, String)> = fs::read_dir(&site_packages)
        .expect("the environment's site-packages reads; run `uv sync` for report-jobs")
        .map(|entry| entry.expect("a site-packages entry reads").file_name())
        .map(|name| {
            name.to_str()
                .expect("an installed name is UTF-8")
                .to_owned()
        })
        .filter(|name| name.ends_with(".dist-info"))
        .map(|name| {
            let text = fs::read_to_string(site_packages.join(&name).join("RECORD"))
                .expect("every .dist-info has a RECORD");
            (name, text)
        })
        .collect();
    records.sort();
    records
}

/// Every `.py` file under `tests/fixtures`, outside its environment.
fn fixtures() -> Project {
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("the fixture folder reads") {
            let path = entry.expect("a fixture folder entry reads").path();
            if path.is_dir() && !path.ends_with(".venv") {
                walk(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "py") {
                found.push(path);
            }
        }
    }
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut found = Vec::new();
    walk(&dir, &mut found);
    found.sort();
    let files = found
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&dir)
                .expect("a fixture lies under the fixture folder")
                .to_str()
                .expect("a fixture path is UTF-8");
            let text = fs::read_to_string(&path).expect("a fixture reads as UTF-8");
            (relative.to_owned(), text)
        })
        .collect();
    let unit = unit("fx", ".", ".venv");
    let greet = Pin {
        id: Id::parse("fx/unit:fx/pin:greet").expect("a pin ID"),
        package: "greet".to_owned(),
        version: "1.0".to_owned(),
        origin: Origin::Registry("https://pypi.org/simple".to_owned()),
        source: Source::Extractor,
    };
    Project {
        records: records(&dir.join(".venv")),
        dir,
        unit,
        pins: vec![greet],
        files,
    }
}

/// Resolves the project and checks what holds for every build: one anchor
/// per site, each quote at its lines, and IDs unique.
fn build(project: &Project) -> Result<(Sites, Vec<Diagnostic>), ResolveError> {
    let files: Vec<PythonFile> = project
        .files
        .iter()
        .map(|(path, text)| PythonFile { path, text })
        .collect();
    let records: Vec<DistRecord> = project
        .records
        .iter()
        .map(|(dist_info, text)| DistRecord { dist_info, text })
        .collect();
    let environment = Environment {
        project_dir: project.dir.to_str().expect("the project path is UTF-8"),
        records: &records,
    };
    let (sites, diagnostics) = extract_sites(&project.unit, &project.pins, &environment, &files)?;
    let ids: BTreeSet<&Id> = sites.sites.iter().map(|site| &site.id).collect();
    assert_eq!(ids.len(), sites.sites.len(), "every site has its own ID");
    let anchored: Vec<&Id> = sites.anchors.iter().map(|anchor| &anchor.id).collect();
    let listed: Vec<&Id> = sites.sites.iter().map(|site| &site.id).collect();
    assert_eq!(anchored, listed, "one anchor per site");
    for anchor in &sites.anchors {
        let (_, text) = project
            .files
            .iter()
            .find(|(path, _)| *path == anchor.path)
            .expect("an anchor names a file that was listed");
        let [first, last]: [u32; 2] = anchor.lines.into();
        let lines: Vec<&str> = text
            .lines()
            .skip(first as usize - 1)
            .take((last - first + 1) as usize)
            .collect();
        assert_eq!(lines.join("\n"), anchor.quote, "{}", anchor.id);
    }
    Ok((sites, diagnostics))
}

/// `resolved:<module>:<name>`, `external:<pin or stdlib>:<module>:<name>`
/// or `unresolved:<reason>`.
fn written(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Resolved { module, name } => format!("resolved:{module}:{name}"),
        Outcome::External {
            package,
            module,
            name,
        } => {
            let package = match package {
                Package::Pin(pin) => pin.as_str().rsplit_once(':').expect("a pin ID").1,
                Package::Stdlib => "stdlib",
            };
            format!("external:{package}:{module}:{name}")
        }
        Outcome::Unresolved { reason } => {
            let reason = match reason {
                Reason::MissingModule => "missing_module",
                Reason::NoType => "no_type",
                Reason::NotFound => "not_found",
                Reason::Ambiguous => "ambiguous",
                Reason::Dynamic => "dynamic",
                Reason::Unreachable => "unreachable",
            };
            format!("unresolved:{reason}")
        }
    }
}

/// `(line, kind, slug, outcome)` of each site, as `# $ kind=slug@outcome`
/// writes them; the outcome is compared only in a file whose marks give it.
type Seen = BTreeSet<(u32, String, String, String)>;

fn expected(text: &str) -> Seen {
    let mut seen = Seen::new();
    for (number, line) in (1..).zip(text.lines()) {
        let Some((_, marks)) = line.split_once(MARK) else {
            continue;
        };
        for mark in marks.split_whitespace() {
            let (kind, site) = mark
                .split_once('=')
                .expect("an expectation is written `kind=slug`");
            let (slug, outcome) = site.split_once('@').unwrap_or((site, ""));
            seen.insert((number, kind.to_owned(), slug.to_owned(), outcome.to_owned()));
        }
    }
    seen
}

fn found(sites: &Sites, path: &str, with_outcomes: bool) -> Seen {
    sites
        .sites
        .iter()
        .zip(&sites.anchors)
        .filter(|(_, anchor)| anchor.path == path)
        .map(|(site, anchor)| {
            let [first, _]: [u32; 2] = anchor.lines.into();
            let (_, slug) = site
                .id
                .as_str()
                .split_once("/site:")
                .expect("a site ID ends in a site segment");
            let outcome = if with_outcomes {
                written(&site.outcome)
            } else {
                String::new()
            };
            (
                first,
                site.kind.as_str().to_owned(),
                slug.to_owned(),
                outcome,
            )
        })
        .collect()
}

#[test]
fn every_fixture_lists_the_sites_and_outcomes_its_lines_expect() {
    let project = fixtures();
    let (sites, diagnostics) = build(&project).unwrap();
    for (path, text) in &project.files {
        let with_outcomes = expected(text)
            .iter()
            .any(|(_, _, _, outcome)| !outcome.is_empty());
        assert_eq!(found(&sites, path, with_outcomes), expected(text), "{path}");
        let broken = diagnostics.iter().any(|diagnostic| {
            diagnostic.code == Code::SyntaxError
                && diagnostic.at.as_ref().is_some_and(|at| at.path == *path)
        });
        assert_eq!(broken, path.starts_with("broken/"), "{path}");
    }
}

#[test]
fn a_syntax_error_says_where_and_what_to_do() {
    let (_, diagnostics) = build(&fixtures()).unwrap();
    let syntax: Vec<Diagnostic> = diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.code == Code::SyntaxError)
        .collect();
    assert_eq!(
        jsonl::write(&syntax),
        r#"{"code":"syntax-error","at":{"path":"broken/recovery.py","lines":[4,4]},"message":"`broken/recovery.py` does not parse here: Expected `)`, found newline; the sites of the rest of the file come from the tree the parser recovered, so fix the syntax to list them all"}
{"code":"syntax-error","at":{"path":"broken/recovery.py","lines":[4,4]},"message":"`broken/recovery.py` does not parse here: Expected a parameter or the end of the parameter list; the sites of the rest of the file come from the tree the parser recovered, so fix the syntax to list them all"}
"#
    );
}

#[test]
fn each_import_of_a_module_nowhere_is_a_module_missing_error() {
    let (sites, diagnostics) = build(&fixtures()).unwrap();
    let missing: BTreeSet<&Id> = sites
        .sites
        .iter()
        .filter(|site| written(&site.outcome) == "unresolved:missing_module")
        .map(|site| &site.id)
        .collect();
    let reported: BTreeSet<&Id> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == Code::ModuleMissing)
        .filter_map(|diagnostic| diagnostic.id.as_ref())
        .collect();
    assert_eq!(reported, missing);
    let nowhere = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .id
                .as_ref()
                .is_some_and(|id| id.as_str() == "fx/unit:fx/site:resolve-outcomes-import-nowhere")
        })
        .unwrap();
    assert_eq!(
        nowhere.message,
        "the import `nowhere` names a module that is neither under a root nor in the \
         environment; run `uv sync`, or add the package that provides it with `uv add`"
    );
}

#[test]
fn the_order_of_files_changes_nothing() {
    let project = fixtures();
    let mut reversed = project.clone();
    reversed.files.reverse();
    assert_eq!(build(&reversed).unwrap(), build(&project).unwrap());
}

/// Runs the fixture test in a child process whose environment points ty
/// elsewhere: a decoy `greet` on `PYTHONPATH`, another environment, a
/// user configuration, a reversed constraint order and one thread.
#[test]
fn ambient_variables_change_no_outcome() {
    if std::env::var_os("MENTAL_DECOY_CHILD").is_some() {
        return;
    }
    let decoy = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/decoy");
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "every_fixture_lists_the_sites_and_outcomes_its_lines_expect",
            "--exact",
        ])
        .env("MENTAL_DECOY_CHILD", "1")
        .env("PYTHONPATH", &decoy)
        .env("VIRTUAL_ENV", &decoy)
        .env("CONDA_PREFIX", &decoy)
        .env("XDG_CONFIG_HOME", &decoy)
        .env("TY_CONSTRAINT_SET_ORDER", "reverse")
        .env("RAYON_NUM_THREADS", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");
    assert!(
        stdout.contains("1 passed"),
        "the fixture test ran: {stdout}"
    );
}

#[test]
fn a_python_version_ty_does_not_check_is_an_error() {
    let mut project = fixtures();
    project.unit.python = Some(Python {
        version: "2.7".to_owned(),
        from: PythonFrom::PythonVersionFile,
    });
    let error = build(&project).unwrap_err();
    assert!(
        matches!(error, ResolveError::PythonVersion { .. }),
        "{error:?}"
    );
    assert!(
        error
            .to_string()
            .starts_with("the type checker does not check Python 2.7: "),
        "{error}"
    );
}

#[test]
fn a_unit_with_no_environment_or_python_version_is_an_error() {
    let mut project = fixtures();
    project.unit.environment = None;
    assert_eq!(
        build(&project).unwrap_err().to_string(),
        "the build unit records no environment, which the type checker needs; close the gaps \
         of the build model first"
    );
    project.unit.python = None;
    assert_eq!(
        build(&project).unwrap_err(),
        ResolveError::Unprepared {
            missing: "Python version"
        }
    );
}

/// ty makes the environment's path canonical; a project reached through a
/// symbolic link, as `/tmp` is on macOS, resolves alike.
#[cfg(unix)]
#[test]
fn a_project_reached_through_a_symbolic_link_resolves_alike() {
    let project = fixtures();
    let link = std::env::temp_dir().join(format!("mental-fixtures-{}", std::process::id()));
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(&project.dir, &link).unwrap();
    let mut linked = project.clone();
    linked.dir.clone_from(&link);
    let through_link = build(&linked);
    fs::remove_file(&link).unwrap();
    assert_eq!(through_link.unwrap(), build(&project).unwrap());
}

#[test]
fn a_project_directory_ty_cannot_match_is_an_error() {
    let mut project = fixtures();
    project.dir = project.dir.join("../fixtures");
    let error = build(&project).unwrap_err();
    assert!(matches!(error, ResolveError::ProjectDir(_)), "{error:?}");
    assert!(
        error
            .to_string()
            .ends_with("is relative or has a `..` segment; pass it absolute and with no `..`, as the type checker matches it against the roots"),
        "{error}"
    );
    project.dir = PathBuf::from("tests/fixtures");
    assert!(matches!(
        build(&project).unwrap_err(),
        ResolveError::ProjectDir(_)
    ));
}

#[test]
fn an_environment_that_is_not_there_is_an_error() {
    let mut project = fixtures();
    project.unit.environment = Some("no-such-environment".to_owned());
    let error = build(&project).unwrap_err();
    assert!(matches!(error, ResolveError::Project(_)), "{error:?}");
    assert!(
        error
            .to_string()
            .starts_with("the type checker could not open the project: "),
        "{error}"
    );
}

#[test]
fn a_file_that_is_not_on_disk_is_an_error() {
    let mut project = fixtures();
    project
        .files
        .push(("gone/missing.py".to_owned(), "x = 1\n".to_owned()));
    let error = build(&project).unwrap_err();
    assert!(matches!(error, ResolveError::File { .. }), "{error:?}");
    assert!(
        error
            .to_string()
            .starts_with("the type checker could not read `gone/missing.py`: "),
        "{error}"
    );
}

/// The number of sites of each syntax kind in a file, counted apart from
/// the extractor: every call that is not a decorator, decorator, imported
/// name, class base, attribute assigned to and type an `except` names.
#[derive(Default)]
struct Count(BTreeMap<&'static str, usize>);

impl Count {
    fn add(&mut self, kind: &'static str, n: usize) {
        *self.0.entry(kind).or_default() += n;
    }
}

impl<'a> Visitor<'a> for Count {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Import(import) => self.add("import", import.names.len()),
            Stmt::ImportFrom(import) => self.add("import", import.names.len()),
            Stmt::ClassDef(class) => self.add("base-class", class.bases().len()),
            _ => {}
        }
        visitor::walk_stmt(self, stmt);
    }

    fn visit_decorator(&mut self, decorator: &'a Decorator) {
        self.add("decorator", 1);
        if matches!(decorator.expression, Expr::Call(_)) {
            self.add("decorator-call", 1);
        }
        visitor::walk_decorator(self, decorator);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Call(_) => self.add("call", 1),
            Expr::Attribute(attribute) if attribute.ctx.is_store() => {
                self.add("write-receiver", 1);
            }
            _ => {}
        }
        visitor::walk_expr(self, expr);
    }

    fn visit_except_handler(&mut self, handler: &'a ExceptHandler) {
        let ExceptHandler::ExceptHandler(clause) = handler;
        let types = match clause.type_.as_deref() {
            None => 0,
            Some(Expr::Tuple(tuple)) => tuple.elts.len(),
            Some(_) => 1,
        };
        self.add("handler-type", types);
        visitor::walk_except_handler(self, handler);
    }
}

fn counted(text: &str) -> BTreeMap<&'static str, usize> {
    let mut count = Count::default();
    count.visit_body(
        &parse_unchecked_source(text, PySourceType::Python)
            .syntax()
            .body,
    );
    let calls = count.0.remove("call").unwrap_or(0);
    let decorator_calls = count.0.remove("decorator-call").unwrap_or(0);
    count.add("callee", calls - decorator_calls);
    count.0.retain(|_, n| *n > 0);
    count.0
}

fn listed(sites: &[&Site]) -> BTreeMap<&'static str, usize> {
    let mut count = Count::default();
    for site in sites.iter().filter(|site| site.kind != SiteKind::Escape) {
        count.add(site.kind.as_str(), 1);
    }
    count.0
}

fn check_every_site_listed_once(project: &Project) {
    let (sites, _) = build(project).expect("the project resolves");
    for (path, text) in &project.files {
        let of_file: Vec<&Site> = sites
            .sites
            .iter()
            .zip(&sites.anchors)
            .filter(|(_, anchor)| anchor.path == *path)
            .map(|(site, _)| site)
            .collect();
        assert_eq!(listed(&of_file), counted(text), "{path}");
    }
}

#[test]
fn every_site_of_every_fixture_is_listed_once() {
    check_every_site_listed_once(&fixtures());
}

/// report-jobs with its own environment, which `uv sync` makes; the
/// tests that need it run when `RUN_SLOW_TESTS` is set.
fn report_jobs() -> Option<Project> {
    std::env::var_os("RUN_SLOW_TESTS")?;
    Some(report_jobs_in(".venv"))
}

/// report-jobs checked against `environment`, relative to its directory.
fn report_jobs_in(environment: &str) -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/report-jobs")
        .canonicalize()
        .expect("report-jobs is in examples/");
    let files = ["api", "cli", "db", "models", "tasks"]
        .into_iter()
        .map(|module| {
            let path = format!("src/app/{module}.py");
            let text = fs::read_to_string(dir.join(&path)).expect("report-jobs' files read");
            (path, text)
        })
        .collect();
    let pins = jsonl::read(
        &fs::read_to_string(dir.join("model/pin.jsonl")).expect("the pin reference reads"),
    )
    .expect("the pin reference is canonical");
    Project {
        records: records(&dir.join(environment)),
        dir,
        unit: unit("reports", "src", environment),
        pins,
        files,
    }
}

/// report-jobs checked against the fixtures' environment, where none of its
/// dependencies is installed: every site of the syntax tree, with its ID,
/// kind and anchor, is that of its reference whatever its outcome. Escapes
/// are left out, since telling one apart needs its dependencies.
fn report_jobs_without_its_dependencies() -> Project {
    report_jobs_in("../../extract/tests/fixtures/.venv")
}

#[test]
fn report_jobs_lists_its_reference_sites_without_its_dependencies() {
    let project = report_jobs_without_its_dependencies();
    let (sites, _) = build(&project).unwrap();
    let reference = project.dir.join("model");
    let read = |name: &str| fs::read_to_string(reference.join(name)).unwrap();
    let syntax = |site: &Site| site.kind != SiteKind::Escape;
    let listed: BTreeSet<(String, SiteKind)> = sites
        .sites
        .iter()
        .filter(|site| syntax(site))
        .map(|site| (site.id.to_string(), site.kind))
        .collect();
    let expected: BTreeSet<(String, SiteKind)> = jsonl::read::<Site>(&read("site.jsonl"))
        .unwrap()
        .into_iter()
        .filter(syntax)
        .map(|site| (site.id.to_string(), site.kind))
        .collect();
    assert_eq!(listed, expected);
    let escape = |id: &Id| id.as_str().contains("-escape-");
    let anchors: Vec<Anchor> = jsonl::read::<Anchor>(&read("anchors.jsonl"))
        .unwrap()
        .into_iter()
        .filter(|anchor| anchor.id.as_str().contains("/site:") && !escape(&anchor.id))
        .collect();
    let built: Vec<Anchor> = sites
        .anchors
        .into_iter()
        .filter(|anchor| !escape(&anchor.id))
        .collect();
    assert_eq!(jsonl::write(&built), jsonl::write(&anchors));
}

#[test]
fn every_site_of_report_jobs_is_listed_once() {
    check_every_site_listed_once(&report_jobs_without_its_dependencies());
}

#[test]
fn report_jobs_resolves_to_its_reference_twice_alike() {
    let Some(project) = report_jobs() else {
        return;
    };
    let reference = project.dir.join("model");
    let read = |name: &str| fs::read_to_string(reference.join(name)).unwrap();
    let (sites, diagnostics) = build(&project).unwrap();
    assert_eq!(diagnostics, []);
    assert_eq!(jsonl::write(&sites.sites), read("site.jsonl"));
    let anchors: Vec<Anchor> = jsonl::read::<Anchor>(&read("anchors.jsonl"))
        .unwrap()
        .into_iter()
        .filter(|anchor| anchor.id.as_str().contains("/site:"))
        .collect();
    assert_eq!(jsonl::write(&sites.anchors), jsonl::write(&anchors));
    assert_eq!(build(&project).unwrap(), (sites, diagnostics));
}
