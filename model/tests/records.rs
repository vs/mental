//! What the record types accept when read.

use mental_model::{BuildUnit, Pin, Site, jsonl};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Reads one line as a record of `T`: `ok`, or the error.
#[track_caller]
fn check<T: DeserializeOwned + Serialize + Ord>(line: &str) -> String {
    match jsonl::read::<T>(&format!("{line}\n")) {
        Ok(_) => "ok".to_owned(),
        Err(error) => error.to_string(),
    }
}

#[test]
fn a_unit_needs_only_its_directory_and_roots() {
    assert_eq!(
        check::<BuildUnit>(
            r#"{"id":"shop/unit:shop","dir":".","roots":["."],"source":"extractor"}"#
        ),
        "ok"
    );
}

#[test]
fn roots_are_a_sorted_set() {
    assert_eq!(
        check::<BuildUnit>(
            r#"{"id":"shop/unit:shop","dir":".","roots":["src","lib"],"source":"extractor"}"#
        ),
        "line 1, column 44: the line differs here from how the record is written"
    );
}

#[test]
fn a_root_is_listed_once() {
    assert_eq!(
        check::<BuildUnit>(
            r#"{"id":"shop/unit:shop","dir":".","roots":["src","src"],"source":"extractor"}"#
        ),
        "line 1, column 48: the line differs here from how the record is written"
    );
}

#[test]
fn the_python_version_names_its_file() {
    assert_eq!(
        check::<BuildUnit>(
            r#"{"id":"shop/unit:shop","dir":".","roots":["."],"python":{"version":"3.12","from":"pyproject.toml"},"source":"extractor"}"#
        ),
        "line 1, column 97: unknown variant `pyproject.toml`, expected `.python-version` or `pyvenv.cfg`"
    );
}

#[test]
fn a_record_names_its_source() {
    assert_eq!(
        check::<BuildUnit>(r#"{"id":"shop/unit:shop","dir":".","roots":["."]}"#),
        "line 1, column 47: missing field `source`"
    );
}

#[test]
fn an_unknown_origin_is_rejected() {
    assert_eq!(
        check::<Pin>(
            r#"{"id":"shop/unit:shop/pin:a","package":"a","version":"1","origin":{"svn":"https://x/a"},"source":"extractor"}"#
        ),
        "line 1, column 72: unknown variant `svn`, expected one of `registry`, `git`, `url`, `path`"
    );
}

#[test]
fn a_git_origin_names_its_commit() {
    assert_eq!(
        check::<Pin>(
            r#"{"id":"shop/unit:shop/pin:a","package":"a","version":"1","origin":{"git":{"url":"https://x/a"}},"source":"extractor"}"#
        ),
        "line 1, column 94: missing field `commit`"
    );
}

#[test]
fn an_unknown_site_kind_is_rejected() {
    assert_eq!(
        check::<Site>(
            r#"{"id":"shop/unit:shop/site:shop-escape-f","kind":"escape","source":"extractor"}"#
        ),
        "line 1, column 57: unknown variant `escape`, expected one of `import`, `decorator`, `callee`, `base-class`, `write-receiver`, `handler-type`"
    );
}
