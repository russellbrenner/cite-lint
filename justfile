# Shared entrypoint (plan 00 T7): humans and loops call these, not raw cargo.

# Run every local gate (mirrors CI).
check: fmt-check clippy test

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

build:
    cargo build --workspace

# Coverage gate (CONTRIBUTING: >= 90% lines). Requires cargo-llvm-cov.
cov:
    cargo llvm-cov --workspace --fail-under-lines 90

# Lint the repo's own docs as a smoke test of the CLI.
selfcheck:
    cargo run -p cite-lint-cli -- check testdata/e2e/memo.md
