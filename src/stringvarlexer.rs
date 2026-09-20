// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use crate::chunk::Chunk;
use crate::chunkkind::ChunkKind;
use crate::error::{LexError, LexErrorKind, Result};

#[derive(Debug, Clone, Copy)]
pub struct StringVarLexer<'a> {
    text: &'a str,
    offset: usize,
}

impl<'a> StringVarLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let trimmed = input.trim();
        let leading = input.len() - input.trim_start().len();

        let (text, offset) = if has_matching_outer_quotes(trimmed) {
            (&trimmed[1..trimmed.len() - 1], leading + 1)
        } else {
            (trimmed, leading)
        };

        Self { text, offset }
    }

    pub fn lex(&self) -> Result<Vec<Chunk<'a>>> {
        let mut chunks = Vec::new();
        let mut pos = 0;

        while pos < self.text.len() {
            let rest = &self.text[pos..];

            if rest.chars().next().is_some_and(char::is_whitespace) {
                let start = pos;
                while pos < self.text.len() {
                    let Some(ch) = self.text[pos..].chars().next() else {
                        break;
                    };
                    if !ch.is_whitespace() {
                        break;
                    }
                    pos += ch.len_utf8();
                }
                chunks.push(self.chunk(ChunkKind::Whitespace, start, pos));
                continue;
            }

            if rest.starts_with("${") {
                let start = pos;
                pos += 2;
                let mut level = 1;

                while pos < self.text.len() && level > 0 {
                    let rest = &self.text[pos..];
                    if rest.starts_with("${") {
                        level += 1;
                        pos += 2;
                    } else if rest.starts_with('}') {
                        level -= 1;
                        pos += 1;
                    } else {
                        pos += rest.chars().next().map_or(0, char::len_utf8);
                    }
                }

                if level != 0 {
                    return Err(LexError::new(
                        LexErrorKind::UnterminatedVariableExpression,
                        self.offset + start,
                    ));
                }

                chunks.push(self.chunk(ChunkKind::String, start, pos));
                continue;
            }

            let start = pos;
            while pos < self.text.len() {
                let rest = &self.text[pos..];
                if rest.starts_with("${")
                    || rest.chars().next().is_some_and(char::is_whitespace)
                {
                    break;
                }
                pos += rest.chars().next().map_or(0, char::len_utf8);
            }
            chunks.push(self.chunk(ChunkKind::String, start, pos));
        }

        Ok(chunks)
    }

    fn chunk(&self, kind: ChunkKind, start: usize, end: usize) -> Chunk<'a> {
        Chunk::new(
            kind,
            &self.text[start..end],
            self.offset + start,
            self.offset + end,
        )
    }
}

fn has_matching_outer_quotes(text: &str) -> bool {
    let Some(first) = text.chars().next() else {
        return false;
    };
    let Some(last) = text.chars().next_back() else {
        return false;
    };

    text.len() >= 2 && matches!(first, '"' | '\'') && first == last
}
