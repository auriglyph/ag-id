# Python witness for Ag^id v1

This is a standalone Python implementation of the Ag^id v1 derivation and parse
shape, used to independently verify [`test-vectors/v1.json`](../../../test-vectors/v1.json).

The script does not read or import any Rust code. It does not depend on this
project's crate. If you remove the rest of the repository and keep only this
directory plus the JSON test-vector file, the witness still runs.

## Install

The script depends on a single third-party package providing a BLAKE3 binding:

```sh
pip install blake3
```

Python 3.10+ is recommended (uses no syntax beyond that level).

## Run

From the repository root:

```sh
python3 conformance/witnesses/python/agid_v1_witness.py
```

Exit code:

- `0` — all positive vectors verified and all negative cases recognised.
- `1` — at least one positive vector mismatch or negative case not recognised.

The script also accepts an alternate path to a vector file as the first
argument, for testing against modified files:

```sh
python3 conformance/witnesses/python/agid_v1_witness.py /path/to/v1.json
```

## What it checks

For every entry in the JSON's `vectors` array:

1. Independently computes `BLAKE3(b"agid:v1:" || domain_byte || input)` and
   asserts the hex matches the recorded `raw_hex`.
2. Independently base58-encodes the 32 bytes (Bitcoin alphabet, leading-zero
   handling) and asserts that prepending `did:agid:` yields the recorded
   `did_string`.

For every entry in `negative_cases.parse_rejections`:

1. Runs the same lightweight parse-shape checks the spec requires (prefix
   present, payload length, ASCII, base58 alphabet) and asserts that the
   recorded `expected_error` would fire.

For `negative_cases.derivation_rejections`:

1. Confirms that `domain_byte = 0x00` is treated as the reserved value.

## What it does NOT check

- Performance — it is a correctness witness, not a benchmark.
- Round-trip semantics in any specific host language — that is the host's job.
- Anything beyond the JSON's content.
