use super::*;

#[test]
fn a_patch_release_checks_as_its_minor_version() {
    assert_eq!(version("3.13.1").unwrap(), version("3.13").unwrap());
}

#[test]
fn a_version_that_is_not_one_is_an_error() {
    assert!(matches!(
        version("three"),
        Err(ResolveError::PythonVersion { .. })
    ));
}

#[test]
fn a_definition_on_no_search_path_is_not_found() {
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let project = format!("{crate_dir}/tests/fixtures");
    let unit = BuildUnit {
        id: mental_model::Id::parse("fx/unit:fx").unwrap(),
        dir: ".".to_owned(),
        roots: std::collections::BTreeSet::from([".".to_owned()]),
        python: Some(mental_model::Python {
            version: "3.12".to_owned(),
            from: mental_model::PythonFrom::PythonVersionFile,
        }),
        requires_python: None,
        lock: None,
        environment: Some(".venv".to_owned()),
        source: mental_model::Source::Extractor,
    };
    let environment = Environment {
        project_dir: &project,
        records: &[],
    };
    let resolver = Resolver::new(&unit, &[], &environment, &[]).unwrap();
    let outside = SystemPathBuf::from(format!("{crate_dir}/src/lib.rs"));
    let file = system_path_to_file(&resolver.db, &outside).unwrap();
    let (outcome, _) = resolver.target(file, TextRange::default());
    assert_eq!(
        outcome,
        Outcome::Unresolved {
            reason: Reason::NotFound
        }
    );
}
