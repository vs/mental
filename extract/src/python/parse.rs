//! One Python file to its syntax tree, recovering from syntax errors.

use mental_model::{Code, Diagnostic};
use ruff_python_ast::{ModModule, PySourceType};
use ruff_python_parser::parse_unchecked_source;

/// The tree of `text`, with a `syntax-error` for each error the parser
/// recovered from.
pub(super) fn parse(path: &str, text: &str) -> (ModModule, Vec<Diagnostic>) {
    let parsed = parse_unchecked_source(text, PySourceType::Python);
    let diagnostics = parsed
        .errors()
        .iter()
        .map(|error| Diagnostic {
            id: None,
            code: Code::SyntaxError,
            at: Some(crate::anchor::location(path, text, error.location.into())),
            message: format!(
                "`{path}` does not parse here: {}; the sites of the rest of the file come from \
                 the tree the parser recovered, so fix the syntax to list them all",
                error.error
            ),
        })
        .collect();
    (parsed.into_syntax(), diagnostics)
}

#[cfg(test)]
mod tests;
