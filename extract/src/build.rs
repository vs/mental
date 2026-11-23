//! Stage 1: the build model, from a project's build files.

mod lock;
mod pyproject;
mod python;

use std::collections::BTreeSet;

use mental_model::{Anchor, BuildUnit, Code, Diagnostic, Id, Kind, Pin, Slug, Source, slugify};

use pyproject::Project;
pub use pyproject::{Workspace, workspace};

use crate::anchor::{anchor, location};

/// The build files of the build directory, as read; `None` for a file that
/// is not there.
#[derive(Clone, Copy, Debug)]
pub struct ProjectFiles<'a> {
    /// The build directory's name, which names the system when it is a
    /// workspace root with no `[project]`.
    pub name: &'a str,
    pub pyproject: Option<&'a str>,
    /// Whether the build directory has a `src/` directory.
    pub has_src: bool,
    /// The `uv.lock`, `.python-version` and `.venv/pyvenv.cfg` where uv keeps
    /// them: in the build directory, or in the root of the workspace it is
    /// a member of.
    pub lock: Option<&'a str>,
    pub python_version: Option<&'a str>,
    pub pyvenv: Option<&'a str>,
    /// When the build directory is a member of a workspace: its root.
    pub root: Option<Root<'a>>,
    /// When the build directory is a workspace's root: each member.
    pub members: &'a [Member<'a>],
}

/// The root of the workspace a member built on its own belongs to.
#[derive(Clone, Copy, Debug)]
pub struct Root<'a> {
    /// Relative to the build directory: `..`, `../..` and so on.
    pub dir: &'a str,
    /// The root directory's name, which names the system when the root has
    /// no `[project]`.
    pub name: &'a str,
    pub pyproject: &'a str,
}

/// A member of the workspace whose root is the build directory.
#[derive(Clone, Copy, Debug)]
pub struct Member<'a> {
    /// Relative to the root, with `/` between folders.
    pub dir: &'a str,
    pub pyproject: Option<&'a str>,
    pub has_src: bool,
}

/// The records of stage 1, with the anchor of each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildModel {
    pub units: Vec<BuildUnit>,
    pub pins: Vec<Pin>,
    pub anchors: Vec<Anchor>,
}

/// A project to build, as its `pyproject.toml` declares it.
struct Declared<'a> {
    dir: &'a str,
    text: &'a str,
    project: Project,
    has_src: bool,
}

/// Reads the build files into the build units and their pins: one unit for
/// a project, one for each member of a workspace and for its root when it
/// is a project too, or the one member built on its own. Every gap is a
/// diagnostic, and a field whose input is missing is left out.
#[must_use]
pub fn build_model(files: &ProjectFiles) -> (BuildModel, Vec<Diagnostic>) {
    let mut model = BuildModel {
        units: Vec::new(),
        pins: Vec::new(),
        anchors: Vec::new(),
    };
    let mut diagnostics = Vec::new();
    let (system, projects) = match declared(files, &mut diagnostics) {
        Ok(found) => found,
        Err(problem) => {
            diagnostics.push(problem);
            return (model, diagnostics);
        }
    };
    let shared = files.root.map_or(".", |root| root.dir);
    let lockfile = files.lock.and_then(|text| {
        let (lockfile, problems) = lock::read(text, &join(shared, LOCK));
        diagnostics.extend(problems);
        lockfile
    });
    for declared in projects {
        let unit = system.child(Kind::Unit, &declared.project.slug);
        let gap = |code, message: String| Diagnostic {
            id: Some(unit.clone()),
            code,
            at: None,
            message,
        };
        let python = match python::version(&unit, shared, files.python_version, files.pyvenv) {
            Ok(python) => Some(python),
            Err(problem) => {
                diagnostics.push(problem);
                None
            }
        };
        if let Some(lockfile) = &lockfile {
            let reached = lockfile.pins(&unit, &declared.project.slug);
            model.pins.extend(reached.pins);
            model.anchors.extend(reached.anchors);
            diagnostics.extend(reached.diagnostics);
        } else if files.lock.is_none() {
            let lock = join(shared, LOCK);
            let message = format!("the project has no `{lock}`; run `uv lock` to write it");
            diagnostics.push(gap(Code::LockMissing, message));
        }
        if files.pyvenv.is_none() {
            let venv = join(shared, VENV);
            let message = format!(
                "the project has no environment (`{venv}/pyvenv.cfg`); run `uv sync` to make it"
            );
            diagnostics.push(gap(Code::EnvironmentMissing, message));
        }
        let pyproject = join(declared.dir, pyproject::PATH);
        let name = declared.project.name;
        model
            .anchors
            .push(anchor(unit.clone(), &pyproject, declared.text, name));
        let root = join(declared.dir, if declared.has_src { "src" } else { "." });
        model.units.push(BuildUnit {
            id: unit,
            dir: declared.dir.to_owned(),
            roots: BTreeSet::from([root]),
            python,
            requires_python: declared.project.requires_python,
            lock: lockfile.as_ref().map(|lockfile| lockfile.lock.clone()),
            environment: files.pyvenv.map(|_| join(shared, VENV)),
            source: Source::Extractor,
        });
    }
    diagnostics.sort();
    diagnostics.dedup();
    (model, diagnostics)
}

