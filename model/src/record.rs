//! The records of the model, one struct per kind, each kind in its own JSONL
//! file. Every record declares its `id` first, so its derived order sorts
//! by ID, as [`crate::jsonl`] requires.

mod build;
mod site;

pub use build::{BuildUnit, Lock, Origin, Pin, Python, PythonFrom};
pub use site::{Checker, Outcome, Package, Reason, Site, SiteKind};

use serde::{Deserialize, Serialize};

/// What made a record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// The extractor, from the project's files.
    Extractor,
}
