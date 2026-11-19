//! The build: the project's files in, the model's files and the
//! diagnostics out. It writes nothing; `repo` does.

use mental_extract::{ProjectFiles, build_model};
use mental_model::{Diagnostic, Severity, jsonl};

use crate::Status;

pub(crate) struct Built {
    /// Each file of `.mental/`, by its path in it, with its text. With no
    /// build unit every text is `None`: an earlier build's file is removed,
    /// and a directory that is no project gets no `.mental/`.
    pub(crate) files: Vec<(&'static str, Option<String>)>,
    pub(crate) units: usize,
    pub(crate) pins: usize,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl Built {
    pub(crate) fn status(&self) -> Status {
        let failed = self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.severity() == Severity::Error);
        if failed {
            Status::Failure
        } else {
            Status::Success
        }
    }
}

pub(crate) fn build(files: &ProjectFiles) -> Built {
    let (model, mut diagnostics) = build_model(files);
    diagnostics.sort();
    let written = !model.units.is_empty();
    let file = |path, text: String| (path, written.then_some(text));
    Built {
        files: vec![
            file("model/format.json", jsonl::format_json()),
            file("model/build_unit.jsonl", jsonl::write(&model.units)),
            file("model/pin.jsonl", jsonl::write(&model.pins)),
            file("anchors.jsonl", jsonl::write(&model.anchors)),
            file("diagnostics.jsonl", jsonl::write(&diagnostics)),
        ],
        units: model.units.len(),
        pins: model.pins.len(),
        diagnostics,
    }
}
