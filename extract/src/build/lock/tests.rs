use mental_model::jsonl;

use super::*;

/// Reads `text` as the `uv.lock` of the project `shop`: the pins, their
/// anchors and the diagnostics, one JSON line each.
#[track_caller]
fn check(text: &str) -> String {
    let unit = Id::parse("shop/unit:shop").unwrap();
    let project = slugify("shop").unwrap();
    let (locked, diagnostics) = read(text, &unit, &project);
    let lock = locked.map(|locked| {
        let Lock { path, version } = locked.lock;
        let (pins, anchors) = (jsonl::write(&locked.pins), jsonl::write(&locked.anchors));
        format!("lock {path} {version}\n{pins}{anchors}")
    });
    lock.unwrap_or_default() + &jsonl::write(&diagnostics)
}

/// A lockfile of the project `shop` and one more package with `source`.
fn with_source(source: &str) -> String {
    format!(
        "version = 1\n\n[[package]]\nname = \"a\"\nversion = \"1.0\"\nsource = {source}\n\n\
         [[package]]\nname = \"shop\"\nversion = \"0.1.0\"\nsource = {{ virtual = \".\" }}\n"
    )
}

/// The pin `check` gives for `source`, without its anchor.
#[track_caller]
fn origin_of(source: &str) -> String {
    check(&with_source(source))
        .lines()
        .nth(1)
        .unwrap()
        .to_owned()
}

#[test]
fn each_package_but_the_project_is_pinned_and_anchored() {
    assert_eq!(
        check(&with_source(r#"{ registry = "https://pypi.org/simple" }"#)),
        r#"lock uv.lock 1
{"id":"shop/unit:shop/pin:a","package":"a","version":"1.0","origin":{"registry":"https://pypi.org/simple"},"source":"extractor"}
{"id":"shop/unit:shop/pin:a","path":"uv.lock","lines":[4,5],"quote":"name = \"a\"\nversion = \"1.0\""}
"#
    );
}

#[test]
fn every_source_form_has_an_origin() {
    let origins = [
        r#"{ git = "https://github.com/o/a?rev=main#2f0c62f5e6e290843f03c1fa70817c7a3c7fd661" }"#,
        r#"{ url = "https://example.com/a-1.0-py3-none-any.whl" }"#,
        r#"{ path = "../wheels/a-1.0.tar.gz" }"#,
        r#"{ directory = "../a" }"#,
        r#"{ editable = "packages/a" }"#,
        r#"{ virtual = "packages/a" }"#,
    ]
    .map(|source| {
        origin_of(source)
            .split(r#""origin":"#)
            .nth(1)
            .unwrap()
            .to_owned()
    });
    assert_eq!(
        origins.join("\n"),
        r#"{"git":{"url":"https://github.com/o/a?rev=main","commit":"2f0c62f5e6e290843f03c1fa70817c7a3c7fd661"}},"source":"extractor"}
{"url":"https://example.com/a-1.0-py3-none-any.whl"},"source":"extractor"}
{"path":"../wheels/a-1.0.tar.gz"},"source":"extractor"}
{"path":"../a"},"source":"extractor"}
{"path":"packages/a"},"source":"extractor"}
{"path":"packages/a"},"source":"extractor"}"#
    );
}

#[test]
fn a_source_has_exactly_one_form() {
    let unread = |source: &str| {
        let out = check(&with_source(source));
        out.contains("has a source this build does not read")
            .to_string()
    };
    let verdicts = [
        r#"{ registry = "https://pypi.org/simple", path = "../a" }"#,
        "{}",
        r#"{ git = "https://github.com/o/a" }"#,
        r#"{ git = "https://github.com/o/a#" }"#,
    ]
    .map(unread);
    assert_eq!(verdicts.join(" "), "true true true true");
}

#[test]
fn an_unread_source_is_located() {
    assert_eq!(
        check(&with_source("{}")),
        r#"lock uv.lock 1
{"code":"lock-invalid","at":{"path":"uv.lock","lines":[6,6]},"message":"package `a` has a source this build does not read: it reads one of `registry`, `git` with a commit, `url`, `path`, `directory`, `editable` and `virtual`; run `uv lock` to write it again"}
"#
    );
}

#[test]
fn an_unknown_source_key_does_not_read() {
    assert_eq!(
        check(&with_source(
            r#"{ url = "https://example.com/a.tar.gz", subdirectory = "a" }"#
        )),
        r#"{"code":"lock-invalid","at":{"path":"uv.lock","lines":[6,6]},"message":"`uv.lock` does not read: unknown field `subdirectory`, expected one of `registry`, `git`, `url`, `path`, `directory`, `editable`, `virtual`; run `uv lock` to write it again"}
"#
    );
}

