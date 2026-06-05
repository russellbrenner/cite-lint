# Architecture backbone

The shape every plan builds on. This refines the design doc §4 with the SDK facade,
bindings, the network surface, and the Rule IR — without breaking a single invariant.
Execution detail lives in [`plans/`](plans/); this doc is the map.

## 1. Layers and crates

The design doc's six crates plus the additive layers this roadmap introduces. New
crates are **feature-gated and deferred** to the milestone that needs them (leanness).

| Crate | Layer | Responsibility | Depends on | First milestone |
|-------|-------|----------------|------------|-----------------|
| `cite-lint-data` | data | Load edition tables + **Rule IR**; compile to fast lookups; edition selection | — | M0 |
| `cite-lint-core` | engine | Lexer → chumsky parser → AST → IR rule engine → document pass → `Diagnostic` | `data` | M1 |
| `cite-lint-host` | hosts | Adapters (md, latex, pdf, docx, plain) → citation spans + ranges | tree-sitter etc. | M1 |
| `cite-lint` | **SDK facade** | The one behaviour surface: `lint`, `parse`, `explain`, `fix`, edition mgmt | `core`, `host`, `data` | M1 |
| `cite-lint-cli` | surface | `cite-lint` binary; arg parse + format only | `cite-lint` | M1 |
| `cite-lint-lsp` | surface | `cite-lint-lsp` binary; LSP shell over the SDK | `cite-lint` | M1 |
| `cite-lint-ffi` | binding | C ABI (cbindgen) over the SDK | `cite-lint` | M3 |
| `cite-lint-py` | binding | Python (PyO3/maturin) over the SDK | `cite-lint` | M3 |
| `cite-lint-wasm` | binding | wasm-bindgen over the SDK (hosts feature-gated for size) | `cite-lint` | M3 |
| `cite-lint-mcp` | surface | **MCP server** over the SDK: tools = capabilities; stdio (local) at M3, HTTP/SSE at scale in plan 12 | `cite-lint` | M3 |
| `cite-lint-server` | surface | Scalable HTTP/JSON-RPC service over the SDK; OpenAPI; stateless replicas, near-zero cold-start (plan 12) | `cite-lint` | M6 |
| `cite-lint-ingest` | offline | fetch → decrypt → segment → **LLM-assisted extract** → embed → diff | `data` | M2 |

> Node binding (napi-rs) and JVM/Swift/Kotlin (UniFFI) are **candidates**, gated behind
> [R-SDK-1](research/README.md). We ship the smallest binding set that covers demand.

## 2. Dependency graph (one-way; invariant 1)

```
cite-lint-data ──▶ cite-lint-core ──▶ cite-lint-host ──▶ cite-lint (SDK facade)
      ▲                                                       │  every surface & binding wraps
      │ (offline; nothing depends on it)                      │  the SDK — parity by construction
cite-lint-ingest                                              │
              ┌──────────────┬──────────────┬────────────────┼────────────────┬──────────────┐
              ▼              ▼              ▼                ▼                ▼              ▼
        cite-lint-cli  cite-lint-lsp  cite-lint-mcp   cite-lint-server   cite-lint-ffi   py / wasm
           (CLI)          (LSP)       (MCP tools)      (HTTP @ scale)      (C ABI)       (bindings)
```

- **`core` depends only on `data`.** It never sees a host, a surface, a binding, the
  ingest pipeline, the vector store, or an edition-by-name. *(Invariant 1.)*
- **The SDK facade is the only place `core` + `host` + `data` are wired together.**
  Every surface and binding wraps the SDK; none re-implements behaviour. *(Principle P1.)*
- **`ingest` is a leaf off `data`.** Nothing the engine ships depends on it. *(Design §7.)*
- An **architecture test** (plan 07) asserts this graph in CI so it can't silently rot.

## 3. The capability surface (parity by construction)

The SDK exposes a closed, versioned set of **capabilities**. Each is one function with
a typed request/response. The CLI subcommands, LSP requests, **MCP tools**, server routes, and
binding methods are generated-from or mechanically-mapped-to this set, so parity is structural.

| Capability | What it does | CLI | LSP | MCP | Server |
|------------|--------------|-----|-----|-----|--------|
| `lint` | text/bytes + kind hint + edition → `[Diagnostic]` | `check` | `publishDiagnostics` | `tools/call lint` | `POST /lint` |
| `parse` | citation → typed AST (or partial + errors) | `parse` | (internal) | `tools/call parse` | `POST /parse` |
| `explain` | diagnostic code → AGLC4 rule, rationale, examples | `explain` | `hover` / code desc | `tools/call explain` | `GET /rules/{code}` |
| `fix` | apply safe fix-its → edited text + applied set | `fix` | `codeAction` | `tools/call fix` | `POST /fix` |
| `tokens` | typed CST node kinds → semantic tokens | — | `semanticTokens` | — | — |
| `editions` | list/select editions + metadata | `editions` | initialise option | `tools/call editions` | `GET /editions` |

A **parity-matrix test** (plan 07) enumerates this table and asserts every capability is
reachable from every surface that should expose it (SDK, CLI, LSP, MCP, server, bindings), with
golden-equal results on a shared fixture corpus. Adding a capability without wiring its surfaces
fails CI. **MCP tools = capabilities**, so the same parity guarantee covers agents (plan 11).

## 4. The one diagnostic model (invariant 5)

