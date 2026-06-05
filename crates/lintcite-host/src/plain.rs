//! Plain-text host adapter (plan 04 T3, M1 slice).
//!
//! What: treats each non-empty line as a citation-candidate run (split on
//! `;`), for linting plain citation lists and footnote exports.
//! Depends on: [`crate::candidates`], [`crate::span`].

use crate::candidates::emit_candidates;
use crate::span::{CitationSpan, HostAdapter};

/// The plain-text adapter. Stateless; construct freely.
pub struct PlainAdapter;

impl HostAdapter for PlainAdapter {
    fn extract(&self, text: &str) -> Vec<CitationSpan> {
        let mut out = Vec::new();
        let mut line_start = 0usize;
        for line in text.split_inclusive('\n') {
            let body = line.trim_end_matches(['\n', '\r']);
            emit_candidates(body, line_start, &mut out);
            line_start += line.len();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_line_is_a_candidate_run() {
        let src = "Mabo v Queensland (1992) 175 CLR 1\nnot a citation\nWik v Queensland (1996) 187 CLR 1; Love v Commonwealth [2020] HCA 3\n";
        let spans = PlainAdapter.extract(src);
        assert_eq!(spans.len(), 3);
        for s in &spans {
            assert_eq!(&src[s.range.start..s.range.end], s.text);
        }
    }

    #[test]
    fn empty_and_garbage_input_never_panics() {
        for bad in ["", "\n", ";;;\n;;", "   \n   "] {
            let _ = PlainAdapter.extract(bad);
        }
    }
}
