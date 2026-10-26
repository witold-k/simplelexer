// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

//! Lossless structural table recognition for Markdown and HTML.
//! Spans are byte offsets into the original UTF-8 source.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span { pub start: usize, pub end: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableType { Markdown, Html }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind { Header, Separator, Body }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell<'a> {
    pub text: &'a str,
    pub span: Span,
    pub column: usize,
    pub rowspan: usize,
    pub colspan: usize,
    pub header: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow<'a> {
    pub text: &'a str,
    pub span: Span,
    pub kind: RowKind,
    pub cells: Vec<Cell<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table<'a> {
    pub text: &'a str,
    pub span: Span,
    pub table_type: TableType,
    pub rows: Vec<TableRow<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentChunk<'a> {
    Text { text: &'a str, span: Span },
    Table(Table<'a>),
}

impl<'a> DocumentChunk<'a> {
    pub fn text(&self) -> &'a str {
        match self { Self::Text { text, .. } => text, Self::Table(t) => t.text }
    }
    pub fn span(&self) -> Span {
        match self { Self::Text { span, .. } => *span, Self::Table(t) => t.span }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MarkdownLexer<'a> { text: &'a str }

impl<'a> MarkdownLexer<'a> {
    pub const fn new(text: &'a str) -> Self { Self { text } }

    pub fn lex(&self) -> Vec<DocumentChunk<'a>> {
        let source = self.text;
        let rows = source_lines(source);
        let mut result = Vec::new();
        let mut cursor = 0;
        let mut index = 0;
        let mut fence: Option<(u8, usize)> = None;

        while index < rows.len() {
            let line = rows[index];
            let trimmed = source[line.start..line.end].trim();
            if let Some((mark, width)) = fence {
                if fence_marker(trimmed).is_some_and(|(m,n,rest)| m == mark && n >= width && rest.is_empty()) {
                    fence = None;
                }
                index += 1;
                continue;
            }
            if let Some((mark, width, _)) = fence_marker(trimmed) {
                fence = Some((mark, width));
                index += 1;
                continue;
            }

            let start = line.start + source[line.start..line.end].len()
                - source[line.start..line.end].trim_start().len();
            let parsed = parse_html(source, start).or_else(|| parse_markdown(source, &rows, index));
            if let Some(table) = parsed {
                if table.span.start > cursor {
                    result.push(DocumentChunk::Text {
                        text: &source[cursor..table.span.start],
                        span: Span { start: cursor, end: table.span.start },
                    });
                }
                cursor = table.span.end;
                result.push(DocumentChunk::Table(table));
                while index < rows.len() && rows[index].start < cursor { index += 1; }
            } else {
                index += 1;
            }
        }
        if cursor < source.len() {
            result.push(DocumentChunk::Text {
                text: &source[cursor..], span: Span { start: cursor, end: source.len() },
            });
        }
        result
    }
}

fn source_lines(text: &str) -> Vec<Span> {
    let mut lines = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        lines.push(Span { start: offset, end: offset + line.len() });
        offset += line.len();
    }
    lines
}

fn fence_marker(s: &str) -> Option<(u8, usize, &str)> {
    let first = *s.as_bytes().first()?;
    if first != b'`' && first != b'~' { return None; }
    let count = s.as_bytes().iter().take_while(|b| **b == first).count();
    (count >= 3).then_some((first, count, s[count..].trim()))
}

fn trim_span(text: &str, mut span: Span) -> Span {
    while span.start < span.end {
        let c = text[span.start..span.end].chars().next().unwrap();
        if !c.is_whitespace() { break; }
        span.start += c.len_utf8();
    }
    while span.start < span.end {
        let c = text[span.start..span.end].chars().next_back().unwrap();
        if !c.is_whitespace() { break; }
        span.end -= c.len_utf8();
    }
    span
}

fn escaped(bytes: &[u8], position: usize, start: usize) -> bool {
    let mut p = position;
    while p > start && bytes[p-1] == b'\\' { p -= 1; }
    (position-p) % 2 == 1
}

