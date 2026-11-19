//! `pyproject.toml`: the project's name and `requires-python`.

use std::ops::Range;

use mental_model::{Code, Diagnostic, Slug, slugify};
use serde::Deserialize;
use toml::Spanned;

pub(super) const PATH: &str = "pyproject.toml";

pub(super) struct Project {
    pub(super) slug: Slug,
    /// Where the name stands, for the build unit's anchor.
    pub(super) name: Range<usize>,
    pub(super) requires_python: Option<String>,
}

#[derive(Deserialize)]
struct File {
    project: Table,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Table {
    name: Spanned<String>,
    requires_python: Option<String>,
}

pub(super) fn read(text: &str) -> Result<Project, Diagnostic> {
    let file = match toml::from_str::<File>(text) {
        Ok(file) => file,
        Err(error) => {
            return Err(super::unreadable(
                Code::ProjectInvalid,
                PATH,
                text,
                &error,
                "a project's `pyproject.toml` has a `[project]` table with a `name`, as `uv init` \
                 writes it",
            ));
        }
    };
    let name = file.project.name;
    let Some(slug) = slugify(name.get_ref()) else {
        return Err(Diagnostic {
            id: None,
            code: Code::ProjectInvalid,
            at: Some(super::location(PATH, text, name.span())),
            message: format!(
                "the project name `{}` has no ASCII letter, and the system is named after the \
                 project; give the project a name with one",
                name.get_ref()
            ),
        });
    };
    Ok(Project {
        slug,
        name: name.span(),
        requires_python: file.project.requires_python,
    })
}

#[cfg(test)]
mod tests;
