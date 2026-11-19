use mental_model::jsonl;

use super::*;

/// Reads `text` as a `pyproject.toml`: the project's slug, where its name
/// stands and its `requires-python`, or the diagnostic.
#[track_caller]
fn check(text: &str) -> String {
    match read(text) {
        Ok(project) => format!(
            "{} at {:?}, requires {:?}",
            project.slug.as_str(),
            &text[project.name],
            project.requires_python
        ),
        Err(problem) => jsonl::write(&[problem]),
    }
}

#[test]
fn the_project_is_named_and_may_require_a_python() {
    assert_eq!(
        check("[project]\nname = \"reports\"\nrequires-python = \">=3.12\"\n"),
        r#"reports at "\"reports\"", requires Some(">=3.12")"#
    );
    assert_eq!(
        check("[project]\nname = \"reports\"\n"),
        r#"reports at "\"reports\"", requires None"#
    );
}

#[test]
fn other_tables_and_fields_are_not_read() {
    assert_eq!(
        check(
            "[build-system]\nrequires = []\n\n[project]\nversion = \"1\"\nname = \"a\"\n\n[tool.uv]\n"
        ),
        r#"a at "\"a\"", requires None"#
    );
}

#[test]
fn the_system_is_named_by_the_slug_of_the_project_name() {
    assert_eq!(
        check("[project]\nname = \"Report_Jobs.Core\"\n"),
        r#"report-jobs-core at "\"Report_Jobs.Core\"", requires None"#
    );
}

#[test]
fn a_name_with_no_letter_names_no_system() {
    assert_eq!(
        check("[project]\nname = \"2048\"\n"),
        r#"{"code":"project-invalid","at":{"path":"pyproject.toml","lines":[2,2]},"message":"the project name `2048` has no ASCII letter, and the system is named after the project; give the project a name with one"}
"#
    );
}

#[test]
fn a_file_that_is_not_toml_is_invalid() {
    assert_eq!(
        check("[project\nname = \"a\"\n"),
        r#"{"code":"project-invalid","at":{"path":"pyproject.toml","lines":[1,1]},"message":"`pyproject.toml` does not read: unclosed table, expected `]`; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
"#
    );
}

#[test]
fn a_project_needs_its_table_and_a_name() {
    assert_eq!(
        check("[tool.ruff]\nline-length = 100\n"),
        r#"{"code":"project-invalid","at":{"path":"pyproject.toml","lines":[1,1]},"message":"`pyproject.toml` does not read: missing field `project`; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
"#
    );
    assert_eq!(
        check("[project]\nversion = \"1\"\n"),
        r#"{"code":"project-invalid","at":{"path":"pyproject.toml","lines":[1,1]},"message":"`pyproject.toml` does not read: missing field `name`; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
"#
    );
    assert_eq!(
        check("[project]\nname = 7\n"),
        r#"{"code":"project-invalid","at":{"path":"pyproject.toml","lines":[2,2]},"message":"`pyproject.toml` does not read: invalid type: integer `7`, expected a string; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
"#
    );
}