fn pipe_cells<'a>(text: &'a str, row: Span) -> Option<Vec<Cell<'a>>> {
    let bytes = text.as_bytes();
    let line = trim_span(text, row);
    let (mut start, mut end) = (line.start, line.end);
    if start == end { return None; }

    if bytes[start] == b'|' { start += 1; }
    if end > start && bytes[end-1] == b'|' && !escaped(bytes, end-1, start) { end -= 1; }

    let mut separators = Vec::new();
    let mut ticks = 0usize;
    let mut i = start;
    while i < end {
        if bytes[i] == b'`' {
            let mut count = 1;
            while i+count < end && bytes[i+count] == b'`' { count += 1; }
            if ticks == 0 { ticks = count; }
            else if ticks == count { ticks = 0; }
            i += count;
            continue;
        }
        if bytes[i] == b'|' && ticks == 0 && !escaped(bytes, i, start) {
            separators.push(i);
        }
        i += 1;
    }
    if separators.is_empty() && !text[row.start..row.end].trim().starts_with('|') {
        return None;
    }

    let mut cells = Vec::new();
    let mut left = start;
    for right in separators.into_iter().chain(std::iter::once(end)) {
        let span = trim_span(text, Span { start: left, end: right });
        cells.push(Cell {
            text: &text[span.start..span.end], span,
            column: cells.len(), rowspan: 1, colspan: 1, header: false,
        });
        left = right + 1;
    }
    Some(cells)
}

fn is_separator(cells: &[Cell<'_>]) -> bool {
    !cells.is_empty() && cells.iter().all(|c| {
        let value = c.text.trim();
        let dashes = value.trim_start_matches(':').trim_end_matches(':');
        dashes.len() >= 3 &&
        dashes.bytes().all(|b| b == b'-') &&
        value.bytes().all(|b| b == b'-' || b == b':')
    })
}

fn parse_markdown<'a>(source: &'a str, lines: &[Span], i: usize) -> Option<Table<'a>> {
    let header = pipe_cells(source, *lines.get(i)?)?;
    let separator = pipe_cells(source, *lines.get(i+1)?)?;
    if header.len() != separator.len() || !is_separator(&separator) { return None; }

    let mut rows = Vec::new();
    for (index, kind, mut cells) in [
        (i, RowKind::Header, header),
        (i+1, RowKind::Separator, separator),
    ] {
        for cell in &mut cells { cell.header = kind == RowKind::Header; }
        rows.push(TableRow {
            text: &source[lines[index].start..lines[index].end],
            span: lines[index], kind, cells,
        });
    }
    let mut index = i+2;
    while index < lines.len() {
        let Some(cells) = pipe_cells(source, lines[index]) else { break; };
        if cells.len() != rows[0].cells.len() ||
            !source[lines[index].start..lines[index].end].contains('|') { break; }
        rows.push(TableRow {
            text: &source[lines[index].start..lines[index].end],
            span: lines[index], kind: RowKind::Body, cells,
        });
        index += 1;
    }
    let end = rows.last()?.span.end;
    Some(Table {
        text: &source[lines[i].start..end],
        span: Span { start: lines[i].start, end },
        table_type: TableType::Markdown, rows,
    })
}

fn html_tag(source: &str, start: usize) -> Option<(String, bool, usize, String)> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'<') { return None; }
    let mut i = start+1;
    let closing = bytes.get(i) == Some(&b'/');
    if closing { i += 1; }
    let name_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphanumeric() { i += 1; }
    if i == name_start { return None; }
    let tag = source[name_start..i].to_ascii_lowercase();
    if !matches!(tag.as_str(), "table" | "tr" | "td" | "th" |
        "tbody" | "thead" | "tfoot" | "caption" | "col" | "colgroup") { return None; }
    if i < bytes.len() && !matches!(bytes[i], b' ' | b'\n' | b'\r' | b'\t' | b'>' | b'/') {
        return None;
    }
    let attrs_start = i;
    let mut quote = 0u8;
    while i < bytes.len() {
        if quote != 0 {
            if bytes[i] == quote { quote = 0; }
        } else if bytes[i] == b'\'' || bytes[i] == b'"' { quote = bytes[i]; }
        else if bytes[i] == b'>' {
            return Some((tag, closing, i+1, source[attrs_start..i].to_owned()));
        }
        i += 1;
    }
    None
}

