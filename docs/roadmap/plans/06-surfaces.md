# Plan 06 — Surfaces (CLI · LSP · MCP)

**Crates:** `cite-lint-cli`, `cite-lint-lsp`, `cite-lint-mcp` ·
**Milestones:** M1 (LSP + minimal CLI) → M3 (CLI mature + SARIF, MCP stdio) ·
**Depends on:** 05 · **Size:** M

> The **networked, at-scale service** (HTTP/JSON-RPC + remote MCP) is
> [plan 12](12-service-scale-and-mcp.md). This plan owns the **local** surfaces — including the
> **MCP server over stdio**, which plan 12 then productionises for scale.

## Goal & Definition of Done

Thin shells over the SDK. Each surface only parses input, calls a capability, and formats
output — **no surface contains linting logic** (P1). This is what makes CLI/LSP/MCP parity
true rather than tested-for.

**DoD**
- [ ] LSP (M1): diagnostics, semantic tokens, code actions (fix-its), hover (explain),
      incremental sync — all delegating to the SDK.
- [ ] CLI (M1 minimal → M3): `check`/`parse`/`explain`/`fix`/`editions`; text/JSON/SARIF;
      stable exit codes; stdin/file/glob; config + precedence.
- [ ] MCP (M3): `cite-lint-mcp` over **stdio**, zero-config, exposing the capabilities as tools
      so local agents call them without shelling out.
- [ ] Every subcommand/request/tool maps to a capability; parity test green (plan 07).

## Design context

[`architecture.md`](../architecture.md) §3 (capability → surface mapping) + §6 deployment
topologies. One diagnostic model rendered identically across every surface (P8). Determinism
makes CLI output cacheable + CI-stable (P3).

## Research spikes

| ID | Question | Time-box | Unblocks |
|----|----------|----------|----------|
| [R-LSP-1](../research/README.md#r-lsp-1) | `tower-lsp` vs `lsp-server` (incremental sync, cancellation, maturity) | 1 day | T3 |
| [R-CLI-1](../research/README.md#r-cli-1) | Config precedence model + a versioned, stable JSON output schema | 1 day | T1, T2 |
| [R-MCP-1](../research/README.md#r-mcp-1) | MCP transports (stdio local; HTTP/SSE remote → plan 12) + tool schema | 1 day | T6 |

## Task ladder

- **T1 — CLI skeleton.** clap; subcommands map 1:1 to capabilities. Inputs: stdin, file, glob.
  Config file + flag precedence (R-CLI-1). Body = parse args → SDK → format. *Check:* `cite-lint
  check file.md` runs end-to-end; `--help` lists subcommands matching the capability set.
- **T2 — CLI formatters + exit codes.** `text` (human), `json` (versioned stable schema),
  `sarif` (`ruleId` = `DiagnosticCode`, for code scanning). Exit: `0` clean, `1` diagnostics,
  `2` error. Snapshot tests (insta). *Check:* the three formats snapshot-match; exit codes
  asserted via `assert_cmd`.
- **T3 — LSP server.** Chosen lib (R-LSP-1): `initialize`, `didOpen`/`didChange` (incremental,
  plan 04 T7), `publishDiagnostics`, `semanticTokens` (from `tokens`), `codeAction` (fix-its),
  `hover` (`explain`). Thin over the SDK. *Check:* a scripted LSP session golden test
  (initialize → didOpen → diagnostics → codeAction → applyEdit) passes.
- **T4 — LSP config.** Edition selection, severity, rule enable/disable via settings. *Check:*
  switching edition/severity changes diagnostics in a session test.
- **T5 — Editor extension (M7).** Minimal VS Code client over the LSP (no logic). *Check:*
  extension activates and surfaces diagnostics on a sample doc.
- **T6 — MCP server (stdio, M3).** `cite-lint-mcp` exposes the capabilities as MCP **tools**
  (`lint`/`parse`/`explain`/`fix`/`editions`) over **stdio**, zero-config, for local agents
  (Claude, Codex, …); drives the skills pack (plan 11). Thin over the SDK. HTTP/SSE remote
  transport + scale + multi-tenancy are **plan 12**. *Check:* `tools/list` matches the capability
  set; `tools/call lint` returns diagnostics golden-equal to the SDK; stdio mode needs no config.
- **T7 — Parity wiring.** Each subcommand/request/tool resolves to a capability; the parity
  matrix (plan 07) asserts identical results across SDK/CLI/LSP/MCP on the shared corpus.
  *Check:* a capability missing from a surface fails the parity test.

## Acceptance gate

A Markdown file lints via CLI **and** LSP with diagnostics byte-identical to the SDK (M1);
CLI text/JSON/SARIF snapshots + exit codes tested; an LSP session golden passes; the **MCP stdio**
server lists + calls the capabilities with golden-equal results; parity test green across
SDK/CLI/LSP/MCP. (The networked service's contract + load gates are in plan 12.)

## Lean notes

- **LSP + minimal CLI first** (live drafting is the headline value); JSON/SARIF + **MCP stdio** at
  M3 (MCP is a thin, high-leverage shell that unlocks every agent runtime); the **networked
  service is plan 12** — only for embedders that can't link the SDK or need multi-tenant scale.
- VS Code extension is a thin LSP client; no second implementation of anything.
- Output schemas are **versioned** so downstream CI doesn't break on additions.

## Risks & mitigations

- *Untrusted MCP tool args / CLI files* → validate + bound inputs (plan 08 T7); local stdio MCP is
  the lowest-surface transport. Networked abuse controls (rate-limit, authn, quotas) are plan 12.
- *Surface logic drift* → forbidden by construction (thin shells) + the parity test; reviewers
  reject any rule logic landing in a surface crate.
