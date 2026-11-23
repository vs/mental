use mental_model::jsonl;

use super::*;

/// The Python version from these two files, or the diagnostic.
#[track_caller]
fn check(python_version: Option<&str>, pyvenv: Option<&str>) -> String {
    let unit = Id::parse("shop/unit:shop").unwrap();
    match version(&unit, ".", python_version, pyvenv) {
        Ok(python) => format!("{} from {:?}", python.version, python.from),
        Err(problem) => jsonl::write(&[problem]),
    }
}

const UV_PYVENV: &str = "home = /usr/local/bin\nimplementation = CPython\nuv = 0.9.2\nversion_info = 3.12.8\nprompt = shop\n";

#[test]
fn python_version_file_states_it() {
    assert_eq!(check(Some("3.12\n"), None), "3.12 from PythonVersionFile");
}

#[test]
fn blank_lines_comments_and_spaces_are_skipped() {
    assert_eq!(
        check(Some("# pinned\n\n  3.13.1  \n3.12\n"), None),
        "3.13.1 from PythonVersionFile"
    );
}

#[test]
fn python_version_file_wins_over_the_environment() {
    assert_eq!(
        check(Some("3.13"), Some(UV_PYVENV)),
        "3.13 from PythonVersionFile"
    );
}

#[test]
fn the_environment_states_it_when_there_is_no_file() {
    assert_eq!(check(None, Some(UV_PYVENV)), "3.12.8 from Pyvenv");
    assert_eq!(
        check(None, Some("home = /usr/bin\nversion = 3.11.4\n")),
        "3.11.4 from Pyvenv"
    );
}

#[test]
fn a_file_with_no_version_states_none() {
    assert_eq!(
        check(Some("# none yet\n\n"), Some(UV_PYVENV)),
        r#"{"id":"shop/unit:shop","code":"python-version-unknown","message":"`.python-version` states no Python version; write one in `.python-version`, for example with `uv python pin 3.12`"}
"#
    );
    assert_eq!(
        check(
            None,
            Some("home = /usr/bin\nversion-ish = 3.12\nno key here\n")
        ),
        r#"{"id":"shop/unit:shop","code":"python-version-unknown","message":"`.venv/pyvenv.cfg` states no Python version; write one in `.python-version`, for example with `uv python pin 3.12`"}
"#
    );
}

#[test]
fn with_neither_file_there_is_no_version() {
    assert_eq!(
        check(None, None),
        r#"{"id":"shop/unit:shop","code":"python-version-unknown","message":"there is no `.python-version` and no environment to take the Python version from; write one in `.python-version`, for example with `uv python pin 3.12`"}
"#
    );
}

#[test]
fn a_version_is_two_or_three_numbers() {
    let stated = |text: &str| {
        let verdict = check(Some(text), None);
        let wrong = verdict.contains("which is not a version");
        format!("{text}: {}", if wrong { "no" } else { "yes" })
    };
    let verdicts: Vec<String> = [
        "3",
        "3.12",
        "3.12.1",
        "3.12.1.0",
        "3..1",
        "3.12.",
        "3.13t",
        "pypy@3.10",
    ]
    .into_iter()
    .map(stated)
    .collect();
    assert_eq!(
        verdicts.join("\n"),
        "3: no\n3.12: yes\n3.12.1: yes\n3.12.1.0: no\n3..1: no\n3.12.: no\n3.13t: no\npypy@3.10: no"
    );
}

#[test]
fn a_wrong_version_is_located() {
    assert_eq!(
        check(Some("# pinned\npypy@3.10\n"), None),
        r#"{"id":"shop/unit:shop","code":"python-version-unknown","at":{"path":".python-version","lines":[2,2]},"message":"`.python-version` says the Python version is `pypy@3.10`, which is not a version such as 3.12 or 3.12.1; write one in `.python-version`, for example with `uv python pin 3.12`"}
"#
    );
    assert_eq!(
        check(
            None,
            Some("home = /usr/bin\nversion_info = 3.12.8.final.0\n")
        ),
        r#"{"id":"shop/unit:shop","code":"python-version-unknown","at":{"path":".venv/pyvenv.cfg","lines":[2,2]},"message":"`.venv/pyvenv.cfg` says the Python version is `3.12.8.final.0`, which is not a version such as 3.12 or 3.12.1; write one in `.python-version`, for example with `uv python pin 3.12`"}
"#
    );
}
