//! Which pin an installed file belongs to, from the `RECORD` file that each
//! installed distribution's `.dist-info` folder holds.

use std::collections::BTreeMap;

use mental_model::{Id, Kind, Pin, slugify};

use crate::python::DistRecord;

/// The pin of each installed file, by its path in `site-packages`.
pub(super) struct Packages(BTreeMap<String, Id>);

impl Packages {
    /// Only distributions the build unit pins count: a file of anything
    /// else installed belongs to no package of the unit.
    pub(super) fn new(unit: &Id, pins: &[Pin], records: &[DistRecord]) -> Packages {
        let mut files = BTreeMap::new();
        for record in records {
            let Some(pin) = pin_of(unit, pins, record.dist_info) else {
                continue;
            };
            for path in record.text.lines().filter_map(first_field) {
                if !path.starts_with("../") {
                    files.insert(path, pin.clone());
                }
            }
        }
        Packages(files)
    }

    pub(super) fn pin(&self, path: &str) -> Option<&Id> {
        self.0.get(path)
    }
}

/// The pin of `<name>-<version>.dist-info`: the one whose ID the slug of
/// `<name>` makes, as `uv.lock`'s reader makes it.
fn pin_of<'a>(unit: &Id, pins: &'a [Pin], dist_info: &str) -> Option<&'a Id> {
    let (name, _version) = dist_info.strip_suffix(".dist-info")?.rsplit_once('-')?;
    let id = unit.child(Kind::Pin, &slugify(name)?);
    pins.iter().map(|pin| &pin.id).find(|pin| **pin == id)
}

/// The first field of a line of `RECORD`, a CSV file whose fields may be
/// quoted, with `""` for a quote inside a quoted field.
fn first_field(line: &str) -> Option<String> {
    let Some(quoted) = line.strip_prefix('"') else {
        let field = line.split_once(',').map_or(line, |(field, _)| field);
        return (!field.is_empty()).then(|| field.to_owned());
    };
    let mut field = String::new();
    let mut chars = quoted.chars();
    while let Some(c) = chars.next() {
        if c != '"' {
            field.push(c);
        } else if chars.as_str().starts_with('"') {
            field.push('"');
            chars.next();
        } else {
            return Some(field);
        }
    }
    None
}

#[cfg(test)]
mod tests;
