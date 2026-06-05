# Milestones & sequencing

How the [plans](plans/) compose over time. Workstreams run **in parallel**; milestones are the
**integration points** where a coherent slice ships. Calendar is team-size dependent — *order and
exit criteria* matter more than dates.

## Sequencing principles

- **Gates from M0.** fmt/clippy/test/coverage/deny/audit/arch-tests exist before any behaviour, so
  nothing lands ungated (P5).
- **Research-first per milestone.** Each milestone's research spikes (see [register](research/README.md))
  resolve *before* its build tasks — that's how we stay lean (pick the small option early).
- **Vertical slices.** Every milestone is end-to-end through *some* surface, not a horizontal layer
  — M1 is a thin thread (cases · Markdown · LSP · SDK), proving the whole spine before widening.
- **Additive, never rewrite.** New source types / hosts / surfaces / editions extend the engine
  (P10). If a milestone seems to need a rewrite, that's an ADR, not a milestone.

## Plan × milestone matrix

● = primary work · ○ = extends / hardens · (blank) = not yet.

| Plan | M0 | M1 | M2 | M3 | M4 | M5 | M6 | M7 |
|------|----|----|----|----|----|----|----|----|
| 00 Foundation | ● | | | | | | | |
| 01 Data & Rule IR | ○ | ● cases | ● IR+ingest-verify | ○ legis | | ○ secondary | | |
| 02 Ingestion ⭐ | | | ● LLM-assisted | ○ | | ○ | | ○ |
| 03 Core engine | ○ | ● cases | ○ | ● legislation | ● doc pass | ● secondary | ○ | |
| 04 Host adapters | | ● md, plain | | ○ | | ● latex | ● pdf, docx | |
| 05 SDK & bindings | ○ seam | ● facade | | ● py, wasm, ffi | | ○ | ○ | ● semver freeze |
| 06 Surfaces | ○ stubs | ● LSP, CLI | | ● SARIF, MCP stdio | ○ | ○ | ○ | ● editor ext |
| 07 Testing & conformance | ● harness | ● corpus seed | ○ | ○ | ● doc-pass suite | ○ | ● scale/load | ○ |
| 08 Security & CI/CD | ● baseline | ○ | ○ | ○ wheels/signing | | | ● svc hardening | ● signed GA |
| 09 Documentation | ○ skeleton | ● quickstarts | | ○ | | | ○ | ● full set |
| 10 Dev loops & mem0 | ○ skeleton | | ● online | ○ | ○ | ○ | ○ | ○ |
| 11 Agent skills pack | ○ Claude | | ○ ingest skills | ○ MCP wiring | | | | ● all adapters |
| 12 Service, scale & MCP | | | | | | | ● service+remote MCP | ● hardened, cold-start SLO |

## Milestones

### M0 — Foundation
**Goal:** an empty-but-green workspace whose structure enforces the invariants.
**Ships:** [00] workspace + dep-graph guard + Diagnostic model + SDK seam + parity harness;
[07] test/arch-test harness; [08] CI baseline + supply-chain gates; [09] docs skeleton;
[10] loop runner + mem0 stubs; [11] skill-core schema + Claude adapter stub.
**Exit:** CI green on stubs; `just check` runs every gate; dep-graph + ban-inline-vocab guards
active; coverage gate live (ratcheting).

### M1 — Proving slice (design v1)
**Goal:** cases · Markdown · LSP · SDK, end-to-end, with parity.
**Ships:** [01] hand-authored case tables + table-tests; [03] case parser + IR rule engine +
≥1 fix-it; [04] markdown + plain adapters; [05] SDK facade (all capabilities real); [06] LSP
diagnostics + semantic tokens + code actions, minimal CLI; [07] AGLC4 conformance corpus seeded.
**Exit:** a Markdown file lints via SDK, CLI, **and** LSP with **byte-identical** diagnostics +
working fix-its; golden ASTs (incl. malformed) pass; coverage ≥ 90%; bench baseline recorded;
parity test green for cases.

### M2 — Ingestion v1 + Rule IR (design §7)
**Goal:** reproduce the case tables from the PDF, auditably; loops go online.
**Ships:** [02] LLM-assisted fetch→decrypt→segment→extract→**verify oracle**→review→embed;
[01] Rule IR compiler; [10] dev loops online (extraction + test-hardening lanes) with mem0.
**Exit:** `cite-lint-ingest run aglc4` reproduces M1's case tables within *reviewed* diffs; the
worked-example oracle passes; fixture-excerpt pipeline green in CI (no network, no real PDF);
PDF/embeddings provably uncommitted.

