#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo-deny >/dev/null 2>&1; then
  echo "cargo_deny_gate: installing cargo-deny (one-time)..." >&2
  cargo install cargo-deny --locked --version 0.19.6
fi

cargo deny check advisories
echo "cargo_deny_gate (ag-id): PASS"
