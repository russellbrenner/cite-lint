# Plan 00 — Foundation

**Crates:** workspace (`data`, `core`, `host`, `cite-lint`, `-cli`, `-lsp`) ·
**Milestone:** M0 · **Depends on:** — · **Size:** M

## Goal & Definition of Done

Stand up an **empty-but-green** Cargo workspace whose structure already enforces the
invariants, so every later plan adds *behaviour* into a frame that can't drift. M0 ships
no linting logic — it ships the rails.

**DoD**
- [ ] Workspace builds; `fmt`/`clippy -D warnings`/`test` green on stubs.
- [ ] Dependency-graph test passes (core can't reach host/surfaces).
- [ ] `Diagnostic` model + stable `DiagnosticCode` scheme defined.
- [ ] SDK capability trait defined (typed `Unimplemented` errors, **no panics** in libs).
- [ ] Parity-matrix harness exists (enumerates capabilities; red until wired).
- [ ] CI baseline runs all gates; coverage gate active (realistic bar, ratchets up).
- [ ] One shared entrypoint (`just`/`xtask`) used by humans **and** loops.

## Design context

Implements the crate set and dependency graph in [`architecture.md`](../architecture.md)
§1–2, the diagnostic model §4, and the capability surface §3. Establishes the
gates-never-lowered principle (P5) as shared CI.

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-ARCH-2](../research/README.md#r-arch-2) | FST vs PHF for vocab; runtime-compile vs `build.rs` codegen | 1 day | T-data compile step |
| [R-LSP-1](../research/README.md#r-lsp-1) | `tower-lsp` vs `lsp-server` (incremental, cancellation) | 1 day | plan 06 |

## Task ladder

- **T1 — Workspace + toolchain pin.** Create the 6-crate workspace; add
  `rust-toolchain.toml` (stable channel, `rustfmt clippy llvm-tools-preview`); commit
  `Cargo.lock`. *Check:* `cargo build --workspace` green; CI uses the pinned toolchain.
- **T2 — Dependency-graph guard.** Add an `xtask` (or `cargo-deny` `[bans]`) test asserting
  `core` depends only on `data`, `host` only on `core`+`data`, surfaces only on the SDK.
  *Check:* test fails if you add `cite-lint-host` to `core`'s deps, passes otherwise.
- **T3 — Diagnostic model.** Define `Diagnostic`, `DiagnosticCode` (e.g. `AGLC4-CASE-001`,
  append-only registry), `Severity`, `Confidence`, `FixIt`, `SourceRange`, `AglcRuleRef` in
  `core`. Doc-comment each (what/how/depends — Rust standards). *Check:* `cargo doc` clean;
  `missing_docs` denied for public items.
- **T4 — SDK capability seam.** In `cite-lint`, define the capability trait/functions
  (`lint`, `parse`, `explain`, `fix`, `tokens`, `editions`) with typed request/response and
  `Err(Error::Unimplemented)` bodies. *No `todo!()`/`unwrap` — libs don't panic (P-Rust).*
  *Check:* compiles; calling any returns `Unimplemented`.
- **T5 — Parity-matrix harness.** Add a test that reads the capability set and asserts each
  is declared; leave surface-reachability assertions `#[ignore]` until plan 06/07 wire them.
  *Check:* harness runs; documents the parity contract in code.
- **T6 — CI baseline.** GitHub Actions workflow: `fmt --check`, `clippy -D warnings`,
  `test --workspace`, `cargo-llvm-cov` gate (start where stubs land, ratchet), `cargo-deny`,
  `cargo-audit`, `cargo doc -D warnings`. **Pin action SHAs**, least-privilege
  `permissions:`, concurrency cancel-in-progress. *Check:* CI green on the empty workspace.
- **T7 — Shared entrypoint.** `justfile` (or `cargo xtask`) with `check test cov deny
  fuzz-smoke bench docs` targets; CI and loops call these, not raw cargo. *Check:* `just
  check` reproduces CI locally.
- **T8 — Docs + platform skeletons.** mdBook skeleton (plan 09) with link-check CI;
  `skills/` core schema + Claude adapter stub (plan 11); `tools/loop/` runner stub + mem0
  service stub (plan 10) that just runs `just check`. *Check:* `mdbook build` green;
  skill-schema validation test passes; loop runner exits 0 on a clean tree.

## Acceptance gate

CI green on an empty workspace with every gate active; dependency-graph guard enforced;
`just check` runs the full gate set locally; the parity and skill-schema harnesses exist
(red/ignored where behaviour is absent, never deleted).

## Lean notes

- **4 + 2 crates only.** No `ffi/py/wasm/server/ingest` yet — added at their milestones.
- Coverage bar starts at the honest stub level and **ratchets up**, never down; don't
  pad stubs to hit a number.
- Prefer `just` over a bespoke build tool; one file, no new runtime.

## Risks & mitigations

- *Over-scaffolding* → keep stubs minimal; a stub that needs a test it can't yet have is a
  smell that the layer belongs to a later milestone.
- *CI action supply chain* → pin by SHA from day one (plan 08), not by tag.