fn span_attribute(attrs: &str, key: &str) -> usize {
    let lower = attrs.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(pos) = lower[from..].find(key) {
        let i = from+pos;
        let boundary = (i == 0 || !bytes[i-1].is_ascii_alphanumeric()) &&
            (i+key.len() == bytes.len() || !bytes[i+key.len()].is_ascii_alphanumeric());
        if boundary {
            let mut j = i+key.len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
            if bytes.get(j) == Some(&b'=') {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
                if bytes.get(j) == Some(&b'\'') || bytes.get(j) == Some(&b'"') { j += 1; }
                let start = j;
                while j < bytes.len() && bytes[j].is_ascii_digit() { j += 1; }
                if let Ok(n) = lower[start..j].parse::<usize>() { return n.max(1); }
            }
        }
        from = i+key.len();
    }
    1
}

fn parse_html<'a>(source: &'a str, start: usize) -> Option<Table<'a>> {
    let (name, closing, after, _) = html_tag(source, start)?;
    if name != "table" || closing { return None; }
    let mut pos = after;
    let mut nesting = 1usize;
    let mut closure = None;
    while pos < source.len() {
        if source.as_bytes()[pos] == b'<' && let Some((tag, close, next, _)) = html_tag(source, pos) {
                if tag == "table" {
                    if close {
                        nesting -= 1;
                        if nesting == 0 { closure = Some((pos, next)); break; }
                    } else { nesting += 1; }
                }
                pos = next;
                continue;
        }
        pos += 1;
    }
    let (close_start, end) = closure?;
    let mut rows = Vec::new();
    let mut current_row: Option<(usize, Vec<Cell<'a>>)> = None;
    let mut current_cell: Option<(usize, bool, usize, usize)> = None;
    pos = after;
    nesting = 0;
    while pos < close_start {
        if source.as_bytes()[pos] != b'<' { pos += 1; continue; }
        let Some((tag, close, next, attrs)) = html_tag(source, pos) else {
            pos += 1;
            continue;
        };
        if tag == "table" {
            if close { nesting = nesting.saturating_sub(1); }
            else { nesting += 1; }
            pos = next;
            continue;
        }
        if nesting > 0 { pos = next; continue; }
        match (tag.as_str(), close) {
            ("tr", false) => current_row = Some((pos, Vec::new())),
            ("tr", true) => {
                if let Some((row_start, cells)) = current_row.take() {
                    let kind = if cells.iter().all(|cell| cell.header) {
                        RowKind::Header
                    } else { RowKind::Body };
                    rows.push(TableRow {
                        text: &source[row_start..next],
                        span: Span { start: row_start, end: next }, kind, cells,
                    });
                }
            }
            ("td" | "th", false) if current_row.is_some() => {
                current_cell = Some((
                    next, tag == "th",
                    span_attribute(&attrs, "rowspan"),
                    span_attribute(&attrs, "colspan"),
                ));
            }
            ("td" | "th", true) => {
                if let (Some((begin, header, rowspan, colspan)), Some((_, cells))) =
                    (current_cell.take(), current_row.as_mut()) {
                    let span = Span { start: begin, end: pos };
                    cells.push(Cell {
                        text: &source[begin..pos], span,
                        column: cells.iter().map(|c| c.colspan).sum(),
                        rowspan, colspan, header,
                    });
                }
            }
            _ => {}
        }
        pos = next;
    }
    if rows.is_empty() { return None; }
    Some(Table {
        text: &source[start..end],
        span: Span { start, end }, table_type: TableType::Html, rows,
    })
}


/// A byte-range translation between a source row and its rendered text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowTextMapping {
    pub source: std::ops::Range<usize>,
    pub rendered: std::ops::Range<usize>,
}

/// A plain-text document view with explicit source-to-output row mappings.
/// Parsing and syntax-specific text cleanup stay inside Simplelexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTableText {
    pub text: String,
    pub rows: Vec<RowTextMapping>,
}

