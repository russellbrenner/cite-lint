# Plan 03 — Core engine

**Crate:** `cite-lint-core` · **Milestones:** M1 (cases) → M3 (legislation) → M4 (doc pass)
→ M5 (secondary) · **Depends on:** 00, 01 · **Size:** L

## Goal & Definition of Done

The deterministic heart: a citation string + kind hint → typed AST → diagnostics, with a
document pass for cross-references. Grows by source type across milestones; the engine
stays format/edition/host-agnostic throughout.

**DoD (per milestone slice)**
- [ ] M1: lexer + chumsky **case** parser → typed AST (error-recovering); IR rule engine
      runs plan 01's compiled rule set; ≥1 safe fix-it; case rules with fixtures.
- [ ] Disambiguation order (case → legislation → secondary → unclassified) in **one** place,
      documented + tested per branch and for the fall-through.
- [ ] Low-confidence "could not classify" path instead of confident-wrong output.
- [ ] M3 legislation, M5 secondary added **additively** (new AST nodes + IR, no rewrite).
- [ ] M4 document pass (ibid / above-n / short titles / signals) over ordered ASTs only.
- [ ] criterion benchmark baseline + regression gate.

## Design context

[`architecture.md`](../architecture.md) §6 (perf) and the invariants: per-citation isolation
(3), parser strategy chumsky-only for citations (4), reads vocab/IR from `data` (2), one
`Diagnostic` model (5). Cross-citation logic lives **only** in the document pass (3).

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-ARCH-1](../research/README.md#r-arch-1) | chumsky error-recovery ergonomics for the citation grammar (partial AST on malformed input) | 2 days | T2 |
| [R-CORE-1](../research/README.md#r-core-1) | Disambiguation: how case/legislation/secondary classification consults tables mid-parse without coupling | 1 day | T3 |

## Task ladder

- **T1 — Lexer + tokens.** Borrowed-`&str` tokeniser for citation atoms (names, years,
  brackets, abbrevs, pinpoints, punctuation). *Check:* token golden tests; zero alloc on the
  happy path (verified by a bench/alloc test).
- **T2 — Case parser (chumsky).** Grammar → typed AST (`CaseName`, `Year{bracket}`,
  `Reporter`, `Pinpoint`, `Court`, `Jurisdiction`, `Signal`, `ShortTitle`). **Error recovery
  yields a partial AST.** *Check:* golden-AST tests for well-formed + malformed inputs (parser
  never panics — property test, plan 07).
- **T3 — Disambiguation order.** One documented function: try case → legislation → secondary
  → `Unclassified(Low confidence)`, consulting `data` lookups. *Check:* a test per branch +
  the fall-through case (correctness guardrail).
- **T4 — IR rule engine.** Execute plan 01's compiled `RuleSet` over an AST → `Vec<Diagnostic>`.
  Each rule is a small named unit; the engine is the runner. *Check:* a synthetic IR rule fires
  exactly on its trigger and emits the templated message + AGLC ref.
- **T5 — Case rules.** Round/square-year bracket, reporter punctuation, italicisation,
  element ordering, court/jurisdiction placement. Each rule: AGLC4 ref in code + message,
  compliant **and** non-compliant fixtures asserting the exact diagnostic, ≥1 with a fix-it.
  *Check:* `(1992) 175 CLR 1` passes; `[1992] 175 CLR 1` flags the bracket rule with a fix-it
  to round brackets.
- **T6 — Fix-it engine.** Safe transforms only (never change cited authority). *Check:*
  property test — applying a fix-it makes the doc pass *that* rule and re-running yields no new
  diagnostics (idempotence); a fix never alters reporter/volume/page.
- **T7 — Document pass (M4).** Stateful resolver over the **ordered** list of parsed ASTs:
  ibid, above-n, short titles, signals. Re-run only on order/reference change. *Check:* ordered
  footnote sequences exercise ibid/above-n; a per-citation rule that tries to read a sibling
  fails the architecture test (plan 07).
- **T8 — Legislation & secondary (M3/M5).** Add AST nodes + IR + rules additively; extend the
  disambiguation branches + tests. *Check:* legislation/secondary fixtures pass without
  regressing case tests; disambiguation tests updated.
- **T9 — Benchmarks.** criterion: parse+check one citation; document-pass over N citations.
  *Check:* baseline recorded; CI flags > X% regression (plan 08).

## Acceptance gate

End-to-end (M1): a case citation parses to a golden AST and produces the exact expected
diagnostics + fix-its; malformed input recovers to a partial AST without panic; disambiguation
branches + fall-through tested; fix-its proven idempotent + meaning-preserving; bench baseline
recorded. Later slices add legislation/secondary/doc-pass without regression.

## Lean notes

- **Cases only for M1.** Resist building all source types up front; each is additive.
- **Most rules are data** (plan 01 IR), so this crate stays small — only algorithmic rules are
  hand-written Rust, and they still read vocab from `data`.
- Keep the AST minimal and grow it per source type; don't pre-model nodes no rule uses yet.

## Risks & mitigations

- *AGLC ambiguity edge cases* (design risk #3) → emit `Unclassified` low-confidence, never
  guess; disambiguation order explicit + tested.
- *Parser complexity creep* → golden-AST tests + error-recovery property tests pin behaviour;
  the IR keeps rule logic out of the parser.
