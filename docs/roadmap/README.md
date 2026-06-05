# cite-lint — Engineering Roadmap & Executable Plans

**Date:** 2026-06-05 · **Status:** Draft for review · **Owner:** Russ

This directory is the **plan of record** for building `cite-lint`. It extends the
[design doc](../../README.md) and the invariants in [`CLAUDE.md`](../../CLAUDE.md) into
something an engineer — or a self-improving agent loop (§[10](plans/10-dev-loops-mem0.md))
— can execute top-to-bottom.

It has three layers:

1. **Backbone** (strategic): why & shape.
   [`00-principles.md`](00-principles.md) · [`architecture.md`](architecture.md) · [`milestones.md`](milestones.md)
2. **Executable plans** (one per build layer): step-by-step task ladders with concrete
   commands, acceptance checks, and inline research spikes — under [`plans/`](plans/).
3. **Research register** (the unknowns to resolve to stay *lean*): time-boxed spikes,
   each unblocking a decision — [`research/README.md`](research/README.md).

> The design doc says *what* cite-lint is. The backbone says *how it's shaped and in
> what order*. The plans say *exactly what to do to build each layer*. The research
> register says *what we must learn first to keep it lean*.

---

## 1. Vision (unchanged, restated)

A citation linter is a **trust product**. Priorities, in order:

1. **Correct** — deterministic enforcement off versioned, provenance-tracked rule data;
   every rule cites its AGLC4 rule; never confidently wrong.
2. **Fast** — per-citation isolation, incremental host parsing, a compiled rule set;
   live linting inside a per-keystroke budget.
3. **Embeddable, lean & easy** — a library-first engine any app calls in-process
   (Rust, Python, JS/WASM, C ABI) or over a network API; CLI and LSP are thin shells
   over that same library, so behaviour is **identical everywhere**; `cite-lint check .`
   just works with zero config.

## 2. Build layers → executable plans

Each layer has a self-contained, executable plan. Build order is encoded in
[`milestones.md`](milestones.md); dependencies are listed at the top of each plan.

| # | Plan | Layer / crate(s) | Headline approach |
|---|------|------------------|-------------------|
| 00 | [Foundation](plans/00-foundation.md) | workspace, CI seam | Empty-but-green workspace; invariants enforced by a deps test; SDK seam + parity harness stubbed. |
| 01 | [Data & Rule IR](plans/01-data-and-rule-ir.md) | `cite-lint-data` | Versioned tables **+ a declarative Rule IR**, compiled to fast lookups (FST/PHF). |
| 02 | [Ingestion ⭐](plans/02-ingestion.md) | `cite-lint-ingest` | **LLM-assisted** PDF → tables + Rule-IR drafts, gated by deterministic verification. The lean digestion path. |
| 03 | [Core engine](plans/03-core-engine.md) | `cite-lint-core` | chumsky parser → typed AST → IR-driven rule engine → document pass. |
| 04 | [Host adapters](plans/04-host-adapters.md) | `cite-lint-host` | tree-sitter (md/latex) live; pdf/docx batch; citation-span extraction + range mapping. |
| 05 | [SDK & bindings](plans/05-sdk-and-bindings.md) | `cite-lint` + bindings | The one behaviour surface; Python/WASM/C/Node bindings wrap it. Parity by construction. |
| 06 | [Surfaces](plans/06-surfaces.md) | `-cli`, `-lsp`, `-mcp` | Thin **local** shells over the SDK; text/JSON/SARIF; LSP diagnostics + tokens + code actions; **MCP (stdio)** for agents. |
| 07 | [Testing & conformance](plans/07-testing-and-conformance.md) | all | Pyramid + **AGLC4 conformance corpus**; property/fuzz/mutation; e2e; coverage gates. |
| 08 | [Security & CI/CD](plans/08-security-cicd.md) | all | Threat model, supply-chain hardening, signed/attested releases, least-privilege OIDC CI. |
| 09 | [Documentation](plans/09-documentation.md) | docs site | Diátaxis set, **docs-from-code**, onboarding, CI'd samples. |
| 10 | [Dev loops & mem0](plans/10-dev-loops-mem0.md) | dev platform | Bounded, gated parallel loops on worktrees sharing local **mem0** memory. |
| 11 | [Agent skills pack](plans/11-agent-skills-pack.md) | dev platform | One skill core → generated adapters for Claude / Codex / Hermes / OpenClaw. |
| 12 | [Service, scale & MCP](plans/12-service-scale-and-mcp.md) | `-server`, `-mcp` | Large-scale **secure multi-tenant** service + remote MCP; **one artifact, locally runnable**; **near-zero cold-start**. |

