// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use simplelexer::chunkkind::ChunkKind;
use simplelexer::stringvarlexer::StringVarLexer;

#[test]
fn strips_outer_quotes_and_splits_whitespace() {
    let mut lexer = StringVarLexer::new("\"foo bar\"");
    let chunks = lexer.lex();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].kind, ChunkKind::String);
    assert_eq!(chunks[0].text, "foo");
    assert_eq!(chunks[1].kind, ChunkKind::Whitespace);
    assert_eq!(chunks[2].text, "bar");
}

#[test]
fn keeps_nested_variable_expression_together() {
    let mut lexer = StringVarLexer::new("prefix ${A${B}} suffix");
    let chunks = lexer.lex();

    assert_eq!(chunks[2].text, "${A${B}}");
    assert_eq!(chunks[2].kind, ChunkKind::String);
}

#[test]
fn unicode_input_keeps_valid_boundaries() {
    let mut lexer = StringVarLexer::new("\"Grüße ${WELT} 世界\"");
    let chunks = lexer.lex();

    assert_eq!(chunks[0].text, "Grüße");
    assert_eq!(chunks[2].text, "${WELT}");
    assert_eq!(chunks[4].text, "世界");
}
