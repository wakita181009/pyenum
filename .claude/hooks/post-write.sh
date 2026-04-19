#!/usr/bin/env bash
# PostToolUse hook: delegate to pre-commit so all formatting/lint rules live
# in .pre-commit-config.yaml as the single source of truth.
# `--hook-stage manual` selects the fast auto-fix subset.
set -u

command -v pre-commit >/dev/null 2>&1 || exit 0

f=$(jq -r '.tool_input.file_path // .tool_response.filePath // empty')
[ -n "$f" ] || exit 0

root=$(git -C "$(dirname "$f")" rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$root" || exit 0

pre-commit run --hook-stage manual --files "$f" 2>&1 | tail -40
exit 0
