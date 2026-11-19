//! Turns a project's build files, and later its Python source, into the
//! model's records. It touches no file itself: callers pass file contents in.
#![deny(clippy::wildcard_enum_match_arm)]

mod build;

pub use build::{BuildModel, ProjectFiles, build_model};
