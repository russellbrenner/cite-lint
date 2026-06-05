# Plan 01 — Data & Rule IR

**Crate:** `cite-lint-data` · **Milestones:** M1 → M2 · **Depends on:** 00 · **Size:** L

## Goal & Definition of Done

Make reference data *and the rules themselves* into versioned, provenance-tracked,
fast-to-load data. This is the layer that lets "ingest a rule-heavy PDF → a lintable rule
set" actually pay off: ingestion (plan 02) writes here; the engine (plan 03) reads here.

**DoD**
- [ ] Table file schema (vocab) with mandatory provenance fields.
- [ ] **Rule IR** schema expressive enough for the case-rule subset (proven on ≥10 rules).
- [ ] Loader: parse + validate (no dangling vocab refs; every rule has an AGLC4 ref + provenance).
- [ ] Compile step: tables → FST/PHF lookups; IR → an executable rule set for `core`.
- [ ] Hand-authored AGLC4 **case** tables + table-tests (unblocks M1 before ingestion exists).
- [ ] Edition selection API (`load(edition_id)`, default `aglc4`).
- [ ] CI check that bans inline vocab in `core` (enforces invariant 2).

## Design context

[`architecture.md`](../architecture.md) §5 (where the IR sits) and §6 (compiled lookups).
*Invariant 2: reference data before rules.* The IR **references** vocab; it never inlines
it, and it is not a parser (*invariant 4 untouched*).

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-DATA-1](../research/README.md#r-data-1) | What predicate vocabulary does the Rule IR need to express the case rules without escaping to Rust? Prototype on 10 rules. | 2–3 days | T2, plan 03 |
| [R-ARCH-2](../research/README.md#r-arch-2) | FST vs PHF; runtime-compile vs `build.rs` codegen | 1 day | T4 |

## Task ladder

- **T1 — Vocab table schema.** Define `data/editions/<id>/tables/*.toml`. A reporter row:
  `abbrev`, `full`, `court`, `jurisdiction`, `year_bracket` (`round`|`square`), plus
  `provenance { page, source_sha, tool, tool_version, reviewer }`. *Check:* schema test
  rejects a row missing provenance.
- **T2 — Rule IR schema.** Define `data/editions/<id>/rules/*.ir.toml`: `id`
  (`AGLC4-CASE-001`), `trigger` (AST node kinds), `predicate` (small typed DSL over AST +
  table lookups), `severity`, `confidence`, `message` (template), `aglc_ref { rule, page }`,
  optional `fix` (typed transform). *Check:* schema test + a `serde` round-trip; IR with an
  unknown predicate op fails validation.
- **T3 — Loader.** Parse tables + IR into typed structs; validate: every `predicate` vocab
  reference resolves to a table; every rule has `aglc_ref` and provenance; codes are unique
  and append-only. Typed errors (`thiserror`), no `unwrap`. *Check:* unit tests for each
  validation failure path.
- **T4 — Compile step.** Tables → `fst`/`phf` lookups (decision from R-ARCH-2); IR → an
  executable `RuleSet` `core` runs. Load once, share via `Arc`. *Check:* benchmark shows
  O(1) lookup; compile is deterministic (same input → same artifact hash).
- **T5 — Hand-author AGLC4 case tables.** Reporters (incl. `CLR` → round-year, High Court),
  round-vs-square year list, courts, jurisdictions — each row provenance-tagged to the PDF
  page. Add **table-tests** asserting known entries so later re-ingestion can't regress them
  silently. *Check:* `CLR → round, High Court` and a square-bracket reporter both asserted.
- **T6 — Edition API.** `load(edition_id) -> Result<EditionTables>`; default `aglc4`; expose
  `meta.toml` fields (label, source sha, self-citation). *Check:* `load("aglc4")` returns
  populated tables; unknown id → typed error.
- **T7 — Ban-inline-vocab guard.** CI test that fails if a controlled term (reporter/court
  abbrev) is string-literalled in `core` instead of read from `data`. *Check:* adding
  `"CLR"` to a `core` rule body fails CI.

## Acceptance gate

Loader + compile covered by tests; case table-tests assert known entries with provenance;
IR validates and round-trips; `load("aglc4")` works; the ban-inline-vocab guard is active in
CI. Re-running ingestion (plan 02) over these tables produces a reviewable diff, not a silent
change.

## Lean notes

- **TOML** for authorability + clean git diffs + human review of ingestion output. Defer a
  binary compiled artifact until a benchmark proves load time matters.
- Hand-author **only** the case subset for M1; let ingestion (plan 02) fill and *audit* the
  rest in parallel rather than blocking on it.
- Keep the predicate DSL **small**: add an op only when a real AGLC rule needs it; complex
  one-offs stay Rust (still reading vocab from data).

## Risks & mitigations

- *IR too weak / too baroque* → R-DATA-1 prototype gates the schema before committing; the
  escape hatch (Rust rule) keeps weak-IR from blocking correctness.
- *Provenance rot* → provenance is schema-mandatory and table-tested, not optional.
