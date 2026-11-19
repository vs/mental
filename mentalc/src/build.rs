//! The build: the project's files in, the model's files and the
//! diagnostics out. It writes nothing; `repo` does.

use mental_extract::{ProjectFiles, build_model};
use mental_model::{Diagnostic, Severity, jsonl};

use crate::Status;

pub(crate) struct Built {
    /// Each file of `.mental/`, by its path in it. None when there is no
    /// build unit, so that a directory that is no project gets no `.mental/`.
    pub(crate) files: Vec<(&'static str, String)>,
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
    let files = if model.units.is_empty() {
        Vec::new()
    } else {
        vec![
            ("model/format.json", jsonl::format_json()),
            ("model/build_unit.jsonl", jsonl::write(&model.units)),
            ("model/pin.jsonl", jsonl::write(&model.pins)),
            ("anchors.jsonl", jsonl::write(&model.anchors)),
            ("diagnostics.jsonl", jsonl::write(&diagnostics)),
        ]
    };
    Built {
        files,
        units: model.units.len(),
        pins: model.pins.len(),
        diagnostics,
    }
}
