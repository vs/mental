//! `uv.lock`: a pin for each package a project's dependencies reach.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use mental_model::{Anchor, Code, Diagnostic, Id, Kind, Lock, Origin, Pin, Slug, Source, slugify};
use serde::Deserialize;
use toml::Spanned;

/// The schema version this build reads.
const VERSION: u32 = 1;
const FIX: &str = "run `uv lock` to write it again";

/// A `uv.lock` read once for every build unit it locks.
pub(super) struct Lockfile<'t> {
    text: &'t str,
    pub(super) lock: Lock,
    /// Every entry of each package, by its slug.
    packages: BTreeMap<String, Vec<Package>>,
}

/// A unit's pins, their anchors, and the packages it reaches that cannot be
/// pinned.
#[derive(Default)]
pub(super) struct Reached {
    pub(super) pins: Vec<Pin>,
    pub(super) anchors: Vec<Anchor>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

/// How the lock pins a package, for each unit that reaches it.
struct Pinned {
    version: String,
    origin: Origin,
    /// Where its name and version stand, for the pin's anchor.
    span: Range<usize>,
}

/// Read first, so that a lockfile of another schema version is named as one
/// rather than as a file that does not read.
#[derive(Deserialize)]
struct Head {
    version: Spanned<u32>,
}

#[derive(Deserialize)]
struct File {
    #[serde(default)]
    package: Vec<Package>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Package {
    name: Spanned<String>,
    /// uv leaves it out for a local package whose version is dynamic.
    version: Option<Spanned<String>>,
    source: Spanned<LockedSource>,
    #[serde(default)]
    dependencies: Vec<Dependency>,
    /// By extra: what each extra of the package adds.
    #[serde(default)]
    optional_dependencies: BTreeMap<String, Vec<Dependency>>,
    /// By group: a project's development dependencies, which `uv sync`
    /// installs.
    #[serde(default)]
    dev_dependencies: BTreeMap<String, Vec<Dependency>>,
}

#[derive(Clone, Deserialize)]
struct Dependency {
    name: String,
    /// The extras of the package depended on, as in `fastapi[standard]`.
    #[serde(default)]
    extra: Vec<String>,
}

/// One of these forms is set, as uv writes them.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LockedSource {
    registry: Option<String>,
    /// The repository URL, then `#` and the commit.
    git: Option<String>,
    url: Option<String>,
    /// Where in the `url` archive the package is.
    subdirectory: Option<String>,
    path: Option<String>,
    directory: Option<String>,
    editable: Option<String>,
    r#virtual: Option<String>,
}

/// Reads `text`, the `uv.lock` at `path`. Only a package with no name to
/// key it by is a problem here; one that cannot be pinned is a problem for
/// each unit that reaches it.
pub(super) fn read<'t>(text: &'t str, path: &str) -> (Option<Lockfile<'t>>, Vec<Diagnostic>) {
    let unreadable = |error| {
        (
            None,
            vec![super::unreadable(
                Code::LockInvalid,
                path,
                text,
                &error,
                FIX,
            )],
        )
    };
    let head = match toml::from_str::<Head>(text) {
        Ok(head) => head,
        Err(error) => return unreadable(error),
    };
    if *head.version.get_ref() != VERSION {
        let problem = Diagnostic {
            id: None,
            code: Code::LockVersionUnknown,
            at: Some(crate::anchor::location(path, text, head.version.span())),
            message: format!(
                "`{path}` has schema version {}, and this build reads only version {VERSION}; \
                 lock with a uv that writes version {VERSION}",
                head.version.get_ref()
            ),
        };
        return (None, vec![problem]);
    }
    let file = match toml::from_str::<File>(text) {
        Ok(file) => file,
        Err(error) => return unreadable(error),
    };
    let mut diagnostics = Vec::new();
    let mut lockfile = Lockfile {
        text,
        lock: Lock {
            path: path.to_owned(),
            version: VERSION,
        },
        packages: BTreeMap::new(),
    };
    for package in file.package {
        let Some(slug) = slugify(package.name.get_ref()) else {
            let why = format!(
                "package `{}` has no ASCII letter in its name, so it can have no ID; \
                 remove the dependency on it, or depend on it under a name with a letter",
                package.name.get_ref()
            );
            diagnostics.push(invalid(path, text, package.name.span(), why));
            continue;
        };
        let entries = lockfile.packages.entry(slug.as_str().to_owned());
        entries.or_default().push(package);
    }
    (Some(lockfile), diagnostics)
}

