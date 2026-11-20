use super::*;

use mental_model::{Origin, Source};

fn pin(unit: &Id, name: &str) -> Pin {
    Pin {
        id: unit.child(Kind::Pin, &slugify(name).unwrap()),
        package: name.to_owned(),
        version: "1.0".to_owned(),
        origin: Origin::Registry("https://pypi.org/simple".to_owned()),
        source: Source::Extractor,
    }
}

/// The pin, if any, of each path, given one `RECORD` of `dist_info`.
#[track_caller]
fn check(dist_info: &str, record: &str, paths: &[&str]) -> Vec<Option<String>> {
    let unit = Id::parse("shop/unit:shop").unwrap();
    let pins = [pin(&unit, "annotated-doc"), pin(&unit, "greet")];
    let packages = Packages::new(
        &unit,
        &pins,
        &[DistRecord {
            dist_info,
            text: record,
        }],
    );
    paths
        .iter()
        .map(|path| packages.pin(path).map(ToString::to_string))
        .collect()
}

#[test]
fn a_file_a_record_lists_belongs_to_its_pin() {
    assert_eq!(
        check(
            "annotated_doc-0.0.7.dist-info",
            "annotated_doc/__init__.py,sha256=x,1\n",
            &["annotated_doc/__init__.py", "annotated_doc/other.py"],
        ),
        [Some("shop/unit:shop/pin:annotated-doc".to_owned()), None]
    );
}

#[test]
fn a_quoted_path_is_read_with_its_commas_and_quotes() {
    assert_eq!(
        check(
            "greet-1.0.dist-info",
            "\"greet/a,b.py\",sha256=x,1\n\"greet/say \"\"hi\"\".py\",,\n\"greet/open.py\n",
            &["greet/a,b.py", "greet/say \"hi\".py", "greet/open.py"],
        ),
        [
            Some("shop/unit:shop/pin:greet".to_owned()),
            Some("shop/unit:shop/pin:greet".to_owned()),
            None
        ]
    );
}

#[test]
fn files_outside_site_packages_and_empty_lines_are_skipped() {
    assert_eq!(
        check(
            "greet-1.0.dist-info",
            "../../bin/greet,sha256=x,1\n\n,sha256=y,1\n",
            &["../../bin/greet", ""],
        ),
        [None, None]
    );
}

#[test]
fn a_distribution_the_unit_does_not_pin_owns_nothing() {
    assert_eq!(
        check(
            "stray-2.0.dist-info",
            "stray/__init__.py,,\n",
            &["stray/__init__.py"]
        ),
        [None]
    );
    assert_eq!(
        check(
            "not-a-dist-info",
            "greet/__init__.py,,\n",
            &["greet/__init__.py"]
        ),
        [None]
    );
    assert_eq!(
        check(
            "noversion.dist-info",
            "greet/__init__.py,,\n",
            &["greet/__init__.py"]
        ),
        [None]
    );
    assert_eq!(
        check(
            "1-2.dist-info",
            "greet/__init__.py,,\n",
            &["greet/__init__.py"]
        ),
        [None]
    );
}
