#!/usr/bin/env bash
# lintcite verify pipeline (one-shot, idempotent). Logs everything to
# .verify.log so the orchestrating session can read full output.
set -uo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")"
LOG=.verify.log
{
  echo "=== verify run: $(date -Is) ==="
  df -h /home | tail -1
  echo "--- cargo build ---"
  cargo build --workspace 2>&1
  echo "BUILD_EXIT=$?"
  echo "--- cargo fmt (apply, then check) ---"
  cargo fmt --all 2>&1
  cargo fmt --all -- --check 2>&1
  echo "FMT_EXIT=$?"
  echo "--- cargo clippy ---"
  cargo clippy --workspace --all-targets -- -D warnings 2>&1
  echo "CLIPPY_EXIT=$?"
  echo "--- cargo test ---"
  cargo test --workspace 2>&1
  echo "TEST_EXIT=$?"
  echo "--- e2e: check memo.md ---"
  ./target/debug/lintcite check testdata/e2e/memo.md 2>&1
  echo "E2E_CHECK_EXIT=$?"
  echo "--- e2e: json ---"
  ./target/debug/lintcite check --format json testdata/e2e/memo.md 2>&1 | head -c 800
  echo
  echo "--- e2e: fix ---"
  ./target/debug/lintcite fix testdata/e2e/memo.md 2>&1
  echo "E2E_FIX_EXIT=$?"
  df -h /home | tail -1
  echo "=== verify done: $(date -Is) ==="
} > "$LOG" 2>&1
# Summary to stdout for the invoking agent.
echo "VERIFY SUMMARY"
grep -E "_(EXIT)=" "$LOG"
echo "--- last 25 log lines ---"
tail -25 "$LOG"
