//! The hand-written references, `examples/report-jobs/model/` and
//! `examples/report-jobs-java/model/`, checked against their examples' own
//! files. They are only ever read here.

use mental_model::{Anchor, BuildUnit, Id, Pin, Site, jsonl};

/// One example: its reference records and every file its anchors quote.
struct Reference {
    units: &'static str,
    pins: &'static str,
    sites: &'static str,
    anchors: &'static str,
    files: &'static [(&'static str, &'static str)],
}

macro_rules! example_files {
    ($example:literal: $($path:literal),+ $(,)?) => {
        &[$(($path, include_str!(concat!("../../examples/", $example, "/", $path)))),+]
    };
}

macro_rules! reference {
    ($example:literal: $($path:literal),+ $(,)?) => {
        Reference {
            units: include_str!(concat!("../../examples/", $example, "/model/build_unit.jsonl")),
            pins: include_str!(concat!("../../examples/", $example, "/model/pin.jsonl")),
            sites: include_str!(concat!("../../examples/", $example, "/model/site.jsonl")),
            anchors: include_str!(concat!("../../examples/", $example, "/model/anchors.jsonl")),
            files: example_files!($example: $($path),+),
        }
    };
}

const REPORT_JOBS: Reference = reference!("report-jobs":
    "pyproject.toml",
    "uv.lock",
    "src/app/api.py",
    "src/app/cli.py",
    "src/app/db.py",
    "src/app/models.py",
    "src/app/tasks.py",
);

const REPORT_JOBS_JAVA: Reference = reference!("report-jobs-java":
    "pom.xml",
    "src/main/java/app/Application.java",
    "src/main/java/app/Order.java",
    "src/main/java/app/OrderRepository.java",
    "src/main/java/app/Report.java",
    "src/main/java/app/ReportBeat.java",
    "src/main/java/app/ReportController.java",
    "src/main/java/app/ReportCtl.java",
    "src/main/java/app/ReportQueue.java",
    "src/main/java/app/ReportRepository.java",
    "src/main/java/app/ReportStatus.java",
    "src/main/java/app/ReportTasks.java",
);

const REFERENCES: [Reference; 2] = [REPORT_JOBS, REPORT_JOBS_JAVA];

#[test]
fn every_file_is_canonical() {
    for reference in REFERENCES {
        jsonl::read::<BuildUnit>(reference.units).unwrap();
        jsonl::read::<Pin>(reference.pins).unwrap();
        jsonl::read::<Site>(reference.sites).unwrap();
        jsonl::read::<Anchor>(reference.anchors).unwrap();
    }
}

#[test]
fn there_is_a_pin_for_each_locked_package_but_the_project() {
    let (_, lock) = REPORT_JOBS
        .files
        .iter()
        .find(|(name, _)| *name == "uv.lock")
        .expect("report-jobs' reference test reads its lock");
    let locked = lock.lines().filter(|line| *line == "[[package]]").count();
    assert_eq!(
        jsonl::read::<Pin>(REPORT_JOBS.pins).unwrap().len(),
        locked - 1
    );
}

#[test]
fn every_record_has_one_anchor() {
    for reference in REFERENCES {
        let units = jsonl::read::<BuildUnit>(reference.units)
            .unwrap()
            .into_iter()
            .map(|unit| unit.id);
        let pins = jsonl::read::<Pin>(reference.pins)
            .unwrap()
            .into_iter()
            .map(|pin| pin.id);
        let sites = jsonl::read::<Site>(reference.sites)
            .unwrap()
            .into_iter()
            .map(|site| site.id);
        let mut records: Vec<Id> = units.chain(pins).chain(sites).collect();
        records.sort();
        let anchored: Vec<Id> = jsonl::read::<Anchor>(reference.anchors)
            .unwrap()
            .into_iter()
            .map(|anchor| anchor.id)
            .collect();
        assert_eq!(anchored, records);
    }
}

#[test]
fn every_quote_stands_at_its_lines() {
    for reference in REFERENCES {
        for anchor in jsonl::read::<Anchor>(reference.anchors).unwrap() {
            let file = reference
                .files
                .iter()
                .find(|(name, _)| *name == anchor.path)
                .map_or_else(
                    || {
                        panic!(
                            "{} is not a file of the example this test knows",
                            anchor.path
                        )
                    },
                    |(_, text)| *text,
                );
            let [first, last]: [u32; 2] = anchor.lines.into();
            let lines: Vec<&str> = file
                .lines()
                .skip(first as usize - 1)
                .take((last - first + 1) as usize)
                .collect();
            assert_eq!(lines.join("\n"), anchor.quote, "{}", anchor.id);
        }
    }
}
