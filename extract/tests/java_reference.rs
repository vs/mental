//! The hand-written reference of report-jobs-java against javac and Maven:
//! its sites and their anchors are what `tests/javac/ReferenceSites.java`
//! gets from javac's own trees and elements, and its pins are what Maven
//! resolves, each anchored to the dependency it comes through. They need a
//! JDK 25 and Maven's cache filled by `mvn dependency:go-offline`, so they run
//! when `RUN_SLOW_TESTS` is set (CONTRIBUTING.md, Exceptions).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mental_model::{Anchor, BuildUnit, Origin, Pin, Site, jsonl};

const REGISTRY: &str = "https://repo.maven.apache.org/maven2";
const OFFLINE: &str =
    "run `mvn dependency:go-offline` in examples/report-jobs-java, with network, first";

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/report-jobs-java")
        .canonicalize()
        .expect("report-jobs-java is in examples/")
}

fn reference(name: &str) -> String {
    fs::read_to_string(example().join("model").join(name)).expect("the reference reads")
}

/// A folder of its own for `test`'s outputs, when the slow tests run.
fn scratch(test: &str) -> Option<PathBuf> {
    std::env::var_os("RUN_SLOW_TESTS")?;
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("the last run's outputs can be removed");
    }
    fs::create_dir_all(&dir).expect("the scratch folder can be made");
    Some(dir)
}

fn run(command: &mut Command, fix: &str) {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{command:?} does not start ({error}): {fix}"));
    assert!(
        output.status.success(),
        "{command:?} failed: {fix}\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Maven's output file of `goal`, run offline in the example.
fn maven(dir: &Path, goal: &str, flags: &[&str]) -> PathBuf {
    let out = dir.join(format!("{}.txt", goal.replace(':', "-")));
    let mut command = Command::new("mvn");
    command
        .args(["-q", "-B", "-o", goal])
        .args(flags)
        .arg(format!("-DoutputFile={}", out.display()))
        .current_dir(example());
    run(&mut command, OFFLINE);
    out
}

fn unit() -> BuildUnit {
    let mut units = jsonl::read::<BuildUnit>(&reference("build_unit.jsonl"))
        .expect("the unit reference is canonical");
    assert_eq!(units.len(), 1, "report-jobs-java is one build unit");
    units.remove(0)
}

#[test]
fn every_site_has_the_outcome_and_anchor_javac_gives_it() {
    let Some(dir) = scratch("java-reference-sites") else {
        return;
    };
    let list = maven(
        &dir,
        "dependency:list",
        &["-DoutputAbsoluteArtifactFilename=true"],
    );
    let out = dir.join("javac");
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/javac/ReferenceSites.java");
    run(
        Command::new("java")
            .arg(tool)
            .arg(example())
            .arg(unit().id.as_str())
            .arg(&list)
            .arg(&out),
        "put a JDK 25 first on PATH",
    );
    let written = |name: &str| fs::read_to_string(out.join(name)).expect("the tool wrote it");
    let sites =
        jsonl::read::<Site>(&written("site.jsonl")).expect("the tool writes canonical sites");
    let expected = jsonl::read::<Site>(&reference("site.jsonl")).unwrap();
    assert_eq!(
        sites.first().map(|site| &site.checker),
        expected.first().map(|site| &site.checker),
        "the reference is of the javac named here: put that JDK first on PATH"
    );
    assert_eq!(jsonl::write(&sites), jsonl::write(&expected));
    let anchors: Vec<Anchor> = jsonl::read::<Anchor>(&reference("anchors.jsonl"))
        .unwrap()
        .into_iter()
        .filter(|anchor| anchor.id.as_str().contains("/site:"))
        .collect();
    assert_eq!(written("anchors.jsonl"), jsonl::write(&anchors));
}

/// Maven's tree as `group:artifact` → the direct dependency it comes through.
fn direct_dependencies(tree: &str) -> BTreeMap<String, String> {
    let mut through = BTreeMap::new();
    let mut direct = String::new();
    for line in tree.lines().skip(1) {
        let entry = line.trim_start_matches(['|', '+', '-', '\\', ' ']);
        let depth = (line.len() - entry.len()) / 3;
        let mut parts = entry.split(':');
        let (group, artifact) = (
            parts.next().expect("a tree line names a group"),
            parts.next().expect("a tree line names an artifact"),
        );
        if depth == 1 {
            artifact.clone_into(&mut direct);
        }
        through.insert(format!("{group}:{artifact}"), direct.clone());
    }
    through
}

#[test]
fn every_resolved_artifact_is_a_pin_anchored_to_its_dependency() {
    let Some(dir) = scratch("java-reference-pins") else {
        return;
    };
    let list = fs::read_to_string(maven(&dir, "dependency:list", &[])).unwrap();
    let resolved: BTreeMap<String, String> = list
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.trim().split(" -- ").next()?.split(':').collect();
            (parts.len() >= 5).then(|| {
                let version = parts[parts.len() - 2];
                (format!("{}:{}", parts[0], parts[1]), version.to_owned())
            })
        })
        .collect();
    let pins = jsonl::read::<Pin>(&reference("pin.jsonl")).unwrap();
    let pinned: BTreeMap<String, String> = pins
        .iter()
        .map(|pin| (pin.package.clone(), pin.version.clone()))
        .collect();
    assert_eq!(pinned, resolved);
    assert!(
        pins.iter()
            .all(|pin| pin.origin == Origin::Registry(REGISTRY.to_owned()))
    );

    let through =
        direct_dependencies(&fs::read_to_string(maven(&dir, "dependency:tree", &[])).unwrap());
    let anchors: BTreeMap<String, String> = jsonl::read::<Anchor>(&reference("anchors.jsonl"))
        .unwrap()
        .into_iter()
        .map(|anchor| (anchor.id.to_string(), anchor.quote))
        .collect();
    for pin in &pins {
        let direct = &through[&pin.package];
        assert_eq!(
            anchors[pin.id.as_str()],
            format!("      <artifactId>{direct}</artifactId>"),
            "{}",
            pin.id
        );
    }
}
