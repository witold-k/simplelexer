// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use simplelexer::chunk::Chunk;
use simplelexer::chunkkind::ChunkKind;
use simplelexer::tokenlexer::TokenLexer;

#[test]
fn both_quotes_are_operators() {
    let input = "name = \"hello world\" 'ok'";
    let chunks = TokenLexer::new(input).lex().unwrap();
    let actual: Vec<_> = chunks.iter().map(|c| (c.kind, c.text)).collect();
    assert_eq!(actual, vec![
        (ChunkKind::Code, "name"),
        (ChunkKind::Whitespace, " "),
        (ChunkKind::Assignment, "="),
        (ChunkKind::Whitespace, " "),
        (ChunkKind::Operator, "\""),
        (ChunkKind::Code, "hello"),
        (ChunkKind::Whitespace, " "),
        (ChunkKind::Code, "world"),
        (ChunkKind::Operator, "\""),
        (ChunkKind::Whitespace, " "),
        (ChunkKind::Operator, "'"),
        (ChunkKind::Code, "ok"),
        (ChunkKind::Operator, "'"),
    ]);
}

#[test]
fn unmatched_quotes_are_not_errors() {
    let chunks = TokenLexer::new("\"foo 'bar").lex().unwrap();
    assert_eq!(chunks[0], Chunk::new(ChunkKind::Operator, "\"", 0, 1));
    assert_eq!(chunks[2].kind, ChunkKind::Whitespace);
    assert_eq!(chunks[3].kind, ChunkKind::Operator);
}

#[test]
fn operators_and_assignments_keep_their_kinds() {
    let chunks = TokenLexer::new("a == b += c").lex().unwrap();
    assert_eq!(chunks[2].kind, ChunkKind::Operator);
    assert_eq!(chunks[2].text, "==");
    assert_eq!(chunks[6].kind, ChunkKind::Assignment);
    assert_eq!(chunks[6].text, "+=");
}

#[test]
fn unicode_offsets_and_reconstruction() {
    let input = "Grüße=\"世界\"";
    let chunks = TokenLexer::new(input).lex().unwrap();
    assert_eq!(chunks.iter().map(|c| c.text).collect::<String>(), input);
    for chunk in &chunks {
        assert_eq!(&input[chunk.start..chunk.end], chunk.text);
    }
}

#[test]
fn empty_input_and_repeatability() {
    assert!(TokenLexer::new("").lex().unwrap().is_empty());
    let lexer = TokenLexer::new("'a'");
    assert_eq!(lexer.lex().unwrap(), lexer.lex().unwrap());
}

#[test]
fn backslash_does_not_escape_quote() {
    let chunks = TokenLexer::new("\\\"x").lex().unwrap();
    assert_eq!(chunks[0].kind, ChunkKind::Code);
    assert_eq!(chunks[0].text, "\\");
    assert_eq!(chunks[1].kind, ChunkKind::Operator);
    assert_eq!(chunks[1].text, "\"");
}
