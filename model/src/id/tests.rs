use proptest::prelude::*;

use super::*;

/// Parses `text` both directly and through serde, which must agree, and
/// returns `ok` or the error message.
#[track_caller]
fn check(text: &str) -> String {
    let json = serde_json::to_string(text).unwrap();
    match (Id::parse(text), serde_json::from_str::<Id>(&json)) {
        (Ok(id), Ok(through_serde)) => {
            assert_eq!(id, through_serde);
            assert_eq!(id.to_string(), text);
            assert_eq!(serde_json::to_string(&id).unwrap(), json);
            "ok".to_owned()
        }
        (Err(error), Err(through_serde)) => {
            assert_eq!(through_serde.to_string(), error.to_string());
            error.to_string()
        }
        (direct, through_serde) => panic!("parse gave {direct:?}, serde gave {through_serde:?}"),
    }
}

/// The slug of `code_name`, or `none`.
#[track_caller]
fn slug(code_name: &str) -> String {
    slugify(code_name).map_or("none".to_owned(), |slug| slug.as_str().to_owned())
}

#[test]
fn a_system_name_alone_is_an_id() {
    assert_eq!(check("reports"), "ok");
}

#[test]
fn every_kind_follows_a_parent() {
    assert_eq!(check("reports/unit:reports"), "ok");
    assert_eq!(check("reports/unit:reports/pin:sqlalchemy"), "ok");
    assert_eq!(
        check("reports/unit:reports/site:app-tasks-generate-report-callee-s-commit-0"),
        "ok"
    );
}

#[test]
fn a_slug_may_start_with_a_digit() {
    assert_eq!(check("reports/pin:2fa"), "ok");
}

#[test]
fn an_empty_text_is_not_an_id() {
    assert_eq!(
        check(""),
        "invalid ID `` at byte 0: a segment is empty; look for a doubled or trailing `/`"
    );
}

#[test]
fn a_doubled_slash_leaves_an_empty_segment() {
    assert_eq!(
        check("reports//unit:reports"),
        "invalid ID `reports//unit:reports` at byte 8: a segment is empty; look for a doubled or trailing `/`"
    );
}

#[test]
fn a_trailing_slash_leaves_an_empty_segment() {
    assert_eq!(
        check("reports/"),
        "invalid ID `reports/` at byte 8: a segment is empty; look for a doubled or trailing `/`"
    );
}

#[test]
fn the_root_takes_no_kind() {
    assert_eq!(
        check("unit:reports"),
        "invalid ID `unit:reports` at byte 4: the first segment is the system's name and takes no kind"
    );
}

#[test]
fn a_segment_below_the_root_needs_a_kind() {
    assert_eq!(
        check("reports/worker"),
        "invalid ID `reports/worker` at byte 8: `worker` has no kind; write it as `<kind>:<slug>`"
    );
}

#[test]
fn an_unknown_kind_is_rejected_with_the_known_ones() {
    assert_eq!(
        check("reports/store:report"),
        "invalid ID `reports/store:report` at byte 8: `store` is not a kind; the kinds are unit, pin, site"
    );
}

#[test]
fn a_kind_needs_a_slug() {
    assert_eq!(
        check("reports/unit:"),
        "invalid ID `reports/unit:` at byte 13: the slug after the kind is empty"
    );
}

#[test]
fn uppercase_is_rejected_not_lowered() {
    assert_eq!(
        check("Reports"),
        "invalid ID `Reports` at byte 0: `R` is not allowed in a slug, which has only a-z, 0-9 and single hyphens"
    );
}

#[test]
fn an_underscore_is_rejected() {
    assert_eq!(
        check("reports/unit:my_reports"),
        "invalid ID `reports/unit:my_reports` at byte 15: `_` is not allowed in a slug, which has only a-z, 0-9 and single hyphens"
    );
}

#[test]
fn a_second_colon_is_rejected() {
    assert_eq!(
        check("reports/unit:a:b"),
        "invalid ID `reports/unit:a:b` at byte 14: `:` is not allowed in a slug, which has only a-z, 0-9 and single hyphens"
    );
}

#[test]
fn a_character_outside_ascii_is_shown_whole() {
    assert_eq!(
        check("reports/unit:café"),
        "invalid ID `reports/unit:café` at byte 16: `é` is not allowed in a slug, which has only a-z, 0-9 and single hyphens"
    );
}

