// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use simplelexer::chunk::Chunk;
use simplelexer::chunklist::ChunkList;
use simplelexer::chunkkind::ChunkKind;
use simplelexer::quotelexer::QuoteLexer;
// use simplelexer::stringvarlexer::StringVarLexer;


pub fn make_chunk_list(text: &str) -> ChunkList<'_> {
    let mut lexer = QuoteLexer::new(text);
    ChunkList {
        chunks: lexer.lex(),
    }
}

#[cfg(test)]
fn lex<'a>(text: &'a str) -> Vec<Chunk<'a>> {
    let mut lexer = QuoteLexer::new(text);
    lexer.lex()
}

// Unit Tests in Rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let chunks = lex("\"hello\"");
        assert_eq!(
            chunks,
            vec![
                Chunk::from_str(ChunkKind::String, "\"hello\"")
            ]
        );
    }

    #[test]
    fn test_code_then_string() {
        let chunks = lex("FOO = \"bar\"");
        assert_eq!(
            chunks,
            vec![
                Chunk::new(ChunkKind::Code, "FOO", 0, 3),
                Chunk::new(ChunkKind::Whitespace, " ", 3, 4),
                Chunk::new(ChunkKind::Assign, "=", 4, 5),
                Chunk::new(ChunkKind::Whitespace, " ", 5, 6),
                Chunk::new(ChunkKind::String, "\"bar\"", 6, 11),
            ]
        );
    }

    #[test]
    fn test_src_rev_case() {
        let text = "SRCREV = \"0d2290a8e64b69207382863fec807eebbeff7ed5\"\n";
        let chunks = lex(text);

        assert_eq!(
            chunks,
            vec![
                Chunk::new(ChunkKind::Code, "SRCREV", 0, 6),
                Chunk::new(ChunkKind::Whitespace, " ", 6, 7),
                Chunk::new(ChunkKind::Assign, "=", 7, 8),
                Chunk::new(ChunkKind::Whitespace, " ", 8, 9),
                Chunk::new(ChunkKind::String, "\"0d2290a8e64b69207382863fec807eebbeff7ed5\"", 9, 51),
                Chunk::new(ChunkKind::Whitespace, "\n", 51, 52),
            ]
        );
    }

    #[test]
    fn test_assignment_with_comment() {
        let text = "SRCREV = \"abc123\"    # comment\n";

        let chunks = lex(text);

        assert_eq!(
            chunks,
            vec![
                Chunk::new(ChunkKind::Code, "SRCREV", 0, 6),
                Chunk::new(ChunkKind::Whitespace, " ", 6, 7),
                Chunk::new(ChunkKind::Assign, "=", 7, 8),
                Chunk::new(ChunkKind::Whitespace, " ", 8, 9),
                Chunk::new(ChunkKind::String, r#""abc123""#, 9, 17),
                Chunk::new(ChunkKind::Whitespace, "    ", 17, 21),
                Chunk::new(ChunkKind::Code, "#", 21, 22),
                Chunk::new(ChunkKind::Whitespace, " ", 22, 23),
                Chunk::new(ChunkKind::Code, "comment", 23, 30),
                Chunk::new(ChunkKind::Whitespace, "\n", 30, 31),
            ]
        );
    }

    #[test]
    fn test_escaped_quote() {
        let text = "FOO = \"a string with an escaped quote: \\\" inside\"\n";

        let chunks = lex(text);

        assert_eq!(
            chunks,
            vec![
                Chunk::new(ChunkKind::Code, "FOO", 0, 3),
                Chunk::new(ChunkKind::Whitespace, " ", 3, 4),
                Chunk::new(ChunkKind::Assign, "=", 4, 5),
                Chunk::new(ChunkKind::Whitespace, " ", 5, 6),
                Chunk::new(
                    ChunkKind::String,
                    "\"a string with an escaped quote: \\\" inside\"",
                    6,
                    text.len() - 1,
                ),
                Chunk::new(
                    ChunkKind::Whitespace,
                    "\n",
                    text.len() - 1,
                    text.len(),
                ),
            ]
        );
    }

    #[test]
    fn test_multiline_string() {
        let text = "FOO = \"line1 \\\nline2 \\\nline3\"\n";

        let chunks = lex(text);

        assert_eq!(chunks[0].kind, ChunkKind::Code);
        assert_eq!(chunks[4].kind, ChunkKind::String);
        assert_eq!(chunks[4].text, "\"line1 \\\nline2 \\\nline3\"");
        assert_eq!(chunks[5].kind, ChunkKind::Whitespace);
        assert_eq!(chunks[5].text, "\n");
    }

    #[test]
    fn test_multiple_assignments() {
        let text = concat!(
            "A = \"1\"\n",
            "B = \"2\"\n",
            "C = \"3\"\n",
        );

        let chunks = lex(text);

        assert_eq!(chunks.len(), 18);

        assert_eq!(chunks[0].text, "A");
        assert_eq!(chunks[6].text, "B");
        assert_eq!(chunks[12].text, "C");
    }

    #[test]
    fn test_all_new_operators() {
        let text = r#"A -= "1" B
     Alpine =- "2" C =+ "3" D .= "4" E =. "5""#;

        let chunks = lex(text);

        let assigns: Vec<_> = chunks
            .iter()
            .filter(|c| c.kind == ChunkKind::Assign)
            .map(|c| c.text)
            .collect();

        assert_eq!(assigns, vec!["-=", "=-", "=+", ".=", "=."]);
    }

    #[test]
    fn test_collect_simple_vars() {
        let cl = make_chunk_list(r#"var1="wert1"  var2 = "wert2"
    var3  :=  "wert3""#);

        let res = cl.collect_simple_vars(&["var1", "var2", "var3"]);

        assert_eq!(res["var1"], "\"wert1\"");
        assert_eq!(res["var2"], "\"wert2\"");
        assert_eq!(res["var3"], "\"wert3\"");
    }

    #[test]
    fn test_replace_val_at_begin() {
        let mut cl = make_chunk_list(r#"username = "old_user"
    password = "old_password""#);

        cl.replace_val_at_begin("username", "\"new_user\"");

        let txt = cl.to_string();

        assert!(txt.contains("\"new_user\""));
        assert!(txt.contains("\"old_password\""));
    }

    #[test]
    fn test_collect_combined_vars() {
        let cl = make_chunk_list(
            r#"var = "foo" var += "bar" var += "baz""#
        );

        let res = cl.collect_combined_vars(&["var"]);

        assert_eq!(res["var"], "\"foobarbaz\"");
    }

    #[test]
    fn test_collect_combined_vars_prepend() {
        let cl = make_chunk_list(
            r#"var = "center" var =+ "left" var =+ "outer""#
        );

        let res = cl.collect_combined_vars(&["var"]);

        assert_eq!(res["var"], "\"outerleftcenter\"");
    }

    #[test]
    fn test_collect_combined_vars_remove() {
        let cl = make_chunk_list(
            r#"var = "abcdef" var -= "cd""#
        );

        let res = cl.collect_combined_vars(&["var"]);

        assert_eq!(res["var"], "\"abef\"");
    }

}
