# Plan 12 — Service, scale & MCP

**Crates:** `cite-lint-server`, `cite-lint-mcp` · **Milestones:** M6 (service + MCP) → M7
(hardened GA) · **Depends on:** 05, 06, 08 · **Size:** L

> This is the layer for the brief's "**large-scale production where many lint queries are
> processed securely**" — designed so the *same artifact* is also **locally runnable**, including
> as an **MCP server** for agents.

## Goal & Definition of Done

Run the SDK as a secure, multi-tenant, horizontally-scalable service **and** as a zero-config
local binary / local MCP server, from one codebase and one image. No behaviour is added here —
the service is a thin, scalable shell over the capability set (P1, P8).

**DoD**
- [ ] **One artifact, three run modes:** embedded (SDK) · local single binary · clustered service —
      config-driven, no code change between them.
- [ ] **MCP server** exposing the capabilities as tools, over **stdio** (local agent) and
      **HTTP/SSE** (remote), parity-checked.
- [ ] Multi-tenant isolation: stateless per request, per-request resource bounds, per-tenant authn
      + quotas + rate limits.
- [ ] Secure processing: zero-retention + no-content-logging by default; TLS in transit; optional
      encrypted, content-addressed cache.
- [ ] Horizontal scale to the throughput/latency SLOs; load + failure tests pass.
- [ ] Local mode boots with **zero config and zero external deps** (CI-smoked).
- [ ] **Near-zero cold-start:** a fresh replica is ready in ms (mmap'd precompiled tables, no
      warmup), enabling scale-to-zero / autoscaling without a warm-pool.

## Design context

