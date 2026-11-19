use super::*;

/// Writes `diagnostic` and reads it back, and gives its severity and line.
#[track_caller]
fn check(diagnostic: &Diagnostic) -> String {
    let json = serde_json::to_string(diagnostic).unwrap();
    assert_eq!(
        &serde_json::from_str::<Diagnostic>(&json).unwrap(),
        diagnostic
    );
    format!("{:?} {json}", diagnostic.code.severity())
}

fn about(code: Code) -> Diagnostic {
    Diagnostic {
        id: None,
        code,
        at: None,
        message: "m".to_owned(),
    }
}

#[test]
fn every_code_has_a_stable_name_and_a_severity() {
    let lines: Vec<String> = Code::ALL
        .into_iter()
        .map(|code| check(&about(code)))
        .collect();
    assert_eq!(
        lines.join("\n"),
        r#"Error {"code":"project-missing","message":"m"}
Error {"code":"lock-missing","message":"m"}
Error {"code":"lock-version-unknown","message":"m"}
Error {"code":"python-version-unknown","message":"m"}
Error {"code":"environment-missing","message":"m"}
Error {"code":"module-missing","message":"m"}
Warning {"code":"syntax-error","message":"m"}"#
    );
}

#[test]
fn a_diagnostic_names_its_record_and_place() {
    let diagnostic = Diagnostic {
        id: Some(Id::parse("reports/unit:reports").unwrap()),
        at: Some(Location {
            path: "uv.lock".to_owned(),
            lines: Lines::new(1, 1).unwrap(),
        }),
        ..about(Code::LockVersionUnknown)
    };
    assert_eq!(
        check(&diagnostic),
        r#"Error {"id":"reports/unit:reports","code":"lock-version-unknown","at":{"path":"uv.lock","lines":[1,1]},"message":"m"}"#
    );
}
