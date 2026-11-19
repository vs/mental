//! The reference sites of Python files: every fixture in `tests/fixtures/`
//! against the `# $ kind=slug` expectations on its lines, an independent
//! count of every site, every prefix of every fixture, and report-jobs
//! against its hand-written reference.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use mental_extract::{PythonFile, Sites, list_sites};
use mental_model::{Anchor, BuildUnit, Code, Diagnostic, Id, Site, Source, jsonl};
use ruff_python_ast::visitor::{self, Visitor};
use ruff_python_ast::{Decorator, ExceptHandler, Expr, PySourceType, Stmt};
use ruff_python_parser::parse_unchecked_source;

const MARK: &str = "# $ ";

fn unit(name: &str, root: &str) -> BuildUnit {
    BuildUnit {
        id: Id::parse(&format!("{name}/unit:{name}")).expect("the test names units with slugs"),
        dir: ".".to_owned(),
        roots: BTreeSet::from([root.to_owned()]),
        python: None,
        requires_python: None,
        lock: None,
        environment: None,
        source: Source::Extractor,
    }
}

/// Lists the sites of `files` and checks what holds for every listing: one
/// anchor per site, each quote at its lines, and IDs unique.
fn list(unit: &BuildUnit, files: &[(String, String)]) -> (Sites, Vec<Diagnostic>) {
    let python: Vec<PythonFile> = files
        .iter()
        .map(|(path, text)| PythonFile { path, text })
        .collect();
    let (sites, diagnostics) = list_sites(unit, &python);
    let ids: BTreeSet<&Id> = sites.sites.iter().map(|site| &site.id).collect();
    assert_eq!(ids.len(), sites.sites.len(), "every site has its own ID");
    let anchored: Vec<&Id> = sites.anchors.iter().map(|anchor| &anchor.id).collect();
    let listed: Vec<&Id> = sites.sites.iter().map(|site| &site.id).collect();
    assert_eq!(anchored, listed, "one anchor per site");
    for anchor in &sites.anchors {
        let (_, text) = files
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
    (sites, diagnostics)
}

fn fixtures() -> Vec<(String, String)> {
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("the fixture folder reads") {
            let path = entry.expect("a fixture folder entry reads").path();
            if path.is_dir() {
                walk(&path, found);
            } else {
                found.push(path);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut found = Vec::new();
    walk(&root, &mut found);
    found.sort();
    found
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&root)
                .expect("a fixture lies under the fixture folder")
                .to_str()
                .expect("a fixture path is UTF-8");
            (
                relative.to_owned(),
                fs::read_to_string(&path).expect("a fixture reads as UTF-8"),
            )
        })
        .collect()
}

/// `(line, kind, slug)` of each site, as `# $ kind=slug` writes them.
type Seen = BTreeSet<(u32, String, String)>;

fn expected(text: &str) -> Seen {
    let mut seen = Seen::new();
    for (number, line) in (1..).zip(text.lines()) {
        let Some((_, marks)) = line.split_once(MARK) else {
            continue;
        };
        for mark in marks.split_whitespace() {
            let (kind, slug) = mark
                .split_once('=')
                .expect("an expectation is written `kind=slug`");
            seen.insert((number, kind.to_owned(), slug.to_owned()));
        }
    }
    seen
}

fn found(sites: &Sites) -> Seen {
    sites
        .sites
        .iter()
        .zip(&sites.anchors)
        .map(|(site, anchor)| {
            let [first, _]: [u32; 2] = anchor.lines.into();
            let (_, slug) = site
                .id
                .as_str()
                .split_once("/site:")
                .expect("a site ID ends in a site segment");
            (first, site.kind.as_str().to_owned(), slug.to_owned())
        })
        .collect()
}

#[test]
fn every_fixture_lists_the_sites_its_lines_expect() {
    for (path, text) in fixtures() {
        let (sites, diagnostics) = list(&unit("fx", "."), &[(path.clone(), text.clone())]);
        assert_eq!(found(&sites), expected(&text), "{path}");
        let broken = diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == Code::SyntaxError);
        assert!(broken, "{path}: {diagnostics:?}");
        assert_eq!(
            !diagnostics.is_empty(),
            path.starts_with("broken/"),
            "{path}"
        );
    }
}

