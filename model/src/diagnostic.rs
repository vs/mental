//! The one record every tool reports a problem with.

use serde::{Deserialize, Serialize};

use crate::{Id, Lines};

/// What a diagnostic reports. Written as its kebab-case name, which never
/// changes: agents, tests and CI filter on it. Declared in the byte order of
/// those names, so diagnostics sort by code name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Code {
    /// The project has no `.venv`.
    EnvironmentMissing,
    /// The `uv.lock` locks one package more than once, such as at one
    /// version per platform; a pin has one version.
    LockAmbiguous,
    /// The `uv.lock` does not read as a lockfile, does not lock the project,
    /// or holds a package with no fixed version or with a source this build
    /// does not read.
    LockInvalid,
    /// The project has no `uv.lock`.
    LockMissing,
    /// The `uv.lock` has a schema version this build does not read.
    LockVersionUnknown,
    /// An import names a module found neither under a root nor in the environment.
    ModuleMissing,
    /// An import that a handler of `ImportError` or `ModuleNotFoundError`
    /// guards names a module found neither under a root nor in the
    /// environment, so the handler runs instead.
    ModuleOptional,
    /// The `pyproject.toml` does not read, has no `[project]` name, or its
    /// name has no ASCII letter to name the system with.
    ProjectInvalid,
    /// The directory has no `pyproject.toml`.
    ProjectMissing,
    /// Neither `.python-version` nor the environment states a Python version.
    PythonVersionUnknown,
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
    pub const ALL: [Code; 11] = [
        Code::EnvironmentMissing,
        Code::LockAmbiguous,
        Code::LockInvalid,
        Code::LockMissing,
        Code::LockVersionUnknown,
        Code::ModuleMissing,
        Code::ModuleOptional,
        Code::ProjectInvalid,
        Code::ProjectMissing,
        Code::PythonVersionUnknown,
        Code::SyntaxError,
    ];

    #[must_use]
    pub fn severity(self) -> Severity {
        match self {
            Code::EnvironmentMissing
            | Code::LockAmbiguous
            | Code::LockInvalid
            | Code::LockMissing
            | Code::LockVersionUnknown
            | Code::ModuleMissing
            | Code::ProjectInvalid
            | Code::ProjectMissing
            | Code::PythonVersionUnknown => Severity::Error,
            Code::ModuleOptional | Code::SyntaxError => Severity::Warning,
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
