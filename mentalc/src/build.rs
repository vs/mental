//! The build: stage 1's model and, when stage 1 found no error, stage 2's
//! sites, assembled into the model's files and the diagnostics. It reads
//! and writes nothing; `repo` does.

use mental_extract::{BuildModel, Sites};
use mental_model::{Diagnostic, Resolution, Severity, Tally, jsonl};

use crate::Status;

pub(crate) struct Built {
    /// Each file of `.mental/`, by its path in it, with its text. A file
    /// with no text is removed: with no build unit that is every file, and
    /// a directory that is no project gets no `.mental/`; when stage 2 did
    /// not run, its files.
    pub(crate) files: Vec<(&'static str, Option<String>)>,
    pub(crate) units: usize,
    pub(crate) pins: usize,
    /// How the sites resolved, when stage 2 ran.
    pub(crate) sites: Option<Tally>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl Built {
    pub(crate) fn status(&self) -> Status {
        if failed(&self.diagnostics) {
            Status::Failure
        } else {
            Status::Success
        }
    }
}

fn failed(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.severity() == Severity::Error)
}

/// Whether stage 2 runs: when stage 1 found no error, as the build stops at
/// the first stage that fails.
pub(crate) fn ready(diagnostics: &[Diagnostic]) -> bool {
    !failed(diagnostics)
}

pub(crate) fn build(
    model: BuildModel,
    mut diagnostics: Vec<Diagnostic>,
    stage_two: Option<(Sites, Vec<Diagnostic>)>,
) -> Built {
    let mut anchors = model.anchors;
    let (sites, resolution) = match stage_two {
        Some((sites, found)) => {
            diagnostics.extend(found);
            anchors.extend(sites.anchors);
            let resolution = Resolution::of(&sites.sites);
            (Some(sites.sites), Some(resolution))
        }
        None => (None, None),
    };
    diagnostics.sort();
    let written = !model.units.is_empty();
    let file = |path, text: String| (path, written.then_some(text));
    Built {
        files: vec![
            file("model/format.json", jsonl::format_json()),
            file("model/build_unit.jsonl", jsonl::write(&model.units)),
            file("model/pin.jsonl", jsonl::write(&model.pins)),
            ("model/site.jsonl", sites.as_deref().map(jsonl::write)),
            file("anchors.jsonl", jsonl::write(&anchors)),
            file("diagnostics.jsonl", jsonl::write(&diagnostics)),
            (
                "resolution.json",
                resolution.as_ref().map(Resolution::to_json),
            ),
        ],
        units: model.units.len(),
        pins: model.pins.len(),
        sites: resolution.map(|resolution| resolution.all),
        diagnostics,
    }
}
