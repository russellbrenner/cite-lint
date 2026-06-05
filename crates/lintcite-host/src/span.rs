//! The span contract between hosts and the engine (plan 04 T1).

use lintcite_core::SourceRange;

/// Which host formats the M1 slice supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKind {
    /// Markdown documents (footnote definitions and inline footnotes).
    Markdown,
    /// Plain text (line/`;`-separated citation lists).
    Plain,
}

/// A hint about what kind of citation a span likely holds. The engine's
/// disambiguation order (core `classify`) makes the final call; the hint
/// exists so future adapters (e.g. a bibliography host) can narrow it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindHint {
    /// No prior knowledge — try the full disambiguation order.
    Unknown,
}

/// One citation-candidate span extracted from a host document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationSpan {
    /// The candidate text, exactly as it appears in the document.
    pub text: String,
    /// The host-document byte range `text` was sliced from. Diagnostic
    /// ranges are produced by offsetting citation-local ranges by
    /// `range.start`, so fixes apply cleanly to the original bytes.
    pub range: SourceRange,
    /// Citation-kind hint (see [`KindHint`]).
    pub kind_hint: KindHint,
}

/// A host adapter: document text in, citation spans out.
///
/// Adapters never parse citations (invariant 4 keeps citation grammar in
/// the engine); they only locate spans and preserve ranges. Malformed input
/// must never panic — adapters are fixture- and property-tested on garbage.
pub trait HostAdapter {
    /// Extract citation-candidate spans from `text`.
    fn extract(&self, text: &str) -> Vec<CitationSpan>;
}

/// True when `candidate` is citation-shaped: contains a bracketed
/// four-digit year. Used by adapters to gate noise (prose footnotes)
/// out of the engine.
pub(crate) fn citation_like(candidate: &str) -> bool {
    let bytes = candidate.as_bytes();
    for i in 0..bytes.len().saturating_sub(5) {
        let open = bytes[i];
        if open != b'(' && open != b'[' {
            continue;
        }
        let close = if open == b'(' { b')' } else { b']' };
        if i + 5 < bytes.len()
            && bytes[i + 1..i + 5].iter().all(u8::is_ascii_digit)
            && bytes[i + 5] == close
        {
            // Plausible year range guard, mirroring the parser.
            if let Ok(y) = candidate[i + 1..i + 5].parse::<u32>() {
                if (1500..=2099).contains(&y) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citation_like_accepts_both_bracket_styles() {
        assert!(citation_like("Mabo v Queensland (1992) 175 CLR 1"));
        assert!(citation_like("Love v Commonwealth [2020] HCA 3"));
    }

    #[test]
    fn citation_like_rejects_prose_and_non_years() {
        assert!(!citation_like("See the discussion in chapter 3."));
        assert!(!citation_like("section (12) of the Act"));
        assert!(!citation_like("(9999) out of range"));
        assert!(!citation_like(""));
    }
}
