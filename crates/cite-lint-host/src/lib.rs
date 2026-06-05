//! # cite-lint-host
//!
//! Host document adapters (plan 04): locate citation spans inside host
//! documents and hand them to the engine with exact byte ranges. The engine
//! never learns what a host is — adapters produce [`CitationSpan`]s.
//!
//! What: Markdown and plain-text adapters for the M1 slice (LaTeX at M5,
//! PDF/docx at M6, per the milestones).
//! How: call [`adapter_for`] (or an adapter directly) and feed each returned
//! span's text to `cite-lint-core`; offset the resulting diagnostics by the
//! span's range start.
//! Depends on: `cite-lint-core` for the shared range type.
//!
//! The Markdown adapter is a deterministic line scanner, provisional while
//! R-HOST-1 (tree-sitter incremental extraction) is open — see
//! docs/adr/0001-m1-provisional-choices.md. The span contract is what the
//! rest of the system builds on; swapping the extractor is additive.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod candidates;
mod markdown;
mod plain;
mod span;

pub use markdown::MarkdownAdapter;
pub use plain::PlainAdapter;
pub use span::{CitationSpan, HostAdapter, HostKind, KindHint};

/// Select the adapter for a host kind.
pub fn adapter_for(kind: HostKind) -> Box<dyn HostAdapter> {
    match kind {
        HostKind::Markdown => Box::new(MarkdownAdapter),
        HostKind::Plain => Box::new(PlainAdapter),
    }
}

/// Guess the host kind from a file extension (zero-config default:
/// unknown extensions are treated as plain text).
pub fn host_for_path(path: &str) -> HostKind {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".md") || lower.ends_with(".markdown") {
        HostKind::Markdown
    } else {
        HostKind::Plain
    }
}
