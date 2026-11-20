//! The record format of Mental's model: semantic IDs, records, anchors,
//! diagnostics and canonical JSONL. It touches no file, clock or network:
//! callers pass strings in and get strings out.
#![deny(clippy::wildcard_enum_match_arm)]

mod anchor;
mod diagnostic;
mod id;
pub mod jsonl;
mod record;

pub use anchor::{Anchor, Lines};
pub use diagnostic::{Code, Diagnostic, Location, Severity};
pub use id::{Id, IdError, IdReason, Kind, Slug, slugify};
pub use record::{
    BuildUnit, Checker, Lock, Origin, Outcome, Package, Pin, Python, PythonFrom, Reason, Site,
    SiteKind, Source,
};
