# Plan 07 — Testing & conformance

**Crates:** all · **Milestones:** M0 → M7 · **Depends on:** 00 · **Size:** L

## Goal & Definition of Done

Prove the product is *correct, fast, parity-true, and secure at scale* with a layered test
strategy whose backbone is an **AGLC4 conformance corpus**. Tests are first-class deliverables,
not afterthoughts (CONTRIBUTING: a rule without a failing-case test is incomplete).

**DoD**
- [ ] Unit + golden-AST + table tests; property + fuzz + mutation tests.
- [ ] **Functional/e2e** for every surface: CLI, LSP, SDK, bindings, **MCP**, server.
- [ ] **Parity-matrix** test: every capability reachable + golden-equal across all surfaces.
- [ ] **AGLC4 conformance corpus** + AGLC-rule-coverage metric.
- [ ] Architecture tests enforce invariants 1–5 in CI.
- [ ] Coverage ≥ 90% (ratchet) + mutation score on core rules + perf/scale/determinism gates.

## Design context

Determinism (P3) makes everything testable: same (input, edition) → byte-identical output.
The parity test is the structural guarantee behind the SDK/CLI/MCP/server promise (P1, P8).

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-TEST-1](../research/README.md#r-test-1) | Seed the conformance corpus from the guide's worked examples (reuse plan 02 T4 `example` sections) — format + tooling | 1 day | T8 |
| [R-TEST-2](../research/README.md#r-test-2) | Load-test tool (k6 vs oha vs locust) + scale SLO targets for the service | 1 day | T11 |

## Task ladder

- **T1 — Unit conventions.** Each rule: compliant + non-compliant fixtures asserting the exact
  diagnostic (+ fix-it). Table-tests assert known vocab entries with provenance. Loader/IR tests.
  *Check:* a rule PR without a failing-case fixture fails review/CI.
- **T2 — Golden-AST tests.** Well-formed + malformed citations → expected AST / partial AST.
  *Check:* error-recovery goldens exist for each source type.
- **T3 — Property tests (proptest).** (a) parser never panics on arbitrary input; (b) fix-it
  idempotence + meaning-preservation; (c) **determinism**: same (input, edition, config) →
  identical diagnostics. *Check:* all three properties run in CI.
- **T4 — Fuzzing (cargo-fuzz).** Targets: citation parser, host adapters (pdf/docx), IR loader,
  **MCP/server request decoders**. Small committed corpus; crashes become regression tests. PR
  smoke (seconds) + scheduled deep run. *Check:* fuzz smoke green on PR; a known-bad input is a
  pinned regression.
- **T5 — Snapshot tests (insta).** CLI `text`/`json`/`sarif`; LSP responses. *Check:* snapshots
  reviewed on change; no accidental output drift.
- **T6 — Functional / e2e.** CLI via `assert_cmd` over fixture corpora (exit codes, stdin/file/
  glob, config precedence); scripted **LSP** session; SDK e2e; **bindings** smoke (pytest, Node);
  **MCP** protocol conformance (tool-list + tool-call over stdio); **server** OpenAPI contract.
  *Check:* each surface has at least one real end-to-end run in CI.
- **T7 — Parity-matrix test.** Enumerate the capability set; assert each is reachable from every
  surface that should expose it (SDK, CLI, LSP, MCP, server, bindings) and returns golden-equal
  results on a shared corpus. *Check:* adding a capability without wiring a surface fails CI.
- **T8 — AGLC4 conformance corpus.** Curated `(input → expected diagnostics)` pairs from the
  guide's worked examples (R-TEST-1) + community contributions; versioned + edition-tagged.
  Track **AGLC-rule coverage** = % of AGLC4 rules with ≥1 positive **and** ≥1 negative case.
  *Check:* the corpus runs as a test; rule-coverage is reported per PR and ratchets up.
- **T9 — Architecture tests.** Dep-graph (invariant 1); per-citation isolation — a per-citation
  rule cannot read a sibling (invariant 3); ban-inline-vocab (invariant 2); vector-store-absent-
  from-`core` (P3); one-`Diagnostic`-model re-export (invariant 5). *Check:* violating any fails CI.
- **T10 — Coverage + mutation gates.** `cargo-llvm-cov` ≥ 90% line (ratchet, never down);
  rule-coverage gate (every rule has a failing-case test); `cargo-mutants` on `core` rules to
  catch tests that assert nothing (CONTRIBUTING). *Check:* mutation survivors in `core` fail CI.
- **T11 — Perf + scale gates.** criterion micro/throughput regression gate; LSP p99 latency SLO;
  **service load test** (R-TEST-2) for throughput + concurrency + tail-latency SLOs at scale.
  *Check:* a > X% regression or a missed SLO fails the gate.
- **T12 — Determinism gate.** Byte-stable diagnostics across repeated runs and platforms (ties
  to caching in plan 12). *Check:* cross-platform diff is empty.

## Acceptance gate

The full pyramid is green; the conformance corpus runs with AGLC-rule-coverage reported; the
parity test covers SDK/CLI/LSP/MCP/server/bindings; architecture tests enforce the invariants;
coverage + mutation + perf + scale + determinism gates are active in CI.

## Lean notes

- The conformance corpus **grows per rule/milestone** — don't front-load it; every new rule adds
  its pos/neg cases.
- **Mutation testing scoped to `core` rules** (where wrong = trust failure), not the whole tree,
  to keep CI fast.
- Fuzz **smoke in PR, deep on a schedule**; one shared fixture corpus feeds parity + conformance.

## Risks & mitigations

- *Coverage theatre* → mutation testing + reviewers rejecting assertion-free tests
  (CONTRIBUTING "no fabricated tests").
- *Flaky scale tests* → SLOs measured as stable percentiles over fixed corpora; determinism gate
  removes output nondeterminism as a flake source.