### M3 — Legislation, CLI/SARIF, MCP, bindings (design v2)
**Goal:** a second source type, CI-consumable output, agent + multi-language reach.
**Ships:** [03]/[01] legislation & treaties rules+tables; [06] CLI text/JSON/**SARIF** + exit
codes, **MCP stdio** server; [05] Python + WASM + C bindings with packaging CI; [08] signed wheels/
npm via OIDC; [11] skills invoke cite-lint via MCP.
**Exit:** CLI gates a repo via SARIF in CI; an agent lints through MCP with parity-equal results;
Python wheel + WASM package build + pass smoke; parity test covers cases + legislation across
SDK/CLI/LSP/MCP.

### M4 — Document pass (design v3)
**Goal:** cross-reference correctness.
**Ships:** [03] ibid / above-n / short-title / signals resolver over ordered ASTs; [07] ordered-
footnote conformance suite; [04] incremental re-lint validated under edits.
**Exit:** document-pass conformance green; per-citation isolation enforced by the arch test; LSP
p99 latency SLO met under edits.

### M5 — Secondary sources + LaTeX (design v4)
**Goal:** breadth of authorities + a second live host.
**Ships:** [01]/[03] books/journals/reports/online rules+tables; [04] LaTeX adapter; disambiguation
order fully exercised + tested.
**Exit:** secondary-source conformance green; LaTeX functional tests green; disambiguation
fall-through (`Unclassified`) tested.

### M6 — Batch hosts + secure at-scale service (design v5)
**Goal:** finished-document checking + the large-scale, secure, locally-runnable service.
**Ships:** [04] pdf + docx adapters (read-only, fuzzed, zip-bomb-guarded); [12] `cite-lint-server`
+ remote MCP (HTTP/SSE), multi-tenant isolation + quotas + zero-retention, **near-zero cold-start**,
batch/streaming API; [08] distroless signed image + svc hardening; [07] load/scale tests.
**Exit:** batch-check a real docx/PDF corpus; the **same artifact** runs local single-binary and
N stateless replicas at the throughput/latency SLOs; cold-start < 50 ms; load + failure tests pass;
threat model reviewed.

### M7 — GA / 1.0
**Goal:** freeze the contract; make it trustworthy and easy to adopt.
**Ships:** [05] semver-frozen SDK (`cargo-semver-checks` gate); [09] full Diátaxis docs +
generated rule catalogue/CLI/API/OpenAPI/MCP refs; [08] signed + SBOM'd + SLSA-attested release
across binaries/wheels/npm/images; [11] Claude/Codex/Hermes/OpenClaw adapters generated + tested;
[12] hardened service + cold-start SLO; [06] VS Code extension.
**Exit:** 1.0 release checklist green; public API frozen under semver; AGLC-rule coverage at target;
all surfaces (SDK/CLI/LSP/MCP/server/bindings) parity-green.

## KPIs by milestone

| KPI | M1 | M3 | M5 | M7 |
|-----|----|----|----|----|
| AGLC-rule coverage (pos+neg) | cases | + legislation | + secondary | target % |
| Line coverage | ≥ 90% | ≥ 90% (ratchet) | ratchet | ratchet |
| Mutation score (core rules) | baseline | rising | rising | target |
| Live-lint p99 | measured | < 5 ms | < 5 ms | < 5 ms |
| Cold-start (service) | — | — | — | < 50 ms |
| Parity surfaces green | SDK/CLI/LSP | + MCP/bindings | all | all |
| Signed releases + SBOM | — | wheels/npm | — | all artifacts |

## Risk register (extends design §12)

| # | Risk | Impact | Mitigation | Plan |
|---|------|--------|------------|------|
| 1 | WAF-blocked PDF fetch | can't ingest | curl-headers + Playwright fallback + TOFU pin | 02 |
| 2 | PDF table extraction fidelity | wrong rules | worked-example oracle + dual-model + round-trip + human gate + table-tests | 02, 07 |
| 3 | AGLC ambiguity edge cases | confident-wrong | `Unclassified` low-confidence; disambiguation tested | 03 |
| 4 | docx/PDF range mapping lossy | poor locations | report by footnote no.; documented fallback; tested | 04 |
| 5 | Copyright / confidentiality | legal/privacy | never commit PDF/embeddings; local-first; documented data-handling | 02, 08 |
| 6 | LLM hallucinated rule | trust failure | schema-constrained + `verify-aglc-ref` + oracle + human gate | 02, 10 |
| 7 | Binding maintenance burden | drift/cost | one capability def → generated bindings; demand-gated; CI builds all | 05 |
| 8 | API breakage | breaks embedders | `cargo-semver-checks` + `#[non_exhaustive]` + append-only codes | 05 |
| 9 | Multi-tenant data leakage | security incident | stateless isolation + zero-retention + no-content-logging + per-request bounds | 08, 12 |
| 10 | Service DoS / abuse | outage | rate-limit + size/time caps + fuzzed parsers + graceful overload | 08, 12 |
| 11 | AI-loop quality drift | silent regressions | same gates + mutation + oracle + human gate on semantics; never commit red | 10 |
| 12 | mem0 data governance | leakage of derived material | local-first mem0; memories are hints, gates are truth; decay/curation | 10 |
| 13 | OpenClaw format unknown | skills adapter stalls | contract-first adapter; binding confirmed via research, never fabricated | 11 |
| 14 | Supply-chain compromise | tampered artifacts | pinned SHAs + lockfile + vetting + SBOM + SLSA provenance + signing | 08 |
