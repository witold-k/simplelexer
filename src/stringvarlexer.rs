// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use crate::chunk::Chunk;
use crate::chunkkind::ChunkKind;

pub struct StringVarLexer<'a> {
    text: &'a str,
    pos: usize,
    len: usize,
}

impl<'a> StringVarLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        // 1. strip() analog zu Python
        let mut trimmed = input.trim();

        // 2. Umschließende Quotes entfernen ("..." oder '...')
        if trimmed.len() >= 2 {
            let bytes = trimmed.as_bytes();
            let first = bytes[0];
            let last = bytes[bytes.len() - 1];

            if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
                trimmed = &trimmed[1..trimmed.len() - 1];
            }
        }

        Self {
            text: trimmed,
            pos: 0,
            len: trimmed.len(),
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.len
    }

    fn peek(&self, n: usize) -> &'a str {
        let end = std::cmp::min(self.pos + n, self.len);
        &self.text[self.pos..end]
    }

    fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    pub fn lex(&mut self) -> Vec<Chunk<'a>> {
        let mut chunks = Vec::new();

        while !self.eof() {
            // 1. Whitespace verarbeiten
            if self.peek(1).chars().next().is_some_and(|c| c.is_whitespace()) {
                let start = self.pos;
                while !self.eof() && self.peek(1).chars().next().is_some_and(|c| c.is_whitespace()) {
                    self.advance(1);
                }
                chunks.push(Chunk {
                    kind: ChunkKind::Whitespace,
                    text: &self.text[start..self.pos],
                    start,
                    end: self.pos,
                });
                continue;
            }

            // 2. BitBake Expression ${...} verschachtelt matchen
            if self.peek(2) == "${" {
                let start = self.pos;
                self.advance(2);
                let mut bracket_level = 1;

                while !self.eof() && bracket_level > 0 {
                    if self.peek(2) == "${" {
                        bracket_level += 1;
                        self.advance(2);
                    } else if self.peek(1) == "}" {
                        bracket_level -= 1;
                        self.advance(1);
                    } else {
                        self.advance(1);
                    }
                }
                chunks.push(Chunk {
                    kind: ChunkKind::String,
                    text: &self.text[start..self.pos],
                    start,
                    end: self.pos,
                });
                continue;
            }

            // 3. Normale Wörter / Zeichen (Alles andere wird als STRING konsumiert)
            let start = self.pos;
            while !self.eof()
                && !self.peek(1).chars().next().is_some_and(|c| c.is_whitespace())
                && self.peek(2) != "${"
            {
                self.advance(1);
            }
            chunks.push(Chunk {
                kind: ChunkKind::String,
                text: &self.text[start..self.pos],
                start,
                end: self.pos,

            });
        }

        chunks
    }
}

