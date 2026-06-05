# aglc4-lsp — Agent Guardrails

A fast LSP server + CLI linter for the Australian Guide to Legal Citation (4th ed).
This file holds the architectural invariants and rules an agent must not violate;
it is self-contained, so the repo is fully specified without external docs. See
`CONTRIBUTING.md` for the contribution process and coverage gate. (A fuller design
doc may exist locally under `docs/superpowers/specs/` — it is a Claude working doc
and intentionally git-ignored, not a source of truth.)

## Architectural invariants (do not break these)

1. **The engine is format- and editor-agnostic.** `aglc-core` accepts a citation
   string + kind hint + source range and returns diagnostics. It MUST NOT depend on
   `aglc-host`, `aglc-lsp`, or `aglc-cli`. Dependency direction is one-way:
   `aglc-data → aglc-core → {aglc-host} → {aglc-lsp, aglc-cli}`.
   If you find yourself importing a host or surface type into core, stop — the
   abstraction is wrong, fix the boundary instead.
2. **Reference data before rules.** Every rule that consults a controlled vocabulary
   (reporters, courts, jurisdictions, signals, round/square-bracket rules) reads it
   from `aglc-data`. Never hard-code a vocabulary inside a rule. Tables are versioned
   data files with a re-runnable extraction script and table-tests.
3. **Per-citation isolation is the performance contract.** A citation is parsed and
   validated in isolation. Cross-citation logic lives ONLY in the document pass,
   which operates on an ordered list of already-parsed ASTs. Do not reach across
   citations from inside per-citation rules.
4. **Parser strategy is fixed:** hand-written / PEG (chumsky) for the *citation*
   grammar; tree-sitter for *host* document structure only. Do not introduce a
   tree-sitter grammar for citations or a regex-based citation parser.
5. **One diagnostic model.** All surfaces render the same `Diagnostic` (range,
   message, AGLC4 rule reference, severity, optional fix-it). Surfaces translate;
   they do not invent diagnostics.

## Correctness guardrails (the product is trust)

- **Never guess an AGLC4 rule.** Every rule cites the specific AGLC4 rule/page it
  enforces, in a code comment and in the diagnostic message. If the guide is
  ambiguous, encode the ambiguity as a low-confidence "could not classify"
  diagnostic — never emit a confident-but-wrong diagnostic.
- **Fix-its must be safe.** A quick-fix may only produce a citation that is *more*
  compliant; never one that changes the cited authority's meaning. When unsure,
  offer the diagnostic without a fix-it.
- **Disambiguation order is explicit and tested.** "Try case → legislation →
  secondary" lives in one place, documented, with tests for each branch and for the
  fall-through-to-unclassified case.

## Rust standards

- Edition 2021+, `cargo fmt` and `cargo clippy -- -D warnings` clean before commit.
- **No `unwrap()` / `expect()` / `panic!` in library crates** (`aglc-data`,
  `aglc-core`, `aglc-host`). Return `Result`. Panics are acceptable only in tests
  and in binary `main` startup where failure must abort.
- Errors: typed errors with `thiserror` in libraries; `anyhow` only in the binary
  crates (`aglc-lsp`, `aglc-cli`).
- Prefer `&str`/borrowed data through the parser; allocate at the edges.
- Public items in library crates carry doc comments stating *what it does, how to
  use it, what it depends on*.
- No `unsafe` without a `// SAFETY:` comment justifying it and review.

## Testing discipline

- **Every rule ships with fixtures:** at least one compliant input and one
  non-compliant input, asserting the exact diagnostic (+ fix-it if any). A rule
  without a failing-case test is incomplete.
- Parser changes carry golden-AST tests, including malformed inputs (error recovery).
- Reference-table changes carry table-tests asserting known entries, so PDF
  re-extraction cannot silently regress.
- Surfaces have integration tests: LSP request/response goldens; CLI output
  snapshots (text/JSON/SARIF) + exit codes.
- Run the full suite before claiming work complete. State what you ran and the result.

## Commit conventions

- **No commit trailers.** Do NOT add `Co-Authored-By`, `Generated with`, or any
  AI-attribution trailer. Commits are attributable via the repo's git identity.
- Imperative subject line, scoped where useful (e.g. `core: add round-year rule`).
- Keep commits focused; one logical change per commit.
- This repo pushes with a GitHub noreply email (email privacy is on). Do not change
  the committer email to a private real address — pushes will be rejected.
- Never commit secrets, the AGLC4 PDF binary, or `target/`.

## Working agreements

- Read the spec before extending scope. New source types / hosts are *additive* on
  the existing engine — if a change needs new architecture, raise it, don't smuggle
  it in.
- When the AGLC4 PDF and this code disagree, the PDF wins; fix the code and the
  table-test, and note the rule reference.
