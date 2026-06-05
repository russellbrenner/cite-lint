# cite-lint

A fast LSP server and CLI linter for the Australian Guide to Legal Citation (4th ed).

- **Live drafting** (LSP): Markdown, LaTeX
- **Batch checking** (CLI): PDF, Word/docx, Markdown, LaTeX, plain text

Status: **M1 slice** — case citations · Markdown/plain hosts · CLI · SDK.
Architecture and guardrails: [`CLAUDE.md`](CLAUDE.md) · plan of record:
[`docs/roadmap/`](docs/roadmap/README.md) · rule catalogue (generated):
[`docs/rules.md`](docs/rules.md).

## Quickstart (CLI)

```bash
cargo build --workspace
target/debug/cite-lint check your-memo.md         # lint (exit 1 on findings)
target/debug/cite-lint fix your-memo.md           # corrected text to stdout
target/debug/cite-lint explain AGLC4-CASE-001     # the rule behind a code
target/debug/cite-lint check --format json -      # JSON from stdin, for CI
```

```text
$ cite-lint check memo.md
memo.md:9:31: error[AGLC4-CASE-001] year bracket should be round brackets for CLR: found square brackets (AGLC4 r 2.2.1)
    fix-it: '(1992)'
```

## Quickstart (SDK)

```rust
let diagnostics = cite_lint::lint("[^1]: Mabo v Queensland (No 2) [1992] 175 CLR 1.")?;
assert_eq!(diagnostics[0].code.0, "AGLC4-CASE-001");
```

Every diagnostic names the AGLC4 rule it enforces; ambiguous citations get a
low-confidence "could not verify" rather than a confident guess. Fix-its are
safe-only: they never change the cited authority.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Contributions require 90%+ test coverage
and every linting rule must cite the AGLC4 rule it enforces. AI-assisted
contributions are welcome and held to a higher bar.

## Licence

[Apache License 2.0](LICENSE). Attribution per the [`NOTICE`](NOTICE) file.
