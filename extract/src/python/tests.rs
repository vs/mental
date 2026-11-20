use std::fs;
use std::path::{Path, PathBuf};

use super::*;

#[track_caller]
fn module(path: &str, roots: &[&str]) -> String {
    module_name(path, &roots.iter().map(|root| (*root).to_owned()).collect())
}

#[test]
fn a_module_is_its_path_under_the_root() {
    assert_eq!(module("app/jobs/run.py", &["."]), "app.jobs.run");
    assert_eq!(module("src/app/run.py", &["src"]), "app.run");
}

#[test]
fn a_package_is_its_folder() {
    assert_eq!(module("app/__init__.py", &["."]), "app");
    assert_eq!(module("__init__.py", &["."]), "");
    assert_eq!(module("app/x__init__.py", &["."]), "app.x__init__");
}

#[test]
fn the_deepest_root_names_the_module() {
    assert_eq!(module("src/app/run.py", &[".", "src"]), "app.run");
    assert_eq!(module("tools/run.py", &[".", "src"]), "tools.run");
}

#[test]
fn a_group_drops_only_trailing_digit_segments() {
    assert_eq!(group("m-callee-s-commit-0-12"), "m-callee-s-commit");
    assert_eq!(group("m-callee-s3-put-object"), "m-callee-s3-put-object");
    assert_eq!(group("m-callee-2-x"), "m-callee-2-x");
}

fn fixtures(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() && !path.ends_with(".venv") {
            fixtures(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "py") {
            found.push(path);
        }
    }
}

/// Parsing and listing see one file and no checker, so every prefix of
/// every fixture is listed without a panic and with each syntax error.
#[test]
fn no_prefix_of_a_fixture_panics_and_each_syntax_error_is_reported() {
    let mut found = Vec::new();
    fixtures(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        &mut found,
    );
    for path in found {
        let text = fs::read_to_string(&path).unwrap();
        for (end, _) in text.char_indices() {
            let prefix = &text[..end];
            let (module, diagnostics) = parse::parse("fixture.py", prefix);
            sites::sites("fixture", &module.body);
            let errors = ruff_python_parser::parse_unchecked_source(
                prefix,
                ruff_python_ast::PySourceType::Python,
            )
            .errors()
            .len();
            assert_eq!(
                diagnostics.len(),
                errors,
                "{} up to byte {end}",
                path.display()
            );
        }
    }
}
