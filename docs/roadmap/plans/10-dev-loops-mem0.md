# Plan 10 — Self-improving dev loops & mem0

**Deliverable:** `tools/loop/` + a local mem0 memory service · **Milestones:** M0 (skeleton)
→ M2 (online) → ongoing · **Depends on:** 00, 07, 08 · **Size:** M

## Goal & Definition of Done

A development pipeline built around **bounded, gated, parallel loops** that consume the
executable plans as their backlog, run on isolated git worktrees, and share a **local mem0
memory** so the system gets better at building itself over time — *without ever lowering a gate
or committing red.*

**DoD**
- [ ] A loop runner: pick a plan task → implement → run `just check` → commit on green /
      roll back on red → record outcome to mem0.
- [ ] Parallel loops on isolated worktrees (no collision); an integration loop merges green branches.
- [ ] Local mem0 with per-lane namespaces; retrieval-augmented action (consult before acting).
- [ ] Human review gate on anything changing rule semantics or edition data.
- [ ] Loop observability: green-rate, time-to-green, coverage + AGLC-rule-coverage trend, mem0 hit-rate.

## Design context

The executable plans (this directory) are deliberately small, machine-checkable tasks — that's
what makes them loop-consumable. Gates are never lowered (P5); enforcement stays deterministic
(P3); AI-authored rules still cite a verified AGLC ref (correctness guardrail). The vector store
from ingestion (plan 02) is reused for memory retrieval.

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-LOOP-1](../research/README.md#r-loop-1) | mem0 local/self-hosted deploy + API + memory schema + decay/curation; keep AGLC-derived material local | 2 days | T3 |
| [R-LOOP-2](../research/README.md#r-loop-2) | Orchestration substrate: git worktrees + runner; Claude Code headless / Codex CLI as executors; how gating is wired | 2 days | T1, T2 |

## Task ladder

- **T1 — Loop runner (M0 skeleton).** `tools/loop/run` picks a task (a plan `T-ID`), invokes an
  executor (R-LOOP-2), runs `just check`, commits on green / discards on red, logs the outcome.
  Safe no-op on a clean tree. *Check:* runner exits 0 and commits nothing on a clean tree; on a
  seeded task it produces a green commit or a clean rollback.
- **T2 — Worktree isolation + parallelism.** Each loop runs in its own `git worktree`/branch so
  lanes run concurrently without collision. *Check:* two loops run in parallel and produce
  independent branches; no shared-tree corruption.
- **T3 — Local mem0 memory.** Self-hosted/local mem0 (R-LOOP-1) with namespaces per lane. Store:
  AGLC interpretations + page provenance, PDF-extraction quirks, recurring clippy/test fix
  patterns, known gotchas (e.g. docx range mapping is lossy), reviewer decisions on edition diffs,
  parity/conformance failures + resolutions. **AGLC-derived material stays local** (P7). *Check:*
  add + search round-trips; a stored extraction quirk is retrievable by a later ingestion task.
- **T4 — Retrieval-augmented action.** Loops query mem0 **before** acting (don't repeat known
  mistakes) and write learnings **after**. *Check:* a task that previously failed and was recorded
  is solved faster on a re-run (measured by time-to-green).
- **T5 — The lanes.** Parallel, each a bounded cycle with its gate: rule-authoring,
  ingestion/extraction, test-hardening, docs-sync, perf-guard, security-sweep, and triage
  (turn a reported false-positive/negative into a conformance case + fix). *Check:* each lane has
  a defined task source (a plan) and acceptance gate.
- **T6 — Integration loop.** Merge green lane branches, resolve conflicts, re-run the full gate.
  *Check:* two green branches integrate with the suite still green; a conflict halts for review.
- **T7 — Safety & governance.** Gates never lowered; rule/edition-data changes are **human-gated**;
  AI-authored rules pass `verify-aglc-ref` (the cited page must contain the rule) + the conformance
  oracle; **no release-signing authority** is delegated to a loop. *Check:* a loop attempting to
  weaken a gate or commit an unverified rule is blocked.
- **T8 — Feedback → memory + regression.** Every failing conformance case, fuzz crash, bench
  regression, coverage gap, doc-link failure, and user-reported FP/FN becomes a mem0 entry **and**
  a regression test. *Check:* a new fuzz crash auto-files a pinned regression + a memory note.
- **T9 — Observability.** A dashboard/metrics file: tasks attempted, green-rate, time-to-green,
  coverage + AGLC-rule-coverage trend, mem0 retrieval hit-rate. *Check:* metrics emitted per loop
  cycle; the status board (plans/README) reflects progress.

## Acceptance gate

A loop picks a plan task, implements it, passes `just check`, commits on green, and records to
mem0; parallel loops run on worktrees without collision; mem0 retrieval measurably speeds repeat
tasks; the human gate blocks semantic/edition-data changes; the loop **never** commits red or
lowers a gate.

## Lean notes

- **Reuse, don't reinvent:** the loops' backlog *is* the executable plans; their gate *is* `just
  check`; their memory store *is* the ingestion vector store. No separate task tracker, CI, or DB.
- **Local-first mem0** — no hosted dependency, keeps confidential material in-house.
- **Start with one lane** (rule-authoring or test-hardening), prove the cycle, then fan out. Don't
  build seven lanes before one works.

## Risks & mitigations

- *AI quality drift* → the same gates as humans + mutation tests + conformance oracle + human gate
  on semantics; a loop that can't pass stops and records why.
- *Memory poisoning / staleness* → curation + decay policy (R-LOOP-1); memories are hints, never
  authority — the deterministic gates remain the source of truth.
- *Runaway loops* → bounded cycles, resource caps, and a human-owned integration gate.
