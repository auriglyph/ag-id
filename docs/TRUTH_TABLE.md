# TRUTH_TABLE

| Claim | Status | Evidence file | Reproduce command | Public wording allowed | Forbidden wording | Notes |
|---|---|---|---|---|---|---|
| A-2 same input always produces the same identifier | verified | `tests/determinism.rs`, `tests/vectors_json.rs`, `test-vectors/v1.json` | `cargo test --test determinism --test vectors_json` | same input always produces the same identifier | cross-platform CI proof | Verified on current Linux HEAD; cross-platform CI remains a limit. |
| A-4 42 / 42 tests passing | verified | `evidence/validation_summary_v1.json` | `cargo test --workspace` | tests passed on HEAD | stale counts from earlier heads | Count is recorded from the current validation run. |
| A-7 hash input layout is PREFIX + domain byte + input | verified | `src/derive.rs`, `conformance/witnesses/python/agid_v1_witness.py` | inspect `src/derive.rs` and witness parity | fixed protocol layout | alternative byte layout | This is the canonical v1 layout. |
| A-16 conformance witnesses prove cross-implementation determinism for v1 corpus | verified for v1 corpus | `conformance_v1.json`, `.github/workflows/ci.yml`, `conformance/witnesses/python/agid_v1_witness.py` | `python3 conformance/witnesses/python/agid_v1_witness.py` | cross-implementation determinism for the v1 corpus | universal proof for arbitrary inputs | Evidence is corpus-bound, not exhaustive. |
| external audit completed | HOLD | `docs/CLAIMS_LEDGER.md` A-15, A-17, A-18 | N/A | audit pending | externally audited production-ready | Must stay a hold until independent audit exists. |
