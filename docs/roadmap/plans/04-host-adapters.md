# Plan 04 — Host adapters

**Crate:** `cite-lint-host` · **Milestones:** M1 (md, plain) → M5 (latex) → M6 (pdf, docx) ·
**Depends on:** 00, 03 · **Size:** M

## Goal & Definition of Done

Locate citation spans inside host documents and map engine diagnostics back to host ranges.
Live hosts (Markdown, LaTeX) use tree-sitter **incremental** reparse; batch hosts (PDF, docx,
plain) are read-only. The engine never learns what a host is — adapters hand it spans.

**DoD**
- [ ] `HostAdapter::extract(bytes) -> Vec<CitationSpan>` with `{ text, host_range, kind_hint }`.
- [ ] Markdown + plain adapters (M1) with fixture-tested spans + ranges.
- [ ] Incremental reparse path for LSP: an edit re-extracts only the changed region.
- [ ] LaTeX (M5); PDF + docx (M6) — read-only, **fuzzed**, resource-limited.
- [ ] Per-format fixture tests: document → expected spans + ranges, incl. malformed input.

## Design context

Design §4 (host adapters) and §3 (incremental host parsing is what needs to be cheap; the
*citation* re-parse is already tiny per invariant 3). tree-sitter is for **host structure
only** (invariant 4). Binary formats may report by footnote number when char ranges are lossy
(design risk #4).

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-HOST-1](../research/README.md#r-host-1) | tree-sitter incremental extraction of footnote/citation spans for md + latex; range mapping | 2 days | T2, T7 |
| [R-HOST-2](../research/README.md#r-host-2) | PDF/docx range fidelity + docx zip-bomb/resource limits | 2 days | T5, T6 |

## Task ladder

- **T1 — Span contract.** Define `CitationSpan` + the `HostAdapter` trait (shared types in
  `core` so the engine and adapters agree without `core` depending on `host`). *Check:*
  compiles; the dep-graph test still passes (invariant 1).
- **T2 — Markdown adapter (M1).** tree-sitter-md; extract footnote/inline citation spans +
  ranges. *Check:* a fixture `.md` yields the expected spans with correct byte/char ranges.
- **T3 — Plain adapter (M1).** Line/block extraction for plain citation lists. *Check:* fixture
  → expected spans; blank-line/block boundaries handled.
- **T4 — LaTeX adapter (M5).** tree-sitter-latex; `\footnote{}`/citation commands → spans.
  *Check:* fixture `.tex` → expected spans.
- **T5 — PDF adapter (M6).** pdfium/lopdf, **read-only**; extract footnote text + best-effort
  ranges (fall back to footnote number). Fuzzed; size/time limits; memory-safe. *Check:* fixture
  PDF → spans; a fuzz target runs in CI smoke (plan 07/08).
- **T6 — docx adapter (M6).** OOXML unzip, **read-only**; **zip-bomb guard** (decompression
  ratio + total-size caps); footnote parts → spans. Fuzzed. *Check:* a crafted high-ratio zip is
  rejected under the cap; fixture docx → spans.
- **T7 — Incremental reparse.** Wire tree-sitter incremental edits so an LSP keystroke
  re-extracts only the changed region, re-linting one footnote (invariant 3). *Check:* an edit
  to footnote N doesn't re-extract footnote M; latency within budget (plan 07 SLO).
- **T8 — Adapter test suite.** Per-format fixtures (well-formed + malformed) → expected spans +
  ranges. *Check:* all green; malformed input never panics.

## Acceptance gate

Markdown + plain adapters extract correct spans/ranges on fixtures and feed the engine end-to-
end (M1); incremental reparse re-lints only the changed footnote; PDF/docx adapters (M6) are
read-only, fuzzed, and resource-limited with range-fallback documented.

## Lean notes

- **Markdown + plain first**; LaTeX (M5) and the binary formats (M6) are deferred — most live
  drafting value is in Markdown.
- **Feature-gate heavy hosts** (pdf/docx, tree-sitter) so the WASM binding (plan 05) can drop
  them and stay small ([R-SDK-2](../research/README.md#r-sdk-2)).
- Reuse one span contract across all hosts; don't special-case the engine per format.

## Risks & mitigations

- *Untrusted binary input* (PDF/docx parse hostile bytes) → fuzz + resource limits + memory-safe
  libs + no shelling out (P7, plan 08).
- *Lossy range mapping in binaries* (design risk #4) → report by footnote number when char
  ranges are unreliable; document the fallback; test it.
