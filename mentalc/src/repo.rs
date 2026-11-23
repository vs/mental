//! The file-system edge: reads a project's build files, its Python files
//! and its environment's `RECORD`s, and writes `.mental/`.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, anyhow, bail};
use mental_extract::{Member, ProjectFiles, Root, Workspace, workspace};
use mental_model::BuildUnit;

/// The build files of the build directory and of the workspace it belongs
/// to, as read; `None` for a file that is not there.
pub(crate) struct Tree {
    /// The build directory's path, which is UTF-8.
    pub(crate) path: String,
    name: String,
    pyproject: Option<String>,
    has_src: bool,
    lock: Option<String>,
    python_version: Option<String>,
    pyvenv: Option<String>,
    /// For a member built on its own: the root's folder relative to the
    /// build directory, its name and its `pyproject.toml`.
    root: Option<(String, String, String)>,
    /// For a workspace root: each member's folder, `pyproject.toml` and
    /// whether it has `src/`.
    members: Vec<(String, Option<String>, bool)>,
}

impl Tree {
    pub(crate) fn files<'a>(&'a self, members: &'a [Member<'a>]) -> ProjectFiles<'a> {
        ProjectFiles {
            name: &self.name,
            pyproject: self.pyproject.as_deref(),
            has_src: self.has_src,
            lock: self.lock.as_deref(),
            python_version: self.python_version.as_deref(),
            pyvenv: self.pyvenv.as_deref(),
            root: self.root.as_ref().map(|(dir, name, pyproject)| Root {
                dir,
                name,
                pyproject,
            }),
            members,
        }
    }

    pub(crate) fn members(&self) -> Vec<Member<'_>> {
        self.members
            .iter()
            .map(|(dir, pyproject, has_src)| Member {
                dir,
                pyproject: pyproject.as_deref(),
                has_src: *has_src,
            })
            .collect()
    }
}

/// A file's text, or `None` when there is no such file.
fn optional(path: &Path, shown: &str) -> anyhow::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| {
            format!("cannot read `{shown}`; make it a readable UTF-8 file, or remove it")
        }),
    }
}

/// The build files of `dir`. A workspace root's members are found by its
/// patterns; a project that is no root may be a member of the nearest
/// workspace above it, whose root holds the lock, Python version and
/// environment, as uv keeps them.
pub(crate) fn read(dir: &Path) -> anyhow::Result<Tree> {
    let path = dir.to_str().with_context(|| {
        format!(
            "the project directory `{}` is not UTF-8; build from a directory whose path is",
            dir.display()
        )
    })?;
    let pyproject = optional(&dir.join("pyproject.toml"), "pyproject.toml")?;
    let own = pyproject.as_deref().and_then(workspace);
    let (root, members) = match &own {
        Some(workspace) => (None, members(dir, workspace)?),
        None if pyproject.is_some() => (workspace_root(dir)?, Vec::new()),
        None => (None, Vec::new()),
    };
    let shared = root.as_ref().map_or(".", |(dir, _, _)| dir.as_str());
    let file = |name: &str| {
        let shown = if shared == "." {
            name.to_owned()
        } else {
            format!("{shared}/{name}")
        };
        optional(&dir.join(&shown), &shown)
    };
    Ok(Tree {
        path: path.to_owned(),
        name: dir_name(dir),
        has_src: dir.join("src").is_dir(),
        lock: file("uv.lock")?,
        python_version: file(".python-version")?,
        pyvenv: file(".venv/pyvenv.cfg")?,
        pyproject,
        root,
        members,
    })
}

/// The name of `dir`, a folder on the build directory's path, which `read`
/// checked is UTF-8; the root of the file system has none.
fn dir_name(dir: &Path) -> String {
    dir.file_name().map_or_else(String::new, |name| {
        name.to_str()
            .expect("the build directory's path is UTF-8")
            .to_owned()
    })
}

