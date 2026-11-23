//! Stage 2 for Python: parse each file, list its reference sites, resolve
//! each with the type checker, and give each its ID within the build unit.

mod parse;
mod resolve;
mod sites;

use std::collections::{BTreeMap, BTreeSet};

use mental_model::{
    Anchor, BuildUnit, Checker, Code, Diagnostic, Kind, Outcome, Pin, Reason, Site, SiteKind, Slug,
    Source, slugify,
};
use ruff_text_size::{TextRange, TextSize};

pub use resolve::ResolveError;

/// One Python file of a build unit, as read from the project directory.
#[derive(Clone, Copy, Debug)]
pub struct PythonFile<'a> {
    /// Relative to the project directory, with `/` between folders, under
    /// one of the unit's roots, ending in `.py`.
    pub path: &'a str,
    pub text: &'a str,
}

/// What the type checker needs beside the build unit: where the project is
/// and which files each installed distribution holds. The environment's
/// directory and the Python version come from the build unit.
#[derive(Clone, Copy, Debug)]
pub struct Environment<'a> {
    /// The project directory: absolute, with no `..` segment, which ty would
    /// not match against the roots.
    pub project_dir: &'a str,
    /// The `RECORD` of each installed distribution.
    pub records: &'a [DistRecord<'a>],
}

/// The `RECORD` file of one installed distribution: the files it installed.
#[derive(Clone, Copy, Debug)]
pub struct DistRecord<'a> {
    /// The name of its folder in `site-packages`: `<name>-<version>.dist-info`.
    pub dist_info: &'a str,
    pub text: &'a str,
}

/// The reference sites of a build unit, with the anchor of each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sites {
    pub sites: Vec<Site>,
    pub anchors: Vec<Anchor>,
}

/// Lists every reference site of the unit's files and resolves it with ty.
/// A file that does not parse gives a `syntax-error` per error, and the
/// sites of the tree the parser recovered; an import of a module that is
/// nowhere gives a `module-missing`, or a `module-optional` when a handler
/// of a failed import guards it.
///
/// # Errors
///
/// When the project directory is relative or has a `..` segment, the build
/// unit records no environment or Python version, or ty cannot check that
/// version, open the project, or find one of `files`.
///
/// # Panics
///
/// When a file is not a `.py` file under one of the unit's roots: `mentalc`
/// passes only those.
pub fn extract_sites(
    unit: &BuildUnit,
    pins: &[Pin],
    environment: &Environment,
    files: &[PythonFile],
) -> Result<(Sites, Vec<Diagnostic>), ResolveError> {
    let resolver = resolve::Resolver::new(unit, pins, environment, files)?;
    let mut diagnostics = Vec::new();
    let mut found = Vec::new();
    for file in files {
        let (module, problems) = parse::parse(file.path, file.text);
        diagnostics.extend(problems);
        let listed = sites::sites(&module_name(file.path, &unit.roots), &module.body);
        let checked = resolver.check(file, &listed)?;
        for (site, check) in listed.iter().zip(&checked) {
            if let Some(call) = site.call
                && !escapes(check, &checked[call])
            {
                continue;
            }
            let words = format!("{} {} {}", site.scope, site.kind.as_str(), site.name);
            found.push(Found {
                file,
                range: site.range,
                kind: site.kind,
                written: site.name.clone(),
                guarded: site.guarded,
                slug: slugify(&words).expect("a site's words hold its kind's name"),
                outcome: check.outcome.clone(),
            });
        }
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
        if site.outcome
            == (Outcome::Unresolved {
                reason: Reason::MissingModule,
            })
        {
            let (code, fix) = if site.guarded {
                (
                    Code::ModuleOptional,
                    "the handler that guards it runs instead; add the package with `uv add` if the code needs it",
                )
            } else {
                (
                    Code::ModuleMissing,
                    "run `uv sync`, or add the package that provides it with `uv add`",
                )
            };
            diagnostics.push(Diagnostic {
                id: Some(id.clone()),
                code,
                at: None,
                message: format!(
                    "the import `{}` names a module that is neither under a root nor in the \
                     environment; {fix}",
                    site.written
                ),
            });
        }
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
            outcome: site.outcome,
            checker: Checker {
                name: "ty".to_owned(),
                version: resolve::TY_VERSION.to_owned(),
            },
            source: Source::Extractor,
        });
    }
    Ok((sites, diagnostics))
}

/// Whether a name handed to a call escapes: it is a repository `def` or
/// `class`, or the checker cannot say what it is, and the callee is not
/// in the repository, so outside code may call it.
fn escapes(argument: &resolve::Checked, call: &resolve::Checked) -> bool {
    let outside = !matches!(call.outcome, Outcome::Resolved { .. });
    let callable = match argument.outcome {
        Outcome::Resolved { .. } => argument.definition,
        Outcome::External { .. } => false,
        Outcome::Unresolved { .. } => true,
    };
    outside && callable
}

/// A site with the slug of its words, before its group gives it a counter.
struct Found<'a> {
    file: &'a PythonFile<'a>,
    range: TextRange,
    kind: SiteKind,
    written: String,
    guarded: bool,
    slug: Slug,
    outcome: Outcome,
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
