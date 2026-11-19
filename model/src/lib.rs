//! The record format of Mental's model: semantic IDs, anchors, diagnostics
//! and canonical JSONL. It touches no file, clock or network: callers pass
//! strings in and get strings out.
#![deny(clippy::wildcard_enum_match_arm)]

mod anchor;
mod diagnostic;
mod id;
pub mod jsonl;

pub use anchor::{Anchor, Lines};
pub use diagnostic::{Code, Diagnostic, Location, Severity};
pub use id::{Id, IdError, IdReason, Kind, Slug, slugify};
