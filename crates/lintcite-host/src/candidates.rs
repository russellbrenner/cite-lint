//! Shared candidate-splitting for text-line hosts.
//!
//! What: turns a run of footnote/line text into citation-candidate spans:
//! splits on `;` (footnotes commonly chain citations), trims whitespace
//! while preserving exact byte offsets, and gates candidates through the
//! citation-likeness check so prose never reaches the engine.
//! Depends on: [`crate::span`].

use lintcite_core::SourceRange;

use crate::span::{citation_like, CitationSpan, KindHint};

/// Split `body` (located at `offset` in the host document) into gated
/// citation-candidate spans, appending to `out`.
pub(crate) fn emit_candidates(body: &str, offset: usize, out: &mut Vec<CitationSpan>) {
    let mut start = 0usize;
    for (i, ch) in body.char_indices() {
        if ch == ';' {
            push_trimmed(&body[start..i], offset + start, out);
            start = i + 1;
        }
    }
    push_trimmed(&body[start..], offset + start, out);
}

/// Trim a candidate, adjust offsets, gate it, and emit.
fn push_trimmed(raw: &str, raw_offset: usize, out: &mut Vec<CitationSpan>) {
    let trimmed_start = raw.len() - raw.trim_start().len();
    let trimmed = raw.trim();
    if trimmed.is_empty() || !citation_like(trimmed) {
        return;
    }
    let start = raw_offset + trimmed_start;
    out.push(CitationSpan {
        text: trimmed.to_string(),
        range: SourceRange::new(start, start + trimmed.len()),
        kind_hint: KindHint::Unknown,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_semicolons_with_exact_ranges() {
        let body = "Mabo v Qld (1992) 175 CLR 1; Wik v Qld (1996) 187 CLR 1";
        let mut out = Vec::new();
        emit_candidates(body, 100, &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].text, "Mabo v Qld (1992) 175 CLR 1");
        assert_eq!(out[0].range.start, 100);
        assert_eq!(out[1].text, "Wik v Qld (1996) 187 CLR 1");
        // The second candidate starts after "; " — exact offset check.
        assert_eq!(
            &body[out[1].range.start - 100..],
            "Wik v Qld (1996) 187 CLR 1"
        );
    }

    #[test]
    fn prose_is_gated_out() {
        let mut out = Vec::new();
        emit_candidates("See above for discussion.", 0, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn empty_segments_are_skipped() {
        let mut out = Vec::new();
        emit_candidates(" ; ; ", 0, &mut out);
        assert!(out.is_empty());
    }
}
