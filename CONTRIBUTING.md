# Contributing to lintcite

Thanks for your interest. This project values **correctness and trust** above
velocity: a citation linter that is confidently wrong is worse than useless. Please
read `CLAUDE.md` (architectural invariants and guardrails) before opening a PR.

## Before you start

1. **Check for existing work.** Search open and recently-closed PRs and issues to
   confirm your change isn't already in flight or already resolved. Link the issue
   you're addressing; if none exists, open one first for anything non-trivial.
2. **Stay within the architecture.** New source types, rules, and host adapters are
   *additive* on the existing engine (see `CLAUDE.md` §"Architectural invariants").
   If your change needs new architecture, raise it in an issue before coding.
3. **One logical change per PR.** Keep PRs focused and reviewable.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo llvm-cov --all --fail-under-lines 90   # coverage gate (see below)
```

A PR must be `fmt`-clean, `clippy`-clean (warnings denied), and green on tests and
coverage before review.

## Test coverage requirement

- **Minimum 90% line coverage**, enforced in CI. PRs that drop coverage below the
  threshold will not be merged.
- New code must be covered by tests, not coast on the existing percentage. Every new
  rule ships with at least one compliant and one non-compliant fixture asserting the
  exact diagnostic (and fix-it, if any). A rule without a failing-case test is
  incomplete.
- Bug fixes include a regression test that fails before the fix and passes after.

## Correctness rules (non-negotiable)

- **Cite the AGLC4 rule.** Every linting rule references the specific AGLC4
  rule/page it enforces, in both a code comment and the diagnostic message. Do not
  invent or paraphrase rules from memory; when the guide is ambiguous, emit a
  low-confidence "could not classify" diagnostic rather than a confident wrong one.
- **Fix-its must be safe.** A quick-fix may only make a citation *more* compliant,
  never change the cited authority's meaning. When unsure, ship the diagnostic
  without a fix-it.
- When the AGLC4 PDF and the code disagree, the PDF wins.

## AI-assisted contributions

AI-assisted contributions are welcome and held to a **higher** bar, not a lower one,
because generated citation logic is easy to get plausibly wrong.

- **Disclose** AI assistance in the PR description (which tool, what it did).
- **You are the author.** The human submitter is responsible for every line:
  correctness, licence compatibility, and AGLC4 accuracy. "The model wrote it" is
  not a defence for a wrong rule.
- **Same gates, applied harder.** 90%+ coverage, clippy-clean, every rule citing its
  AGLC4 reference. Verify generated rule references against the actual PDF — models
  hallucinate citation rules.
- **Check for duplication first.** Confirm no existing PR already covers the change
  before generating a new one; do not open near-duplicate AI-generated PRs.
- **No fabricated tests.** Tests must exercise real behaviour, not be shaped to pass
  a coverage gate. Reviewers will reject tests that assert nothing meaningful.

## Commits

- **No commit trailers.** Do not add `Co-Authored-By`, `Generated with`, or any
  AI-attribution / sign-off trailer.
- Imperative, scoped subject lines (e.g. `core: add round-year bracket rule`).
- Never commit secrets, the AGLC4 PDF binary, or `target/`.

## Licence

By contributing, you agree your contributions are licensed under the project's
[Apache License 2.0](LICENSE), per section 5 of that licence. Retain the `NOTICE`
file's attribution in any redistribution.
