#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

LEDGER=docs/CLAIMS_LEDGER.md
test -f "$LEDGER" || { echo "missing $LEDGER" >&2; exit 1; }

grep -q 'Audit date:' "$LEDGER"
grep -q '## Standing rules' "$LEDGER"
grep -q '| A-2 |' "$LEDGER"
grep -q '| A-4 |' "$LEDGER"
grep -q '| A-7 |' "$LEDGER"
grep -q '| A-16 |' "$LEDGER"

jq -e '.blockers[] | select(.id == "I.1") | .status == "resolved"' evidence/release_gate.json >/dev/null

echo "claims_ledger_gate (ag-id): PASS"
