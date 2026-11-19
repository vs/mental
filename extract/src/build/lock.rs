//! `uv.lock`: a pin for each locked package but the project itself.

use std::collections::BTreeMap;
use std::ops::Range;

use mental_model::{Anchor, Code, Diagnostic, Id, Kind, Lock, Origin, Pin, Slug, Source, slugify};
use serde::Deserialize;
use toml::Spanned;

const PATH: &str = "uv.lock";
/// The schema version this build reads.
const VERSION: u32 = 1;
const FIX: &str = "run `uv lock` to write it again";

pub(super) struct Locked {
    pub(super) lock: Lock,
    pub(super) pins: Vec<Pin>,
    pub(super) anchors: Vec<Anchor>,
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
struct Package {
    name: Spanned<String>,
    /// uv leaves it out for a local package whose version is dynamic.
    version: Option<Spanned<String>>,
    source: Spanned<LockedSource>,
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

/// Reads `text`, the project's `uv.lock`. `project` names the package that
/// is the project itself, which gets no pin.
pub(super) fn read(text: &str, unit: &Id, project: &Slug) -> (Option<Locked>, Vec<Diagnostic>) {
    let unreadable = |error| {
        (
            None,
            vec![super::unreadable(
                Code::LockInvalid,
                PATH,
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
            at: Some(super::location(PATH, text, head.version.span())),
            message: format!(
                "`{PATH}` has schema version {}, and this build reads only version {VERSION}; \
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
    let mut packages: BTreeMap<String, (Slug, Vec<Package>)> = BTreeMap::new();
    for package in file.package {
        let Some(slug) = slugify(package.name.get_ref()) else {
            let why = format!(
                "package `{}` has no ASCII letter in its name, so it can have no ID; \
                 remove the dependency on it, or depend on it under a name with a letter",
                package.name.get_ref()
            );
            diagnostics.push(invalid(text, package.name.span(), why));
            continue;
        };
        if slug != *project {
            let entry = packages.entry(slug.as_str().to_owned());
            entry.or_insert_with(|| (slug, Vec::new())).1.push(package);
        }
    }
    let mut locked = Locked {
        lock: Lock {
            path: PATH.to_owned(),
            version: VERSION,
        },
        pins: Vec::new(),
        anchors: Vec::new(),
    };
    for (slug, entries) in packages.into_values() {
        match pin(text, unit, &slug, entries) {
            Ok((pin, anchor)) => {
                locked.pins.push(pin);
                locked.anchors.push(anchor);
            }
            Err(problem) => diagnostics.push(problem),
        }
    }
    (Some(locked), diagnostics)
}

/// The pin of one package, from every entry `uv.lock` has for it.
fn pin(
    text: &str,
    unit: &Id,
    slug: &Slug,
    mut entries: Vec<Package>,
) -> Result<(Pin, Anchor), Diagnostic> {
    let package = entries.pop().expect("a package has at least one entry");
    let name = package.name.get_ref();
    if let Some(first) = entries.first() {
        let span = first.name.span().start..package.name.span().end;
        return Err(Diagnostic {
            id: None,
            code: Code::LockAmbiguous,
            at: Some(super::location(PATH, text, span)),
            message: format!(
                "`{PATH}` locks `{name}` {} times, such as once per platform, and a pin has one \
                 version; let uv lock one version for every platform, for example with a \
                 tighter bound on it or with `environments` under `[tool.uv]`",
                entries.len() + 1
            ),
        });
    }
    let Some(version) = &package.version else {
        let why = format!(
            "package `{name}` has no fixed version, as when its version is dynamic, and a pin \
             has one; give it a static `version` in its own `pyproject.toml`"
        );
        return Err(invalid(text, package.name.span(), why));
    };
    if package.source.get_ref().subdirectory.is_some() {
        let why = format!(
            "package `{name}` comes from a subdirectory of an archive at a URL, which this \
             build does not read yet; depend on a built archive of it, or on a local copy \
             by its path"
        );
        return Err(invalid(text, package.source.span(), why));
    }
    let Some(origin) = origin(package.source.get_ref()) else {
        let why = format!(
            "package `{name}` has a source this build does not read: it reads one of \
             `registry`, `git` with a commit, `url`, `path`, `directory`, `editable` and \
             `virtual`; {FIX}"
        );
        return Err(invalid(text, package.source.span(), why));
    };
    let (name_span, version_span) = (package.name.span(), version.span());
    let span = name_span.start.min(version_span.start)..name_span.end.max(version_span.end);
    let id = unit.child(Kind::Pin, slug);
    let anchor = super::anchor(id.clone(), PATH, text, span);
    let pin = Pin {
        id,
        package: slug.as_str().to_owned(),
        version: version.get_ref().clone(),
        origin,
        source: Source::Extractor,
    };
    Ok((pin, anchor))
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

fn invalid(text: &str, span: Range<usize>, why: String) -> Diagnostic {
    Diagnostic {
        id: None,
        code: Code::LockInvalid,
        at: Some(super::location(PATH, text, span)),
        message: why,
    }
}

#[cfg(test)]
mod tests;
