# Research register

The unknowns to resolve so the build stays **lean and easy**. Every spike exists to do one of
three things: **cut scope**, **pick the lean option**, or **de-risk** a commitment before code.
Each is time-boxed and ends in a recorded decision (ideally an ADR, plan 09).

- **~45 person-days total, highly parallelizable.** None blocks M0 (foundation); each is
  front-loaded just before its owning milestone.
- **Method bias:** prototype on a *fixture/sample*, measure, decide — never a research essay.
- **Output:** a one-line decision recorded in the spike + an ADR for load-bearing choices.

The ingestion cluster (`R-ING-*`) is the deepest, per the brief's emphasis — LLM-assisted
extraction is welcome **iff** the deterministic verification oracle (plan 02 T7) holds.

| Status | Meaning |
|--------|---------|
| ☐ open · ◐ in progress · ☑ decided (link ADR) · ⊘ dropped (scope cut) | |

---

## Architecture & data

### R-ARCH-1
**Question:** Does chumsky give usable error recovery for the citation grammar — a meaningful
*partial* AST on malformed input? **Method:** prototype the case grammar, feed malformed inputs,
inspect recovery. **Time-box:** 2d · **Unblocks:** plan 03 T2 · **Status:** ☐

### R-ARCH-2
**Question:** FST vs PHF for vocab lookups; runtime-compile vs `build.rs` codegen? **Method:**
bench both on the reporters table (load + lookup). **Lean lens:** the faster/smaller wins; avoid
codegen unless it pays. **Time-box:** 1d · **Unblocks:** plans 00, 01 · **Status:** ☐

### R-DATA-1
**Question:** What predicate vocabulary must the Rule IR support to express the **case** rules
without escaping to Rust? **Method:** hand-encode 10 case rules in a draft IR; log every Rust
escape needed. **Lean lens:** keep the DSL minimal — add an op only when a real rule needs it.
**Time-box:** 2–3d · **Unblocks:** plan 01 T2, plan 03 · **Status:** ☐

### R-CORE-1
**Question:** How does case/legislation/secondary classification consult tables mid-parse without
coupling `core` to `data` internals? **Method:** prototype the try-case→legislation→secondary
flow. **Time-box:** 1d · **Unblocks:** plan 03 T3 · **Status:** ☐

## Ingestion ⭐ (LLM-assisted, lean)

### R-ING-1
**Question:** Best extraction modality for the appendix tables — text-layer vs page-image
**vision** vs hybrid? **Method:** extract one reporters page all three ways; score fidelity
against a hand-checked truth. **Lean lens:** pick the one needing the least post-correction.
**Time-box:** 2d · **Unblocks:** plan 02 T5 · **Status:** ☐

### R-ING-2
**Question:** Do the worked-example oracle + dual-model agreement justify **auto-accepting**
high-confidence rows (human-reviewing only the rest)? Set the thresholds. **Method:** run the
oracle on a sample; measure false-accept/false-reject; tune. **Lean lens:** the more we can
auto-accept *safely*, the less human toil. **Time-box:** 2d · **Unblocks:** plan 02 T7/T8 · **Status:** ☐

### R-ING-3
**Question:** Leanest memory-safe decrypt+extract toolchain (qpdf/pikepdf for empty-password AES;
pdfium/pdftotext/render for text+images)? **Method:** decrypt + extract a known page; compare
fidelity + footprint. **Time-box:** 1d · **Unblocks:** plan 02 T2 · **Status:** ☐

### R-ING-4
**Question:** Local vector store (sqlite-vec vs Lance) + a local embedder — offline + small?
**Method:** embed + search sample sections; measure size + recall. **Lean lens:** one store reused
for edition-diff *and* dev-loop memory (plan 10). **Time-box:** 1d · **Unblocks:** plan 02 T9 · **Status:** ☐

### R-ING-5
**Question:** Edition-diff matching — embedding-NN + structural bookmark alignment? **Method:**
synthetic before/after sections; measure match accuracy. **Time-box:** 1d · **Unblocks:** plan 02
T10 · **Status:** ☐

### R-ING-6
**Question:** LLM Rule-IR drafting — schema-constrained output, hallucination guards, how much is
auto-draftable vs human-only? **Method:** draft IR for sample rules with two models; measure
validity + `verify-aglc-ref` pass rate. **Lean lens:** LLM does the toil, the oracle owns truth.
**Time-box:** 2d · **Unblocks:** plan 02 T6 · **Status:** ☐

## Hosts & surfaces

### R-HOST-1
**Question:** tree-sitter incremental extraction of footnote/citation spans for md + latex, with
correct range mapping? **Method:** prototype the md adapter; measure incremental reparse + range
accuracy. **Time-box:** 2d · **Unblocks:** plan 04 T2/T7 · **Status:** ☐

### R-HOST-2
**Question:** PDF/docx range fidelity + docx zip-bomb/resource limits? **Method:** range-map a
sample docx/pdf; test decompression caps. **Time-box:** 2d · **Unblocks:** plan 04 T5/T6 · **Status:** ☐

### R-LSP-1
**Question:** `tower-lsp` vs `lsp-server` — incremental sync, cancellation, maturity? **Method:**
spike a minimal diagnostics server in each. **Time-box:** 1d · **Unblocks:** plan 06 T3 · **Status:** ☐

### R-CLI-1
**Question:** Config precedence model + a versioned, stable JSON output schema? **Method:** draft
the schema + precedence rules; snapshot-test. **Time-box:** 1d · **Unblocks:** plan 06 T1/T2 · **Status:** ☐

## SDK & bindings

