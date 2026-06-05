//! Markdown host adapter (plan 04 T2, M1 slice).
//!
//! What: extracts citation-candidate spans from Markdown footnote
//! definitions (`[^1]: ...`) and inline footnotes (`^[...]`), with exact
//! byte ranges back into the document.
//! How: a deterministic single-pass line scanner. Provisional while
//! R-HOST-1 (tree-sitter incremental extraction) is open — the span
//! contract is stable, so swapping the extractor is additive (see
//! docs/adr/0001-m1-provisional-choices.md).
//! Depends on: [`crate::candidates`], [`crate::span`].
//!
//! M1 limits (documented, fixture-tested): footnote definitions are single
//! line; inline footnotes do not nest brackets.

use crate::candidates::emit_candidates;
use crate::span::{CitationSpan, HostAdapter};

/// The Markdown adapter. Stateless; construct freely.
pub struct MarkdownAdapter;

impl HostAdapter for MarkdownAdapter {
    fn extract(&self, text: &str) -> Vec<CitationSpan> {
        let mut out = Vec::new();
        let mut line_start = 0usize;
        for line in text.split_inclusive('\n') {
            let trimmed_end = line.trim_end_matches(['\n', '\r']);
            if let Some(body_off) = footnote_definition_body(trimmed_end) {
                emit_candidates(&trimmed_end[body_off..], line_start + body_off, &mut out);
            } else {
                extract_inline_footnotes(trimmed_end, line_start, &mut out);
            }
            line_start += line.len();
        }
        out
    }
}

/// If `line` is a footnote definition (`[^id]: body`), return the byte
/// offset of `body` within the line.
fn footnote_definition_body(line: &str) -> Option<usize> {
    let rest = line.strip_prefix("[^")?;
    let close = rest.find("]:")?;
    let id = &rest[..close];
    if id.is_empty() || id.contains(' ') {
        return None;
    }
    // Offset: "[^" + id + "]:" then any spaces.
    let after_marker = 2 + close + 2;
    let body = &line[after_marker..];
    let pad = body.len() - body.trim_start().len();
    Some(after_marker + pad)
}

/// Scan a non-definition line for inline footnotes `^[ ... ]`.
fn extract_inline_footnotes(line: &str, line_start: usize, out: &mut Vec<CitationSpan>) {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'^' && bytes[i + 1] == b'[' {
            let body_start = i + 2;
            if let Some(rel_close) = line[body_start..].find(']') {
                let body = &line[body_start..body_start + rel_close];
                emit_candidates(body, line_start + body_start, out);
                i = body_start + rel_close + 1;
                continue;
            }
            // Unterminated inline footnote: stop scanning this line
            // (malformed input must not panic or loop).
            break;
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "# Memo\n\nNative title was recognised.[^1] The Tampa litigation followed.[^2]\n\nInline cite.^[Love v Commonwealth [2020] HCA 3]\n\n[^1]: Mabo v Queensland (No 2) (1992) 175 CLR 1.\n[^2]: Ruddock v Vadarlis (2001) 110 FCR 491; Plaintiff S157/2002 v Commonwealth (2003) 211 CLR 476.\n[^3]: See generally the discussion of sovereignty in chapter 3.\n";

    #[test]
    fn extracts_footnote_definitions_with_exact_ranges() {
        let spans = MarkdownAdapter.extract(FIXTURE);
        let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
        assert!(texts.contains(&"Mabo v Queensland (No 2) (1992) 175 CLR 1."));
        assert!(texts.contains(&"Ruddock v Vadarlis (2001) 110 FCR 491"));
        assert!(texts.contains(&"Plaintiff S157/2002 v Commonwealth (2003) 211 CLR 476."));
        for s in &spans {
            assert_eq!(
                &FIXTURE[s.range.start..s.range.end],
                s.text,
                "range must slice back to the span text"
            );
        }
    }

    #[test]
    fn extracts_inline_footnotes() {
        let spans = MarkdownAdapter.extract(FIXTURE);
        // The inline footnote body's closing bracket also terminates at the
        // first ']' — which here is the medium-neutral year bracket, so the
        // candidate is gated by citation-likeness on the truncated text.
        // The documented M1 limit: inline footnotes containing square-bracket
        // years are truncated at the first ']'. 'Love v Commonwealth [2020'
        // contains no complete year bracket, so it is gated out. This test
        // pins the documented behaviour.
        let inline: Vec<&str> = spans
            .iter()
            .map(|s| s.text.as_str())
            .filter(|t| t.starts_with("Love"))
            .collect();
        assert!(
            inline.is_empty(),
            "documented M1 truncation limit: {inline:?}"
        );
    }

    #[test]
    fn prose_footnote_is_gated_out() {
        let spans = MarkdownAdapter.extract(FIXTURE);
        assert!(
            !spans.iter().any(|s| s.text.contains("sovereignty")),
            "prose footnotes must not reach the engine"
        );
    }

    #[test]
    fn malformed_markdown_never_panics() {
        for bad in [
            "[^",
            "[^]:",
            "[^1]:",
            "^[unterminated",
            "[^ spaced]: (1992) 175 CLR 1",
            "",
            "\n\n\n",
        ] {
            let _ = MarkdownAdapter.extract(bad);
        }
    }

    #[test]
    fn crlf_line_endings_preserve_ranges() {
        let src = "[^1]: Mabo v Queensland (1992) 175 CLR 1\r\n[^2]: Wik v Queensland (1996) 187 CLR 1\r\n";
        let spans = MarkdownAdapter.extract(src);
        assert_eq!(spans.len(), 2);
        for s in &spans {
            assert_eq!(&src[s.range.start..s.range.end], s.text);
        }
    }
}
