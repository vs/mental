//! Stage 1: how a project is built.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::Id;
use crate::record::Source;

/// One project: one line of `build_unit.jsonl`. Paths are relative to the
/// project directory, with `/` between folders.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildUnit {
    pub id: Id,
    /// The project directory, relative to where `mentalc` ran.
    pub dir: String,
    /// `src` when the project has one, else `.`.
    pub roots: BTreeSet<String>,
    /// Absent when no file states a version (`python-version-unknown`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<Python>,
    /// `requires-python` from `pyproject.toml`, recorded and never used as the version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_python: Option<String>,
    /// Absent when the project has no `uv.lock` (`lock-missing`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock: Option<Lock>,
    /// The environment's directory; absent when there is none (`environment-missing`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    pub source: Source,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Python {
    /// As the file states it, such as `3.12`.
    pub version: String,
    pub from: PythonFrom,
}

/// The file the Python version was taken from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PythonFrom {
    #[serde(rename = ".python-version")]
    PythonVersionFile,
    /// The environment's `pyvenv.cfg`, read when there is no `.python-version`.
    #[serde(rename = "pyvenv.cfg")]
    Pyvenv,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    pub path: String,
    /// The lockfile's schema version: `version` in `uv.lock`.
    pub version: u32,
}

/// One package of a build unit at an exact version: one line of `pin.jsonl`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub id: Id,
    /// The normalized name: lowercase, with `-` for each run of `-`, `_` and `.`.
    pub package: String,
    pub version: String,
    pub origin: Origin,
    pub source: Source,
}

/// Where a pinned package comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Origin {
    /// A package index, by its URL.
    Registry(String),
    /// A commit of a git repository.
    Git { url: String, commit: String },
    /// An archive at a URL, such as a wheel.
    Url(String),
    /// A directory or archive, relative to the project directory.
    Path(String),
}
