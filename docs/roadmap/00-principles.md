# 00 — Cross-cutting principles

These principles govern every workstream. They refine — never contradict — the
[`CLAUDE.md`](../../CLAUDE.md) invariants. Where a principle strengthens an invariant,
it says so.

## P1. Library-first, parity by construction

There is exactly **one behaviour surface**: the `cite-lint` SDK facade crate. The CLI,
LSP, network server, and every language binding are *thin shells* that parse input,
call the SDK, and format output. No surface contains linting logic the SDK lacks.

- *Why:* parity stops being a thing we test for and becomes a thing that is true by
  construction. The parity-matrix test ([plan 07](plans/07-testing-and-conformance.md))
  then guards against regressions only.
- *Consistent with invariant 1* (one-way deps): the facade sits **above** `core`/`host`
  and below the surfaces — `data → core → host → cite-lint(SDK) → {cli, lsp, server, ffi}`.
  `core` still depends on nothing upward.

## P2. Data before rules; rules are (mostly) data too

Controlled vocabularies live in `cite-lint-data`. This roadmap extends that: *format
rules themselves* are expressed as declarative **Rule IR** in `cite-lint-data` and
compiled by `cite-lint-core` (plans [01](plans/01-data-and-rule-ir.md)–[03](plans/03-core-engine.md)).
Only genuinely algorithmic rules are hand-written
Rust — and even those read vocab from data.

- *Consistent with invariant 2.* The IR references vocab tables; it never inlines them.
- *Consequence:* ingesting AGLC5 is a data+IR update plus review, not a code rewrite.

## P3. Deterministic enforcement, probabilistic digestion

Enforcement (linting) is 100% deterministic off committed tables/IR. The vector store,
embeddings, and any LLM assistance live **only** in the offline ingestion/authoring
path and the dev loops — never on the lint hot path.

- *Consistent with invariants 2–3 and design §7.* Same input + edition ⇒ same output,
  always; this is a CI-checked property ([plan 07](plans/07-testing-and-conformance.md)),
  not an aspiration.

## P4. Per-citation isolation is the performance contract

A citation parses and validates in isolation; cross-citation logic lives only in the
document pass over already-parsed ASTs. Scale and incrementality derive from this.

- *Restates invariant 3* — listed here because the scale model ([architecture.md](architecture.md))
  and the loops ([plan 10](plans/10-dev-loops-mem0.md)) both depend on it.

## P5. Gates are never lowered — by humans or by loops

`fmt` clean, `clippy -D warnings`, tests green, coverage ≥ threshold, `cargo-deny`
clean, no `unwrap/expect/panic` in libraries, every rule cites its AGLC4 ref. These
apply identically to human PRs and to the self-improving loops
([plan 10](plans/10-dev-loops-mem0.md)). A loop that
cannot pass the gates does not merge; it records why to mem0 and stops.

## P6. Docs and interfaces are generated from code, not maintained beside it

The rule catalogue, diagnostic-code index, CLI reference, and API reference are
**generated** from the rule registry, the clap definitions, and rustdoc. Hand-written
docs are reserved for tutorials and explanation, where drift is cheap and human
judgement adds value. CI fails if generated docs are stale.

## P7. Security is a property of the defaults, not an add-on

Untrusted input (documents on the CLI, edits over LSP, requests to the server, bytes
to PDF/docx parsers) is assumed hostile: bounded, fuzzed, resource-limited, never
shelled out to. Confidential legal text never leaves the process by default. Releases
are signed, attested (SLSA), and accompanied by an SBOM. See [plan 08](plans/08-security-cicd.md).

## P8. One diagnostic model, everywhere

Every surface and binding renders the same `Diagnostic` (range, message, AGLC4 rule
ref, severity, optional fix-it, stable diagnostic code). Surfaces translate; they never
invent. *Restates invariant 5* — load-bearing for the SDK, server, and docs.

## P9. Provenance or it didn't happen

Every committed table row and Rule-IR entry carries provenance: source edition id, PDF
page/section, extraction tool+version, and reviewer. Every diagnostic code is traceable
to an AGLC4 rule and a test. Trust is auditable, not asserted.

## P10. Additive change; raise architecture, don't smuggle it

New source types, hosts, bindings, and editions are additive on the existing engine. A
change that needs new architecture gets an ADR ([plan 09](plans/09-documentation.md)) and review
*before* code. *Restates
the CLAUDE.md working agreement.*

---

### Principle → enforcement map

| Principle | Enforced by | Plan |
|-----------|-------------|------|
| P1 parity | facade crate boundary + parity-matrix test | [05](plans/05-sdk-and-bindings.md), [07](plans/07-testing-and-conformance.md) |
| P2 data-before-rules | Rule IR in `cite-lint-data`; lint that bans inline vocab | [01](plans/01-data-and-rule-ir.md) |
| P3 deterministic | determinism property test; dep-graph test bans vector store in `core` | [07](plans/07-testing-and-conformance.md) |
| P4 isolation | architecture test: per-citation rules can't see siblings | [07](plans/07-testing-and-conformance.md) |
| P5 gates | shared CI workflow reused by humans and loops | [08](plans/08-security-cicd.md), [10](plans/10-dev-loops-mem0.md) |
| P6 docs-from-code | docs-staleness CI gate | [09](plans/09-documentation.md) |
| P7 security | fuzz targets, `cargo-deny`, resource limits, signing | [08](plans/08-security-cicd.md) |
| P8 one model | single `Diagnostic` type re-exported by every surface | [05](plans/05-sdk-and-bindings.md) |
| P9 provenance | table-tests assert provenance fields; ADR index | [02](plans/02-ingestion.md), [09](plans/09-documentation.md) |
| P10 additive | ADR gate on architectural change | [09](plans/09-documentation.md) |
