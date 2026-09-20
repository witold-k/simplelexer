// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    String,
    Code,
    Decoration, // e.g. ======== or ######## ...
    Whitespace,
    Assign,
    Operator,
}

