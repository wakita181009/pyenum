#!/usr/bin/env bash
# Unified coverage: Rust unit/integration tests + Python pytest suite.
#
# The proc-macro and validation logic is exercised by `cargo test`,
# while the runtime crate (pyenum/cache.rs, construct.rs, register.rs)
# and the cdylib fixture (pyenum-test) are only reached via Python.
# This script instruments both runs under the same cargo-llvm-cov
# session so the final report covers the whole workspace.
#
# Usage:
#   scripts/coverage.sh                    # text summary
#   scripts/coverage.sh --html             # HTML report
#   scripts/coverage.sh --lcov --output-path lcov.info
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Wipe prior .profraw files so the report is clean.
cargo llvm-cov clean --workspace

# Export LLVM_PROFILE_FILE, RUSTC_WRAPPER, and crate allowlist so every
# subsequent cargo/maturin invocation produces instrumented artifacts.
eval "$(cargo llvm-cov show-env --export-prefix 2>/dev/null)"

# Rust tests (proc-macro unit tests, trybuild, pyenum lib tests).
cargo test --workspace --lib --tests

# Build the pyenum-test cdylib against the current Python interpreter.
# `maturin develop` respects RUSTC_WRAPPER + RUSTFLAGS from the env above.
if command -v uv >/dev/null 2>&1 && [[ -d .venv ]]; then
  uv run maturin develop
  uv run pytest tests/
else
  maturin develop
  pytest tests/
fi

# Aggregate the two runs into a single report.
cargo llvm-cov report "$@"

# Re-runs `report` with `--fail-under-lines 80` so the job exits non-zero if
# coverage regresses; the earlier call above still emits whatever format the
# caller asked for (text summary, --html, --lcov, ...).
cargo llvm-cov report --fail-under-lines 80
