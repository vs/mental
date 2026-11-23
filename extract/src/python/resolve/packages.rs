//! Which pin an installed file belongs to, from the `RECORD` file that each
//! installed distribution's `.dist-info` folder holds.

use std::collections::{BTreeMap, BTreeSet};

use mental_model::{Id, Kind, Pin, slugify};

use crate::python::DistRecord;

/// The pin of each installed file, by its path in `site-packages`.
pub(super) struct Packages {
    files: BTreeMap<String, Id>,
    /// The pins that install stubs and no module, as `types-redis` and
    /// `redis-stubs` do: they describe another package.
    stubs: BTreeSet<Id>,
}

impl Packages {
    /// Only distributions the build unit pins count: a file of anything
    /// else installed belongs to no package of the unit.
    pub(super) fn new(unit: &Id, pins: &[Pin], records: &[DistRecord]) -> Packages {
        let mut packages = Packages {
            files: BTreeMap::new(),
            stubs: BTreeSet::new(),
        };
        for record in records {
            let Some(pin) = pin_of(unit, pins, record.dist_info) else {
                continue;
            };
            let paths: Vec<String> = record
                .text
                .lines()
                .filter_map(first_field)
                .filter(|path| !path.starts_with("../"))
                .collect();
            let has = |suffix: &str| paths.iter().any(|path| path.ends_with(suffix));
            if has(".pyi") && !has(".py") {
                packages.stubs.insert(pin.clone());
            }
            for path in paths {
                packages.files.insert(path, pin.clone());
            }
        }
        packages
    }

    /// The pin of the installed file at `path`; for a file of a stub-only
    /// pin, the pin that installs the module it describes, when one does.
    pub(super) fn pin(&self, path: &str) -> Option<&Id> {
        let pin = self.files.get(path)?;
        if !self.stubs.contains(pin) {
            return Some(pin);
        }
        let runtime = runtime_files(path)
            .iter()
            .find_map(|candidate| self.files.get(candidate));
        Some(runtime.unwrap_or(pin))
    }
}

/// The files that may install the module a stub file describes, nearest
/// first: for `redis-stubs/commands/core.pyi`, `redis/commands/core.py`,
/// then the `__init__.py` of `redis/commands/core`, `redis/commands` and
/// `redis`, since a compiled module has no `.py` of its own. None of them
/// belongs to a stub-only pin, which installs no `.py`.
fn runtime_files(stub: &str) -> Vec<String> {
    let module = stub
        .strip_suffix(".pyi")
        .expect("ty finds only `.py` and `.pyi` files, and a stub-only pin installs no `.py`");
    let module = module.strip_suffix("/__init__").unwrap_or(module);
    let mut parts: Vec<&str> = module.split('/').collect();
    parts[0] = parts[0].strip_suffix("-stubs").unwrap_or(parts[0]);
    let mut files = vec![format!("{}.py", parts.join("/"))];
    while !parts.is_empty() {
        files.push(format!("{}/__init__.py", parts.join("/")));
        parts.pop();
    }
    files
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
