// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use crate::chunk::Chunk;
use crate::chunkkind::ChunkKind;

pub struct QuoteLexer<'a> {
    text: &'a str,
    pos: usize,
    len: usize,
}

impl<'a> QuoteLexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            pos: 0,
            len: text.len(),
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.len
    }

    fn peek_char(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }

    fn starts_with(&self, value: &str) -> bool {
        self.text[self.pos..].starts_with(value)
    }

    fn advance_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.pos += ch.len_utf8();
        }
    }

    fn advance_bytes(&mut self, n: usize) {
        self.pos += n;
    }

    fn flush_code_chunk(chunks: &mut Vec<Chunk<'a>>, s_pos: usize, e_pos: usize, text: &'a str) {
        if e_pos > s_pos {
            chunks.push(Chunk::<'a> {
                kind: ChunkKind::Code,
                text: &text[s_pos..e_pos],
                start: s_pos,
                end: e_pos,
            });
        }
    }

    pub fn lex(&mut self) -> Vec<Chunk<'a>> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut in_string = false;

        while !self.eof() {
            if in_string {
                if self.starts_with("\\\"") {
                    self.advance_bytes(2);
                    continue;
                }
                if self.starts_with("\"") {
                    self.advance_char();
                    chunks.push(Chunk {
                        kind: ChunkKind::String,
                        text: &self.text[start..self.pos],
                        start,
                        end: self.pos,
                    });
                    start = self.pos;
                    in_string = false;
                    continue;
                }
                self.advance_char();
                continue;
            }

            if self.starts_with("\"") {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                in_string = true;
                start = self.pos;
                self.advance_char();
                continue;
            }

            const ASSIGN_3: [&str; 10] = [
                "<<=", ">>=", "<=>", "||=", "&&=", "??=", "**=", "::=", "...", "|||",
            ];
            const ASSIGN_2: [&str; 30] = [
                ":=", "?=", "+=", "-=", "=-", "=+", ".=", "=.", "*=", "/=", "%=", "&=", "|=",
                "^=", "~=", "==", "!=", "<=", ">=", "~~", "!~", "<>", "&&", "||", "=>", "->",
                "<-", "<~", "~>", "++",
            ];

            if let Some(op) = ASSIGN_3.iter().find(|op| self.starts_with(op)) {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                let op_start = self.pos;
                self.advance_bytes(op.len());
                chunks.push(Chunk::<'a> {
                    kind: ChunkKind::Assign,
                    text: &self.text[op_start..self.pos],
                    start: op_start,
                    end: self.pos,
                });
                start = self.pos;
                continue;
            }

            if let Some(op) = ASSIGN_2.iter().find(|op| self.starts_with(op)) {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                let op_start = self.pos;
                self.advance_bytes(op.len());
                chunks.push(Chunk::<'a> {
                    kind: ChunkKind::Assign,
                    text: &self.text[op_start..self.pos],
                    start: op_start,
                    end: self.pos,
                });
                start = self.pos;
                continue;
            }

            if self.starts_with("=") {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                let op_start = self.pos;
                self.advance_char();
                chunks.push(Chunk::<'a> {
                    kind: ChunkKind::Assign,
                    text: &self.text[op_start..self.pos],
                    start: op_start,
                    end: self.pos,
                });
                start = self.pos;
                continue;
            }

            // Prüft das erste Zeichen auf Whitespace (ASCII-kompatibel)
            if self.peek_char().is_some_and(|c| c.is_whitespace()) {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                let ws_start = self.pos;
                while !self.eof() && self.peek_char().is_some_and(|c| c.is_whitespace()) {
                    self.advance_char();
                }
                chunks.push(Chunk::<'a> {
                    kind: ChunkKind::Whitespace,
                    text: &self.text[ws_start..self.pos],
                    start: ws_start,
                    end: self.pos,
                });
                start = self.pos;
                continue;
            }

            // Alles andere wird als CODE-Teil konsumiert
            self.advance_char();
        }

        if start < self.len {
            if in_string {
                chunks.push(Chunk {
                    kind: ChunkKind::String,
                    text: &self.text[start..self.len],
                    start,
                    end: self.len,
                });
            } else {
                Self::flush_code_chunk(&mut chunks, start, self.len, self.text);
            }
        }

        chunks
    }
}

