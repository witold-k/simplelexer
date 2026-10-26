// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use simplelexer::tablelexer::{DocumentChunk, MarkdownLexer, RowKind, TableType};

#[test]
fn mixed_text_tables_and_spans() {
    let src = "Prefix\n| One | Two |\n| :--- | ---: |\n| α | a\\|b |\nSuffix\n<table><tr><th>A</th><th>B</th></tr><tr><td rowspan='2'>ä</td><td colspan=2>cell</td></tr></table>\nEnd";
    let chunks = MarkdownLexer::new(src).lex();
    assert_eq!(chunks.iter().map(DocumentChunk::text).collect::<String>(), src);
    assert_eq!(chunks.len(), 5);

    let DocumentChunk::Table(md) = &chunks[1] else { panic!("Markdown table expected") };
    assert_eq!(md.table_type, TableType::Markdown);
    assert_eq!(md.rows.len(), 3);
    assert_eq!(md.rows[0].kind, RowKind::Header);
    assert_eq!(md.rows[1].kind, RowKind::Separator);
    assert_eq!(md.rows[2].cells[0].text, "α");
    assert_eq!(md.rows[2].cells[1].text, "a\\|b");

    let DocumentChunk::Table(html) = &chunks[3] else { panic!("HTML table expected") };
    assert_eq!(html.table_type, TableType::Html);
    assert_eq!(html.rows.len(), 2);
    assert_eq!(html.rows[1].cells[0].rowspan, 2);
    assert_eq!(html.rows[1].cells[1].colspan, 2);

    for chunk in &chunks {
        let span = chunk.span();
        assert_eq!(&src[span.start..span.end], chunk.text());
        if let DocumentChunk::Table(table) = chunk {
            for row in &table.rows {
                assert_eq!(&src[row.span.start..row.span.end], row.text);
                for cell in &row.cells {
                    assert_eq!(&src[cell.span.start..cell.span.end], cell.text);
                }
            }
        }
    }
}

#[test]
fn ignores_pipes_and_html_inside_fenced_code() {
    let source = "```md\n| x | y |\n| --- | --- |\n| 1 | 2 |\n<table><tr><td>fake</td></tr></table>\n```\n";
    assert!(MarkdownLexer::new(source).lex()
        .iter().all(|x| matches!(x, DocumentChunk::Text { .. })));
}

#[test]
fn recognizes_inline_code_and_ignores_plain_pipes() {
    let source = "Company Website: url | Phone: 555\n\n| A | B |\n| --- | --- |\n| `a|b` | text |\n";
    let chunks = MarkdownLexer::new(source).lex();
    let tables: Vec<_> = chunks.iter().filter_map(|part| match part {
        DocumentChunk::Table(table) => Some(table),
        _ => None,
    }).collect();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].rows[2].cells[0].text, "`a|b`");
}

#[test]
fn missing_or_incomplete_tables_remain_plain_text() {
    for source in [
        "| A | B |\n| bad separator | -- |\n",
        "<table><tr><td>not closed",
        "",
        "Plain text\n",
    ] {
        let chunks = MarkdownLexer::new(source).lex();
        assert!(chunks.iter().all(|c| matches!(c, DocumentChunk::Text { .. })));
        assert_eq!(chunks.iter().map(DocumentChunk::text).collect::<String>(), source);
    }
}

#[test]
fn html_grid_accounts_for_rowspan_and_colspan_without_mutation() {
    use simplelexer::tablelexer::GridIssueKind;
    let src = "<table><tr><td rowspan='2'>A</td><td>B</td><td>C</td></tr><tr><td colspan='2'>D</td></tr></table>";
    let chunks = MarkdownLexer::new(src).lex();
    let DocumentChunk::Table(table) = &chunks[0] else { panic!("table expected") };
    let original = table.clone();
    let audit = table.audit_grid();
    assert_eq!(audit.width, 3);
    assert_eq!(audit.placements.iter().map(|p| (p.row, p.column, p.colspan)).collect::<Vec<_>>(),
        vec![(0, 0, 1), (0, 1, 1), (0, 2, 1), (1, 1, 2)]);
    assert!(audit.issues.is_empty(), "{:?}", audit.issues);
    assert_eq!(*table, original);

    let sparse = "<table><tr><td>A</td><td>B</td></tr><tr><td>C</td></tr></table>";
    let chunks = MarkdownLexer::new(sparse).lex();
    let DocumentChunk::Table(table) = &chunks[0] else { panic!("table expected") };
    let audit = table.audit_grid();
    assert_eq!(audit.width, 2);
    assert_eq!(audit.issues.len(), 1);
    assert_eq!(audit.issues[0].kind, GridIssueKind::PossibleMissingCells);
    assert_eq!(audit.issues[0].row, 1);
}


#[test]
fn whole_document_analysis_tracks_sections_and_original_offsets() {
    use simplelexer::tablelexer::DocumentAnalysis;
    let src = "# Root\nIntro\n## First\n| A | B |\n| --- | --- |\n| x | y |\nAfter\n```md\n# Not a heading\n```\n## Second\n<table><tr><td>Z</td></tr></table>\nEnd";
    let doc = DocumentAnalysis::parse(src);
    assert_eq!(doc.source, src);
    assert_eq!(doc.blocks.iter().map(DocumentChunk::text).collect::<String>(), src);
    assert_eq!(doc.headings.iter().map(|h| h.text).collect::<Vec<_>>(),
        vec!["Root", "First", "Second"]);
    assert_eq!(doc.tables.len(), 2);
    let first = &doc.tables[0];
    assert_eq!(first.heading_indices.iter().map(|&i| doc.headings[i].text).collect::<Vec<_>>(),
        vec!["Root", "First"]);
    assert_eq!(doc.table(first).table_type, TableType::Markdown);
    assert!(src[first.preceding.start..first.preceding.end].contains("First"));
    assert!(src[first.following.start..first.following.end].contains("After"));
    let second = &doc.tables[1];
    assert_eq!(second.heading_indices.iter().map(|&i| doc.headings[i].text).collect::<Vec<_>>(),
        vec!["Root", "Second"]);
    assert_eq!(doc.table(second).table_type, TableType::Html);
    assert!(second.grid.issues.is_empty());
}


#[test]
fn observations_preserve_full_gaps_and_cell_offsets() {
    use simplelexer::tablelexer::DocumentAnalysis;
    let source = "# Kapitel\n| A | B |\n| --- | --- |\n| eins | zwei |\n\nZwischentext\n\n<table><tr><td colspan='2'>Ende</td></tr></table>\n";
    let document = DocumentAnalysis::parse(source);
    let observations = document.structural_observations();
    assert_eq!(observations.len(), 2);
    let first = &observations[0];
    let second = &observations[1];
    assert_eq!(first.previous_table, None);
    assert_eq!(first.next_table, Some(1));
    assert_eq!(second.previous_table, Some(0));
    assert_eq!(second.next_table, None);
    let gap = first.gap_after.expect("intervening text");
    assert_eq!(Some(gap), second.gap_before);
    assert!(source[gap.start..gap.end].contains("Zwischentext"));
    assert_eq!(first.heading_spans, second.heading_spans);
    assert_eq!(second.grid_width, 2);
    assert_eq!(second.cells.len(), 1);
    assert_eq!(second.cells[0].colspan, 2);
    let cell = second.cells[0].span;
    assert_eq!(&source[cell.start..cell.end], "Ende");
}
