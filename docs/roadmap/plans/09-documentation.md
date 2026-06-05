# Plan 09 — Documentation

**Deliverable:** the docs site + `docs/adr/` · **Milestones:** M1 → M7 · **Depends on:** 05,
07 · **Size:** M

## Goal & Definition of Done

A clear, well-structured documentation set with strong onboarding, organised by the
**Diátaxis** framework (tutorials · how-to · reference · explanation), where **reference docs
are generated from code** so they can't drift. First-class SDK/API docs sit alongside the CLI.

**DoD**
- [ ] Docs site (mdBook or Starlight) — versioned, searchable, "edit this page".
- [ ] Onboarding: install + 5-minute quickstart for **each** surface (CLI, LSP, SDK, MCP).
- [ ] **Generated** reference: rule catalogue, diagnostic-code index, CLI reference, SDK/API
      reference (rustdoc + per-binding), server OpenAPI, MCP tool schema.
- [ ] Explanation: architecture, Rule IR, ingestion, disambiguation, perf, security, ADRs.
- [ ] All code samples are CI-tested (doctests / tested snippets) — no rotting examples.
- [ ] Docs CI gates: link check, spell check, `missing_docs` denied, staleness check.

## Design context

Principle P6 (docs-from-code) and P9 (provenance — every diagnostic code traces to an AGLC4
rule + test). The rule catalogue is generated from the rule registry; the CLI reference from
clap; API reference from rustdoc → docs.rs + the site. Drift becomes a CI failure, not a chore.

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-DOCS-1](../research/README.md#r-docs-1) | mdBook vs Astro Starlight; integrating rustdoc + generated reference + tested non-Rust samples | 1 day | T1, T5 |

## Task ladder

- **T1 — Site skeleton + IA.** Stand up the site (R-DOCS-1) with the Diátaxis sections and a
  clear nav. Versioned; search; "edit this page". *Check:* `build` green in CI; nav matches the
  four Diátaxis buckets.
- **T2 — Onboarding & quickstarts.** "Install + lint your first document" per surface: CLI
  (`cite-lint check .`), LSP (editor setup), SDK (5-line snippet), **MCP** (point an agent at the
  server). *Check:* each quickstart is runnable end-to-end by a new user; SDK snippet is a doctest.
- **T3 — Tutorials.** Lint a Markdown thesis live in VS Code; CI-gate a repo of legal memos
  (SARIF); embed the SDK in a Python script; **add a new AGLC rule** (contributor tutorial).
  *Check:* each tutorial's commands run in CI or a tested notebook.
- **T4 — How-to guides.** Configure editions; write a safe fix-it; suppress a diagnostic;
  integrate with GitHub Actions/GitLab; consume SARIF in code scanning; run the MCP/server
  locally; deploy the service (→ plan 12). *Check:* commands validated.
- **T5 — Generated reference.** From code: **rule catalogue** (every AGLC rule, id, what it
  checks, examples, fix-it), **diagnostic-code index** (`AGLC4-CASE-001` → explanation, like an
  error index), **CLI reference** (clap → markdown + man pages), **SDK/API reference** (rustdoc +
  Python/JS API docs), **server OpenAPI**, **MCP tool schema**. *Check:* a staleness gate fails CI
  if generated docs differ from `cargo run -- gen-docs` output.
- **T6 — Explanation / architecture.** Publish the architecture, Rule IR, ingestion pipeline,
  disambiguation order, performance + scale model, and security/threat model as explanation pages
  (sourced from this roadmap + `CLAUDE.md`). *Check:* cross-links resolve (link-check gate).
- **T7 — ADRs.** `docs/adr/` for the load-bearing decisions (parser strategy, edition-as-data,
  parity-by-construction, Rule IR, LLM-assisted ingestion, binding-set, optional service, MCP).
  *Check:* each major architectural choice has a dated ADR; the ADR gate (P10) blocks undocumented
  architecture changes.
- **T8 — Docs CI gates.** Link check, spell check (typos), `missing_docs = deny` on public SDK
  items, doctest run, generated-docs staleness. *Check:* a broken link / undocumented public item
  fails CI.

## Acceptance gate

The site builds and is navigable by Diátaxis section; every surface has a runnable quickstart;
the rule catalogue, code index, CLI ref, API ref, OpenAPI, and MCP schema are **generated** and
staleness-gated; all samples are CI-tested; link/spell/missing-docs gates are active; ADRs cover
the major decisions.

## Lean notes

- **Generate, don't hand-maintain** reference docs — the rule catalogue and code index update
  themselves as rules land, so docs scale for free with the engine.
- Reuse this roadmap's explanation content rather than rewriting it; the roadmap is the draft of
  the architecture docs.
- One docs toolchain, doctest-backed samples — no parallel "examples repo" to keep in sync.

## Risks & mitigations

- *Docs drift* → generated + staleness-gated; samples are doctests; the cost of drift is a red CI,
  caught immediately.
- *Onboarding rot* → quickstarts run in CI, so an install/flag change that breaks onboarding fails
  the build.
