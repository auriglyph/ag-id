# Contributing to ag_id

Thank you for considering a contribution. This crate is small and security-relevant, so the bar is high but the surface is also small.

## What to send

Welcome:

- Bug reports — especially anything that produces different bytes for the same input on different platforms, or a panic in production code.
- Documentation fixes, typos, clearer wording in `SPEC.md`/`DESIGN.md`/`README.md`.
- New `Domain` variants with a clear use case (open an issue first to discuss the byte allocation).
- Cross-language re-implementations (link them; we'll add to the README).
- Additional test vectors for `test-vectors/v1.json`, especially ones that exercise edge cases (leading zeros, large inputs, exotic domains).

Likely to be deferred or declined:

- API expansion that has no obvious user. Less surface is better.
- Performance micro-optimisations without a benchmark showing they matter.
- Changes to the wire format. The v1 protocol is frozen; see [`SPEC.md` §10](SPEC.md#10-stability-commitments-v1x).
- Anything that introduces `unsafe` code (the crate is `unsafe_code = "forbid"` and will stay that way).

## Local checks

Before opening a PR, run:

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo build --no-default-features          # confirm no_std build
cargo test --release --test vectors_json    # confirm test-vectors/v1.json parity
cargo bench --bench throughput              # only if you touched the hot path
```

`cargo fmt --check` and clippy must be clean. The lint configuration in `Cargo.toml` is intentionally strict; it is the primary defence against accidental complexity creeping into a small library.

## Adding test vectors

If you add a row to `test-vectors/v1.json`, you must also add the same case to `tests/vector_export.rs` so the generator can produce the canonical hex/DID values. Then run:

```sh
cargo test --release --test vector_export -- --nocapture print_vectors
```

Copy the printed `raw_hex` and `did_string` into the JSON. Re-run `cargo test --test vectors_json` to confirm the JSON file is internally consistent.

## Adding a new `Domain` variant

1. Open an issue first proposing the variant name, byte assignment, and use case. Byte must be in `0x06..=0xFF` and not previously used.
2. Once accepted, update:
   - `src/domain.rs` (enum + `as_byte` + `Display`)
   - `src/domain.rs::Domain::builtins()` if it should be enumerable
   - `README.md` domain table
   - `SPEC.md` §3 table
   - `test-vectors/v1.json` and `tests/vector_export.rs`
   - `CHANGELOG.md` under `[Unreleased] → Added`
3. The byte assignment, once published in a release, is permanent. There is no renumbering.

## Reporting security issues

Do **not** open a public issue for suspected vulnerabilities. Email <security@auriglyph.com> with a minimal reproducer and the commit hash you tested. See [`SECURITY.md`](SECURITY.md) for the full policy.

## Style

- Public items have rustdoc with at least one example.
- Public items in `lib.rs` are re-exported from internal modules; module structure is an implementation detail.
- Tests use `snake_case_with_double_underscores__like_this` — readable as `subject__action__condition`.
- Clippy lints are strict; reach for `#[allow(...)]` only when the lint is genuinely wrong, and put the allow at the smallest possible scope (function or block, not module).

## Licence on contributions

By submitting a contribution you agree it is licensed under the same terms as the crate: MIT OR Apache-2.0.