impl Lockfile<'_> {
    /// The pins of the unit `unit`, whose project is `project`, with their
    /// anchors: one for each package its dependencies reach, with every
    /// extra and development group of its own. Another project it reaches
    /// is a pin too, by its path; one that cannot be pinned is a diagnostic.
    pub(super) fn pins(&self, unit: &Id, project: &Slug) -> Reached {
        let mut pins = Reached::default();
        let Some(own) = self.packages.get(project.as_str()) else {
            pins.diagnostics.push(Diagnostic {
                id: Some(unit.clone()),
                code: Code::LockInvalid,
                at: None,
                message: format!(
                    "`{}` does not lock the project `{}`, so it says nothing of what the \
                     project depends on; run `uv lock` to lock it",
                    self.lock.path,
                    project.as_str()
                ),
            });
            return pins;
        };
        for slug in self.reach(own) {
            if slug == project.as_str() {
                continue;
            }
            let found = match pinned(&self.lock.path, self.text, &self.packages[&slug]) {
                Ok(pinned) => pinned,
                Err(problem) => {
                    pins.diagnostics.push(problem);
                    continue;
                }
            };
            let id = unit.child(
                Kind::Pin,
                &slugify(&slug).expect("a package is keyed by its slug"),
            );
            let span = found.span;
            pins.anchors.push(crate::anchor::anchor(
                id.clone(),
                &self.lock.path,
                self.text,
                span,
            ));
            pins.pins.push(Pin {
                id,
                package: slug,
                version: found.version,
                origin: found.origin,
                source: Source::Extractor,
            });
        }
        pins
    }

    /// The slugs of the packages that `own`, a project's entries, depends
    /// on directly or through others, itself included: through each one's
    /// dependencies and the extras asked of it.
    fn reach(&self, own: &[Package]) -> BTreeSet<String> {
        let mut todo: Vec<Dependency> = own
            .iter()
            .flat_map(|package| {
                let extras = package.optional_dependencies.values();
                let groups = package.dev_dependencies.values();
                package
                    .dependencies
                    .iter()
                    .chain(extras.chain(groups).flatten())
            })
            .cloned()
            .collect();
        let mut seen: BTreeSet<(String, Option<String>)> = BTreeSet::new();
        while let Some(dependency) = todo.pop() {
            let Some(slug) = slugify(&dependency.name) else {
                continue;
            };
            let asked = std::iter::once(None).chain(dependency.extra.into_iter().map(Some));
            for extra in asked {
                if !seen.insert((slug.as_str().to_owned(), extra.clone())) {
                    continue;
                }
                for package in self.packages.get(slug.as_str()).into_iter().flatten() {
                    let adds = match &extra {
                        None => Some(&package.dependencies),
                        Some(extra) => package.optional_dependencies.get(extra),
                    };
                    todo.extend(adds.into_iter().flatten().cloned());
                }
            }
        }
        seen.into_iter()
            .map(|(slug, _)| slug)
            .filter(|slug| self.packages.contains_key(slug))
            .collect()
    }
}

/// How one package is pinned, from every entry `uv.lock` has for it.
fn pinned(path: &str, text: &str, entries: &[Package]) -> Result<Pinned, Diagnostic> {
    let (package, others) = entries
        .split_last()
        .expect("a package has at least one entry");
    let name = package.name.get_ref();
    if let Some(first) = others.first() {
        let span = first.name.span().start..package.name.span().end;
        return Err(Diagnostic {
            id: None,
            code: Code::LockAmbiguous,
            at: Some(crate::anchor::location(path, text, span)),
            message: format!(
                "`{path}` locks `{name}` {} times, such as once per platform, and a pin has one \
                 version; let uv lock one version for every platform, for example with a \
                 tighter bound on it or with `environments` under `[tool.uv]`",
                entries.len()
            ),
        });
    }
    let Some(version) = &package.version else {
        let why = format!(
            "package `{name}` has no fixed version, as when its version is dynamic, and a pin \
             has one; give it a static `version` in its own `pyproject.toml`"
        );
        return Err(invalid(path, text, package.name.span(), why));
    };
    if package.source.get_ref().subdirectory.is_some() {
        let why = format!(
            "package `{name}` comes from a subdirectory of an archive at a URL, which this \
             build does not read yet; depend on a built archive of it, or on a local copy \
             by its path"
        );
        return Err(invalid(path, text, package.source.span(), why));
    }
    let Some(origin) = origin(package.source.get_ref()) else {
        let why = format!(
            "package `{name}` has a source this build does not read: it reads one of \
             `registry`, `git` with a commit, `url`, `path`, `directory`, `editable` and \
             `virtual`; {FIX}"
        );
        return Err(invalid(path, text, package.source.span(), why));
    };
    let (name_span, version_span) = (package.name.span(), version.span());
    Ok(Pinned {
        version: version.get_ref().clone(),
        origin,
        span: name_span.start.min(version_span.start)..name_span.end.max(version_span.end),
    })
}

fn origin(source: &LockedSource) -> Option<Origin> {
    let LockedSource {
        registry,
        git,
        url,
        subdirectory: _,
        path,
        directory,
        editable,
        r#virtual,
    } = source;
    let forms = [registry, git, url, path, directory, editable, r#virtual];
    if forms.iter().filter(|form| form.is_some()).count() != 1 {
        return None;
    }
    if let Some(index) = registry {
        return Some(Origin::Registry(index.clone()));
    }
    if let Some(git) = git {
        let (url, commit) = git
            .split_once('#')
            .filter(|(_, commit)| !commit.is_empty())?;
        return Some(Origin::Git {
            url: url.to_owned(),
            commit: commit.to_owned(),
        });
    }
    if let Some(url) = url {
        return Some(Origin::Url(url.clone()));
    }
    let local = path.as_ref().or(directory.as_ref()).or(editable.as_ref());
    local.or(r#virtual.as_ref()).cloned().map(Origin::Path)
}

fn invalid(path: &str, text: &str, span: Range<usize>, why: String) -> Diagnostic {
    Diagnostic {
        id: None,
        code: Code::LockInvalid,
        at: Some(crate::anchor::location(path, text, span)),
        message: why,
    }
}

#[cfg(test)]
mod tests;
