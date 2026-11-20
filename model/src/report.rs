//! The resolution report: how the sites of a build resolved, by kind and
//! by reason, written to `.mental/resolution.json`.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Outcome, Reason, Site, SiteKind};

/// The outcomes of a build's sites: one line of `resolution.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Resolution {
    /// Every site, whatever its kind.
    pub all: Tally,
    /// The sites of each kind that has any.
    pub kinds: BTreeMap<SiteKind, Tally>,
}

/// How many sites resolved into the repository, to an external package, or
/// not at all, and why not.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Tally {
    pub resolved: u32,
    pub external: u32,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub unresolved: BTreeMap<Reason, u32>,
}

impl Tally {
    fn count(&mut self, outcome: &Outcome) {
        match outcome {
            Outcome::Resolved { .. } => self.resolved += 1,
            Outcome::External { .. } => self.external += 1,
            Outcome::Unresolved { reason } => *self.unresolved.entry(*reason).or_default() += 1,
        }
    }

    /// The sites that did not resolve, for whatever reason.
    #[must_use]
    pub fn unresolved_total(&self) -> u32 {
        self.unresolved.values().sum()
    }
}

impl Resolution {
    #[must_use]
    pub fn of(sites: &[Site]) -> Resolution {
        let mut report = Resolution::default();
        for site in sites {
            report.all.count(&site.outcome);
            report
                .kinds
                .entry(site.kind)
                .or_default()
                .count(&site.outcome);
        }
        report
    }

    /// The text of `resolution.json`: one line and a final newline.
    ///
    /// # Panics
    ///
    /// Never: its map keys are kind and reason names, its values counts.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut text = serde_json::to_string(self)
            .expect("a report serializes: its keys are kind and reason names, its values counts");
        text.push('\n');
        text
    }
}

#[cfg(test)]
mod tests;
