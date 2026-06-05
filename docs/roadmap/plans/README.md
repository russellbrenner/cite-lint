# Executable plans

One plan per build layer. Each is a **task ladder** you can execute top-to-bottom:
every task has a concrete action, the files/commands it touches, and an acceptance
check. Research spikes are called out where a decision must be made first.

These plans are also the **backlog the self-improving loops consume**
([plan 10](10-dev-loops-mem0.md)): tasks are sized small and each has a machine-checkable
acceptance, so a loop can pick a task, do it, run its gate, and commit on green.

## Plan index & build order

| # | Plan | Depends on | Milestone | Size |
|---|------|-----------|-----------|------|
| 00 | [Foundation](00-foundation.md) | — | M0 | M |
| 01 | [Data & Rule IR](01-data-and-rule-ir.md) | 00 | M1→M2 | L |
| 02 | [Ingestion ⭐](02-ingestion.md) | 00, 01 | M2 | L |
| 03 | [Core engine](03-core-engine.md) | 00, 01 | M1→M4 | L |
| 04 | [Host adapters](04-host-adapters.md) | 00, 03 | M1→M6 | M |
| 05 | [SDK & bindings](05-sdk-and-bindings.md) | 00, 03 | M1→M7 | L |
| 06 | [Surfaces](06-surfaces.md) | 05 | M1→M6 | M |
| 07 | [Testing & conformance](07-testing-and-conformance.md) | 00 | M0→M7 | L |
| 08 | [Security & CI/CD](08-security-cicd.md) | 00 | M0→M7 | L |
| 09 | [Documentation](09-documentation.md) | 05, 07 | M1→M7 | M |
| 10 | [Dev loops & mem0](10-dev-loops-mem0.md) | 00, 07, 08 | M0→ongoing | M |
| 11 | [Agent skills pack](11-agent-skills-pack.md) | 10 | M0→M7 | M |
| 12 | [Service, scale & MCP](12-service-scale-and-mcp.md) | 05, 06, 08 | M6→M7 | L |

Size: S ≈ days, M ≈ 1–2 weeks, L ≈ multi-milestone (delivered in slices). Estimates are
indicative and team-size dependent; sequencing matters more than calendar — see
[`../milestones.md`](../milestones.md).

## Plan format (every plan follows this)

```
# Plan NN — <Layer>
Crate(s) · Milestone(s) · Depends on · Size

## Goal & Definition of Done   — one paragraph + a crisp DoD bullet list
## Design context              — short; links to architecture.md / principles, no duplication
## Research spikes             — table: ID · question · time-box · unblocks  (also in research/)
## Task ladder                 — T1..Tn: action / files+commands / acceptance check
## Acceptance gate             — the CI-checkable exit criteria for the whole plan
## Lean notes                  — what we defer/cut to stay lean & easy
## Risks & mitigations
```

### Task convention

- **T-IDs are stable** (`T1`, `T2`, …) so loops, commits, and the status board can
  reference them (e.g. commit `core: T3 round-year bracket rule`).
- Each task is **independently verifiable** — its acceptance check is a command or a test
  that returns pass/fail. No task is "done" without a green check.
- A task that adds a rule **always** ships its compliant + non-compliant fixtures and an
  AGLC4 rule reference (CONTRIBUTING, *invariant: data before rules*). No exceptions for loops.

## Status board

Update this as plans land. (Loops keep this current; humans can too.)

| Plan | Status | Last milestone reached | Notes |
|------|--------|------------------------|-------|
| 00 Foundation | ☐ not started | — | unblocks everything |
| 01 Data & Rule IR | ☐ not started | — | hand-author first, ingest-verify later |
| 02 Ingestion | ☐ not started | — | LLM-assisted; gated by verification |
| 03 Core engine | ☐ not started | — | cases first |
| 04 Host adapters | ☐ not started | — | markdown first |
| 05 SDK & bindings | ☐ not started | — | facade in M1; bindings M3+ |
| 06 Surfaces | ☐ not started | — | LSP first, then CLI |
| 07 Testing & conformance | ☐ not started | — | corpus from M1 |
| 08 Security & CI/CD | ☐ not started | — | CI baseline in M0 |
| 09 Documentation | ☐ not started | — | docs-from-code |
| 10 Dev loops & mem0 | ☐ not started | — | skeleton M0, online M2 |
| 11 Agent skills pack | ☐ not started | — | Claude adapter first |
| 12 Service, scale & MCP | ☐ not started | — | one artifact, run modes; near-zero cold-start |

Legend: ☐ not started · ◐ in progress · ☑ milestone-complete · ⚠ blocked (see notes).
