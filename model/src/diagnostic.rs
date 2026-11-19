//! The one record every tool reports a problem with.

use serde::{Deserialize, Serialize};

use crate::{Id, Lines};

/// What a diagnostic reports. Written as its kebab-case name, which never
/// changes: agents, tests and CI filter on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Code {
    /// The directory has no `pyproject.toml`.
    ProjectMissing,
    /// The project has no `uv.lock`.
    LockMissing,
    /// The `uv.lock` has a schema version this build does not read.
    LockVersionUnknown,
    /// Neither `.python-version` nor the environment states a Python version.
    PythonVersionUnknown,
    /// The project has no `.venv`.
    EnvironmentMissing,
    /// An import names a module found neither under a root nor in the environment.
    ModuleMissing,
    /// A file does not parse; the recovered tree is used.
    SyntaxError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Fails the command: it exits 1.
    Error,
    Warning,
}

impl Code {
    pub const ALL: [Code; 7] = [
        Code::ProjectMissing,
        Code::LockMissing,
        Code::LockVersionUnknown,
        Code::PythonVersionUnknown,
        Code::EnvironmentMissing,
        Code::ModuleMissing,
        Code::SyntaxError,
    ];

    #[must_use]
    pub fn severity(self) -> Severity {
        match self {
            Code::ProjectMissing
            | Code::LockMissing
            | Code::LockVersionUnknown
            | Code::PythonVersionUnknown
            | Code::EnvironmentMissing
            | Code::ModuleMissing => Severity::Error,
            Code::SyntaxError => Severity::Warning,
        }
    }
}

/// A place in a file that no record stands for, such as a syntax error.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    /// Relative to the project directory, with `/` between folders.
    pub path: String,
    pub lines: Lines,
}

/// One line of `diagnostics.jsonl`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    /// The record it is about, when there is one; its anchor is in `anchors.jsonl`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    pub code: Code,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<Location>,
    /// What is wrong and how to fix it, starting lowercase, with no final period.
    pub message: String,
}

#[cfg(test)]
mod tests;
