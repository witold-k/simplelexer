// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use simplelexer::chunk::Chunk;
use simplelexer::chunkkind::ChunkKind;
use simplelexer::chunklist::ChunkList;
use simplelexer::quotelexer::QuoteLexer;

fn make_chunk_list(text: &str) -> ChunkList {
    ChunkList::from_text(text)
}

fn lex(text: &str) -> Vec<Chunk<'_>> {
    let mut lexer = QuoteLexer::new(text);
    lexer.lex()
}

#[test]
fn simple_string() {
    let chunks = lex("\"hello\"");
    assert_eq!(chunks, vec![Chunk::from_str(ChunkKind::String, "\"hello\"")]);
}

#[test]
fn code_then_string() {
    let chunks = lex("FOO = \"bar\"");
    assert_eq!(
        chunks,
        vec![
            Chunk::new(ChunkKind::Code, "FOO", 0, 3),
            Chunk::new(ChunkKind::Whitespace, " ", 3, 4),
            Chunk::new(ChunkKind::Assignment, "=", 4, 5),
            Chunk::new(ChunkKind::Whitespace, " ", 5, 6),
            Chunk::new(ChunkKind::String, "\"bar\"", 6, 11),
        ]
    );
}

#[test]
fn escaped_quote() {
    let text = "FOO = \"a string with an escaped quote: \\\" inside\"\n";
    let chunks = lex(text);

    assert_eq!(chunks[4].kind, ChunkKind::String);
    assert_eq!(chunks[4].text, "\"a string with an escaped quote: \\\" inside\"");
    assert_eq!(chunks[5].text, "\n");
}

#[test]
fn unicode_input_keeps_valid_boundaries() {
    let text = "NAME = \"Grüße 世界\"\n";
    let chunks = lex(text);

    assert_eq!(chunks[0], Chunk::new(ChunkKind::Code, "NAME", 0, 4));
    assert_eq!(chunks[4].kind, ChunkKind::String);
    assert_eq!(chunks[4].text, "\"Grüße 世界\"");
    assert_eq!(chunks[4].start, 7);
    assert_eq!(chunks[4].end, text.len() - 1);
}

#[test]
fn collect_simple_vars() {
    let cl = make_chunk_list("var1=\"wert1\"  var2 = \"wert2\"\nvar3  :=  \"wert3\"");
    let res = cl.collect_simple_vars(&["var1", "var2", "var3"]);

    assert_eq!(res["var1"], "\"wert1\"");
    assert_eq!(res["var2"], "\"wert2\"");
    assert_eq!(res["var3"], "\"wert3\"");
}

#[test]
fn replace_val_at_begin() {
    let mut cl = make_chunk_list("username = \"old_user\"\npassword = \"old_password\"");
    assert!(cl.replace_value("username", "\"new_user\""));

    let txt = cl.to_string();
    assert!(txt.contains("\"new_user\""));
    assert!(txt.contains("\"old_password\""));
}

#[test]
fn collect_combined_vars() {
    let cl = make_chunk_list("var = \"foo\" var += \"bar\" var += \"baz\"");
    let res = cl.collect_combined_vars(&["var"]);

    assert_eq!(res["var"], "\"foobarbaz\"");
}

#[test]
fn collect_combined_vars_prepend() {
    let cl = make_chunk_list("var = \"center\" var =+ \"left\" var =+ \"outer\"");
    let res = cl.collect_combined_vars(&["var"]);

    assert_eq!(res["var"], "\"outerleftcenter\"");
}

#[test]
fn collect_combined_vars_remove() {
    let cl = make_chunk_list("var = \"abcdef\" var -= \"cd\"");
    let res = cl.collect_combined_vars(&["var"]);

    assert_eq!(res["var"], "\"abef\"");
}

#[test]
fn replace_value_accepts_short_lived_input() {
    let mut cl = ChunkList::from_text("name = \"old\"");

    {
        let value = String::from("\"new\"");
        assert!(cl.replace_value("name", &value));
    }

    assert_eq!(cl.to_string(), "name = \"new\"");
}

#[test]
fn replace_value_appends_missing_assignment() {
    let mut cl = ChunkList::from_text("first = \"1\"");
    assert!(!cl.replace_value("second", "\"2\""));

    assert_eq!(cl.to_string(), "first = \"1\"\nsecond = \"2\"");
}

#[test]
fn comparison_is_operator_not_assignment() {
    let lexer = QuoteLexer::new("left == right");
    let chunks = lexer.lex();

    assert_eq!(chunks[2].kind, ChunkKind::Operator);
    assert_eq!(chunks[2].text, "==");
}

#[test]
fn lex_is_repeatable() {
    let lexer = QuoteLexer::new("A = \"1\"");
    assert_eq!(lexer.lex(), lexer.lex());
}
