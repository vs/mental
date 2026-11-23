use mental_model::jsonl;

use super::*;

/// Reads `text` as a `pyproject.toml`: the project's slug, where its name
/// stands and its `requires-python`, and the workspace it declares, or the
/// diagnostic.
#[track_caller]
fn check(text: &str) -> String {
    match read(text, PATH) {
        Ok(declared) => {
            let project = declared.project.map(|project| {
                format!(
                    "{} at {:?}, requires {:?}",
                    project.slug.as_str(),
                    &text[project.name],
                    project.requires_python
                )
            });
            let workspace = declared.workspace.map(|workspace| {
                format!(
                    "workspace {:?} but {:?}",
                    workspace.members, workspace.exclude
                )
            });
            [project, workspace]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("; ")
        }
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
        r#"{"code":"project-invalid","message":"`pyproject.toml` has no `[project]` table and no `[tool.uv.workspace]`, so it is neither a project nor a workspace; a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` writes it"}
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

#[test]
fn a_workspace_root_needs_no_project() {
    assert_eq!(
        check(
            "[tool.uv.workspace]\nmembers = [\"backend\", \"packages/*\"]\nexclude = [\"packages/old\"]\n"
        ),
        r#"workspace ["backend", "packages/*"] but ["packages/old"]"#
    );
    assert_eq!(
        check("[project]\nname = \"shop\"\n\n[tool.uv.workspace]\n"),
        r#"shop at "\"shop\"", requires None; workspace [] but []"#
    );
    assert_eq!(workspace("[project]\nname = \"shop\"\n"), Ok(None));
    assert!(workspace("[tool.uv.workspace]\nmembers = 1\n").is_err());
}

#[test]
fn a_member_pattern_reads_only_star_wildcards() {
    let unread = [
        "pkg?", "[ab]", "a/**", "../a", "/a", "{a,b}", "a\\\\b", "./a", "a//b", "",
    ]
    .map(|pattern| {
        check(&format!("[tool.uv.workspace]\nmembers = [\"{pattern}\"]\n"))
            .contains("is not one this build reads")
            .to_string()
    });
    assert_eq!(
        unread.join(" "),
        "true true true true true true true true true true"
    );
    assert_eq!(
        check("[tool.uv.workspace]\nmembers = [\"a\"]\nexclude = [\"b?\"]\n"),
        r#"{"code":"project-invalid","at":{"path":"pyproject.toml","lines":[3,3]},"message":"the workspace pattern `b?` is not one this build reads: it reads a folder under the root in which `*` stands for any name; list the members that way"}
"#
    );
}

#[test]
fn a_member_is_a_folder_a_pattern_names_and_no_exclude_does() {
    let workspace = Workspace {
        members: vec![
            "backend".to_owned(),
            "packages/*".to_owned(),
            "lib-*-py/".to_owned(),
        ],
        exclude: vec!["packages/old*".to_owned()],
    };
    let included = [
        "backend",
        "packages/a",
        "packages/oldest",
        "packages/a/b",
        "lib-x-py",
        "lib--py",
        "lib-x-pz",
        "lib-é-py",
        "front",
    ]
    .map(|dir| workspace.includes(dir).to_string());
    assert_eq!(
        included.join(" "),
        "true true false false true true false true false"
    );
}
