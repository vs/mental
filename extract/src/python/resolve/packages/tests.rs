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

/// The pin of each path, given the `RECORD` of each `dist_info`; the unit
/// pins the packages its tests install.
#[track_caller]
fn check_all(records: &[(&str, &str)], paths: &[&str]) -> Vec<Option<String>> {
    let unit = Id::parse("shop/unit:shop").unwrap();
    let pins = [
        "redis",
        "types-redis",
        "redis-stubs",
        "types-six",
        "django",
        "django-stubs",
        "pycurl",
        "types-pycurl",
        "fastx",
        "types-fastx",
    ]
    .map(|name| pin(&unit, name));
    let records: Vec<DistRecord> = records
        .iter()
        .map(|&(dist_info, text)| DistRecord { dist_info, text })
        .collect();
    let packages = Packages::new(&unit, &pins, &records);
    paths
        .iter()
        .map(|path| {
            packages
                .pin(path)
                .map(|pin| pin.as_str().rsplit(':').next().unwrap().to_owned())
        })
        .collect()
}

const REDIS: (&str, &str) = (
    "redis-5.0.dist-info",
    "redis/__init__.py,,\nredis/client.py,,\nredis/commands/__init__.py,,\nredis/_speedups.cpython-312-darwin.so,,\n",
);

#[test]
fn a_stub_file_belongs_to_the_pin_of_the_module_it_describes() {
    let types_redis = (
        "types_redis-4.6.dist-info",
        "redis/__init__.pyi,,\nredis/client.pyi,,\nredis/_speedups.pyi,,\nredis/commands/core.pyi,,\nredis/py.typed,,\n",
    );
    assert_eq!(
        check_all(
            &[REDIS, types_redis],
            &[
                "redis/client.pyi",
                "redis/__init__.pyi",
                "redis/_speedups.pyi",
                "redis/commands/core.pyi"
            ],
        ),
        ["redis", "redis", "redis", "redis"].map(|pin| Some(pin.to_owned()))
    );
}

#[test]
fn a_stubs_folder_describes_the_package_without_its_suffix() {
    let stubs = ("redis_stubs-1.0.dist-info", "redis-stubs/client.pyi,,\n");
    assert_eq!(
        check_all(&[REDIS, stubs], &["redis-stubs/client.pyi"]),
        [Some("redis".to_owned())]
    );
}

#[test]
fn a_stub_file_keeps_its_pin_when_no_pin_installs_its_module() {
    let stubs = ("types_six-1.0.dist-info", "six.pyi,,\n");
    assert_eq!(
        check_all(&[REDIS, stubs], &["six.pyi"]),
        [Some("types-six".to_owned())]
    );
}

#[test]
fn a_package_with_its_own_stubs_keeps_them() {
    let inline = (
        "redis-5.0.dist-info",
        "redis/__init__.py,,\nredis/client.py,,\nredis/client.pyi,,\n",
    );
    assert_eq!(
        check_all(&[inline], &["redis/client.pyi"]),
        [Some("redis".to_owned())]
    );
}

#[test]
fn a_stubs_folder_beside_code_still_describes_its_package() {
    let django = (
        "django-5.0.dist-info",
        "django/__init__.py,,\ndjango/db/models/__init__.py,,\n",
    );
    let stubs = (
        "django_stubs-5.0.dist-info",
        "django-stubs/db/models/__init__.pyi,,\nmypy_django_plugin/main.py,,\n",
    );
    assert_eq!(
        check_all(
            &[django, stubs],
            &[
                "django-stubs/db/models/__init__.pyi",
                "mypy_django_plugin/main.py"
            ]
        ),
        [Some("django".to_owned()), Some("django-stubs".to_owned())]
    );
}

#[test]
fn a_stub_of_a_compiled_module_belongs_to_the_pin_that_installs_it() {
    let pycurl = ("pycurl-7.45.dist-info", "pycurl.cpython-312-darwin.so,,\n");
    let stubs = ("types_pycurl-7.45.dist-info", "pycurl.pyi,,\n");
    let fastx = ("fastx-1.0.dist-info", "fastx.cp312-win_amd64.pyd,,\n");
    let fastx_stubs = ("types_fastx-1.0.dist-info", "fastx.pyi,,\n");
    assert_eq!(
        check_all(
            &[pycurl, stubs, fastx, fastx_stubs],
            &["pycurl.pyi", "fastx.pyi"]
        ),
        [Some("pycurl".to_owned()), Some("fastx".to_owned())]
    );
}
