use super::*;

/// The sites of `source` in module `m`, one per line: kind, scope, name.
#[track_caller]
fn check(source: &str) -> String {
    let parsed =
        ruff_python_parser::parse_unchecked_source(source, ruff_python_ast::PySourceType::Python);
    assert_eq!(parsed.errors(), []);
    sites("m", &parsed.syntax().body)
        .iter()
        .map(|site| format!("{} {} {}", site.kind.as_str(), site.scope, site.name))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_name_keeps_calls_and_subscripts_as_marks() {
    assert_eq!(
        check("a.b().c[0].d()\n"),
        "callee m a.b().c[].d\ncallee m a.b"
    );
}

#[test]
fn a_name_drops_what_is_not_a_name() {
    assert_eq!(
        check("', '.join(x)\n(f or g)()\n"),
        "callee m .join\ncallee m "
    );
}

#[test]
fn an_imported_name_follows_its_module() {
    assert_eq!(
        check("from . import x\nfrom ..p import y\nimport a.b\n"),
        "import m .x\nimport m p.y\nimport m a.b"
    );
}

#[test]
fn a_header_belongs_to_its_def_or_class() {
    assert_eq!(
        check("@d(e())\ndef f(x=g()):\n    pass\nclass C(B, k=h()):\n    pass\n"),
        "decorator m.f d\ncallee m.f e\ncallee m.f g\nbase-class m.C B\ncallee m.C h"
    );
}
