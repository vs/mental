//! ty's view of the machine: it reads the repository's files as `mentalc`
//! read them and the environment from disk, and sees no environment
//! variable, command, user configuration, cache or write (X5).

use std::any::Any;
use std::collections::BTreeMap;
use std::io;
use std::sync::Arc;

use ruff_db::system::walk_directory::WalkDirectoryBuilder;
use ruff_db::system::{
    DirectoryEntry, Metadata, OsSystem, System, SystemPath, SystemPathBuf, SystemVirtualPath,
    WhichError, WhichResult, WritableSystem,
};
use ruff_notebook::{Notebook, NotebookError};

#[derive(Debug, Clone)]
pub(super) struct ReadOnly {
    os: OsSystem,
    /// The text of each repository file, by absolute path, as `mentalc`
    /// read it, so ty and the site walk see the same bytes.
    files: Arc<BTreeMap<SystemPathBuf, String>>,
}

impl ReadOnly {
    pub(super) fn new(project: &SystemPath, files: BTreeMap<SystemPathBuf, String>) -> ReadOnly {
        ReadOnly {
            os: OsSystem::new(project),
            files: Arc::new(files),
        }
    }
}

impl System for ReadOnly {
    fn path_metadata(&self, path: &SystemPath) -> io::Result<Metadata> {
        self.os.path_metadata(path)
    }
    fn canonicalize_path(&self, path: &SystemPath) -> io::Result<SystemPathBuf> {
        self.os.canonicalize_path(path)
    }
    fn is_same_file(&self, first: &SystemPath, second: &SystemPath) -> io::Result<bool> {
        self.os.is_same_file(first, second)
    }
    fn which(&self, _binary_name: &str) -> WhichResult {
        Err(WhichError::CannotFindBinaryPath)
    }
    fn read_to_string(&self, path: &SystemPath) -> io::Result<String> {
        match self.files.get(path) {
            Some(text) => Ok(text.clone()),
            None => self.os.read_to_string(path),
        }
    }
    fn read_to_notebook(&self, path: &SystemPath) -> Result<Notebook, NotebookError> {
        self.os.read_to_notebook(path)
    }
    fn read_virtual_path_to_string(&self, path: &SystemVirtualPath) -> io::Result<String> {
        self.os.read_virtual_path_to_string(path)
    }
    fn read_virtual_path_to_notebook(
        &self,
        path: &SystemVirtualPath,
    ) -> Result<Notebook, NotebookError> {
        self.os.read_virtual_path_to_notebook(path)
    }
    fn path_exists(&self, path: &SystemPath) -> bool {
        self.os.path_exists(path)
    }
    fn current_directory(&self) -> &SystemPath {
        self.os.current_directory()
    }
    fn user_config_directory(&self) -> Option<SystemPathBuf> {
        None
    }
    fn cache_dir(&self) -> Option<SystemPathBuf> {
        None
    }
    fn read_directory<'a>(
        &'a self,
        path: &SystemPath,
    ) -> io::Result<Box<dyn Iterator<Item = io::Result<DirectoryEntry>> + 'a>> {
        self.os.read_directory(path)
    }
    fn walk_directory(&self, path: &SystemPath) -> WalkDirectoryBuilder {
        self.os.walk_directory(path)
    }
    fn as_writable(&self) -> Option<&dyn WritableSystem> {
        None
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn dyn_clone(&self) -> Box<dyn System> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests;
