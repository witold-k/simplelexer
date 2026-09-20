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

    fn peek(&self, n: usize) -> &'a str {
        let end = std::cmp::min(self.pos + n, self.len);
        &self.text[self.pos..end]
    }

    fn advance(&mut self, n: usize) {
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
                if self.peek(2) == "\\\"" {
                    self.advance(2);
                    continue;
                }
                if self.peek(1) == "\"" {
                    self.advance(1);
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
                self.advance(1);
                continue;
            }

            if self.peek(1) == "\"" {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                in_string = true;
                start = self.pos;
                self.advance(1);
                continue;
            }

            let next_3 = self.peek(3);
            match next_3 {
                // 3-Zeichen-Operatoren
                "<<=" | ">>=" | "<=>" | "||=" | "&&=" | "??=" | "**=" | "::=" | "..." | "|||" | "&&&" => {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                    let op_start = self.pos;
                    self.advance(3);
                    chunks.push(Chunk::<'a> {
                        kind: ChunkKind::Assign,
                        text: &self.text[op_start..self.pos],
                        start: op_start,
                        end: self.pos,
                    });
                    start = self.pos;
                    continue;
                }
                _ => {}
            }

            let next_2 = self.peek(2);
            match next_2 {
                // 2-Zeichen-Operatoren
                ":=" | "?=" | "+=" | "-=" | "=-" | "=+" | ".=" | "=." |
                "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "~=" | "^^=" |
                "==" | "!=" | "<=" | ">=" | "~~" | "!~" | "<>" | "=:=" | "=@" |
                "&&" | "||" | "=>" | "->" | "<-" | "<~" | "~>" | "++" | "--" |
                "**" | "::" | ".." => {
                    Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                    let op_start = self.pos;
                    self.advance(2);
                    chunks.push(Chunk::<'a> {
                        kind: ChunkKind::Assign,
                        text: &self.text[op_start..self.pos],
                        start: op_start,
                        end: self.pos,
                    });
                    start = self.pos;
                    continue;
                }
                _ => {}
            }

            if self.peek(1) == "=" {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                let op_start = self.pos;
                self.advance(1);
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
            if self.peek(1).chars().next().is_some_and(|c| c.is_whitespace()) {
                Self::flush_code_chunk(&mut chunks, start, self.pos, self.text);
                let ws_start = self.pos;
                while !self.eof() && self.peek(1).chars().next().is_some_and(|c| c.is_whitespace()) {
                    self.advance(1);
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
            self.advance(1);
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