### R-SDK-1
**Question:** UniFFI (one def → Python/Swift/Kotlin) vs hand-rolled PyO3 + wasm-bindgen + napi —
least total binding code that fits the `Diagnostic` model? **Method:** spike UniFFI on the facade
vs separate bindings. **Lean lens:** fewest artifacts to maintain wins. **Time-box:** 2d ·
**Unblocks:** plan 05 T5–T8 · **Status:** ☐

### R-SDK-2
**Question:** WASM size budget; can we feature-gate pdf/docx/tree-sitter out of the WASM build?
**Method:** build WASM with/without hosts; measure bundle. **Time-box:** 1d · **Unblocks:** plan 05
T7 · **Status:** ☐

### R-SDK-3
**Question:** Sync core + async wrapper, or two APIs (LSP needs async, CLI sync)? **Method:**
prototype both ergonomics. **Lean lens:** one core + a thin async shim if it suffices. **Time-box:**
1d · **Unblocks:** plan 05 T1 · **Status:** ☐

## Service, scale & MCP

### R-SRV-1
**Question:** Service transport — HTTP+JSON vs gRPC vs JSON-RPC? **Method:** map the capability set
to each; weigh client ergonomics + OpenAPI. **Time-box:** 1d · **Unblocks:** plan 12 T1 · **Status:** ☐

### R-MCP-1
**Question:** MCP transports (stdio local, HTTP/SSE remote) + auth for remote MCP? **Method:**
stand up a stdio MCP server; test an agent calling `lint`; design remote auth. **Lean lens:** stdio
(zero-config local) is the default; remote only when needed. **Time-box:** 1d · **Unblocks:** plan 12
T3 · **Status:** ☐

### R-SCALE-1
**Question:** Scale model — stateless replicas + LB vs a work queue; content-addressed cache
design? **Method:** load-model both; prototype the `hash(input+edition+config)` cache. **Lean
lens:** replicas+LB unless load forces a queue. **Time-box:** 2d · **Unblocks:** plan 12 T6 · **Status:** ☐

### R-SCALE-2
**Question:** Multi-tenancy isolation + per-tenant quota/rate-limit model? **Method:** design
per-request bounds + quota accounting; threat-check cross-tenant leakage. **Time-box:** 1d ·
**Unblocks:** plan 12 T4 · **Status:** ☐

### R-SCALE-3
**Question:** What delivers **near-zero cold-start** (replica ready in ms) + scale-to-zero — static
binary + **mmap'd precompiled edition tables** + no startup deps; and is a WASM/edge isolate viable
for sub-ms starts? **Method:** measure binary cold-start with mmap'd tables vs parsed-at-boot;
prototype a WASM/edge variant. **Lean lens:** precompiled + memory-mapped = instant readiness, no
warm-pool to run. **Time-box:** 2d · **Unblocks:** plan 12 T11 · **Status:** ☐

## Testing, security, docs

### R-TEST-1
**Question:** Seed the conformance corpus from the guide's worked examples — format + tooling?
**Method:** convert plan 02 T4 `example` sections into `(input → expected)` cases. **Time-box:** 1d
· **Unblocks:** plan 07 T8 · **Status:** ☐

### R-TEST-2
**Question:** Load-test tool (k6 vs oha vs locust) + the scale SLO targets? **Method:** trial one
against the local service; set throughput/latency SLOs. **Time-box:** 1d · **Unblocks:** plan 07 T11
· **Status:** ☐

### R-SEC-1
**Question:** Dependency vetting (`cargo-vet` vs `cargo-crev`); target SLSA level; sigstore keyless
via OIDC? **Method:** trial vetting on the dep set; pick the SLSA level we can sustain. **Lean
lens:** keyless + OIDC = no secret management. **Time-box:** 1d · **Unblocks:** plan 08 T3/T5 · **Status:** ☐

### R-SEC-2
**Question:** Reproducible-build feasibility for the Rust binaries + WASM/wheels? **Method:** build
twice, diff artifacts. **Time-box:** 1d · **Unblocks:** plan 08 T5 · **Status:** ☐

### R-DOCS-1
**Question:** mdBook vs Astro Starlight; integrating rustdoc + generated reference + tested
non-Rust samples? **Method:** stand up a skeleton in each; test a generated rule-catalogue page.
**Time-box:** 1d · **Unblocks:** plan 09 T1/T5 · **Status:** ☐

## Dev platform

### R-LOOP-1
**Question:** mem0 local/self-hosted deploy + API + memory schema + decay/curation policy, keeping
AGLC-derived material local? **Method:** run mem0 locally; design namespaces + schema; test
add/search. **Time-box:** 2d · **Unblocks:** plan 10 T3 · **Status:** ☐

### R-LOOP-2
**Question:** Orchestration substrate — git worktrees + a runner; Claude Code headless / Codex CLI
as executors; how gating is wired? **Method:** prototype a one-lane loop that runs `just check` and
commits on green. **Time-box:** 2d · **Unblocks:** plan 10 T1/T2 · **Status:** ☐

### R-SKILL-1
**Question:** Concrete skill/tool formats per runtime — Claude `SKILL.md`, Codex `AGENTS.md`,
Hermes tool-call JSON, **OpenClaw (format unknown — confirm from its docs)**; define the adapter
contract. **Method:** read each runtime's spec; write one skill's adapter for each. **Time-box:** 2d
· **Unblocks:** plan 11 T3 · **Status:** ☐

### R-SKILL-2
**Question:** How should skills invoke cite-lint — MCP tools (agent-native) vs SDK binding
(embedded) vs CLI — per runtime? **Method:** wire one skill via MCP + via a binding; compare.
**Lean lens:** prefer MCP as the uniform call path. **Time-box:** 1d · **Unblocks:** plan 11 T4 · **Status:** ☐
