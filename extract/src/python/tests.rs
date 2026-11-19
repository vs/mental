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
