// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use simplelexer::chunkkind::ChunkKind;
use simplelexer::stringvarlexer::StringVarLexer;

#[test]
fn strips_outer_quotes_and_splits_whitespace() {
    let lexer = StringVarLexer::new("\"foo bar\"");
    let chunks = lexer.lex().unwrap();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].kind, ChunkKind::String);
    assert_eq!(chunks[0].text, "foo");
    assert_eq!(chunks[1].kind, ChunkKind::Whitespace);
    assert_eq!(chunks[2].text, "bar");
}

#[test]
fn keeps_nested_variable_expression_together() {
    let lexer = StringVarLexer::new("prefix ${A${B}} suffix");
    let chunks = lexer.lex().unwrap();

    assert_eq!(chunks[2].text, "${A${B}}");
    assert_eq!(chunks[2].kind, ChunkKind::String);
}

#[test]
fn unicode_input_keeps_valid_boundaries() {
    let lexer = StringVarLexer::new("\"Grüße ${WELT} 世界\"");
    let chunks = lexer.lex().unwrap();

    assert_eq!(chunks[0].text, "Grüße");
    assert_eq!(chunks[2].text, "${WELT}");
    assert_eq!(chunks[4].text, "世界");
}

#[test]
fn offsets_reference_original_input() {
    let input = "  \"Grüße ${WELT}\"  ";
    let lexer = StringVarLexer::new(input);
    let chunks = lexer.lex().unwrap();

    assert_eq!(&input[chunks[0].start..chunks[0].end], "Grüße");
    assert_eq!(&input[chunks[2].start..chunks[2].end], "${WELT}");
}

#[test]
fn lex_is_repeatable() {
    let lexer = StringVarLexer::new("\"foo ${BAR}\"");
    assert_eq!(lexer.lex().unwrap(), lexer.lex().unwrap());
}

#[test]
fn unterminated_variable_expression_is_an_error() {
    use simplelexer::error::LexErrorKind;

    let lexer = StringVarLexer::new("prefix ${OPEN");
    let error = lexer.lex().unwrap_err();

    assert_eq!(error.kind, LexErrorKind::UnterminatedVariableExpression);
    assert_eq!(error.position, 7);
}

#[test]
fn unmatched_outer_quote_is_an_error() {
    use simplelexer::error::LexErrorKind;

    let lexer = StringVarLexer::new("  \"unterminated");
    let error = lexer.lex().unwrap_err();

    assert_eq!(error.kind, LexErrorKind::UnterminatedString);
    assert_eq!(error.position, 2);
}
