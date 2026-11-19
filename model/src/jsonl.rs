//! Canonical JSONL: one record per line, lines in the records' order, each
//! exactly as [`write`] makes it, and a final newline. Every record type
//! declares its `id` first, so its derived order sorts by ID.

use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// The file of `records`, sorted.
pub fn write<T: Serialize + Ord>(records: &[T]) -> String {
    let mut sorted: Vec<&T> = records.iter().collect();
    sorted.sort();
    let mut text = String::new();
    for record in sorted {
        text.push_str(&line(record));
        text.push('\n');
    }
    text
}

/// The records of a file [`write`] made. Anything else is rejected, so an
/// accepted file is written back to the same bytes.
///
/// # Errors
///
/// At the first line that is not a record, not canonical or out of order,
/// or when the file does not end with a newline.
pub fn read<T: DeserializeOwned + Serialize + Ord>(text: &str) -> Result<Vec<T>, ReadError> {
    let mut records: Vec<T> = Vec::new();
    if text.is_empty() {
        return Ok(records);
    }
    let Some(body) = text.strip_suffix('\n') else {
        let line = text.split('\n').count();
        return Err(ReadError {
            line,
            reason: ReadReason::NoFinalNewline,
        });
    };
    for (index, text_line) in body.split('\n').enumerate() {
        let line_number = index + 1;
        let fail = |reason| ReadError {
            line: line_number,
            reason,
        };
        let record: T = serde_json::from_str(text_line).map_err(|error| fail(invalid(&error)))?;
        if line(&record) != text_line {
            return Err(fail(ReadReason::NotCanonical));
        }
        if records.last().is_some_and(|previous| *previous > record) {
            return Err(fail(ReadReason::Unsorted));
        }
        records.push(record);
    }
    Ok(records)
}

fn line<T: Serialize>(record: &T) -> String {
    serde_json::to_string(record)
        .expect("a record serializes: its fields are strings, numbers, IDs and records")
}

fn invalid(error: &serde_json::Error) -> ReadReason {
    let message = error.to_string();
    let position = format!(" at line {} column {}", error.line(), error.column());
    ReadReason::Invalid {
        column: error.column(),
        message: message
            .strip_suffix(&position)
            .unwrap_or(&message)
            .to_owned(),
    }
}

/// Why a file is not canonical JSONL, at a line counted from 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadError {
    pub line: usize,
    pub reason: ReadReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadReason {
    NoFinalNewline,
    /// Not a record: bad JSON, a missing or unknown field, a bad value.
    Invalid {
        column: usize,
        message: String,
    },
    NotCanonical,
    Unsorted,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = self.line;
        match &self.reason {
            ReadReason::NoFinalNewline => {
                write!(f, "line {line}: the file does not end with a newline")
            }
            ReadReason::Invalid { column, message } => {
                write!(f, "line {line}, column {column}: {message}")
            }
            ReadReason::NotCanonical => write!(
                f,
                "line {line}: not in canonical form: fields out of order, extra spaces, or an empty field written out"
            ),
            ReadReason::Unsorted => {
                write!(f, "line {line}: the record sorts before the one above it")
            }
        }
    }
}

impl std::error::Error for ReadError {}

#[cfg(test)]
mod tests;
