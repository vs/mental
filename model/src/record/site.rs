//! Stage 2: the places where the code names something, and what each names.

use serde::{Deserialize, Serialize};

use crate::Id;
use crate::record::Source;

/// A place where the code names something: one line of `site.jsonl`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub id: Id,
    pub kind: SiteKind,
    pub outcome: Outcome,
    pub checker: Checker,
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
    /// A repository function or class, or a name nothing defines, handed to
    /// a call whose callee is not in the repository: `s.query(Report)`.
    Escape,
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
            SiteKind::Escape => "escape",
            SiteKind::HandlerType => "handler-type",
        }
    }
}

/// What a site points to. A write receiver points to the class of the
/// object written to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Outcome {
    /// A definition in the repository.
    Resolved {
        module: String,
        /// The qualified name in the module; empty for the module itself.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
    },
    /// A definition outside the repository, where the checker found it.
    External {
        package: Package,
        module: String,
        /// The qualified name in the module; empty for the module itself.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        name: String,
    },
    /// The checker cannot say.
    Unresolved { reason: Reason },
}

/// The package an external definition belongs to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Package {
    /// A pinned package of the build unit, whose installed files hold the definition.
    Pin(Id),
    /// The standard library, as the checker's own stubs describe it.
    Stdlib,
}

/// Why a site is unresolved: a closed list, written in `snake_case`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// An import names a module neither under a root nor in the environment.
    MissingModule,
    /// The receiver has no type: an unannotated parameter, an untyped container element.
    NoType,
    /// Nothing the checker knows defines the name.
    NotFound,
    /// Several definitions can be meant.
    Ambiguous,
    /// The name is computed at run time: a star import, a callee with no name.
    Dynamic,
    /// The checker typed the receiver `Never`: the code cannot run there.
    Unreachable,
}

/// The tool that answered, with its version.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checker {
    pub name: String,
    pub version: String,
}

#[cfg(test)]
mod tests;