/// The folders under `dir` that `workspace` names as members, sorted, each
/// with its `pyproject.toml` and whether it has `src/`. A `*` segment
/// stands for any folder whose name does not start with `.`.
fn members(
    dir: &Path,
    workspace: &Workspace,
) -> anyhow::Result<Vec<(String, Option<String>, bool)>> {
    let mut found = std::collections::BTreeSet::new();
    for pattern in &workspace.members {
        let mut paths = vec![String::new()];
        for segment in pattern.trim_end_matches('/').split('/') {
            let mut next = Vec::new();
            for path in paths {
                let join = |name: &str| {
                    if path.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{path}/{name}")
                    }
                };
                if segment.contains('*') {
                    let folder = if path.is_empty() { "." } else { path.as_str() };
                    for (name, _) in entries(dir, folder)? {
                        if !name.starts_with('.') && dir.join(join(&name)).is_dir() {
                            next.push(join(&name));
                        }
                    }
                } else if dir.join(join(segment)).is_dir() {
                    next.push(join(segment));
                }
            }
            paths = next;
        }
        found.extend(paths.into_iter().filter(|path| workspace.includes(path)));
    }
    found
        .into_iter()
        .map(|member| {
            let shown = format!("{member}/pyproject.toml");
            let pyproject = optional(&dir.join(&shown), &shown)?;
            let has_src = dir.join(&member).join("src").is_dir();
            Ok((member, pyproject, has_src))
        })
        .collect()
}

/// The root of the nearest workspace above `dir`, when it lists `dir` as a
/// member: its folder relative to `dir`, its name and its `pyproject.toml`.
fn workspace_root(dir: &Path) -> anyhow::Result<Option<(String, String, String)>> {
    let mut up = Vec::new();
    let mut below: Vec<String> = Vec::new();
    for (child, parent) in dir.ancestors().zip(dir.ancestors().skip(1)) {
        below.insert(0, dir_name(child));
        up.push("..");
        let shown = format!("{}/pyproject.toml", up.join("/"));
        let Some(text) = optional(&parent.join("pyproject.toml"), &shown)? else {
            continue;
        };
        let Some(workspace) = workspace(&text) else {
            continue;
        };
        return Ok(workspace
            .includes(&below.join("/"))
            .then(|| (up.join("/"), dir_name(parent), text)));
    }
    Ok(None)
}

/// What stage 2 reads: the unit's Python files and the `RECORD` of each
/// distribution installed in its environment, each as `(name, text)`.
pub(crate) struct Sources {
    /// Relative to the project directory, with `/` between folders.
    pub(crate) files: Vec<(String, String)>,
    /// By the name of its `.dist-info` folder.
    pub(crate) records: Vec<(String, String)>,
}

/// Every `.py` file under the unit's roots, leaving out files and folders
/// whose name starts with `.` (as `.venv`) and `__pycache__`, and each `RECORD` of the
/// environment's `site-packages`. A symbolic link is not followed: one to a
/// folder, or one whose name ends in `.py`, is an error, so no file goes
/// unread unsaid; a broken one, like any other, is ignored.
/// Folders in `skip`, the other units of a workspace, are left out.
pub(crate) fn read_sources(dir: &Path, unit: &BuildUnit, skip: &[&str]) -> anyhow::Result<Sources> {
    let mut files = Vec::new();
    for root in &unit.roots {
        let prefix = if root == "." {
            String::new()
        } else {
            format!("{root}/")
        };
        python_files(dir, &prefix, skip, &mut files)?;
    }
    files.sort();
    let environment = unit
        .environment
        .as_deref()
        .expect("stage 2 runs only when stage 1 found no gap, so the unit has its environment");
    let mut records = Vec::new();
    for site_packages in site_packages(dir, environment)? {
        dist_records(dir, &site_packages, &mut records)?;
    }
    records.sort();
    Ok(Sources { files, records })
}

/// What failed, with the path relative to the project directory.
fn cannot_read(relative: &str) -> String {
    format!(
        "cannot read `{relative}`; make it readable, as a UTF-8 file if it is one, or remove it"
    )
}

