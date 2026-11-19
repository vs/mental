//! Semantic IDs: `<system>/<kind>:<slug>/<kind>:<slug>…`, built only by
//! [`Id::parse`] or from slugs, so every `Id` has its one canonical text form.

use std::fmt;
use std::ops::Range;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The kind of a record, named by the prefix of the last segment of its ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A build unit: `reports/unit:reports`.
    Unit,
    /// A package pinned by a build unit's lockfile: `reports/unit:reports/pin:sqlalchemy`.
    Pin,
    /// A reference site: `reports/unit:reports/site:app-tasks-generate-report-callee-s-commit-0`.
    Site,
}

impl Kind {
    const ALL: [Kind; 3] = [Kind::Unit, Kind::Pin, Kind::Site];

    fn prefix(self) -> &'static str {
        match self {
            Kind::Unit => "unit",
            Kind::Pin => "pin",
            Kind::Site => "site",
        }
    }

    fn from_prefix(prefix: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.prefix() == prefix)
    }
}

/// A name in the slug alphabet: `[a-z0-9]+(-[a-z0-9]+)*`, with at least one
/// letter. Made from a code name by [`slugify`], and from nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slug(String);

impl Slug {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The slug of a code name: ASCII letters lowercased, digits kept, and each
/// run of other characters made one hyphen, with none at either end.
/// `None` when the name has no ASCII letter.
#[must_use]
pub fn slugify(code_name: &str) -> Option<Slug> {
    let mut slug = String::with_capacity(code_name.len());
    for c in code_name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    parse_slug(&slug, 0).is_ok().then_some(Slug(slug))
}

/// A semantic ID. Its order is the byte order of its text.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(String);

impl Id {
    /// Accepts only the canonical form; nothing is normalized.
    ///
    /// # Errors
    ///
    /// When `text` is not an ID: the error names the reason and its place.
    pub fn parse(text: &str) -> Result<Id, IdError> {
        let mut at = 0;
        for (index, segment) in text.split('/').enumerate() {
            parse_segment(segment, at, index == 0).map_err(|(span, reason)| IdError {
                input: text.to_owned(),
                span,
                reason,
            })?;
            at += segment.len() + 1;
        }
        Ok(Id(text.to_owned()))
    }

    /// The ID of a system, named after its project.
    #[must_use]
    pub fn root(name: &Slug) -> Id {
        Id(name.0.clone())
    }

    #[must_use]
    pub fn child(&self, kind: Kind, slug: &Slug) -> Id {
        Id(format!("{}/{}:{}", self.0, kind.prefix(), slug.0))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Id, D::Error> {
        let text = String::deserialize(deserializer)?;
        Id::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// Why a text is not an ID, at the byte span of `input` that shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdError {
    pub input: String,
    pub span: Range<usize>,
    pub reason: IdReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdReason {
    EmptySegment,
    KindOnRoot,
    MissingKind,
    UnknownKind,
    EmptySlug,
    BadCharacter,
    BadHyphen,
    NoLetter,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = &self.input[self.span.clone()];
        let explanation = match self.reason {
            IdReason::EmptySegment => {
                "a segment is empty; look for a doubled or trailing `/`".to_owned()
            }
            IdReason::KindOnRoot => {
                "the first segment is the system's name and takes no kind".to_owned()
            }
            IdReason::MissingKind => format!("`{shown}` has no kind; write it as `<kind>:<slug>`"),
            IdReason::UnknownKind => {
                let kinds: Vec<&str> = Kind::ALL.into_iter().map(Kind::prefix).collect();
                format!(
                    "`{shown}` is not a kind; the kinds are {}",
                    kinds.join(", ")
                )
            }
            IdReason::EmptySlug => "the slug after the kind is empty".to_owned(),
            IdReason::BadCharacter => format!(
                "`{shown}` is not allowed in a slug, which has only a-z, 0-9 and single hyphens"
            ),
            IdReason::BadHyphen => {
                "a hyphen in a slug must stand between two letters or digits".to_owned()
            }
            IdReason::NoLetter => format!("the slug `{shown}` has no letter"),
        };
        let (input, at) = (&self.input, self.span.start);
        write!(f, "invalid ID `{input}` at byte {at}: {explanation}")
    }
}

impl std::error::Error for IdError {}

type Fault = (Range<usize>, IdReason);

fn parse_segment(segment: &str, at: usize, is_root: bool) -> Result<(), Fault> {
    if segment.is_empty() {
        return Err((at..at, IdReason::EmptySegment));
    }
    if is_root {
        if let Some(colon) = segment.find(':') {
            return Err((at + colon..at + colon + 1, IdReason::KindOnRoot));
        }
        return parse_slug(segment, at);
    }
    let Some((prefix, slug)) = segment.split_once(':') else {
        return Err((at..at + segment.len(), IdReason::MissingKind));
    };
    if Kind::from_prefix(prefix).is_none() {
        return Err((at..at + prefix.len(), IdReason::UnknownKind));
    }
    let slug_at = at + prefix.len() + 1;
    if slug.is_empty() {
        return Err((slug_at..slug_at, IdReason::EmptySlug));
    }
    parse_slug(slug, slug_at)
}

/// The one definition of the slug alphabet; `slug` starts at byte `at` of
/// the text being parsed.
fn parse_slug(slug: &str, at: usize) -> Result<(), Fault> {
    for (i, c) in slug.char_indices() {
        let span = at + i..at + i + c.len_utf8();
        match c {
            'a'..='z' | '0'..='9' => {}
            '-' => {
                if i == 0 || i + 1 == slug.len() || slug[..i].ends_with('-') {
                    return Err((span, IdReason::BadHyphen));
                }
            }
            _ => return Err((span, IdReason::BadCharacter)),
        }
    }
    if !slug.bytes().any(|b| b.is_ascii_lowercase()) {
        return Err((at..at + slug.len(), IdReason::NoLetter));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
