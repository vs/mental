use super::*;

#[test]
fn a_file_that_parses_has_no_diagnostic() {
    let (module, diagnostics) = parse("a.py", "x = f(1)\n");
    assert_eq!(module.body.len(), 1);
    assert_eq!(diagnostics, []);
}

#[test]
fn an_error_at_the_end_of_the_file_stands_on_its_last_line() {
    let (_, diagnostics) = parse("a.py", "x = 1\ny = (");
    let lines: Vec<[u32; 2]> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.at.as_ref().unwrap().lines.into())
        .collect();
    assert_eq!(lines, [[2, 2]]);
}
