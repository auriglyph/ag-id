#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

run() { echo "==> $*"; "$@"; }

echo "validation_gate (ag-id): $(git rev-parse --short HEAD)"

run cargo fmt --all -- --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run bash scripts/claims_ledger_gate.sh
run bash scripts/cargo_deny_gate.sh

jq -e '.status == "pass"' evidence/release_gate.json >/dev/null
jq -e '[.blockers[].status] | all(. == "resolved")' evidence/release_gate.json >/dev/null
jq -e '.tests.status == "pass"' evidence/validation_summary_v1.json >/dev/null
jq -e '.status == "pass"' evidence/conformance_v1.json >/dev/null

echo "validation_gate (ag-id): PASS"