/// Render recognized HTML tables into the historical line-oriented table
/// representation, preserving non-table text. Markdown tables are unchanged.
///
/// The output is intended for consumers that need a textual view; the source
/// structure and row spans remain authoritative for ownership decisions.
pub fn normalize_table_text(source: &str) -> NormalizedTableText {
    let mut text = String::with_capacity(source.len());
    let mut rows = Vec::new();
    let mut cursor = 0;
    for chunk in MarkdownLexer::new(source).lex() {
        let DocumentChunk::Table(table) = chunk else { continue };
        if table.table_type != TableType::Html { continue; }
        text.push_str(&source[cursor..table.span.start]);
        for row in &table.rows {
            if row.cells.is_empty() { continue; }
            let cells: Vec<String> = row.cells.iter().map(|cell| {
                cell.text
                    .replace("<br/>", " ")
                    .replace("<br>", " ")
                    .replace("&nbsp;", " ")
                    .replace("&amp;", "&")
            }).collect();
            let rendered_start = text.len();
            if cells[0].trim().eq_ignore_ascii_case("Type") {
                text.push_str("Type | Description | Parameters\n");
            } else if cells[0].trim().len() <= 8
                && cells[0].trim().chars().all(|c| c.is_ascii_alphanumeric())
            {
                text.push_str("Type: ");
                text.push_str(cells[0].trim());
                for cell in cells.iter().skip(1) {
                    text.push_str(" | ");
                    text.push_str(cell.trim());
                }
                text.push('\n');
            } else {
                text.push_str(&cells.join(" | "));
                text.push('\n');
            }
            rows.push(RowTextMapping {
                source: row.span.start..row.span.end,
                rendered: rendered_start..text.len(),
            });
        }
        cursor = table.span.end;
    }
    text.push_str(&source[cursor..]);
    NormalizedTableText { text, rows }
}

#[cfg(test)]
mod normalized_table_text_tests {
    use super::*;

    #[test]
    fn preserves_text_and_maps_rows() {
        let source = "before\n<table><tr><td>Type</td><td>Description</td></tr><tr><td>q</td><td>alpha<br>beta</td></tr></table>\nafter";
        let result = normalize_table_text(source);
        assert_eq!(result.text, "before\nType | Description | Parameters\nType: q | alpha beta\n\nafter");
        assert_eq!(result.rows.len(), 2);
        assert_eq!(&source[result.rows[1].source.clone()], "<tr><td>q</td><td>alpha<br>beta</td></tr>");
        assert_eq!(&result.text[result.rows[1].rendered.clone()], "Type: q | alpha beta\n");
    }

    #[test]
    fn markdown_and_fenced_html_are_unchanged() {
        let source = "| A | B |\n| --- | --- |\n| x | y |\n\n```html\n<table><tr><td>x</td></tr></table>\n```\n";
        let result = normalize_table_text(source);
        assert_eq!(result.text, source);
        assert!(result.rows.is_empty());
    }
}