/// The system and the projects to build: the build directory's own, and a
/// workspace root's members. A member that declares no project is a
/// diagnostic in `diagnostics`; with no project or system, nothing builds.
fn declared<'a>(
    files: &ProjectFiles<'a>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(Id, Vec<Declared<'a>>), Diagnostic> {
    let text = files.pyproject.ok_or_else(|| Diagnostic {
        id: None,
        code: Code::ProjectMissing,
        at: None,
        message: "there is no `pyproject.toml` here, so there is no project to build; run \
                  mentalc in a project's directory, or make one with `uv init`"
            .to_owned(),
    })?;
    let declared = pyproject::read(text, pyproject::PATH)?;
    let system = match files.root {
        Some(root) => {
            let root_declared = pyproject::read(root.pyproject, &join(root.dir, pyproject::PATH))?;
            system_name(root_declared.project, root.name)?
        }
        None => system_name(declared.project.clone(), files.name)?,
    };
    let mut projects = Vec::new();
    if let Some(project) = declared.project {
        projects.push(Declared {
            dir: ".",
            text,
            project,
            has_src: files.has_src,
        });
    }
    if files.root.is_none() && declared.workspace.is_some() {
        for member in files.members {
            match member_project(member) {
                Ok(project) => projects.push(project),
                Err(problem) => diagnostics.push(problem),
            }
        }
    }
    if projects.is_empty() && files.members.is_empty() {
        return Err(Diagnostic {
            id: None,
            code: Code::ProjectInvalid,
            at: None,
            message: "the workspace root has no `[project]`, and its `members` name no folder, \
                      so there is nothing to build; fix the patterns, or give the root a \
                      `[project]`"
                .to_owned(),
        });
    }
    Ok((Id::root(&system), projects))
}

const LOCK: &str = "uv.lock";
const VENV: &str = ".venv";

/// The system's name: the root's project's, else the root directory's.
fn system_name(project: Option<Project>, dir_name: &str) -> Result<Slug, Diagnostic> {
    if let Some(project) = project {
        return Ok(project.slug);
    }
    slugify(dir_name).ok_or_else(|| Diagnostic {
        id: None,
        code: Code::ProjectInvalid,
        at: None,
        message: format!(
            "the workspace root has no `[project]`, so the system is named after its \
             directory, and `{dir_name}` has no ASCII letter; rename the directory, or give \
             the root a `[project]` with a name"
        ),
    })
}

/// The project a member declares.
fn member_project<'a>(member: &Member<'a>) -> Result<Declared<'a>, Diagnostic> {
    let path = join(member.dir, pyproject::PATH);
    let Some(text) = member.pyproject else {
        return Err(Diagnostic {
            id: None,
            code: Code::ProjectMissing,
            at: None,
            message: format!(
                "the workspace member `{}` has no `pyproject.toml`; give it one with \
                 `uv init`, or leave it out of the workspace's `members`",
                member.dir
            ),
        });
    };
    let project = pyproject::read(text, &path)?
        .project
        .ok_or_else(|| Diagnostic {
            id: None,
            code: Code::ProjectInvalid,
            at: None,
            message: format!(
                "the workspace member `{}` declares no `[project]`, so it is no project; give \
             `{path}` a `[project]` table with a `name`",
                member.dir
            ),
        })?;
    Ok(Declared {
        dir: member.dir,
        text,
        project,
        has_src: member.has_src,
    })
}

/// `name` in the folder `dir`, either of which may be `.`.
fn join(dir: &str, name: &str) -> String {
    match (dir, name) {
        (".", _) => name.to_owned(),
        (_, ".") => dir.to_owned(),
        _ => format!("{dir}/{name}"),
    }
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
