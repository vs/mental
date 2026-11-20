//! The hand-written reference of report-jobs, `examples/report-jobs/model/`,
//! checked against report-jobs' own files. It is only ever read here.

use mental_model::{Anchor, BuildUnit, Id, Pin, Site, jsonl};

const UNITS: &str = include_str!("../../examples/report-jobs/model/build_unit.jsonl");
const PINS: &str = include_str!("../../examples/report-jobs/model/pin.jsonl");
const SITES: &str = include_str!("../../examples/report-jobs/model/site.jsonl");
const ANCHORS: &str = include_str!("../../examples/report-jobs/model/anchors.jsonl");
const PYPROJECT: &str = include_str!("../../examples/report-jobs/pyproject.toml");
const LOCK: &str = include_str!("../../examples/report-jobs/uv.lock");
const PYTHON: [(&str, &str); 5] = [
    (
        "src/app/api.py",
        include_str!("../../examples/report-jobs/src/app/api.py"),
    ),
    (
        "src/app/cli.py",
        include_str!("../../examples/report-jobs/src/app/cli.py"),
    ),
    (
        "src/app/db.py",
        include_str!("../../examples/report-jobs/src/app/db.py"),
    ),
    (
        "src/app/models.py",
        include_str!("../../examples/report-jobs/src/app/models.py"),
    ),
    (
        "src/app/tasks.py",
        include_str!("../../examples/report-jobs/src/app/tasks.py"),
    ),
];

#[test]
fn every_file_is_canonical() {
    jsonl::read::<BuildUnit>(UNITS).unwrap();
    jsonl::read::<Pin>(PINS).unwrap();
    jsonl::read::<Site>(SITES).unwrap();
    jsonl::read::<Anchor>(ANCHORS).unwrap();
}

#[test]
fn there_is_a_pin_for_each_locked_package_but_the_project() {
    let locked = LOCK.lines().filter(|line| *line == "[[package]]").count();
    assert_eq!(jsonl::read::<Pin>(PINS).unwrap().len(), locked - 1);
}

#[test]
fn every_record_has_one_anchor() {
    let units = jsonl::read::<BuildUnit>(UNITS)
        .unwrap()
        .into_iter()
        .map(|unit| unit.id);
    let pins = jsonl::read::<Pin>(PINS)
        .unwrap()
        .into_iter()
        .map(|pin| pin.id);
    let sites = jsonl::read::<Site>(SITES)
        .unwrap()
        .into_iter()
        .map(|site| site.id);
    let mut records: Vec<Id> = units.chain(pins).chain(sites).collect();
    records.sort();
    let anchored: Vec<Id> = jsonl::read::<Anchor>(ANCHORS)
        .unwrap()
        .into_iter()
        .map(|anchor| anchor.id)
        .collect();
    assert_eq!(anchored, records);
}

#[test]
fn every_quote_stands_at_its_lines() {
    for anchor in jsonl::read::<Anchor>(ANCHORS).unwrap() {
        let file = match anchor.path.as_str() {
            "pyproject.toml" => PYPROJECT,
            "uv.lock" => LOCK,
            path => PYTHON.iter().find(|(name, _)| *name == path).map_or_else(
                || panic!("{path} is not a file of report-jobs this test knows"),
                |(_, text)| *text,
            ),
        };
        let [first, last]: [u32; 2] = anchor.lines.into();
        let lines: Vec<&str> = file
            .lines()
            .skip(first as usize - 1)
            .take((last - first + 1) as usize)
            .collect();
        assert_eq!(lines.join("\n"), anchor.quote, "{}", anchor.id);
    }
}