/// A source-preserving, *observational* projection of a table onto a grid.
/// No cells are inserted, rewritten, or inferred. A short row is a warning,
/// not proof that the original HTML/PDF export was malformed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableGridAudit {
    pub width: usize,
    pub placements: Vec<GridPlacement>,
    pub issues: Vec<GridIssue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPlacement {
    pub row: usize,
    pub cell: usize,
    pub column: usize,
    pub rowspan: usize,
    pub colspan: usize,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridIssueKind {
    /// Fewer occupied columns than the maximum observed table width.
    /// HTML permits sparse rows; this is only a review candidate.
    PossibleMissingCells,
    /// A declared rowspan extends past the last explicit row.
    RowspanBeyondTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridIssue {
    pub kind: GridIssueKind,
    pub row: usize,
    pub span: Span,
    pub occupied_columns: usize,
    pub expected_columns: usize,
}

impl Table<'_> {
    /// Calculate logical column positions, honoring HTML rowspan/colspan.
    /// The original `Cell::column` and all source spans are left unchanged.
    pub fn audit_grid(&self) -> TableGridAudit {
        let mut active: Vec<usize> = Vec::new();
        let mut placements = Vec::new();
        let mut occupied_per_row = Vec::new();
        let mut width = 0usize;
        let mut overflow = Vec::new();

        for (row_index, row) in self.rows.iter().enumerate() {
            // active[c] counts rows including this row covered by an earlier cell.
            let mut occupied: Vec<bool> = active.iter().map(|&n| n > 0).collect();
            let mut cursor = 0usize;
            for (cell_index, cell) in row.cells.iter().enumerate() {
                let colspan = cell.colspan.max(1);
                let rowspan = cell.rowspan.max(1);
                // Find the first contiguous free range large enough for the cell.
                loop {
                    if occupied.len() < cursor.saturating_add(colspan) {
                        occupied.resize(cursor.saturating_add(colspan), false);
                    }
                    if let Some(blocked) = (cursor..cursor + colspan).find(|&c| occupied[c]) {
                        cursor = blocked + 1;
                    } else {
                        break;
                    }
                }
                let column = cursor;
                for slot in occupied.iter_mut().skip(column).take(colspan) {
                    *slot = true;
                }
                placements.push(GridPlacement {
                    row: row_index, cell: cell_index, column,
                    rowspan, colspan, span: cell.span,
                });
                if rowspan > self.rows.len() - row_index {
                    overflow.push(GridIssue {
                        kind: GridIssueKind::RowspanBeyondTable,
                        row: row_index, span: cell.span,
                        occupied_columns: column + colspan,
                        expected_columns: self.rows.len() - row_index,
                    });
                }
                cursor = column + colspan;
            }
            let row_width = occupied.iter().rposition(|&b| b).map_or(0, |i| i + 1);
            width = width.max(row_width);
            occupied_per_row.push(occupied);

            // Decrement existing spans, then install spans originating here.
            for n in &mut active { *n = n.saturating_sub(1); }
            for placement in placements.iter().rev().take_while(|p| p.row == row_index) {
                let duration = placement.rowspan.saturating_sub(1);
                if active.len() < placement.column + placement.colspan {
                    active.resize(placement.column + placement.colspan, 0);
                }
                for slot in active.iter_mut().skip(placement.column).take(placement.colspan) {
                    *slot = duration;
                }
            }
        }

        let mut issues = overflow;
        for (row_index, occupied) in occupied_per_row.iter().enumerate() {
            let count = occupied.iter().filter(|&&b| b).count();
            if count < width {
                issues.push(GridIssue {
                    kind: GridIssueKind::PossibleMissingCells,
                    row: row_index, span: self.rows[row_index].span,
                    occupied_columns: count, expected_columns: width,
                });
            }
        }
        TableGridAudit { width, placements, issues }
    }
}


/// Whole-document analysis. The caller supplies one complete source string;
/// spans and table indices refer to that same source, never to token windows.
#[derive(Debug, Clone)]
pub struct DocumentAnalysis<'a> {
    pub source: &'a str,
    pub blocks: Vec<DocumentChunk<'a>>,
    pub headings: Vec<DocumentHeading<'a>>,
    pub tables: Vec<DocumentTableAnalysis>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentHeading<'a> {
    pub level: usize,
    pub text: &'a str,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DocumentTableAnalysis {
    /// Index into DocumentAnalysis::blocks.
    pub block_index: usize,
    /// Indices of the active ancestor headings, from outermost to innermost.
    pub heading_indices: Vec<usize>,
    /// Source range before this table, ending at its start. May be empty.
    pub preceding: Span,
    /// Source range after this table, starting at its end. May be empty.
    pub following: Span,
    pub grid: TableGridAudit,
}

