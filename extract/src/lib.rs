//! Turns a project's build files and Python source into the model's
//! records. It touches no file itself: callers pass file contents in.
#![deny(clippy::wildcard_enum_match_arm)]

mod anchor;
mod build;
mod python;

pub use build::{BuildModel, ProjectFiles, build_model};
pub use python::{PythonFile, Sites, list_sites};