⭐ = the layer the brief calls out for depth (ingestion, LLM-assisted, lean).

## 3. Milestones at a glance

| Milestone | Theme | Design tie-in | Plans advanced |
|-----------|-------|---------------|----------------|
| **M0** | Foundation: workspace, CI, SDK seam, parity harness, loop/skill skeletons | — | 00, (07/08/10/11 stubs) |
| **M1** | Proving slice: cases · Markdown · LSP · SDK (parity) | v1 | 01, 03, 04, 05, 06, 07 |
| **M2** | Ingestion v1 + Rule IR; reproduce the case tables from the PDF | §7 | 01, 02, 10 |
| **M3** | Legislation & treaties; CLI + SARIF; **MCP (stdio)**; Python + WASM bindings | v2 | 01, 03, 05, 06, 08 |
| **M4** | Document pass: ibid / above-n / short titles / signals | v3 | 03, 07 |
| **M5** | Secondary sources; LaTeX host adapter | v4 | 01, 03, 04 |
| **M6** | PDF/docx batch; **secure at-scale service + remote MCP**; near-zero cold-start | v5 | 04, 06, 08, 12 |
| **M7** | GA / 1.0: semver-frozen SDK, full docs, signed release, all skill adapters, hardened service | — | 05, 08, 09, 11, 12 |

Detail, dependencies, exit criteria, KPIs, risks: [`milestones.md`](milestones.md).

## 4. The leanness contract

Everything here is filtered through *lean + easy to use*:

- **Fewest crates that preserve the invariants.** Bindings, server, and extra hosts are
  **feature-gated and deferred** until a milestone needs them (see each plan's *Lean notes*).
- **Buy don't build, but only memory-safe + auditable.** Prefer a vetted crate over a
  bespoke parser — *except* where determinism/trust require our own (the citation parser).
- **LLMs do the toil, gates own the truth.** Ingestion may be LLM-assisted; enforcement
  never is. Every LLM output passes a deterministic verification gate before it's committed.
- **One behaviour surface.** No logic duplicated across CLI/LSP/MCP/SDK/server → less to test,
  document, and keep in sync.
- **One artifact, many run modes.** Embedded → local binary → local **MCP** (stdio) → clustered
  secure service is *configuration*, not new code; **near-zero cold-start** (mmap'd precompiled
  tables, no warmup) makes scale-to-zero a deployment choice, so "large-scale service" and
  "locally runnable" cost one build, not two.
- **Zero-config default.** `cite-lint check .` and a 5-line SDK snippet must work before
  any flag is learned.

## 5. Success metrics (tracked from M0)

- **AGLC-rule coverage** — % of AGLC4 rules implemented *and* covered by ≥1 positive and
  ≥1 negative conformance case. The headline trust metric.
- **Line coverage** ≥ 90% (ratcheting) + **mutation score** on core rules.
- **Live-lint latency** — p99 single-footnote re-lint < 5 ms (target).
- **Cold-start** — fresh / scaled-from-zero service replica ready < 50 ms (mmap'd precompiled tables).
- **Parity** — 100% of SDK capabilities reachable from CLI and vice versa.
- **Determinism** — identical (input, edition) ⇒ byte-identical diagnostics.
- **Supply chain** — zero unreviewed advisories; signed release + SBOM on every tag.
- **Loop health** — green-rate, time-to-green, mem0 retrieval hit-rate.

## 6. How to read / execute

- New here? → [`00-principles.md`](00-principles.md) → [`architecture.md`](architecture.md).
- Building a layer? → its plan in [`plans/`](plans/) + [`milestones.md`](milestones.md) for order.
- Each plan ends with **Acceptance gate** (done = CI-checkable true) and **Lean notes**.
- Research spikes are inline in each plan **and** aggregated in
  [`research/README.md`](research/README.md) with time-boxes and the decision each unblocks.
- Invariant-touching choices are flagged *“(consistent with invariant N)”* → `CLAUDE.md`.