The engine is **stateless + deterministic + offline** (P3, P4) — the ideal substrate for secure
multi-tenant scale: no cross-request state to leak, output is cacheable, and replicas are
interchangeable. [`architecture.md`](../architecture.md) §6 (run modes + scale). The service is
**off the lint hot path** — embedding the SDK is always the fastest option.

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-SRV-1](../research/README.md#r-srv-1) | Transport: HTTP+JSON vs gRPC vs JSON-RPC for the service | 1 day | T1 |
| [R-MCP-1](../research/README.md#r-mcp-1) | MCP transports (stdio local, HTTP/SSE remote) + auth for remote MCP | 1 day | T3 |
| [R-SCALE-1](../research/README.md#r-scale-1) | Scale model: stateless replicas + LB vs a work queue; content-addressed cache design | 2 days | T6 |
| [R-SCALE-2](../research/README.md#r-scale-2) | Multi-tenancy isolation + per-tenant quota/rate-limit model | 1 day | T4 |
| [R-SCALE-3](../research/README.md#r-scale-3) | Near-zero cold-start: mmap'd precompiled tables + static binary + scale-to-zero; optional WASM/edge isolate | 2 days | T11 |

## Task ladder

- **T1 — Service core.** SDK behind a thin transport (R-SRV-1); routes = the capability set;
  publish **OpenAPI**. Stateless workers sharing `Arc`/mmap'd edition tables. *Check:* `POST /lint`
  returns diagnostics golden-equal to the SDK; OpenAPI contract test green (plan 07).
- **T2 — Run modes.** Same binary: `--local` (single process, no deps) ↔ clustered (config-driven,
  N replicas). Document the one-artifact-many-modes story (plan 09). *Check:* `cite-lint-server
  --local` serves with zero config; the clustered config differs only in env, not code.
- **T3 — MCP server.** `cite-lint-mcp` exposes tools = capabilities (`lint`/`parse`/`explain`/
  `fix`/`editions`) over **stdio** (local agents: Claude, Codex, …) and **HTTP/SSE** (remote)
  (R-MCP-1). Drives the skills pack (plan 11). *Check:* MCP `tools/list` matches the capability set;
  `tools/call lint` matches the SDK (parity, plan 07); stdio mode needs no config.
- **T4 — Multi-tenancy + isolation.** Stateless per request (no cross-tenant state by
  construction); per-request CPU/mem/time bounds; per-tenant authn (API key / OIDC / mTLS) +
  quotas + rate limits (R-SCALE-2). *Check:* a tenant over quota gets `429`; a runaway request hits
  its time/mem cap, not the neighbour's latency.
- **T5 — Secure processing.** **Zero-retention by default** (no citation content persisted/logged);
  TLS in transit; optional **content-addressed cache** keyed by `hash(input + edition + config)`
  for repeated queries — opt-in, encrypted, TTL'd, per-tenant keyed (deterministic output makes
  this safe). Reject pathological inputs (size/nesting caps; plan 08 T7). *Check:* logs contain no
  citation text by default; cache hit returns identical diagnostics; an oversized payload is
  rejected pre-parse.
- **T6 — Horizontal scale.** Stateless replicas behind an LB (queue only if load demands —
  R-SCALE-1); autoscale on latency/queue depth; warm start via mmap'd tables; **bulk/batch API +
  streaming** results for many citations/docs per request. *Check:* throughput scales ~linearly
  with replicas up to the SLO; batch endpoint streams incrementally.
- **T7 — Observability (privacy-preserving).** Metrics (throughput, latency percentiles, error
  rate, per-tenant usage), traces, and **structured audit logs without citation content**;
  health/readiness probes. *Check:* a Grafana-style dashboard shows the SLOs; audit log records
  who/when/counts, never content.
- **T8 — Deployment.** Distroless, non-root, **signed** image (plan 08); Helm chart + Compose for
  local; resource requests/limits; graceful shutdown + backpressure. *Check:* image is signed +
  SBOM'd; `docker compose up` serves locally; rolling deploy drains cleanly.
- **T9 — Scale + failure validation.** Load test to throughput/concurrency/tail-latency SLOs (plan
  07 T11); failure tests (replica loss, overload → graceful `429`, slow-loris). *Check:* SLOs met;
  overload degrades gracefully, never corrupts or leaks.
- **T10 — Local-runnable guarantees.** `cite-lint-server --local` and `cite-lint-mcp` (stdio) start
  with zero config/deps; CI smoke proves both boot and serve. *Check:* a cold machine runs both
  with a single command, no network.

- **T11 — Near-zero cold-start.** Engineer the readiness path so a replica serves almost
  immediately: **static binary** (no runtime/JIT warmup), **memory-mapped precompiled edition
  tables** (FST/PHF artifact `mmap`'d, not parsed at boot — plan 01 T4), **no startup network
  deps** (offline engine), lazy per-edition load. Set + measure a **cold-start SLO** (target:
  replica ready < 50 ms; first-request latency within the steady-state budget). Validate
  **scale-to-zero** (cold replica handles the first request without a warm-pool) and prototype a
  **WASM/edge isolate** variant for sub-ms starts (R-SCALE-3, reuses plan 05's WASM binding).
  *Check:* measured cold-start meets the SLO in CI (plan 07 T11); a scaled-from-zero replica
  answers the first request within budget.

## Acceptance gate

The same artifact runs as a zero-config local binary, a local MCP (stdio) server, and N stateless
replicas at the throughput/latency SLOs; **a fresh/scaled-from-zero replica meets the cold-start
SLO (ready in ms via mmap'd precompiled tables, no warm-pool);** the MCP server is parity-checked over stdio + HTTP/SSE;
multi-tenant isolation + quotas + zero-retention are enforced and tested; the image is distroless,
non-root, and signed; load + failure tests pass.

## Lean notes

- **One artifact, config-driven modes** — no separate "local" vs "prod" codebase to maintain.
- **Stateless replicas + LB beats a queue** for leanness; deterministic output gives a *simple*
  content-addressed cache instead of a complex invalidation scheme.
- **MCP stdio = zero-config local agent access**; remote MCP/HTTP only when an org needs it.
- **Reuse the SDK + capability set** — the service invents nothing, so it inherits all the engine's
  tests, parity, and security for free.
- **Cold-start is free leverage:** because the engine is a static binary over mmap'd precompiled,
  read-only tables with no startup deps, replicas are ready in ms — so scale-to-zero / serverless /
  edge are deployment *choices*, not re-architectures. No warm-pool to pay for or keep hot.

## Risks & mitigations

- *Multi-tenant data leakage* → stateless isolation + zero-retention + no-content-logging +
  per-request bounds; isolation is **by construction**, not by policy.
- *DoS / abuse* → rate-limits, size/time caps, fuzzed parsers (plan 04/07), graceful overload.
- *Cache privacy* → opt-in, encrypted, TTL'd, per-tenant keyed; off by default for confidential work.
- *Remote MCP exposure* → authn required for HTTP/SSE; stdio (local) is the default, lowest-surface
  path (R-MCP-1).
