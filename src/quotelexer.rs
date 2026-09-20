// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use crate::chunk::Chunk;
use crate::chunkkind::ChunkKind;
use crate::error::{LexError, LexErrorKind, Result};

const ASSIGNMENTS: [&str; 17] = [
    "<<=", ">>=", "||=", "&&=", "??=", "**=", "::=", ":=", "?=", "+=", "-=", "=-", "=+", ".=",
    "=.", "*=", "/=",
];

const MORE_ASSIGNMENTS: [&str; 7] = ["%=", "&=", "|=", "^=", "~=", "=:=", "=@"];

const OPERATORS: [&str; 21] = [
    "<=>", "...", "|||", "^^=", "==", "!=", "<=", ">=", "~~", "!~", "<>", "&&", "||", "=>", "->",
    "<-", "<~", "~>", "++", "--", "::",
];

#[derive(Debug, Clone, Copy)]
pub struct QuoteLexer<'a> {
    text: &'a str,
}

impl<'a> QuoteLexer<'a> {
    pub const fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn lex(&self) -> Result<Vec<Chunk<'a>>> {
        let mut chunks = Vec::new();
        let mut pos = 0;
        let mut start = 0;
        let mut in_string = false;

        while pos < self.text.len() {
            let rest = &self.text[pos..];

            if in_string {
                if rest.starts_with("\\\"") {
                    pos += 2;
                    continue;
                }
                if rest.starts_with('"') {
                    pos += 1;
                    chunks.push(Chunk::new(
                        ChunkKind::String,
                        &self.text[start..pos],
                        start,
                        pos,
                    ));
                    start = pos;
                    in_string = false;
                    continue;
                }

                pos += next_char_len(rest);
                continue;
            }

            if rest.starts_with('"') {
                push_chunk(&mut chunks, ChunkKind::Code, self.text, start, pos);
                in_string = true;
                start = pos;
                pos += 1;
                continue;
            }

            if let Some(op) = longest_match(rest, &ASSIGNMENTS)
                .or_else(|| longest_match(rest, &MORE_ASSIGNMENTS))
            {
                push_chunk(&mut chunks, ChunkKind::Code, self.text, start, pos);
                let op_start = pos;
                pos += op.len();
                chunks.push(Chunk::new(
                    ChunkKind::Assignment,
                    &self.text[op_start..pos],
                    op_start,
                    pos,
                ));
                start = pos;
                continue;
            }

            if let Some(op) = longest_match(rest, &OPERATORS) {
                push_chunk(&mut chunks, ChunkKind::Code, self.text, start, pos);
                let op_start = pos;
                pos += op.len();
                chunks.push(Chunk::new(
                    ChunkKind::Operator,
                    &self.text[op_start..pos],
                    op_start,
                    pos,
                ));
                start = pos;
                continue;
            }

            if rest.starts_with('=') {
                push_chunk(&mut chunks, ChunkKind::Code, self.text, start, pos);
                chunks.push(Chunk::new(ChunkKind::Assignment, "=", pos, pos + 1));
                pos += 1;
                start = pos;
                continue;
            }

            if rest.chars().next().is_some_and(char::is_whitespace) {
                push_chunk(&mut chunks, ChunkKind::Code, self.text, start, pos);
                let ws_start = pos;
                while pos < self.text.len() {
                    let rest = &self.text[pos..];
                    let Some(ch) = rest.chars().next() else {
                        break;
                    };
                    if !ch.is_whitespace() {
                        break;
                    }
                    pos += ch.len_utf8();
                }
                chunks.push(Chunk::new(
                    ChunkKind::Whitespace,
                    &self.text[ws_start..pos],
                    ws_start,
                    pos,
                ));
                start = pos;
                continue;
            }

            pos += next_char_len(rest);
        }

        if in_string {
            return Err(LexError::new(LexErrorKind::UnterminatedString, start));
        }

        if start < self.text.len() {
            push_chunk(
                &mut chunks,
                ChunkKind::Code,
                self.text,
                start,
                self.text.len(),
            );
        }

        Ok(chunks)
    }
}

fn next_char_len(text: &str) -> usize {
    text.chars().next().map_or(0, char::len_utf8)
}

fn longest_match<'a>(text: &str, candidates: &'a [&str]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .filter(|candidate| text.starts_with(candidate))
        .max_by_key(|candidate| candidate.len())
}

fn push_chunk<'a>(
    chunks: &mut Vec<Chunk<'a>>,
    kind: ChunkKind,
    text: &'a str,
    start: usize,
    end: usize,
) {
    if end > start {
        chunks.push(Chunk::new(kind, &text[start..end], start, end));
    }
}
