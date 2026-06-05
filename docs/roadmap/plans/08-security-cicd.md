# Plan 08 — Security & CI/CD

**Crates:** all · **Milestones:** M0 (baseline) → M7 (signed GA) · **Depends on:** 00 ·
**Size:** L

## Goal & Definition of Done

Make security a property of the **defaults** (P7): every untrusted input is bounded and
fuzzed, every dependency is vetted, every release is signed + attested, and the same gates
apply to humans and the self-improving loops. The deep multi-tenant service hardening lives in
[plan 12](12-service-scale-and-mcp.md); this plan owns the cross-cutting + supply-chain + CI/CD.

**DoD**
- [ ] Secure-coding baseline enforced by lints (`forbid(unsafe)` except `ffi`; no
      `unwrap/expect/panic` in libs; resource limits on all parsers/decoders).
- [ ] Threat model (STRIDE) over CLI / LSP / MCP / server / fetch, with mitigations.
- [ ] Supply chain: `cargo-deny` + `cargo-audit` + dep-vetting + pinned toolchain + `Cargo.lock`.
- [ ] CI security: least-privilege OIDC, pinned action SHAs, secret scanning, SAST, container scan.
- [ ] Signed, attested releases: SBOM + SLSA provenance + sigstore signatures on every artifact.
- [ ] `SECURITY.md` + disclosure + supported-versions policy.

## Design context

Untrusted bytes reach the PDF/docx parsers (plan 04), the MCP/server decoders (plans 06/12),
and the ingestion fetch (plan 02). Confidential legal text means **local-by-default,
zero-retention, no content logging** (P7). Determinism (P3) makes signed, reproducible builds
meaningful.

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-SEC-1](../research/README.md#r-sec-1) | Dependency vetting: `cargo-vet` vs `cargo-crev`; target SLSA level; sigstore keyless via OIDC | 1 day | T3, T5 |
| [R-SEC-2](../research/README.md#r-sec-2) | Reproducible-build feasibility for the Rust binaries + WASM/wheels | 1 day | T5 |

## Task ladder

- **T1 — Secure-coding baseline.** `#![forbid(unsafe_code)]` workspace-wide except
  `cite-lint-ffi` (every `unsafe` `SAFETY:`-commented); clippy lints denying `unwrap/expect/panic`
  in libs; input validation + size/time/recursion caps on every parser and request decoder.
  *Check:* CI denies a new `unwrap` in a lib crate; a pathological input hits the cap, not OOM.
- **T2 — Threat model.** STRIDE over each surface: CLI (untrusted files), LSP (untrusted docs),
  **MCP** (untrusted tool args from an agent), **server** (untrusted multi-tenant requests),
  fetch (network + copyright). Document mitigations + the confidential-data posture (no content
  in logs/telemetry by default). *Check:* `docs/security/threat-model.md` reviewed; each surface
  has named mitigations traced to tests.
- **T3 — Supply chain.** `cargo-deny` (licenses/advisories/bans/sources), `cargo-audit`, dep
  vetting (R-SEC-1), `rust-toolchain.toml` pin, committed `Cargo.lock`, Renovate/Dependabot.
  *Check:* a banned license or known-vuln dep fails CI; lockfile drift is flagged.
- **T4 — CI security.** Least-privilege `GITHUB_TOKEN` `permissions:`; **action SHAs pinned**
  (not tags); OIDC trusted publishing (no long-lived registry secrets); secret scanning
  (gitleaks); SAST (CodeQL + clippy security lints); container scan (trivy/grype); branch
  protection + required reviews + signed tags. *Check:* a leaked secret or a CodeQL finding fails
  CI; releases use OIDC, not stored tokens.
- **T5 — Release integrity.** SBOM (CycloneDX/syft) per artifact; **SLSA provenance**; sigstore/
  cosign signatures on binaries, container images, and wheels; reproducible builds where feasible
  (R-SEC-2). *Check:* every release artifact has an attached SBOM + signature verifiable by a
  third party.
- **T6 — Release pipeline.** `cargo-dist` for multi-platform binaries (`cite-lint`, `-lsp`,
  `-mcp`); maturin wheels (PyPI); WASM npm; distroless non-root **signed** images for the
  service + MCP-HTTP (plan 12); `git-cliff` changelog; publish on tag via OIDC. *Check:* a tagged
  release produces signed binaries + wheels + npm + image + SBOM + provenance in one run.
- **T7 — Cross-cutting secure-processing.** Per-request resource bounds; stateless isolation (no
  cross-request/cross-tenant state); reject pathological inputs (zip-bomb in docx, deep nesting,
  oversized payloads); rate-limit + size caps at **every** network surface; encryption in
  transit; no secrets/PII/citation-content in logs. (Service-scale specifics → plan 12.)
  *Check:* fuzz + the docx zip-bomb test (plan 04) + a request-cap test all green.
- **T8 — Security testing in CI.** Fuzz smoke (plan 07 T4); dependency review on PRs; container
  + image scans; SAST. *Check:* all run on PR; deep fuzz on a schedule.
- **T9 — Disclosure.** `SECURITY.md` (contact, scope, safe-harbor), advisory workflow, supported-
  versions + backport policy. *Check:* file present; a dry-run advisory follows the process.

## Acceptance gate

All security + supply-chain gates active in CI from M0 and ratcheting; threat model documented
and reviewed; M7 releases are signed + SBOM'd + SLSA-attested across binaries/wheels/npm/images;
no unreviewed advisories; `SECURITY.md` + disclosure process live.

## Lean notes

- **Keyless signing (sigstore) + OIDC publishing** → no key/secret management to run.
- **One shared CI workflow** is reused by humans and loops (plan 10) — gates defined once.
- **Distroless, non-root images** keep the attack surface (and size) minimal.
- Prefer the platform's built-ins (CodeQL, secret scanning, dependency review) over bespoke tools.

## Risks & mitigations

- *Supply-chain compromise* → pinned SHAs + lockfile + vetting + SBOM + provenance make tampering
  detectable and dependencies auditable.
- *Confidential document leakage* → zero-retention + no-content-logging defaults; local-first
  deployment; documented data-handling for any hosted LLM in ingestion (plan 02).
- *Loop misuse of credentials* → loops run with the same least-privilege token; no release
  signing authority is delegated to an automated loop (human-gated tags).
