// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    UnterminatedString,
    UnterminatedVariableExpression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub position: usize,
}

impl LexError {
    pub const fn new(kind: LexErrorKind, position: usize) -> Self {
        Self { kind, position }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            LexErrorKind::UnterminatedString => "unterminated quoted string",
            LexErrorKind::UnterminatedVariableExpression => "unterminated variable expression",
        };
        write!(f, "{message} at byte {}", self.position)
    }
}

impl std::error::Error for LexError {}

pub type Result<T> = std::result::Result<T, LexError>;
