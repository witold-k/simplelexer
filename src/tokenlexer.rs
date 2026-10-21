// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use crate::chunk::Chunk;
use crate::error::Result;
use crate::quotelexer::lex_impl;

/// Tokenizes input without treating quotes as string delimiters.
#[derive(Debug, Clone, Copy)]
pub struct TokenLexer<'a> {
    text: &'a str,
}

impl<'a> TokenLexer<'a> {
    pub const fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn lex(&self) -> Result<Vec<Chunk<'a>>> {
        lex_impl(self.text, false)
    }
}
