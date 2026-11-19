//! The JSONL format, frozen: a small model with every record kind and every
//! form of each field. A change here is a change of the format.

use std::collections::BTreeSet;

use mental_model::{
    Anchor, BuildUnit, Id, Lines, Lock, Origin, Pin, Python, PythonFrom, Site, SiteKind, Source,
    jsonl,
};

fn id(text: &str) -> Id {
    Id::parse(text).expect("the test writes valid IDs")
}

fn units() -> Vec<BuildUnit> {
    vec![
        BuildUnit {
            id: id("shop/unit:shop"),
            dir: ".".to_owned(),
            roots: BTreeSet::from(["src".to_owned()]),
            python: Some(Python {
                version: "3.12".to_owned(),
                from: PythonFrom::PythonVersionFile,
            }),
            requires_python: Some(">=3.12".to_owned()),
            lock: Some(Lock {
                path: "uv.lock".to_owned(),
                version: 1,
            }),
            environment: Some(".venv".to_owned()),
            source: Source::Extractor,
        },
        BuildUnit {
            id: id("shop/unit:admin"),
            dir: "admin".to_owned(),
            roots: BTreeSet::from([".".to_owned()]),
            python: Some(Python {
                version: "3.13.1".to_owned(),
                from: PythonFrom::Pyvenv,
            }),
            requires_python: None,
            lock: None,
            environment: None,
            source: Source::Extractor,
        },
    ]
}

fn pins() -> Vec<Pin> {
    let pin = |name: &str, origin| Pin {
        id: id(&format!("shop/unit:shop/pin:{name}")),
        package: name.to_owned(),
        version: "1.0.0".to_owned(),
        origin,
        source: Source::Extractor,
    };
    vec![
        pin(
            "requests",
            Origin::Registry("https://pypi.org/simple".to_owned()),
        ),
        pin(
            "flask",
            Origin::Git {
                url: "https://github.com/pallets/flask".to_owned(),
                commit: "2f0c62f5e6e290843f03c1fa70817c7a3c7fd661".to_owned(),
            },
        ),
        pin("tools", Origin::Path("../tools".to_owned())),
        pin(
            "wheel",
            Origin::Url("https://example.com/wheel-1.0.0-py3-none-any.whl".to_owned()),
        ),
    ]
}

fn sites() -> Vec<Site> {
    let site = |slug: &str, kind| Site {
        id: id(&format!("shop/unit:shop/site:{slug}")),
        kind,
        source: Source::Extractor,
    };
    vec![
        site("shop-import-shop-db-session", SiteKind::Import),
        site("shop-api-create-decorator-app-post", SiteKind::Decorator),
        site("shop-api-create-callee-s-commit-0", SiteKind::Callee),
        site("shop-models-order-base-class-base", SiteKind::BaseClass),
        site(
            "shop-api-create-write-receiver-order-status",
            SiteKind::WriteReceiver,
        ),
        site(
            "shop-api-create-handler-type-timeout",
            SiteKind::HandlerType,
        ),
    ]
}

fn anchors() -> Vec<Anchor> {
    vec![Anchor {
        id: id("shop/unit:shop/pin:requests"),
        path: "uv.lock".to_owned(),
        lines: Lines::new(12, 13).expect("12 to 13 is a range of lines"),
        quote: "name = \"requests\"\nversion = \"1.0.0\"".to_owned(),
    }]
}

#[test]
fn build_units() {
    let text = jsonl::write(&units());
    assert_eq!(jsonl::read::<BuildUnit>(&text).unwrap().len(), 2);
    insta::assert_snapshot!(text, @r#"
    {"id":"shop/unit:admin","dir":"admin","roots":["."],"python":{"version":"3.13.1","from":"pyvenv.cfg"},"source":"extractor"}
    {"id":"shop/unit:shop","dir":".","roots":["src"],"python":{"version":"3.12","from":".python-version"},"requires_python":">=3.12","lock":{"path":"uv.lock","version":1},"environment":".venv","source":"extractor"}
    "#);
}

#[test]
fn pins_with_every_origin() {
    let text = jsonl::write(&pins());
    assert_eq!(jsonl::read::<Pin>(&text).unwrap().len(), 4);
    insta::assert_snapshot!(text, @r#"
    {"id":"shop/unit:shop/pin:flask","package":"flask","version":"1.0.0","origin":{"git":{"url":"https://github.com/pallets/flask","commit":"2f0c62f5e6e290843f03c1fa70817c7a3c7fd661"}},"source":"extractor"}
    {"id":"shop/unit:shop/pin:requests","package":"requests","version":"1.0.0","origin":{"registry":"https://pypi.org/simple"},"source":"extractor"}
    {"id":"shop/unit:shop/pin:tools","package":"tools","version":"1.0.0","origin":{"path":"../tools"},"source":"extractor"}
    {"id":"shop/unit:shop/pin:wheel","package":"wheel","version":"1.0.0","origin":{"url":"https://example.com/wheel-1.0.0-py3-none-any.whl"},"source":"extractor"}
    "#);
}

#[test]
fn sites_of_every_kind() {
    let text = jsonl::write(&sites());
    assert_eq!(jsonl::read::<Site>(&text).unwrap().len(), 6);
    insta::assert_snapshot!(text, @r#"
    {"id":"shop/unit:shop/site:shop-api-create-callee-s-commit-0","kind":"callee","source":"extractor"}
    {"id":"shop/unit:shop/site:shop-api-create-decorator-app-post","kind":"decorator","source":"extractor"}
    {"id":"shop/unit:shop/site:shop-api-create-handler-type-timeout","kind":"handler-type","source":"extractor"}
    {"id":"shop/unit:shop/site:shop-api-create-write-receiver-order-status","kind":"write-receiver","source":"extractor"}
    {"id":"shop/unit:shop/site:shop-import-shop-db-session","kind":"import","source":"extractor"}
    {"id":"shop/unit:shop/site:shop-models-order-base-class-base","kind":"base-class","source":"extractor"}
    "#);
}

#[test]
fn anchors_of_records() {
    let text = jsonl::write(&anchors());
    assert_eq!(jsonl::read::<Anchor>(&text).unwrap().len(), 1);
    insta::assert_snapshot!(text, @r#"{"id":"shop/unit:shop/pin:requests","path":"uv.lock","lines":[12,13],"quote":"name = \"requests\"\nversion = \"1.0.0\""}"#);
}
