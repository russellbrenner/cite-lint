# Plan 05 — SDK & bindings

**Crates:** `cite-lint` (facade) + `cite-lint-ffi`, `cite-lint-py`, `cite-lint-wasm`
(candidates: node, UniFFI targets) · **Milestones:** M1 (facade) → M3 (py, wasm) → M7
(semver freeze) · **Depends on:** 00, 03 · **Size:** L

## Goal & Definition of Done

A **first-class, embeddable SDK** that is the *only* behaviour surface, with language
bindings that wrap it so any application uses cite-lint **in-process** — no shelling out to
the CLI, and with **full parity** to the CLI because the CLI is itself a wrapper over this SDK.

**DoD**
- [ ] `cite-lint` facade exposes every capability (`lint`, `parse`, `explain`, `fix`,
      `tokens`, `editions`) with typed request/response and the **one** `Diagnostic` model.
- [ ] Zero-config defaults: `lint(text)` works (markdown + `aglc4`) in ~5 lines.
- [ ] Semver-stable public API; `cargo-semver-checks` gate on the facade crate.
- [ ] Python (PyO3/maturin) + WASM (wasm-bindgen) bindings with smoke tests + packaging CI.
- [ ] C ABI (`cite-lint-ffi`, cbindgen) with a compiled+run example.
- [ ] **Parity by construction**: bindings/CLI/LSP/server all map to one capability set;
      the parity-matrix test (plan 07) is green.

## Design context

[`architecture.md`](../architecture.md) §3 (capability surface) + §4 (one diagnostic model).
Principle P1 (library-first, parity by construction) and P8 (one model). This crate is the
single place `core`+`host`+`data` are wired (invariant 1 preserved: `core` depends on nothing
upward).

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-SDK-1](../research/README.md#r-sdk-1) | UniFFI (one def → Python/Swift/Kotlin) vs hand-rolled PyO3 + wasm-bindgen + napi — least total binding code that fits the `Diagnostic` model? | 2 days | T5–T8 |
| [R-SDK-2](../research/README.md#r-sdk-2) | WASM size budget; feature-gate pdf/docx/tree-sitter out of the WASM build | 1 day | T7 |
| [R-SDK-3](../research/README.md#r-sdk-3) | Sync core + async wrapper (LSP needs async, CLI sync) — one API or two? | 1 day | T1 |

## Task ladder

- **T1 — Facade API.** Implement the capability functions with typed `Request`/`Response`
  structs and a `Session`/builder for config (edition, host hint, severity filter). Provide a
  **sync** core and a thin **async** wrapper (R-SDK-3). Re-export the `Diagnostic` model so
  every consumer shares one type. *Check:* `lint`/`parse`/`explain`/`fix`/`tokens`/`editions`
  return real results (no more `Unimplemented`); rustdoc complete (`missing_docs` denied).
- **T2 — Single wiring point.** Wire `core`+`host`+`data` here and **only** here. *Check:*
  dep-graph test confirms no surface/binding re-implements engine logic.
- **T3 — Ergonomics & zero-config.** Defaults that "just work": `cite_lint::lint("...")` picks
  markdown + `aglc4`; batch returns an iterator/stream. *Check:* the 5-line quickstart in the
  docs is a compiled doctest (plan 09).
- **T4 — Stability policy.** `#[non_exhaustive]` on growable enums/structs; documented semver
  policy (capabilities are additive; codes append-only). Add `cargo-semver-checks` to CI on the
  facade. *Check:* a breaking change to a public signature fails CI.
- **T5 — C ABI (`cite-lint-ffi`).** cbindgen-generated header; `lint`/`fix`/`explain` over a
  stable C struct; an `examples/` C program. *Check:* the example compiles and runs against the
  built lib; a `SAFETY:`-commented boundary (the only place `unsafe` is allowed).
- **T6 — Python (`cite-lint-py`).** PyO3 module mirroring the capabilities; type stubs
  (`.pyi`); `pytest` smoke; wheels via maturin/cibuildwheel across platforms (CI in plan 08).
  *Check:* `pip install` the wheel, `cite_lint.lint(...)` returns diagnostics matching the SDK.
- **T7 — WASM (`cite-lint-wasm`).** wasm-bindgen; **hosts feature-gated out** for size
  (R-SDK-2); npm package; JS smoke (browser + Node). *Check:* bundle under the size budget;
  `lint()` in Node matches the SDK on a fixture.
- **T8 — Candidate bindings.** Node (napi-rs) and/or UniFFI multi-target **only if** R-SDK-1
  shows lower total cost / real demand. *Check:* gated decision recorded as an ADR (plan 09).
- **T9 — Parity adapters.** Drive every binding's method list from the one capability
  definition; the parity-matrix test (plan 07) asserts SDK ↔ CLI ↔ bindings agree on a shared
  golden corpus. *Check:* adding a capability without wiring a binding fails the parity test.

## Acceptance gate

All six capabilities functional from the Rust SDK with golden-equal results to the CLI (parity
test green); `cargo-semver-checks` active on the facade; Python wheel + WASM npm package build
and pass smoke tests in CI; the C example compiles and runs; the 5-line quickstart is a CI'd
doctest.

## Lean notes

- **Smallest binding set that covers demand: Python + WASM first.** Add Node/JVM/Swift only on
  real pull, and prefer **UniFFI** if it generates several from one definition (less to maintain).
- **One capability definition drives every binding** → no per-binding behaviour drift, less to
  test and document.
- WASM stays small by feature-gating heavy hosts out; embedders that need pdf/docx use native
  bindings.

## Risks & mitigations

- *Binding maintenance burden* → generate/wrap from one definition; gate new bindings behind
  demand + R-SDK-1; CI builds all bindings on every release so breakage is caught early.
- *Accidental API breakage* → `cargo-semver-checks` + `#[non_exhaustive]` + append-only codes.
- *`unsafe` at the C boundary* → confined to `cite-lint-ffi`, every block `SAFETY:`-commented and
  reviewed (Rust standards); the rest of the workspace stays `#![forbid(unsafe_code)]`.
