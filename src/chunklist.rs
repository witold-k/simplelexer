// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::collections::HashSet;
use std::collections::HashMap;
use crate::chunk::Chunk;
use crate::chunkkind::ChunkKind;
use crate::quotelexer::QuoteLexer;


#[derive(Debug, Clone, Default)]
pub struct ChunkList<'a> {
    pub chunks: Vec<Chunk<'a>>,
}

impl<'a> ChunkList<'a> {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    // staticmethod equivalent
    pub fn from_text(text: &'a str) -> Self {
        let mut lexer = QuoteLexer::new(text);
        // Wir konvertieren hier die Slices des Lexers in eigene Strings
        let chunks = lexer.lex().into_iter().map(|c| Chunk {
            kind: c.kind,
            text: c.text,
            start: c.start,
            end: c.end,
        }).collect();

        Self { chunks }
    }

    pub fn replace_val_at_begin(&mut self, name: &'a str, value: &'a str) {
        let mut target_idx = None;

        // 1. Suche nach ChunkKind::Code mit dem Namen
        for (i, chunk) in self.chunks.iter().enumerate() {
            if chunk.kind == ChunkKind::Code && chunk.text == name {
                target_idx = Some(i);
                break;
            }
        }

        // 2. Wenn nicht gefunden, anhängen und beenden
        let target_idx = match target_idx {
            None => {
                if let Some(last) = self.chunks.last()
                    && (last.kind != ChunkKind::Whitespace || !last.text.contains('\n')) {
                    self.chunks.push(Chunk::<'a>::new(ChunkKind::Whitespace, "\n", 0, 1));
                }
                self.chunks.push(Chunk::<'a>::new(ChunkKind::Code, name, 0, name.len()));
                self.chunks.push(Chunk::<'a>::new(ChunkKind::Whitespace, " ", 0, 1));
                self.chunks.push(Chunk::<'a>::new(ChunkKind::Assign, "=", 0, 1));
                self.chunks.push(Chunk::<'a>::new(ChunkKind::Whitespace, " ", 0, 1));
                self.chunks.push(Chunk::<'a>::new(ChunkKind::String, value, 0, value.len()));
                return;
            }
            Some(idx) => idx,
        };

        // 3. Sequenz validieren: (WHITESPACE)* ASSIGN (WHITESPACE)*
        let mut cursor = target_idx + 1;

        while cursor < self.chunks.len() && self.chunks[cursor].kind == ChunkKind::Whitespace {
            cursor += 1;
        }

        if cursor >= self.chunks.len() || self.chunks[cursor].kind != ChunkKind::Assign {
            // Rekursiver Fallback (In Rust über Schleife gelöst, um Borrow-Checker-Konflikte zu umgehen)
            // Wir simulieren das Python-Verhalten, indem wir das Element löschen und neu anhängen
            self.chunks.push(Chunk::<'a>::new(ChunkKind::Whitespace, "\n", 0, 1));
            self.chunks.push(Chunk::<'a>::new(ChunkKind::Code, name, 0, name.len()));
            self.chunks.push(Chunk::<'a>::new(ChunkKind::Whitespace, " ", 0, 1));
            self.chunks.push(Chunk::<'a>::new(ChunkKind::Assign, "=", 0, 1));
            self.chunks.push(Chunk::<'a>::new(ChunkKind::Whitespace, " ", 0, 1));
            self.chunks.push(Chunk::<'a>::new(ChunkKind::String, value, 0, value.len()));
            return;
        }

        cursor += 1; // Weiter nach ASSIGN

        while cursor < self.chunks.len() && self.chunks[cursor].kind == ChunkKind::Whitespace {
            cursor += 1;
        }

        // 4. Folgendes ChunkKind::String ersetzen oder anhängen
        if cursor < self.chunks.len() && self.chunks[cursor].kind == ChunkKind::String {
            self.chunks[cursor].text = value;
        } else {
            self.chunks.push(Chunk::<'a>::new(ChunkKind::String, value, 0, value.len()));
            self.chunks.push(Chunk::<'a>::new(ChunkKind::Whitespace, "\n", 0, 1));
        }
    }

    pub fn collect_simple_vars(&self, var_names: &[&str]) -> HashMap<String, String> {
        let mut result = HashMap::<String, String>::new();
        let allowed_names: HashSet<&str> = var_names.iter().cloned().collect();
        let num_chunks = self.chunks.len();

        for i in 0..num_chunks {
            let chunk = &self.chunks[i];
            let trimmed_name = chunk.text.trim();
            if chunk.kind != ChunkKind::Code || !allowed_names.contains(trimmed_name) {
                continue;
            }

            let mut idx = i + 1;
            while idx < num_chunks && self.chunks[idx].kind == ChunkKind::Whitespace {
                idx += 1;
            }

            if idx >= num_chunks || self.chunks[idx].kind != ChunkKind::Assign {
                continue;
            }
            idx += 1;

            while idx < num_chunks && self.chunks[idx].kind == ChunkKind::Whitespace {
                idx += 1;
            }

            if idx < num_chunks && self.chunks[idx].kind == ChunkKind::String {
                result.insert(trimmed_name.to_string(), self.chunks[idx].text.to_string());
            }
        }
        result
    }

    pub fn collect_combined_vars(&self, var_names: &[&str]) -> HashMap<String, String> {
        let mut result: HashMap<String, String> = HashMap::new();
        let allowed_names: HashSet<&str> = var_names.iter().cloned().collect();
        let num_chunks = self.chunks.len();

        for i in 0..num_chunks {
            let chunk = &self.chunks[i];
            let trimmed_name = chunk.text.trim();
            if chunk.kind != ChunkKind::Code || !allowed_names.contains(trimmed_name) {
                continue;
            }

            let mut idx = i + 1;
            while idx < num_chunks && self.chunks[idx].kind == ChunkKind::Whitespace {
                idx += 1;
            }

            if idx >= num_chunks || self.chunks[idx].kind != ChunkKind::Assign {
                continue;
            }
            let op = self.chunks[idx].text.trim();
            idx += 1;

            while idx < num_chunks && self.chunks[idx].kind == ChunkKind::Whitespace {
                idx += 1;
            }

            if idx < num_chunks && self.chunks[idx].kind == ChunkKind::String {
                let raw_val: &str = self.chunks[idx].text;
                let mut val_content = raw_val;

                // Quotes entfernen analog zu Python
                if raw_val.len() >= 2 {
                    let first = raw_val.as_bytes()[0];
                    let last = raw_val.as_bytes()[raw_val.len() - 1];
                    if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
                        val_content = &raw_val[1..raw_val.len() - 1];
                    }
                }

                let current_content = result.get(trimmed_name).cloned().unwrap_or_default();
                let new_content = match op {
                    "?=" | ":=" | "=" => val_content.to_string(),
                    "+=" => format!("{}{}", current_content, val_content),
                    "=+" => format!("{}{}", val_content, current_content),
                    "-=" | "=-" => current_content.replace(val_content, ""),
                    _ => continue,
                };

                result.insert(trimmed_name.to_string(), new_content);
            }
        }

        // Am Ende alle Values in Anführungszeichen setzen f'"{val}"'
        result.into_iter().map(|(k, v)| (k, format!("\"{}\"", v))).collect()
    }

    // Helper für die Python-Len- und Index-Methoden
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

// Indexierung wie `list[idx]` in Python
impl<'a> std::ops::Index<usize> for ChunkList<'a> {
    type Output = Chunk<'a>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.chunks[index]
    }
}

// String-Zusammenfassung via `ToString` bzw. `fmt::Display`
impl<'a> std::fmt::Display for ChunkList<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let combined: String = self.chunks.iter().map(|ch| ch.text).collect();
        write!(f, "{}", combined)
    }
}

