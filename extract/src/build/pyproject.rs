//! `pyproject.toml`: the project's name and `requires-python`, and the
//! members of the uv workspace it is the root of.

use std::ops::Range;

use mental_model::{Code, Diagnostic, Slug, slugify};
use serde::Deserialize;
use toml::Spanned;

pub(super) const PATH: &str = "pyproject.toml";
const FIX: &str = "a project's `pyproject.toml` has a `[project]` table with a `name`, as \
                   `uv init` writes it";

#[derive(Clone)]
pub(super) struct Project {
    pub(super) slug: Slug,
    /// Where the name stands, for the build unit's anchor.
    pub(super) name: Range<usize>,
    pub(super) requires_python: Option<String>,
}

/// What a `pyproject.toml` declares: a project, a workspace, or both.
pub(super) struct Declared {
    pub(super) project: Option<Project>,
    pub(super) workspace: Option<Workspace>,
}

/// The members of a uv workspace: `[tool.uv.workspace]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    /// Each a folder relative to the root, `*` standing for any one name.
    pub members: Vec<String>,
    pub exclude: Vec<String>,
}

impl Workspace {
    /// Whether the folder at `dir`, relative to the root with `/` between
    /// folders, is a member: some member pattern matches it and no
    /// `exclude` pattern does.
    #[must_use]
    pub fn includes(&self, dir: &str) -> bool {
        let matched = |patterns: &[String]| patterns.iter().any(|pattern| matches(pattern, dir));
        matched(&self.members) && !matched(&self.exclude)
    }
}

/// Whether `dir` is the folder `pattern` names, segment by segment, where a
/// `*` in a segment stands for any run of characters but `/`.
fn matches(pattern: &str, dir: &str) -> bool {
    let (patterns, names): (Vec<&str>, Vec<&str>) = (
        pattern.trim_end_matches('/').split('/').collect(),
        dir.split('/').collect(),
    );
    patterns.len() == names.len()
        && patterns
            .iter()
            .zip(&names)
            .all(|(pattern, name)| segment_matches(pattern, name))
}

fn segment_matches(pattern: &str, name: &str) -> bool {
    let Some((head, rest)) = pattern.split_once('*') else {
        return pattern == name;
    };
    let Some(after) = name.strip_prefix(head) else {
        return false;
    };
    (0..=after.len())
        .filter(|&skip| after.is_char_boundary(skip))
        .any(|skip| segment_matches(rest, &after[skip..]))
}

#[derive(Deserialize)]
struct File {
    project: Option<Table>,
    tool: Option<Tool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Table {
    name: Spanned<String>,
    requires_python: Option<String>,
}

#[derive(Deserialize)]
struct Tool {
    uv: Option<Uv>,
}

#[derive(Deserialize)]
struct Uv {
    workspace: Option<WorkspaceTable>,
}

#[derive(Deserialize)]
struct WorkspaceTable {
    #[serde(default)]
    members: Vec<Spanned<String>>,
    #[serde(default)]
    exclude: Vec<Spanned<String>>,
}

/// Only the workspace table, so that a `pyproject.toml` that holds only
/// tool settings, or a `[project]` the build would reject, is no workspace.
#[derive(Deserialize)]
struct WorkspaceFile {
    tool: Option<Tool>,
}

/// The workspace `text` declares, if any, for finding the members before
/// the build, whatever else the file holds.
///
/// # Errors
///
/// When `text` is not TOML, or its workspace table does not read or holds
/// a pattern this build does not read; the build reports it.
pub fn workspace(text: &str) -> Result<Option<Workspace>, Diagnostic> {
    let file = toml::from_str::<WorkspaceFile>(text)
        .map_err(|error| super::unreadable(Code::ProjectInvalid, PATH, text, &error, FIX))?;
    file.tool
        .and_then(|tool| tool.uv)
        .and_then(|uv| uv.workspace)
        .map(|table| patterns(text, PATH, table))
        .transpose()
}

/// Reads `text`, the `pyproject.toml` at `path`. It must declare a project,
/// a workspace, or both.
pub(super) fn read(text: &str, path: &str) -> Result<Declared, Diagnostic> {
    let file = match toml::from_str::<File>(text) {
        Ok(file) => file,
        Err(error) => {
            return Err(super::unreadable(
                Code::ProjectInvalid,
                path,
                text,
                &error,
                FIX,
            ));
        }
    };
    let workspace = file
        .tool
        .and_then(|tool| tool.uv)
        .and_then(|uv| uv.workspace)
        .map(|table| patterns(text, path, table))
        .transpose()?;
    let Some(table) = file.project else {
        return if workspace.is_some() {
            Ok(Declared {
                project: None,
                workspace,
            })
        } else {
            Err(Diagnostic {
                id: None,
                code: Code::ProjectInvalid,
                at: None,
                message: format!(
                    "`{path}` has no `[project]` table and no `[tool.uv.workspace]`, so it is \
                     neither a project nor a workspace; {FIX}"
                ),
            })
        };
    };
    let name = table.name;
    let Some(slug) = slugify(name.get_ref()) else {
        return Err(Diagnostic {
            id: None,
            code: Code::ProjectInvalid,
            at: Some(crate::anchor::location(path, text, name.span())),
            message: format!(
                "the project name `{}` has no ASCII letter, and the system is named after the \
                 project; give the project a name with one",
                name.get_ref()
            ),
        });
    };
    Ok(Declared {
        project: Some(Project {
            slug,
            name: name.span(),
            requires_python: table.requires_python,
        }),
        workspace,
    })
}

/// The member and exclude patterns, each a relative folder in which only
/// `*` is a wildcard: uv's other glob forms are not read yet.
fn patterns(text: &str, path: &str, table: WorkspaceTable) -> Result<Workspace, Diagnostic> {
    let check = |patterns: Vec<Spanned<String>>| {
        patterns
            .into_iter()
            .map(|pattern| {
                let written = pattern.get_ref();
                let unread = written.contains(['?', '[', ']', '{', '}', '\\'])
                    || written
                        .trim_end_matches('/')
                        .split('/')
                        .any(|part| matches!(part, "**" | ".." | "." | ""))
                    || written.starts_with('/');
                if unread {
                    Err(Diagnostic {
                        id: None,
                        code: Code::ProjectInvalid,
                        at: Some(crate::anchor::location(path, text, pattern.span())),
                        message: format!(
                            "the workspace pattern `{written}` is not one this build reads: it \
                             reads a folder under the root in which `*` stands for any name; \
                             list the members that way"
                        ),
                    })
                } else {
                    Ok(pattern.into_inner())
                }
            })
            .collect::<Result<Vec<String>, Diagnostic>>()
    };
    Ok(Workspace {
        members: check(table.members)?,
        exclude: check(table.exclude)?,
    })
}

#[cfg(test)]
mod tests;
