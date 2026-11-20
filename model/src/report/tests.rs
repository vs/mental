use super::*;
use crate::{Checker, Id, Package, Source};

fn site(slug: &str, kind: SiteKind, outcome: Outcome) -> Site {
    Site {
        id: Id::parse(&format!("shop/unit:shop/site:{slug}")).unwrap(),
        kind,
        outcome,
        checker: Checker {
            name: "ty".to_owned(),
            version: "0.0.105".to_owned(),
        },
        source: Source::Extractor,
    }
}

#[test]
fn sites_are_counted_by_kind_and_reason() {
    let resolved = Outcome::Resolved {
        module: "shop".to_owned(),
        name: "f".to_owned(),
    };
    let external = Outcome::External {
        package: Package::Stdlib,
        module: "os".to_owned(),
        name: String::new(),
    };
    let no_type = Outcome::Unresolved {
        reason: Reason::NoType,
    };
    let report = Resolution::of(&[
        site("shop-callee-f", SiteKind::Callee, resolved),
        site("shop-callee-g", SiteKind::Callee, no_type.clone()),
        site("shop-callee-h", SiteKind::Callee, no_type),
        site("shop-import-os", SiteKind::Import, external),
    ]);
    assert_eq!(report.all.unresolved_total(), 2);
    assert_eq!(
        report.to_json(),
        r#"{"all":{"resolved":1,"external":1,"unresolved":{"no_type":2}},"kinds":{"import":{"resolved":0,"external":1},"callee":{"resolved":1,"external":0,"unresolved":{"no_type":2}}}}
"#
    );
}

#[test]
fn no_sites_make_an_empty_report() {
    assert_eq!(
        Resolution::of(&[]).to_json(),
        "{\"all\":{\"resolved\":0,\"external\":0},\"kinds\":{}}\n"
    );
}
