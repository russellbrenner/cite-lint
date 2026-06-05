# Plan 11 — Agent skills pack

**Deliverable:** `skills/` (core + adapters) · **Milestones:** M0 (Claude adapter) → M7 (all
adapters) · **Depends on:** 10 · **Size:** M

## Goal & Definition of Done

A **portable skills pack** so agent runtimes — Claude, Codex, Hermes, OpenClaw — can drive
cite-lint's workflows (author a rule, ingest an edition, harden tests, …) consistently. One
**runtime-agnostic skill core** is the single source of truth; per-runtime adapters are
**generated** from it (no drift). Skills invoke cite-lint via the **MCP server** (plan 12), so
agents never shell out.

**DoD**
- [ ] Skill core: each skill = a schema-validated manifest (when-to-use, inputs, procedure, gates,
      invariant references).
- [ ] Skill catalogue covering the key workflows, each encoding the guardrails.
- [ ] Generated adapters: Claude (`SKILL.md` + slash commands + hooks), Codex (`AGENTS.md`),
      Hermes (tool-schema JSON), OpenClaw (per its format — confirmed via research).
- [ ] Skills call cite-lint through MCP tools; shared memory via mem0 (plan 10).
- [ ] Pack-conformance test: manifests validate; every adapter is generated, not hand-edited.

## Design context

Parity-by-construction applied to the skills pack itself: one core → generated adapters, like the
SDK → surfaces. Skills reuse the executable plans' procedures + gates (this directory) rather than
re-describing them. MCP gives every runtime a uniform, no-shell way to call the engine.

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-SKILL-1](../research/README.md#r-skill-1) | Concrete skill/tool formats per runtime — Claude `SKILL.md`, Codex `AGENTS.md`, Hermes tool-call JSON, **OpenClaw (format unknown — confirm from its docs)**; define the adapter contract | 2 days | T3 |
| [R-SKILL-2](../research/README.md#r-skill-2) | How skills invoke cite-lint: MCP tools (agent-native) vs SDK binding (embedded) vs CLI — pick per runtime | 1 day | T4 |

## Task ladder

- **T1 — Skill core schema (M0).** Define the neutral manifest: `name`, `when_to_use`/trigger,
  `inputs`, `procedure` (steps), `gates` (which `just` checks must pass), `invariants` (links to
  `CLAUDE.md`). Markdown + machine-readable front-matter, JSON-schema validated. *Check:* a
  manifest missing `gates` or `invariants` fails validation.
- **T2 — Skill catalogue.** Author the core skills, each encoding the guardrails: `add-aglc-rule`,
  `ingest-edition`, `extract-table`, `verify-aglc-ref`, `harden-tests`, `parity-check`,
  `bench-guard`, `triage-false-positive`, `write-binding`, `doc-sync`, `security-sweep`. *Check:*
  each runs its gate and refuses to "finish" red (mirrors plan 10's safety).
- **T3 — Adapter generator.** `skills/` core → per-runtime artifacts (R-SKILL-1): **Claude**
  (`SKILL.md` + slash commands + hooks), **Codex** (`AGENTS.md` + config), **Hermes** (tool-schema
  JSON + system-prompt descriptions), **OpenClaw** (adapter per its confirmed format). Single
  source → no hand-divergence. *Check:* regenerating adapters yields a clean diff; editing a
  generated adapter by hand fails the conformance test (T6).
- **T4 — MCP wiring.** Skills invoke cite-lint via the **MCP server** tools (plan 12) — `lint`,
  `parse`, `explain`, `fix`, `editions` — chosen per runtime (R-SKILL-2). *Check:* a skill lints a
  fixture through MCP and gets the same diagnostics as the SDK (parity, plan 07).
- **T5 — Shared memory.** Skills read/write mem0 (plan 10) so an agent benefits from prior
  learnings. *Check:* a skill consults mem0 before acting and records its outcome.
- **T6 — Pack-conformance test.** Validate every manifest against the schema; assert each adapter
  is byte-for-byte generated from the core. *Check:* green only when no adapter has drifted.
- **T7 — Distribution.** Package the pack with per-runtime install instructions (plan 09). *Check:*
  a fresh runtime can install + invoke a skill end-to-end.

## Acceptance gate

The skill core is schema-validated; the catalogue covers the key workflows; adapters for Claude,
Codex, and Hermes are generated from one source (OpenClaw once its format is confirmed); skills
call cite-lint via MCP with parity-equal results; the pack-conformance test blocks adapter drift.

## Lean notes

- **One skill core, generated adapters** — adding a runtime is an adapter, not a rewrite.
- **Skills reuse the executable plans + `just` gates** — no parallel description of procedures to
  keep in sync.
- **MCP is the uniform call path** — every runtime invokes the engine the same way; no per-runtime
  shell-out glue.
- **Defer OpenClaw's concrete binding** behind R-SKILL-1 rather than guessing its format — the
  adapter contract is defined now; the binding lands when the format is confirmed.

## Risks & mitigations

- *Runtime format churn* → the neutral core absorbs change; only the (small, generated) adapters
  move.
- *Skills bypassing gates* → skills embed the gates from plan 10; an agent that "finishes" red
  fails the skill's own acceptance check.
- *Unknown OpenClaw specifics* → researched + contract-first, never fabricated (correctness over
  confidence).