#[test]
fn a_syntax_error_says_where_and_what_to_do() {
    let path = "broken/recovery.py".to_owned();
    let text = fixtures()
        .into_iter()
        .find(|(name, _)| *name == path)
        .unwrap()
        .1;
    let (_, diagnostics) = list(&unit("fx", "."), &[(path, text)]);
    assert_eq!(
        jsonl::write(&diagnostics),
        r#"{"code":"syntax-error","at":{"path":"broken/recovery.py","lines":[4,4]},"message":"`broken/recovery.py` does not parse here: Expected `)`, found newline; the sites of the rest of the file come from the tree the parser recovered, so fix the syntax to list them all"}
{"code":"syntax-error","at":{"path":"broken/recovery.py","lines":[4,4]},"message":"`broken/recovery.py` does not parse here: Expected a parameter or the end of the parameter list; the sites of the rest of the file come from the tree the parser recovered, so fix the syntax to list them all"}
"#
    );
}

/// The number of sites of each kind in a file, counted apart from the
/// extractor: every call that is not a decorator, decorator, imported name,
/// class base, attribute assigned to and type an `except` names.
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
    for site in sites {
        count.add(site.kind.as_str(), 1);
    }
    count.0
}

#[test]
fn every_site_of_every_fixture_and_of_report_jobs_is_listed_once() {
    let report_jobs = report_jobs();
    let fixtures = fixtures();
    for (unit, files) in [
        (unit("fx", "."), &fixtures),
        (unit("reports", "src"), &report_jobs),
    ] {
        let (sites, _) = list(&unit, files);
        for (path, text) in files {
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
}

#[test]
fn no_prefix_of_a_fixture_panics_and_each_syntax_error_is_reported() {
    for (path, text) in fixtures() {
        for (end, _) in text.char_indices() {
            let prefix = &text[..end];
            let (_, diagnostics) = list(&unit("fx", "."), &[(path.clone(), prefix.to_owned())]);
            let errors = parse_unchecked_source(prefix, PySourceType::Python)
                .errors()
                .len();
            assert_eq!(diagnostics.len(), errors, "{path} up to byte {end}");
        }
    }
}

fn report_jobs() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/report-jobs");
    ["api", "cli", "db", "models", "tasks"]
        .into_iter()
        .map(|module| {
            let path = format!("src/app/{module}.py");
            let text = fs::read_to_string(root.join(&path)).expect("report-jobs' files read");
            (path, text)
        })
        .collect()
}

#[test]
fn report_jobs_lists_its_reference_sites() {
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/report-jobs/model");
    let read = |name: &str| fs::read_to_string(reference.join(name)).unwrap();
    let files = report_jobs();
    let (sites, diagnostics) = list(&unit("reports", "src"), &files);
    assert_eq!(diagnostics, []);
    assert_eq!(jsonl::write(&sites.sites), read("site.jsonl"));
    let anchors: Vec<Anchor> = jsonl::read::<Anchor>(&read("anchors.jsonl"))
        .unwrap()
        .into_iter()
        .filter(|anchor| anchor.id.as_str().contains("/site:"))
        .collect();
    assert_eq!(jsonl::write(&sites.anchors), jsonl::write(&anchors));
    let mut reversed = files.clone();
    reversed.reverse();
    assert_eq!(
        list(&unit("reports", "src"), &reversed).0,
        sites,
        "the order of files changes nothing"
    );
}

#[test]
fn a_group_spans_the_files_of_the_unit_in_file_order() {
    let package = (
        "app/__init__.py".to_owned(),
        "def tasks():\n    f()\n".to_owned(),
    );
    let module = ("app/tasks.py".to_owned(), "f()\n".to_owned());
    let ids = |files: &[(String, String)]| -> Vec<(String, String)> {
        let (sites, _) = list(&unit("fx", "."), files);
        sites
            .sites
            .iter()
            .zip(&sites.anchors)
            .map(|(site, anchor)| (anchor.path.clone(), site.id.to_string()))
            .collect()
    };
    let listed = ids(&[package.clone(), module.clone()]);
    assert_eq!(
        listed,
        [
            (
                "app/__init__.py".to_owned(),
                "fx/unit:fx/site:app-tasks-callee-f-0".to_owned()
            ),
            (
                "app/tasks.py".to_owned(),
                "fx/unit:fx/site:app-tasks-callee-f-1".to_owned()
            ),
        ]
    );
    assert_eq!(ids(&[module, package]), listed);
}
