# ADR 0001 — M1 provisional implementation choices (zero-dependency slice)

**Date:** 2026-06-05 · **Status:** Accepted (provisional, spike-gated) ·
**Owner:** Russ · **Drafted by:** claude-opus-4.8 (AI-assisted; disclosed per
CONTRIBUTING)

## Context

The M0/M1 slice (workspace foundation → cases · Markdown · CLI) was built
while several roadmap research spikes were still open, and under a build
environment where third-party crate availability could not be assumed. The
roadmap names candidate crates (chumsky, tree-sitter, clap, serde/toml,
fst/phf) but gates each behind a spike (R-ARCH-1, R-HOST-1, R-CLI-1,
R-ARCH-2). CLAUDE.md invariant 4 explicitly allows a hand-written citation
parser ("hand-written / PEG (chumsky)").

## Decision

Ship the M1 slice with **zero external dependencies** across all six crates:

| Concern | Roadmap candidate | M1 choice | Gated by |
|---------|-------------------|-----------|----------|
| Citation parser | chumsky (PEG) | hand-written recursive descent with error recovery | R-ARCH-1 |
| Markdown host extraction | tree-sitter-md (incremental) | deterministic line scanner over the span contract | R-HOST-1 |
| CLI argument parsing | clap | hand-rolled (5 subcommands, 3 flags) | R-CLI-1 |
| Data-file parsing | serde + toml | strict-subset TOML reader (`minitoml`), line-numbered errors | dependency policy |
| Vocab lookups | fst / phf | `BTreeMap` (deterministic iteration) behind the same API | R-ARCH-2 |
| Table loading | runtime file I/O | compile-time `include_str!` embedding | — |

## Consequences

- **Builds are reproducible and supply-chain-clean by construction**: no
  third-party code on the lint path (P7), `cargo-deny` will be trivially
  green, and the engine starts with zero I/O (the near-zero cold-start
  property in architecture §6 holds from day one).
- **The data files remain valid TOML**, so adopting the `toml` crate later
  changes one loader module and no data.
- **Each provisional choice is replaceable behind a stable seam**: the span
  contract (host), the `RuleSet` API (lookups), the capability surface
  (CLI). Swaps are additive changes (P10), made when their spike lands.
- **Costs accepted**: `minitoml` supports a documented subset only; the
  Markdown scanner handles single-line footnote definitions and truncates
  inline footnotes at the first `]` (limits pinned by tests); `BTreeMap`
  lookups are O(log n) — fine at vocabulary scale, re-measured by R-ARCH-2.
- Each spike's resolution must revisit the corresponding row and either
  adopt the candidate crate or close the spike with this implementation
  confirmed.
