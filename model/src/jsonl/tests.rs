use super::*;
use crate::{Anchor, Diagnostic};

const UNIT: &str =
    r#"{"id":"reports/unit:reports","path":"pyproject.toml","lines":[1,1],"quote":"[project]"}"#;
const PIN: &str = r#"{"id":"reports/unit:reports/pin:sqlalchemy","path":"uv.lock","lines":[40,44],"quote":"name = \"sqlalchemy\""}"#;

/// Reads `text` as records of `T` and writes them back, which must give the
/// same bytes, or gives the error.
#[track_caller]
fn check<T: DeserializeOwned + Serialize + Ord>(text: &str) -> String {
    match read::<T>(text) {
        Ok(records) => {
            assert_eq!(write(&records), text);
            format!("{} records", records.len())
        }
        Err(error) => error.to_string(),
    }
}

#[test]
fn an_empty_file_has_no_records() {
    assert_eq!(check::<Anchor>(""), "0 records");
}

#[test]
fn a_sorted_file_is_read() {
    assert_eq!(check::<Anchor>(&format!("{UNIT}\n{PIN}\n")), "2 records");
}

#[test]
fn identical_lines_are_read_for_the_checker_to_report() {
    assert_eq!(check::<Anchor>(&format!("{UNIT}\n{UNIT}\n")), "2 records");
}

#[test]
fn the_writer_sorts_by_id() {
    let records: Vec<Anchor> = [PIN, UNIT]
        .map(|line| serde_json::from_str(line).unwrap())
        .into();
    assert_eq!(write(&records), format!("{UNIT}\n{PIN}\n"));
}

#[test]
fn a_file_ends_with_a_newline() {
    assert_eq!(
        check::<Anchor>(&format!("{UNIT}\n{PIN}")),
        "line 2: the file does not end with a newline"
    );
}

#[test]
fn a_blank_line_is_not_a_record() {
    assert_eq!(
        check::<Anchor>(&format!("{UNIT}\n\n{PIN}\n")),
        "line 2, column 0: EOF while parsing a value"
    );
}

#[test]
fn a_lone_newline_is_a_blank_line() {
    assert_eq!(
        check::<Anchor>("\n"),
        "line 1, column 0: EOF while parsing a value"
    );
}

#[test]
fn an_unsorted_file_is_rejected() {
    assert_eq!(
        check::<Anchor>(&format!("{PIN}\n{UNIT}\n")),
        "line 2: the record sorts before the one above it"
    );
}

#[test]
fn spaces_are_not_canonical() {
    assert_eq!(
        check::<Anchor>(&format!("{}\n", UNIT.replace(',', ", "))),
        "line 1, column 30: the line differs here from how the record is written"
    );
}

#[test]
fn a_carriage_return_is_not_canonical() {
    assert_eq!(
        check::<Anchor>(&format!("{UNIT}\r\n")),
        "line 1, column 88: the line differs here from how the record is written"
    );
}

#[test]
fn fields_out_of_order_are_not_canonical() {
    assert_eq!(
        check::<Anchor>(
            r#"{"path":"pyproject.toml","id":"reports/unit:reports","lines":[1,1],"quote":"[project]"}
"#
        ),
        "line 1, column 3: the line differs here from how the record is written"
    );
}

#[test]
fn an_empty_field_written_out_is_not_canonical() {
    assert_eq!(
        check::<Diagnostic>(
            r#"{"id":null,"code":"lock-missing","message":"m"}
"#
        ),
        "line 1, column 3: the line differs here from how the record is written"
    );
}

#[test]
fn an_unknown_field_is_named() {
    assert_eq!(
        check::<Diagnostic>(
            r#"{"code":"lock-missing","message":"m","fix":"uv lock"}
"#
        ),
        "line 1, column 42: unknown field `fix`, expected one of `id`, `code`, `at`, `message`"
    );
}

#[test]
fn an_unknown_code_is_named() {
    assert_eq!(
        check::<Diagnostic>(
            r#"{"code":"lock-lost","message":"m"}
"#
        ),
        "line 1, column 19: unknown variant `lock-lost`, expected one of `environment-missing`, `lock-missing`, `lock-version-unknown`, `module-missing`, `project-missing`, `python-version-unknown`, `syntax-error`"
    );
}

#[test]
fn a_bad_id_gives_the_id_parser_message() {
    assert_eq!(
        check::<Anchor>(&format!(
            "{UNIT}\n{}\n",
            PIN.replace("sqlalchemy\"", "SQLAlchemy\"")
        )),
        "line 2, column 43: invalid ID `reports/unit:reports/pin:SQLAlchemy` at byte 25: `S` is not allowed in a slug, which has only a-z, 0-9 and single hyphens"
    );
}

#[test]
fn an_escape_serde_would_not_write_is_not_canonical() {
    assert_eq!(
        check::<Anchor>(&format!("{}\n", UNIT.replace("[project]", "\\u00e9"))),
        "line 1, column 77: the line differs here from how the record is written"
    );
}
