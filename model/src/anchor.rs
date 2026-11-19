//! Anchors: the file, lines and text a record comes from. They live apart
//! from the records, in `anchors.jsonl`, so moving code touches only them.

use serde::{Deserialize, Serialize};

use crate::Id;

/// A range of lines, counted from 1, both ends included; `[first, last]` in JSON.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "[u32; 2]", into = "[u32; 2]")]
pub struct Lines {
    first: u32,
    last: u32,
}

impl Lines {
    /// `None` unless `1 <= first <= last`.
    #[must_use]
    pub fn new(first: u32, last: u32) -> Option<Lines> {
        (1 <= first && first <= last).then_some(Lines { first, last })
    }
}

impl TryFrom<[u32; 2]> for Lines {
    type Error = &'static str;

    fn try_from([first, last]: [u32; 2]) -> Result<Lines, Self::Error> {
        Lines::new(first, last)
            .ok_or("lines are `[first, last]`, counted from 1, with first <= last")
    }
}

impl From<Lines> for [u32; 2] {
    fn from(lines: Lines) -> [u32; 2] {
        [lines.first, lines.last]
    }
}

/// The anchor of one record: one line of `anchors.jsonl`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub id: Id,
    /// Relative to the project directory, with `/` between folders.
    pub path: String,
    pub lines: Lines,
    /// The text the record rests on, as it stands in those lines.
    pub quote: String,
}

#[cfg(test)]
mod tests;