#[test]
fn a_slug_cannot_start_with_a_hyphen() {
    assert_eq!(
        check("reports/unit:-a"),
        "invalid ID `reports/unit:-a` at byte 13: a hyphen in a slug must stand between two letters or digits"
    );
}

#[test]
fn a_slug_cannot_end_with_a_hyphen() {
    assert_eq!(
        check("reports/unit:a-"),
        "invalid ID `reports/unit:a-` at byte 14: a hyphen in a slug must stand between two letters or digits"
    );
}

#[test]
fn a_slug_cannot_double_a_hyphen() {
    assert_eq!(
        check("reports/unit:a--b"),
        "invalid ID `reports/unit:a--b` at byte 15: a hyphen in a slug must stand between two letters or digits"
    );
}

#[test]
fn a_slug_of_digits_is_rejected() {
    assert_eq!(
        check("reports/unit:123"),
        "invalid ID `reports/unit:123` at byte 13: the slug `123` has no letter"
    );
}

#[test]
fn a_slug_of_digit_groups_is_rejected() {
    assert_eq!(
        check("reports/unit:1-2"),
        "invalid ID `reports/unit:1-2` at byte 13: the slug `1-2` has no letter"
    );
}

#[test]
fn a_non_string_is_not_an_id() {
    let error = serde_json::from_str::<Id>("5").unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid type: integer `5`, expected a string at line 1 column 1"
    );
}

#[test]
fn ids_are_built_from_slugs() {
    let root = Id::root(&slugify("reports").unwrap());
    let built: Vec<String> = Kind::ALL
        .into_iter()
        .map(|kind| {
            root.child(kind, &slugify("SQLAlchemy").unwrap())
                .to_string()
        })
        .collect();
    assert_eq!(
        built,
        [
            "reports/unit:sqlalchemy",
            "reports/pin:sqlalchemy",
            "reports/site:sqlalchemy"
        ]
    );
}

#[test]
fn slugify_joins_words_with_single_hyphens() {
    assert_eq!(
        slug("app.tasks.generate_report"),
        "app-tasks-generate-report"
    );
    assert_eq!(slug("x__y"), "x-y");
}

#[test]
fn slugify_lowers_ascii_letters() {
    assert_eq!(slug("SQLAlchemy"), "sqlalchemy");
}

#[test]
fn slugify_keeps_digits() {
    assert_eq!(slug("s.commit_0"), "s-commit-0");
}

#[test]
fn slugify_trims_both_ends() {
    assert_eq!(slug("__init__"), "init");
}

#[test]
fn slugify_treats_other_characters_as_separators() {
    assert_eq!(slug("naïve"), "na-ve");
}

#[test]
fn slugify_needs_an_ascii_letter() {
    assert_eq!(slug(""), "none");
    assert_eq!(slug("_1"), "none");
    assert_eq!(slug("データ"), "none");
}

/// A valid ID: a root and up to three kinded segments, each slug with a letter.
fn valid_id() -> impl Strategy<Value = Id> {
    let name = "[a-z][a-z0-9]{0,3}(-[a-z0-9]{1,3}){0,2}";
    let kind = prop::sample::select(Kind::ALL.to_vec());
    (name, prop::collection::vec((kind, name), 0..4)).prop_map(|(root, children)| {
        children
            .into_iter()
            .fold(Id::root(&slugify(&root).unwrap()), |id, (kind, name)| {
                id.child(kind, &slugify(&name).unwrap())
            })
    })
}

proptest! {
    #[test]
    fn an_accepted_id_displays_as_its_input(text in "[a-zA-Z0-9:/_é-]{0,24}") {
        if let Ok(id) = Id::parse(&text) {
            prop_assert_eq!(id.to_string(), text);
        }
    }

    #[test]
    fn a_built_id_parses_back(id in valid_id()) {
        prop_assert_eq!(Id::parse(id.as_str()), Ok(id));
    }

    #[test]
    fn id_order_is_text_order(a in valid_id(), b in valid_id()) {
        prop_assert_eq!(a.cmp(&b), a.as_str().cmp(b.as_str()));
    }

    #[test]
    fn a_slug_is_an_id(code_name in any::<String>()) {
        match slugify(&code_name) {
            Some(slug) => prop_assert!(Id::parse(slug.as_str()).is_ok()),
            None => prop_assert!(!code_name.bytes().any(|b| b.is_ascii_alphabetic())),
        }
    }
}
