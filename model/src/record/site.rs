//! Stage 2: the places where the code names something.

use serde::{Deserialize, Serialize};

use crate::Id;
use crate::record::Source;

/// A place where the code names something: one line of `site.jsonl`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub id: Id,
    pub kind: SiteKind,
    pub source: Source,
}

/// What a site names. Written as [`SiteKind::as_str`], which is also the
/// kind's word in a site's ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SiteKind {
    /// A name an import binds: `from app.db import Session`.
    Import,
    /// `@app.post("/reports")`; a decorator that is a call is this site, not a callee.
    Decorator,
    /// Every other call: `s.commit()`.
    Callee,
    /// A positional base of a class: `class Report(Base)`.
    BaseClass,
    /// An attribute that is assigned to: `report.status = …`.
    WriteReceiver,
    /// A type an `except` clause names: `except ClientError:`.
    HandlerType,
}

impl SiteKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            SiteKind::Import => "import",
            SiteKind::Decorator => "decorator",
            SiteKind::Callee => "callee",
            SiteKind::BaseClass => "base-class",
            SiteKind::WriteReceiver => "write-receiver",
            SiteKind::HandlerType => "handler-type",
        }
    }
}

#[cfg(test)]
mod tests;