#[test]
fn a_package_locked_twice_is_ambiguous() {
    let text = "version = 1\n\n\
                [[package]]\nname = \"numpy\"\nversion = \"1.26.4\"\nsource = { registry = \"https://pypi.org/simple\" }\n\n\
                [[package]]\nname = \"numpy\"\nversion = \"2.1.0\"\nsource = { registry = \"https://pypi.org/simple\" }\n\n\
                [[package]]\nname = \"Requests\"\nversion = \"2.32.3\"\nsource = { registry = \"https://pypi.org/simple\" }\n";
    assert_eq!(
        check(text),
        r#"lock uv.lock 1
{"id":"shop/unit:shop/pin:requests","package":"requests","version":"2.32.3","origin":{"registry":"https://pypi.org/simple"},"source":"extractor"}
{"id":"shop/unit:shop/pin:requests","path":"uv.lock","lines":[14,15],"quote":"name = \"Requests\"\nversion = \"2.32.3\""}
{"code":"lock-ambiguous","at":{"path":"uv.lock","lines":[4,9]},"message":"`uv.lock` locks `numpy` 2 times, such as once per platform, and a pin has one version; let uv lock one version for every platform, for example with a tighter bound on it or with `environments` under `[tool.uv]`"}
"#
    );
}

#[test]
fn names_that_share_a_slug_are_one_package() {
    let text = "version = 1\n\n\
                [[package]]\nname = \"zope.interface\"\nversion = \"7.2\"\nsource = { registry = \"https://pypi.org/simple\" }\n\n\
                [[package]]\nname = \"zope-interface\"\nversion = \"7.2\"\nsource = { registry = \"https://pypi.org/simple\" }\n";
    assert!(check(text).contains("locks `zope-interface` 2 times"));
}

#[test]
fn a_package_with_no_fixed_version_has_no_pin() {
    assert_eq!(
        check(
            "version = 1\n\n[[package]]\nname = \"tools\"\nsource = { editable = \"../tools\" }\n"
        ),
        r#"lock uv.lock 1
{"code":"lock-invalid","at":{"path":"uv.lock","lines":[4,4]},"message":"package `tools` has no fixed version, as when its version is dynamic, and a pin has one; give it a static `version` in its own `pyproject.toml`"}
"#
    );
}

#[test]
fn the_project_needs_no_fixed_version() {
    assert_eq!(
        check("version = 1\n\n[[package]]\nname = \"shop\"\nsource = { editable = \".\" }\n"),
        "lock uv.lock 1\n"
    );
}

#[test]
fn a_package_name_with_no_letter_has_no_pin() {
    assert_eq!(
        check(
            &with_source(r#"{ registry = "https://pypi.org/simple" }"#)
                .replace("\"a\"", "\"2048\"")
        ),
        r#"lock uv.lock 1
{"code":"lock-invalid","at":{"path":"uv.lock","lines":[4,4]},"message":"package `2048` has no ASCII letter in its name, so it can have no ID"}
"#
    );
}

#[test]
fn the_anchor_spans_name_and_version_in_either_order() {
    let text = "version = 1\n\n[[package]]\nversion = \"1.0\"\nsource = { registry = \"https://pypi.org/simple\" }\nname = \"a\"\n";
    assert!(
        check(text).contains(r#""lines":[4,6],"quote":"version = \"1.0\"\nsource = { registry = \"https://pypi.org/simple\" }\nname = \"a\"""#),
        "{}",
        check(text)
    );
}

#[test]
fn a_lock_with_no_packages_has_no_pins() {
    assert_eq!(check("version = 1\nrevision = 3\n"), "lock uv.lock 1\n");
}

#[test]
fn only_schema_version_1_is_read() {
    assert_eq!(
        check("version = 2\n\n[[package]]\nnot = \"a package of version 1\"\n"),
        r#"{"code":"lock-version-unknown","at":{"path":"uv.lock","lines":[1,1]},"message":"`uv.lock` has schema version 2, and this build reads only version 1; lock with a uv that writes version 1"}
"#
    );
}

#[test]
fn a_file_that_does_not_read_is_invalid() {
    let unreadable = [
        "version = 1\n[[package]\n",
        "revision = 3\n",
        "version = \"1\"\n",
        "version = 1\n\n[[package]]\nname = \"a\"\nversion = \"1.0\"\n",
    ]
    .map(check);
    assert_eq!(
        unreadable.join(""),
        r#"{"code":"lock-invalid","at":{"path":"uv.lock","lines":[2,2]},"message":"`uv.lock` does not read: unclosed array table, expected `]`; run `uv lock` to write it again"}
{"code":"lock-invalid","at":{"path":"uv.lock","lines":[1,1]},"message":"`uv.lock` does not read: missing field `version`; run `uv lock` to write it again"}
{"code":"lock-invalid","at":{"path":"uv.lock","lines":[1,1]},"message":"`uv.lock` does not read: invalid type: string \"1\", expected u32; run `uv lock` to write it again"}
{"code":"lock-invalid","at":{"path":"uv.lock","lines":[3,3]},"message":"`uv.lock` does not read: missing field `source`; run `uv lock` to write it again"}
"#
    );
}
