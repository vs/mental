use super::*;

#[test]
fn a_kind_is_written_as_its_word_in_ids() {
    for kind in [
        SiteKind::Import,
        SiteKind::Decorator,
        SiteKind::Callee,
        SiteKind::BaseClass,
        SiteKind::WriteReceiver,
        SiteKind::Escape,
        SiteKind::HandlerType,
    ] {
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, format!("\"{}\"", kind.as_str()));
        assert_eq!(serde_json::from_str::<SiteKind>(&json).unwrap(), kind);
    }
}

#[test]
fn a_reason_is_written_in_snake_case() {
    let reasons = [
        Reason::MissingModule,
        Reason::NoType,
        Reason::NotFound,
        Reason::Ambiguous,
        Reason::Dynamic,
        Reason::Unreachable,
    ];
    let json = serde_json::to_string(&reasons).unwrap();
    assert_eq!(
        json,
        r#"["missing_module","no_type","not_found","ambiguous","dynamic","unreachable"]"#
    );
    assert_eq!(serde_json::from_str::<Vec<Reason>>(&json).unwrap(), reasons);
}
