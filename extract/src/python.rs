//! Stage 2 for Python: parse each file, list its reference sites, and give
//! each site its ID within the build unit.

mod parse;
mod sites;

use std::collections::{BTreeMap, BTreeSet};

use mental_model::{Anchor, BuildUnit, Diagnostic, Kind, Site, SiteKind, Slug, Source, slugify};
use ruff_text_size::{TextRange, TextSize};

/// One Python file of a build unit, as read from the project directory.
#[derive(Clone, Copy, Debug)]
pub struct PythonFile<'a> {
    /// Relative to the project directory, with `/` between folders, under
    /// one of the unit's roots, ending in `.py`.
    pub path: &'a str,
    pub text: &'a str,
}

/// The reference sites of a build unit, with the anchor of each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sites {
    pub sites: Vec<Site>,
    pub anchors: Vec<Anchor>,
}

/// Lists every reference site of the unit's files. A file that does not
/// parse gives a `syntax-error` per error, and the sites of the tree the
/// parser recovered.
///
/// # Panics
///
/// When a file is not a `.py` file under one of the unit's roots: `mentalc`
/// passes only those.
#[must_use]
pub fn list_sites(unit: &BuildUnit, files: &[PythonFile]) -> (Sites, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut found = Vec::new();
    for file in files {
        let (module, problems) = parse::parse(file.path, file.text);
        diagnostics.extend(problems);
        let name = module_name(file.path, &unit.roots);
        found.extend(sites::sites(&name, &module.body).into_iter().map(|site| {
            let words = format!("{} {} {}", site.scope, site.kind.as_str(), site.name);
            Found {
                file,
                range: site.range,
                kind: site.kind,
                slug: slugify(&words).expect("a site's words hold its kind's name"),
            }
        }));
    }
    found.sort_by(|a, b| a.position().cmp(&b.position()));
    let mut group_sizes: BTreeMap<String, usize> = BTreeMap::new();
    for site in &found {
        *group_sizes.entry(group(site.slug.as_str())).or_default() += 1;
    }
    let mut next_in_group: BTreeMap<String, usize> = BTreeMap::new();
    let mut sites = Sites {
        sites: Vec::new(),
        anchors: Vec::new(),
    };
    for site in found {
        let key = group(site.slug.as_str());
        let slug = if group_sizes[&key] > 1 {
            let next = next_in_group.entry(key).or_default();
            let order = *next;
            *next += 1;
            slugify(&format!("{}-{order}", site.slug.as_str()))
                .expect("a slug with a counter appended is a slug")
        } else {
            site.slug
        };
        let id = unit.id.child(Kind::Site, &slug);
        let (path, text) = (site.file.path, site.file.text);
        sites.anchors.push(crate::anchor::anchor(
            id.clone(),
            path,
            text,
            site.range.into(),
        ));
        sites.sites.push(Site {
            id,
            kind: site.kind,
            source: Source::Extractor,
        });
    }
    (sites, diagnostics)
}

/// A site with the slug of its words, before its group gives it a counter.
struct Found<'a> {
    file: &'a PythonFile<'a>,
    range: TextRange,
    kind: SiteKind,
    slug: Slug,
}

impl Found<'_> {
    /// The order of sites: by file, then where each starts and ends.
    fn position(&self) -> (&str, TextSize, TextSize) {
        (self.file.path, self.range.start(), self.range.end())
    }
}

/// The group a site's slug counts in: the slug without its trailing
/// segments made only of digits, so `s.commit()` and `s.commit_0()` share
/// one and a counter never repeats a name.
fn group(slug: &str) -> String {
    let mut segments: Vec<&str> = slug.split('-').collect();
    while segments
        .last()
        .is_some_and(|segment| segment.bytes().all(|byte| byte.is_ascii_digit()))
    {
        segments.pop();
    }
    segments.join("-")
}

/// The module a file is: its path under the deepest root that holds it,
/// without `.py` or a final `__init__`, with `.` between folders.
fn module_name(path: &str, roots: &BTreeSet<String>) -> String {
    let under_root = roots
        .iter()
        .filter_map(|root| {
            if root == "." {
                Some(path)
            } else {
                path.strip_prefix(root.as_str())?.strip_prefix('/')
            }
        })
        .min_by_key(|relative| relative.len())
        .expect("mentalc passes only files under one of the unit's roots");
    let stem = under_root
        .strip_suffix(".py")
        .expect("mentalc passes only `.py` files");
    let package = match stem.strip_suffix("__init__") {
        Some(package) if package.is_empty() || package.ends_with('/') => {
            package.trim_end_matches('/')
        }
        Some(_) | None => stem,
    };
    package.replace('/', ".")
}

#[cfg(test)]
mod tests;