```rust
pub struct Diagnostic {
    pub code: DiagnosticCode,     // stable, e.g. "AGLC4-CASE-001"
    pub message: String,          // human message, names the AGLC4 rule
    pub rule_ref: AglcRuleRef,    // edition id + rule/page, machine-readable
    pub severity: Severity,       // Error | Warning | Info | Hint
    pub range: SourceRange,       // host-space range (may be footnote-no for binaries)
    pub confidence: Confidence,   // High | Low("could not classify") — never confident-wrong
    pub fix: Option<FixIt>,       // safe-only; absent when unsure
}
```

- **Stable diagnostic codes** are the spine of the docs (rule catalogue, error index),
  the conformance corpus, suppression comments, and SARIF `ruleId`. Codes are
  append-only and never reused. *(Principles P8, P9.)*
- `confidence = Low` encodes AGLC ambiguity instead of guessing. *(Correctness guardrail.)*

## 5. Where the Rule IR sits (the lean digestion path)

The IR is the bridge from "unstructured rule-heavy PDF" to "fast lintable rule set":

```
PDF (unstructured) ──ingest(plan 02)──▶ data/editions/<id>/
                                          ├─ tables/*.toml        (vocab: reporters, courts, …)
                                          └─ rules/*.ir.toml      (declarative Rule IR)
                                                     │ load + compile (plan 01/03)
                                                     ▼
                              cite-lint-core: compiled rule set (FST/PHF-backed)
```

- **Vocab tables** stay data (*invariant 2*). **Most format rules** become declarative IR
  entries (trigger node-kinds, predicate over AST+tables, severity, message template,
  AGLC ref, optional fix transform). **Algorithmic rules** stay hand-written Rust but still
  read vocab from data. The IR **references** tables; it never inlines them.
- The IR is **not** a parser and **not** a grammar — it's a rule/condition spec evaluated
  over an already-parsed AST. *(Invariant 4 untouched: chumsky for citations, tree-sitter
  for hosts only.)*
- Consequence: AGLC5 (or a corrected AGLC4 table) is a **data + IR update + review**, not a
  code change. The ingestion pipeline (plan 02) drafts IR; deterministic gates verify it.

## 6. Performance & scale model

Speed is structural (design §3), then mechanically defended:

- **Per-citation isolation** (*invariant 3*): an edit re-lints only the changed footnote;
  the document pass is a linear walk re-run only on order/reference change.
- **Compiled lookups:** vocab tables compile to `fst`/`phf` at load (or build-time
  codegen) → O(1) reporter/court/jurisdiction resolution, no per-call parsing.
- **Borrowed data:** `&str` through the parser; arena/bump allocation for AST nodes;
  interned vocab keys. Allocate at the edges only. *(Rust standards.)*
- **Concurrency:** the engine is `Send + Sync` over an `Arc<EditionTables>` and stateless
  per call → batch CLI fans out with `rayon`; LSP uses `tokio` with request cancellation;
  the server scales horizontally as stateless workers sharing mmap'd tables.
- **Near-zero cold-start** (*plan 12 T11*): a replica needs no warmup — a **static binary**
  (no runtime/JIT) over **memory-mapped precompiled tables** (the `fst`/`phf` artifact is
  `mmap`'d, not parsed at boot) with **no startup network deps** (the engine is offline). A
  fresh replica is ready in ms, so **scale-to-zero / serverless / edge** are deployment
  *choices*, not re-architectures — no warm-pool to keep hot.
- **SLOs (gated in CI, plan 07/08/12):** p99 single-footnote re-lint < 5 ms; batch throughput
  (citations/sec) tracked by `criterion` with a regression gate; cold-start < 50 ms;
  determinism = byte-stable output (which also makes the service cache content-addressable).

### Deployment topologies — one artifact, config-driven run modes

The *same* binary/image runs every mode; only config + replica count change (plan 12). Secure
multi-tenant scale and local single-user both fall out of a **stateless, deterministic, offline**
engine.

1. **Embedded library** (default, fastest) — in-process via the SDK or a binding. Zero services.
2. **Local binaries** — `cite-lint` (CLI/CI) and `cite-lint-lsp` (editors), single static binary,
   no runtime.
3. **Local MCP server** — `cite-lint-mcp` over **stdio**, zero-config, so a local agent (Claude,
   Codex, …) calls the capabilities as tools without shelling out (plans 11/12).
4. **Clustered service** (plan 12) — `cite-lint-server` (+ MCP over HTTP/SSE): N **stateless**
   replicas behind an LB sharing mmap'd tables; per-tenant authn + quotas; zero-retention; signed
   distroless image; **near-zero cold-start** enables autoscaling/scale-to-zero. Off the lint hot
   path — embedding is always faster.

## 7. Leanness rules (architecture-level)

- Start at **4 crates** (`data`, `core`, `host`, `cite-lint`) + 2 thin binaries. Everything
  else is additive and milestone-gated.
- Every dependency passes `cargo-deny` (license/advisory/bans) and earns its weight; the
  citation parser is the one place we *build* rather than buy (determinism + trust).
- Bindings are generated/wrapped, never hand-divergent. The service is **one artifact in
  config-driven run modes** (embedded → local → MCP → clustered), not a separate codebase — so
  "large-scale secure service" and "locally runnable" cost us one build, not two.
- WASM builds feature-gate heavy hosts (pdf/docx) out to keep the bundle small
  ([R-SDK-2](research/README.md)).

## 8. ADRs

Irreversible/load-bearing decisions get a short ADR under `docs/adr/` (plan 09): parser
strategy, edition-as-data, parity-by-construction, the Rule IR, LLM-assisted ingestion
with deterministic gates, binding-set selection, optional server. ADRs are the audit
trail for *why*, so future agents/contributors don't relitigate or smuggle in drift.