/// The entries of the folder at `relative`, each with its name.
fn entries(dir: &Path, relative: &str) -> anyhow::Result<Vec<(String, fs::DirEntry)>> {
    let mut found = Vec::new();
    let folder = dir.join(relative);
    for entry in fs::read_dir(folder).with_context(|| cannot_read(relative))? {
        let entry = entry.with_context(|| cannot_read(relative))?;
        let name = entry.file_name().into_string().map_err(|name| {
            anyhow!(
                "`{relative}` holds `{}`, whose name is not UTF-8; rename it",
                name.to_string_lossy()
            )
        })?;
        found.push((name, entry));
    }
    Ok(found)
}

fn python_files(
    dir: &Path,
    prefix: &str,
    skip: &[&str],
    files: &mut Vec<(String, String)>,
) -> anyhow::Result<()> {
    let folder = prefix.strip_suffix('/').unwrap_or(".");
    for (name, entry) in entries(dir, folder)? {
        let path = format!("{prefix}{name}");
        if name.starts_with('.') || name == "__pycache__" || skip.contains(&path.as_str()) {
            continue;
        }
        let kind = entry.file_type().with_context(|| cannot_read(&path))?;
        let python = Path::new(&name).extension().is_some_and(|ext| ext == "py");
        if kind.is_symlink() {
            let to_folder = fs::metadata(entry.path()).is_ok_and(|target| target.is_dir());
            if to_folder || python {
                bail!(
                    "`{path}` is a symbolic link, which mentalc does not follow, so the files \
                     it leads to would go unread; replace it with what it links to, or remove it"
                );
            }
        } else if kind.is_dir() {
            python_files(dir, &format!("{path}/"), skip, files)?;
        } else if kind.is_file() && python {
            let text = fs::read_to_string(entry.path()).with_context(|| cannot_read(&path))?;
            files.push((path, text));
        }
    }
    Ok(())
}

/// The environment's `site-packages`, relative to the project directory:
/// `lib/python3.12/site-packages`, the layout of an environment on Unix.
/// None when the environment has no `lib`.
fn site_packages(dir: &Path, environment: &str) -> anyhow::Result<Vec<String>> {
    let lib = format!("{environment}/lib");
    if !dir.join(&lib).exists() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for (name, _) in entries(dir, &lib)? {
        let path = format!("{lib}/{name}/site-packages");
        if dir.join(&path).is_dir() {
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}

fn dist_records(
    dir: &Path,
    site_packages: &str,
    records: &mut Vec<(String, String)>,
) -> anyhow::Result<()> {
    for (name, entry) in entries(dir, site_packages)? {
        if !name.ends_with(".dist-info") {
            continue;
        }
        let record = entry.path().join("RECORD");
        match fs::read_to_string(&record) {
            Ok(text) => records.push((name, text)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| cannot_read(&format!("{site_packages}/{name}/RECORD")));
            }
        }
    }
    Ok(())
}

/// Writes each file that has a text under `.mental/`, and removes each that
/// has none. Every file is written whole beside its place before any is
/// moved into place, and none is removed before all are in place, so a
/// failed write leaves the earlier model as it was.
pub(crate) fn write(dir: &Path, files: &[(&str, Option<String>)]) -> anyhow::Result<()> {
    let place = |path: &str| dir.join(".mental").join(path);
    let fail = |doing: &str, path: &str| {
        format!("cannot {doing} `.mental/{path}`; check that the project directory is writable")
    };
    for (path, text) in files {
        if let Some(text) = text {
            let target = place(path);
            let folder = target.parent().expect("every file is inside `.mental/`");
            fs::create_dir_all(folder).with_context(|| fail("make the folder of", path))?;
            let partial = target.with_extension("partial");
            fs::write(partial, text).with_context(|| fail("write", path))?;
        }
    }
    for (path, _) in files.iter().filter(|(_, text)| text.is_some()) {
        let target = place(path);
        let partial = target.with_extension("partial");
        fs::rename(partial, &target).with_context(|| fail("replace", path))?;
    }
    for (path, _) in files.iter().filter(|(_, text)| text.is_none()) {
        match fs::remove_file(place(path)) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                return Err(error).with_context(|| fail("remove", path));
            }
            Ok(()) | Err(_) => {}
        }
    }
    Ok(())
}
