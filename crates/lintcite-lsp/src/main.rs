//! `lintcite-lsp` — the LSP surface (plan 06 T3).
//!
//! M0/M1 stub: the crate exists so the workspace shape, dependency rules,
//! and parity harness are enforced from the start (plan 00), but the
//! server implementation lands with the R-LSP-1 decision (`tower-lsp` vs
//! `lsp-server`). The binary states this honestly rather than half-working.

use std::process::ExitCode;

fn main() -> ExitCode {
    // Touch the SDK so the dependency direction (surface → facade) is real
    // and the linker keeps the parity relationship honest.
    let edition_count = lintcite::Session::new("aglc4")
        .and_then(|s| s.editions())
        .map(|e| e.len())
        .unwrap_or(0);
    eprintln!(
        "lintcite-lsp: not yet implemented (M1 stub; {edition_count} edition(s) loaded). \
         See docs/roadmap/plans/06-surfaces.md T3 — the LSP server lands \
         after the R-LSP-1 spike."
    );
    ExitCode::from(2)
}
