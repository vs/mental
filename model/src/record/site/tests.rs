use super::*;

#[test]
fn a_kind_is_written_as_its_word_in_ids() {
    for kind in [
        SiteKind::Import,
        SiteKind::Decorator,
        SiteKind::Callee,
        SiteKind::BaseClass,
        SiteKind::WriteReceiver,
        SiteKind::HandlerType,
    ] {
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, format!("\"{}\"", kind.as_str()));
        assert_eq!(serde_json::from_str::<SiteKind>(&json).unwrap(), kind);
    }
}
