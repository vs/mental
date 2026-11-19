use super::*;

/// Reads one anchor and writes it back, or gives the error.
#[track_caller]
fn check(json: &str) -> String {
    match serde_json::from_str::<Anchor>(json) {
        Ok(anchor) => serde_json::to_string(&anchor).unwrap(),
        Err(error) => error.to_string(),
    }
}

#[test]
fn an_anchor_writes_its_lines_as_a_pair() {
    let json = r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[1,3],"quote":"[project]"}"#;
    assert_eq!(check(json), json);
}

#[test]
fn one_line_is_a_range_whose_ends_meet() {
    let json = r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[4,4],"quote":"[project]"}"#;
    assert_eq!(check(json), json);
}

#[test]
fn lines_count_from_one() {
    assert_eq!(
        check(
            r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[0,3],"quote":"[project]"}"#
        ),
        "lines are `[first, last]`, counted from 1, with first <= last at line 1 column 66"
    );
}

#[test]
fn the_last_line_cannot_come_before_the_first() {
    assert_eq!(
        check(
            r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[3,2],"quote":"[project]"}"#
        ),
        "lines are `[first, last]`, counted from 1, with first <= last at line 1 column 66"
    );
}

#[test]
fn lines_are_a_pair() {
    assert_eq!(
        check(
            r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[3],"quote":"[project]"}"#
        ),
        "invalid length 1, expected an array of length 2 at line 1 column 64"
    );
}

#[test]
fn an_unknown_field_is_rejected() {
    assert_eq!(
        check(
            r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[1,1],"quote":"[project]","symbol":"x"}"#
        ),
        "unknown field `symbol`, expected one of `id`, `path`, `lines`, `quote` at line 1 column 95"
    );
}

#[test]
fn every_field_is_required() {
    assert_eq!(
        check(r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[1,1]}"#),
        "missing field `quote` at line 1 column 67"
    );
}
