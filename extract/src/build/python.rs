//! The Python version: from `.python-version`, else from the environment's
//! `pyvenv.cfg`. `requires-python` is a range, never a version.

use mental_model::{Code, Diagnostic, Id, Lines, Location, Python, PythonFrom};

const PYTHON_VERSION: &str = ".python-version";
const PYVENV: &str = ".venv/pyvenv.cfg";

/// The version of `unit`, from the files at `root`, the folder that holds
/// them relative to the build directory.
pub(super) fn version(
    unit: &Id,
    root: &str,
    python_version: Option<&str>,
    pyvenv: Option<&str>,
) -> Result<Python, Diagnostic> {
    let version_file = super::join(root, PYTHON_VERSION);
    let fix = format!("write one in `{version_file}`, for example with `uv python pin 3.12`");
    let problem = |at, message| Diagnostic {
        id: Some(unit.clone()),
        code: Code::PythonVersionUnknown,
        at,
        message,
    };
    let (file, from, stated) = match (python_version, pyvenv) {
        (Some(text), _) => (
            PYTHON_VERSION,
            PythonFrom::PythonVersionFile,
            first_line(text),
        ),
        (None, Some(text)) => (PYVENV, PythonFrom::Pyvenv, version_key(text)),
        (None, None) => {
            return Err(problem(
                None,
                format!(
                    "there is no `{version_file}` and no environment to take the Python \
                     version from; {fix}"
                ),
            ));
        }
    };
    let path = super::join(root, file);
    let Some((line, version)) = stated else {
        return Err(problem(
            None,
            format!("`{path}` states no Python version; {fix}"),
        ));
    };
    if !is_version(version) {
        return Err(problem(
            Some(Location {
                path: path.clone(),
                lines: Lines::new(line, line).expect("lines are counted from 1"),
            }),
            format!(
                "`{path}` says the Python version is `{version}`, which is not a version such \
                 as 3.12 or 3.12.1; {fix}"
            ),
        ));
    }
    Ok(Python {
        version: version.to_owned(),
        from,
    })
}

/// The first line that is neither blank nor a `#` comment, as uv reads it.
fn first_line(text: &str) -> Option<(u32, &str)> {
    (1..)
        .zip(text.lines().map(str::trim))
        .find(|(_, line)| !line.is_empty() && !line.starts_with('#'))
}

/// `version_info`, as uv writes it, or `version`, as Python's `venv` writes it.
fn version_key(text: &str) -> Option<(u32, &str)> {
    (1..).zip(text.lines()).find_map(|(number, line)| {
        let (key, value) = line.split_once('=')?;
        matches!(key.trim(), "version_info" | "version").then(|| (number, value.trim()))
    })
}

/// `3.12` or `3.12.1`: two or three numbers.
fn is_version(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    (2..=3).contains(&parts.len())
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests;
