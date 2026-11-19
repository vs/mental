//! The file-system edge: reads a project's build files and writes `.mental/`.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::Context;
use mental_extract::ProjectFiles;

/// A project's build files as read; `None` for a file that is not there.
pub(crate) struct Tree {
    pyproject: Option<String>,
    lock: Option<String>,
    python_version: Option<String>,
    pyvenv: Option<String>,
    has_src: bool,
}

impl Tree {
    pub(crate) fn files(&self) -> ProjectFiles<'_> {
        ProjectFiles {
            pyproject: self.pyproject.as_deref(),
            lock: self.lock.as_deref(),
            python_version: self.python_version.as_deref(),
            pyvenv: self.pyvenv.as_deref(),
            has_src: self.has_src,
        }
    }
}

pub(crate) fn read(dir: &Path) -> anyhow::Result<Tree> {
    let file = |name: &str| match fs::read_to_string(dir.join(name)) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| {
            format!("cannot read `{name}`; make it a readable UTF-8 file, or remove it")
        }),
    };
    Ok(Tree {
        pyproject: file("pyproject.toml")?,
        lock: file("uv.lock")?,
        python_version: file(".python-version")?,
        pyvenv: file(".venv/pyvenv.cfg")?,
        has_src: dir.join("src").is_dir(),
    })
}

/// Writes each file that has a text under `.mental/`, and removes each that
/// has none. Every file is written whole beside its place before any is
/// moved into place, so a failed write leaves the earlier model as it was.
pub(crate) fn write(dir: &Path, files: &[(&str, Option<String>)]) -> anyhow::Result<()> {
    let place = |path: &str| dir.join(".mental").join(path);
    let fail = |doing: &str, path: &str| {
        format!("cannot {doing} `.mental/{path}`; check that the project directory is writable")
    };
    for (path, text) in files {
        let target = place(path);
        if let Some(text) = text {
            let folder = target.parent().expect("every file is inside `.mental/`");
            fs::create_dir_all(folder).with_context(|| fail("make the folder of", path))?;
            let partial = target.with_extension("partial");
            fs::write(partial, text).with_context(|| fail("write", path))?;
        } else {
            match fs::remove_file(&target) {
                Err(error) if error.kind() != io::ErrorKind::NotFound => {
                    return Err(error).with_context(|| fail("remove", path));
                }
                Ok(()) | Err(_) => {}
            }
        }
    }
    for (path, _) in files.iter().filter(|(_, text)| text.is_some()) {
        let target = place(path);
        let partial = target.with_extension("partial");
        fs::rename(partial, &target).with_context(|| fail("replace", path))?;
    }
    Ok(())
}
