# Ag^id — Conformance suite

This directory exists for one purpose: to make it easy to verify that a third-party
implementation of the Ag^id v1 protocol matches the Rust reference implementation
byte-for-byte.

## What is canonical

The authoritative test data lives in [`test-vectors/v1.json`](../test-vectors/v1.json)
at the repository root. That file is referenced from
[`SPEC.md`](../SPEC.md#13-test-vectors) §13 as the conformance contract. It contains:

- A `vectors` array — positive vectors that every conforming implementation MUST
  reproduce exactly.
- A `negative_cases` block — inputs that every conforming implementation MUST
  reject with the documented error.

## What lives here

`conformance/` holds independent verification tooling and language-specific
witnesses. The witnesses do **not** define new conformance requirements; they
exercise `test-vectors/v1.json` from outside the Rust source tree.

- [`witnesses/python/agid_v1_witness.py`](witnesses/python/agid_v1_witness.py) —
  a small Python program that reads `test-vectors/v1.json`, computes
  `BLAKE3(b"agid:v1:" || domain_byte || input)` for every positive vector, and
  performs the parse-shape checks for the negative cases. It uses only the
  third-party `blake3` PyPI package and the standard library. It does not read
  any Rust source. If the Python witness disagrees with the Rust integration
  test (`tests/vectors_json.rs`), one of them has a bug.

## Adding a new-language witness

Open a PR with a `witnesses/<lang>/` directory containing:

1. A README that documents how to install the language's BLAKE3 dependency.
2. A short program that reads `test-vectors/v1.json` from this repo and asserts
   parity for every positive vector and rejection for every negative case.
3. A standalone exit code: `0` on success, non-zero on any mismatch.

Witnesses should be small, boring, and easy to audit. They are evidence, not a
secondary specification.

## What conformance does NOT prove

Two implementations agreeing on `test-vectors/v1.json` proves that the protocol
is implementable in at least two languages. It does NOT prove that
[`SPEC.md`](../SPEC.md) is sufficient on its own — that requires an external
engineer to write a conformant implementation using only the spec text (no
access to existing code). That acceptance test is tracked separately in
[`ROADMAP.md`](../ROADMAP.md).
