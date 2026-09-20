// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::collections::{HashMap, HashSet};

use crate::chunkkind::ChunkKind;
use crate::error::Result;
use crate::quotelexer::QuoteLexer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedChunk {
    pub kind: ChunkKind,
    pub text: String,
}

impl OwnedChunk {
    pub fn new(kind: ChunkKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChunkList {
    chunks: Vec<OwnedChunk>,
}

impl ChunkList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_text(text: &str) -> Result<Self> {
        let lexer = QuoteLexer::new(text);
        let chunks = lexer
            .lex()?
            .into_iter()
            .map(|chunk| OwnedChunk::new(chunk.kind, chunk.text))
            .collect();

        Ok(Self { chunks })
    }

    pub fn chunks(&self) -> &[OwnedChunk] {
        &self.chunks
    }

    pub fn replace_value(&mut self, name: &str, value: &str) -> bool {
        let Some(name_idx) = self
            .chunks
            .iter()
            .position(|chunk| chunk.kind == ChunkKind::Code && chunk.text.trim() == name)
        else {
            self.append_assignment(name, value);
            return false;
        };

        let Some(assign_idx) = self.next_non_whitespace(name_idx + 1) else {
            self.append_assignment(name, value);
            return false;
        };
        if self.chunks[assign_idx].kind != ChunkKind::Assignment {
            self.append_assignment(name, value);
            return false;
        }

        let Some(value_idx) = self.next_non_whitespace(assign_idx + 1) else {
            self.append_assignment(name, value);
            return false;
        };
        if self.chunks[value_idx].kind != ChunkKind::String {
            self.append_assignment(name, value);
            return false;
        }

        self.chunks[value_idx].text = value.to_owned();
        true
    }

    pub fn collect_simple_vars(&self, var_names: &[&str]) -> HashMap<String, String> {
        let allowed_names: HashSet<&str> = var_names.iter().copied().collect();
        let mut result = HashMap::new();

        for (index, chunk) in self.chunks.iter().enumerate() {
            let name = chunk.text.trim();
            if chunk.kind != ChunkKind::Code || !allowed_names.contains(name) {
                continue;
            }

            let Some(assign_idx) = self.next_non_whitespace(index + 1) else {
                continue;
            };
            if self.chunks[assign_idx].kind != ChunkKind::Assignment {
                continue;
            }

            let Some(value_idx) = self.next_non_whitespace(assign_idx + 1) else {
                continue;
            };
            if self.chunks[value_idx].kind == ChunkKind::String {
                result.insert(name.to_owned(), self.chunks[value_idx].text.clone());
            }
        }

        result
    }

    pub fn collect_combined_vars(&self, var_names: &[&str]) -> HashMap<String, String> {
        let allowed_names: HashSet<&str> = var_names.iter().copied().collect();
        let mut result = HashMap::<String, String>::new();

        for (index, chunk) in self.chunks.iter().enumerate() {
            let name = chunk.text.trim();
            if chunk.kind != ChunkKind::Code || !allowed_names.contains(name) {
                continue;
            }

            let Some(assign_idx) = self.next_non_whitespace(index + 1) else {
                continue;
            };
            if self.chunks[assign_idx].kind != ChunkKind::Assignment {
                continue;
            }

            let Some(value_idx) = self.next_non_whitespace(assign_idx + 1) else {
                continue;
            };
            if self.chunks[value_idx].kind != ChunkKind::String {
                continue;
            }

            let op = self.chunks[assign_idx].text.as_str();
            let value = strip_matching_quotes(&self.chunks[value_idx].text);
            let current = result.get(name).map(String::as_str).unwrap_or("");

            let combined = match op {
                "=" | ":=" | "?=" => value.to_owned(),
                "+=" => format!("{current}{value}"),
                "=+" => format!("{value}{current}"),
                "-=" | "=-" => current.replace(value, ""),
                _ => continue,
            };

            result.insert(name.to_owned(), combined);
        }

        result
            .into_iter()
            .map(|(name, value)| (name, format!("\"{value}\"")))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    fn next_non_whitespace(&self, mut index: usize) -> Option<usize> {
        while self
            .chunks
            .get(index)
            .is_some_and(|chunk| chunk.kind == ChunkKind::Whitespace)
        {
            index += 1;
        }
        (index < self.chunks.len()).then_some(index)
    }

    fn append_assignment(&mut self, name: &str, value: &str) {
        if self
            .chunks
            .last()
            .is_some_and(|chunk| chunk.kind != ChunkKind::Whitespace || !chunk.text.contains('\n'))
        {
            self.chunks
                .push(OwnedChunk::new(ChunkKind::Whitespace, "\n"));
        }

        self.chunks.extend([
            OwnedChunk::new(ChunkKind::Code, name),
            OwnedChunk::new(ChunkKind::Whitespace, " "),
            OwnedChunk::new(ChunkKind::Assignment, "="),
            OwnedChunk::new(ChunkKind::Whitespace, " "),
            OwnedChunk::new(ChunkKind::String, value),
        ]);
    }
}

fn strip_matching_quotes(value: &str) -> &str {
    if value.len() < 2 {
        return value;
    }

    let bytes = value.as_bytes();
    let quoted = matches!((bytes[0], bytes[value.len() - 1]), (b'"', b'"') | (b'\'', b'\''));

    if quoted {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

impl std::ops::Index<usize> for ChunkList {
    type Output = OwnedChunk;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chunks[index]
    }
}

impl std::fmt::Display for ChunkList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for chunk in &self.chunks {
            f.write_str(&chunk.text)?;
        }
        Ok(())
    }
}
