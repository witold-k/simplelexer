// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use crate::chunkkind::ChunkKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub kind: ChunkKind,
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

impl<'a> Chunk<'a> {
    pub fn new(kind: ChunkKind, text: &'a str, start: usize, end: usize) -> Self {
        Self { kind, text, start, end }
    }

    pub fn from_str(kind: ChunkKind, text: &'a str) -> Self {
        Self { kind, text, start: 0, end: text.len() }
    }


    pub fn last_line(&self) -> &str {
        self.text.lines().next_back().unwrap_or("")
    }
}

use std::fmt;
impl<'a> fmt::Display for Chunk<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Chunk {:?} [{}:{}] {:?}>", self.kind, self.start, self.end, self.text)
    }
}

