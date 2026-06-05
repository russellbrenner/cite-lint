# cite-lint

A fast LSP server and CLI linter for the Australian Guide to Legal Citation (4th ed).

- **Live drafting** (LSP): Markdown, LaTeX
- **Batch checking** (CLI): PDF, Word/docx, Markdown, LaTeX, plain text
- **Embed** (SDK/API): Rust, Python, WASM, C — a first-class library with full CLI parity
- **Agents & scale**: an MCP server plus a secure, multi-tenant service — the *same artifact* runs
  locally (including as an MCP server) or as a near-zero-cold-start cluster

Status: design phase. Architecture and guardrails: see [`CLAUDE.md`](CLAUDE.md).

## Roadmap

The full engineering plan — executable, per-layer build plans plus a research register — lives in
[`docs/roadmap/`](docs/roadmap/README.md). Start with the
[roadmap index](docs/roadmap/README.md), the [principles](docs/roadmap/00-principles.md), and the
[architecture backbone](docs/roadmap/architecture.md).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Contributions require 90%+ test coverage
and every linting rule must cite the AGLC4 rule it enforces. AI-assisted
contributions are welcome and held to a higher bar.

## Licence

[Apache License 2.0](LICENSE). Attribution per the [`NOTICE`](NOTICE) file.
