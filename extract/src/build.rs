//! Stage 1: the build model, from a project's build files.

mod lock;
mod pyproject;
mod python;

use std::collections::BTreeSet;

use mental_model::{Anchor, BuildUnit, Code, Diagnostic, Id, Kind, Pin, Source};

use crate::anchor::{anchor, location};

/// A project's build files, as read from its directory; `None` for a file
/// that is not there.
#[derive(Clone, Copy, Debug)]
pub struct ProjectFiles<'a> {
    pub pyproject: Option<&'a str>,
    pub lock: Option<&'a str>,
    pub python_version: Option<&'a str>,
    /// `.venv/pyvenv.cfg`: the environment, when there is one.
    pub pyvenv: Option<&'a str>,
    /// Whether the project has a `src/` directory.
    pub has_src: bool,
}

/// The records of stage 1, with the anchor of each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildModel {
    pub units: Vec<BuildUnit>,
    pub pins: Vec<Pin>,
    pub anchors: Vec<Anchor>,
}

/// Reads a project's build files into its build unit and pins. Every gap is
/// a diagnostic, and a field whose input is missing is left out.
#[must_use]
pub fn build_model(files: &ProjectFiles) -> (BuildModel, Vec<Diagnostic>) {
    let mut model = BuildModel {
        units: Vec::new(),
        pins: Vec::new(),
        anchors: Vec::new(),
    };
    let mut diagnostics = Vec::new();
    let Some(text) = files.pyproject else {
        diagnostics.push(Diagnostic {
            id: None,
            code: Code::ProjectMissing,
            at: None,
            message: "there is no `pyproject.toml` here, so there is no project to build; run \
                      mentalc in a project's directory, or make one with `uv init`"
                .to_owned(),
        });
        return (model, diagnostics);
    };
    let project = match pyproject::read(text) {
        Ok(project) => project,
        Err(problem) => {
            diagnostics.push(problem);
            return (model, diagnostics);
        }
    };
    let unit = Id::root(&project.slug).child(Kind::Unit, &project.slug);
    let gap = |code, message: &str| Diagnostic {
        id: Some(unit.clone()),
        code,
        at: None,
        message: message.to_owned(),
    };
    let python = match python::version(&unit, files.python_version, files.pyvenv) {
        Ok(python) => Some(python),
        Err(problem) => {
            diagnostics.push(problem);
            None
        }
    };
    let lock = if let Some(text) = files.lock {
        let (locked, problems) = lock::read(text, &unit, &project.slug);
        diagnostics.extend(problems);
        locked.map(|locked| {
            model.pins = locked.pins;
            model.anchors = locked.anchors;
            locked.lock
        })
    } else {
        diagnostics.push(gap(
            Code::LockMissing,
            "the project has no `uv.lock`; run `uv lock` to write it",
        ));
        None
    };
    if files.pyvenv.is_none() {
        diagnostics.push(gap(
            Code::EnvironmentMissing,
            "the project has no environment (`.venv/pyvenv.cfg`); run `uv sync` to make it",
        ));
    }
    model
        .anchors
        .push(anchor(unit.clone(), pyproject::PATH, text, project.name));
    model.units.push(BuildUnit {
        id: unit,
        dir: ".".to_owned(),
        roots: BTreeSet::from([if files.has_src { "src" } else { "." }.to_owned()]),
        python,
        requires_python: project.requires_python,
        lock,
        environment: files.pyvenv.map(|_| ".venv".to_owned()),
        source: Source::Extractor,
    });
    (model, diagnostics)
}

/// A build file that does not read as TOML of the expected shape.
fn unreadable(
    code: Code,
    path: &str,
    text: &str,
    error: &toml::de::Error,
    fix: &str,
) -> Diagnostic {
    Diagnostic {
        id: None,
        code,
        at: error.span().map(|span| location(path, text, span)),
        message: format!(
            "`{path}` does not read: {}; {fix}",
            error.message().trim_end()
        ),
    }
}
