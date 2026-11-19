use mental_model::{Code, Id, Lines, Location};

use super::*;

/// The text line of a diagnostic with `code`, at `at` and about `id`.
#[track_caller]
fn check(code: Code, at: Option<(&str, u32, u32)>, id: Option<&str>) -> String {
    text(&Diagnostic {
        id: id.map(|id| Id::parse(id).unwrap()),
        code,
        at: at.map(|(path, first, last)| Location {
            path: path.to_owned(),
            lines: Lines::new(first, last).unwrap(),
        }),
        message: "m".to_owned(),
    })
}

#[test]
fn a_diagnostic_names_its_place_or_else_its_record() {
    let lines = [
        check(Code::LockAmbiguous, Some(("uv.lock", 6, 11)), None),
        check(
            Code::PythonVersionUnknown,
            Some((".python-version", 1, 1)),
            Some("shop/unit:shop"),
        ),
        check(Code::LockMissing, None, Some("shop/unit:shop")),
        check(Code::ProjectMissing, None, None),
        check(Code::SyntaxError, Some(("src/app.py", 3, 3)), None),
    ];
    assert_eq!(
        lines.join("\n"),
        "error[lock-ambiguous] uv.lock:6-11: m
error[python-version-unknown] .python-version:1: m
error[lock-missing] shop/unit:shop: m
error[project-missing]: m
warning[syntax-error] src/app.py:3: m"
    );
}
