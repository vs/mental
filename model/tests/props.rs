//! Round trips of canonical JSONL through the public API.

use mental_model::{Anchor, Code, Diagnostic, Id, Lines, Location, jsonl};
use proptest::prelude::*;

fn id() -> impl Strategy<Value = Id> {
    "[a-z]{1,3}(/(unit|pin|site):[a-z][a-z0-9]{0,2}(-[a-z0-9]{1,2})?){0,2}"
        .prop_map(|text| Id::parse(&text).expect("the pattern makes only IDs"))
}

fn lines() -> impl Strategy<Value = Lines> {
    (1_u32..60, 0_u32..4).prop_map(|(first, more)| {
        Lines::new(first, first + more).expect("first is at least 1 and last no less")
    })
}

fn anchor() -> impl Strategy<Value = Anchor> {
    (id(), "[a-z_./]{1,12}", lines(), any::<String>()).prop_map(|(id, path, lines, quote)| Anchor {
        id,
        path,
        lines,
        quote,
    })
}

fn diagnostic() -> impl Strategy<Value = Diagnostic> {
    let at = ("[a-z_./]{1,12}", lines()).prop_map(|(path, lines)| Location { path, lines });
    (
        prop::option::of(id()),
        prop::sample::select(Code::ALL.to_vec()),
        prop::option::of(at),
        any::<String>(),
    )
        .prop_map(|(id, code, at, message)| Diagnostic {
            id,
            code,
            at,
            message,
        })
}

/// The bytes of `text` with `edit` applied at the character boundary nearest
/// `at`: a character inserted, or the character there removed.
fn edited(text: &str, at: prop::sample::Index, edit: Option<char>) -> String {
    let boundaries: Vec<usize> = text
        .char_indices()
        .map(|(i, _)| i)
        .chain([text.len()])
        .collect();
    let at = boundaries[at.index(boundaries.len())];
    let mut text = text.to_owned();
    match edit {
        Some(c) => text.insert(at, c),
        None if at < text.len() => drop(text.remove(at)),
        None => {}
    }
    text
}

fn edit() -> impl Strategy<Value = Option<char>> {
    prop::option::of(prop::sample::select(vec![
        ' ', '\n', '\r', ',', '"', '{', '}', '[', ']', '0', 'a', '/',
    ]))
}

proptest! {
    #[test]
    fn anchors_read_back_sorted(anchors in prop::collection::vec(anchor(), 0..6)) {
        let mut sorted = anchors.clone();
        sorted.sort();
        prop_assert_eq!(jsonl::read::<Anchor>(&jsonl::write(&anchors)), Ok(sorted));
    }

    #[test]
    fn diagnostics_read_back_sorted(diagnostics in prop::collection::vec(diagnostic(), 0..6)) {
        let mut sorted = diagnostics.clone();
        sorted.sort();
        prop_assert_eq!(jsonl::read::<Diagnostic>(&jsonl::write(&diagnostics)), Ok(sorted));
    }

    #[test]
    fn an_accepted_anchor_file_writes_back_to_its_bytes(
        anchors in prop::collection::vec(anchor(), 0..4),
        at in any::<prop::sample::Index>(),
        edit in edit(),
    ) {
        let text = edited(&jsonl::write(&anchors), at, edit);
        if let Ok(records) = jsonl::read::<Anchor>(&text) {
            prop_assert_eq!(jsonl::write(&records), text);
        }
    }

    #[test]
    fn an_accepted_diagnostic_file_writes_back_to_its_bytes(
        diagnostics in prop::collection::vec(diagnostic(), 0..4),
        at in any::<prop::sample::Index>(),
        edit in edit(),
    ) {
        let text = edited(&jsonl::write(&diagnostics), at, edit);
        if let Ok(records) = jsonl::read::<Diagnostic>(&text) {
            prop_assert_eq!(jsonl::write(&records), text);
        }
    }
}
