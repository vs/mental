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

    /// The pin of the installed file at `path`. A stub describes a module,
    /// and `types-redis` or `redis-stubs` installs stubs of another
    /// package's, so a `.pyi` file belongs to the pin that installs the
    /// module it describes, when one does.
    pub(super) fn pin(&self, path: &str) -> Option<&Id> {
        let pin = self.0.get(path)?;
        let Some(stub) = path.strip_suffix(".pyi") else {
            return Some(pin);
        };
        Some(self.module_pin(stub).unwrap_or(pin))
    }

    /// The pin that installs the module of the stub `stub`, written without
    /// `.pyi`, nearest file first: for `redis-stubs/commands/core`, the
    /// module `redis/commands/core.py`, or one compiled as
    /// `redis/commands/core.<tag>.so` or `.pyd`, then the `__init__.py` of
    /// `redis/commands/core`, `redis/commands` and `redis`.
    fn module_pin(&self, stub: &str) -> Option<&Id> {
        let module = stub.strip_suffix("/__init__").unwrap_or(stub);
        let mut parts: Vec<&str> = module.split('/').collect();
        parts[0] = parts[0].strip_suffix("-stubs").unwrap_or(parts[0]);
        let module = parts.join("/");
        let compiled = format!("{module}.");
        let own = self.0.get(&format!("{module}.py")).or_else(|| {
            self.0
                .range(compiled.clone()..)
                .take_while(|(path, _)| path.starts_with(&compiled))
                .find(|(path, _)| {
                    std::path::Path::new(path.as_str())
                        .extension()
                        .is_some_and(|tag| {
                            tag.eq_ignore_ascii_case("so") || tag.eq_ignore_ascii_case("pyd")
                        })
                })
                .map(|(_, pin)| pin)
        });
        own.or_else(|| {
            (1..=parts.len()).rev().find_map(|depth| {
                self.0
                    .get(&format!("{}/__init__.py", parts[..depth].join("/")))
            })
        })
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