impl<'a> DocumentAnalysis<'a> {
    pub fn parse(source: &'a str) -> Self {
        let blocks = MarkdownLexer::new(source).lex();
        let mut headings = Vec::new();
        // Headings are only recognized outside tables and fenced code.
        let mut fence: Option<(u8, usize)> = None;
        for block in &blocks {
            let DocumentChunk::Text { text, span } = block else { continue };
            let mut offset = span.start;
            for line in text.split_inclusive('\n') {
                let raw = line.trim_end_matches(['\r', '\n']);
                let trimmed = raw.trim_start_matches(' ');
                let indent = raw.len() - trimmed.len();
                if let Some((mark, width)) = fence {
                    if fence_marker(trimmed).is_some_and(|(m, n, rest)| {
                        m == mark && n >= width && rest.is_empty()
                    }) {
                        fence = None;
                    }
                } else if let Some((mark, width, _)) = fence_marker(trimmed) {
                    fence = Some((mark, width));
                } else if indent <= 3 {
                    let level = trimmed.bytes().take_while(|&b| b == b'#').count();
                    if (1..=6).contains(&level)
                        && trimmed.as_bytes().get(level).is_some_and(u8::is_ascii_whitespace)
                    {
                        let heading = trimmed[level..].trim();
                        let start = offset + indent;
                        headings.push(DocumentHeading {
                            level,
                            text: heading,
                            span: Span { start, end: offset + raw.len() },
                        });
                    }
                }
                offset += line.len();
            }
        }
        let mut tables = Vec::new();
        for (block_index, block) in blocks.iter().enumerate() {
            let DocumentChunk::Table(table) = block else { continue };
            let mut heading_indices = Vec::new();
            for (i, heading) in headings.iter().enumerate() {
                if heading.span.start >= table.span.start { break; }
                while heading_indices.last().is_some_and(|&last: &usize| {
                    headings[last].level >= heading.level
                }) {
                    heading_indices.pop();
                }
                heading_indices.push(i);
            }
            let preceding = Span {
                start: if block_index == 0 { 0 } else { blocks[block_index - 1].span().start },
                end: table.span.start,
            };
            let following = Span {
                start: table.span.end,
                end: blocks.get(block_index + 1)
                    .map_or(source.len(), |next| next.span().end),
            };
            tables.push(DocumentTableAnalysis {
                block_index, heading_indices, preceding, following,
                grid: table.audit_grid(),
            });
        }
        Self { source, blocks, headings, tables }
    }

    pub fn table(&self, analysis: &DocumentTableAnalysis) -> &Table<'a> {
        match &self.blocks[analysis.block_index] {
            DocumentChunk::Table(table) => table,
            DocumentChunk::Text { .. } => unreachable!("table index points to text"),
        }
    }
}


/// A source-grounded observation suitable for comparing with an external
/// resolver. It does not assert that two blocks belong to one logical table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralObservation {
    pub table_index: usize,
    pub table_span: Span,
    pub heading_spans: Vec<Span>,
    pub previous_table: Option<usize>,
    pub next_table: Option<usize>,
    /// Exact intervening source range from the preceding table, if any.
    pub gap_before: Option<Span>,
    /// Exact intervening source range to the following table, if any.
    pub gap_after: Option<Span>,
    pub grid_width: usize,
    pub cells: Vec<StructuralCellObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralCellObservation {
    pub row: usize,
    pub column: usize,
    pub span: Span,
    pub rowspan: usize,
    pub colspan: usize,
}

impl DocumentAnalysis<'_> {
    /// Describe all tables in document order, including neighboring tables,
    /// complete gaps and logical cell coordinates. No repair or identity
    /// inference takes place; offsets always refer to the original source.
    pub fn structural_observations(&self) -> Vec<StructuralObservation> {
        self.tables.iter().enumerate().map(|(i, item)| {
            let table = self.table(item);
            let previous_table = i.checked_sub(1);
            let next_table = (i + 1 < self.tables.len()).then_some(i + 1);
            let gap_before = previous_table.map(|prev| Span {
                start: self.table(&self.tables[prev]).span.end,
                end: table.span.start,
            });
            let gap_after = next_table.map(|next| Span {
                start: table.span.end,
                end: self.table(&self.tables[next]).span.start,
            });
            StructuralObservation {
                table_index: i,
                table_span: table.span,
                heading_spans: item.heading_indices.iter()
                    .map(|&index| self.headings[index].span).collect(),
                previous_table, next_table, gap_before, gap_after,
                grid_width: item.grid.width,
                cells: item.grid.placements.iter().map(|p| StructuralCellObservation {
                    row: p.row, column: p.column, span: p.span,
                    rowspan: p.rowspan, colspan: p.colspan,
                }).collect(),
            }
        }).collect()
    }
}
