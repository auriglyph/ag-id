# Changelog

All notable changes to `ag_id` are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Stability contract

The following are part of the **MAJOR** version contract. Changing any of
them in a non-major release is a defect:

- The protocol prefix `b"agid:v1:"` fed into BLAKE3.
- The byte values assigned to each `Domain` variant
  (`User=0x01`, `Document=0x02`, `Session=0x03`, `Device=0x04`, `Concept=0x05`).
- The base58 alphabet (Bitcoin variant) and the `did:agid:` URI prefix.
- The output length (32 raw bytes; ≤44 base58 characters).

Anything affecting the `(domain, input) → Did` mapping requires a major bump.
Adding new `Domain` variants is **MINOR** (the byte assignments above are
frozen; new variants get new bytes). Bug fixes that do not change output are
**PATCH**.

## [Unreleased]

### Changed

- `Did::derive`, `derive`, and `derive_str` now accept `DeriveDomain` instead
  of `Domain`, so the parsed-value sentinel `Domain::Opaque` cannot be used as
  a derivation domain.

### Added

- `DeriveDomain` — derivation-only domain type with built-in domains and
  `DeriveDomain::custom(byte) -> Result<_, Error>` for non-zero custom bytes.

## [0.1.0] — 2026-05-10

First public release of `ag_id` (display name **Ag^id**), under the
crate name `ag_id` on `github.com/auriglyph/ag-id`.

### Project rename (relative to internal `determin-id` predecessor)

This release supersedes an internal-only crate previously named
`determin-id`. Identifiers minted under the old name are intentionally
**not** bit-compatible with `ag_id` v1 — both the BLAKE3 protocol prefix
and the DID URI scheme were rotated when the project was renamed.

| Field | Old (internal `determin-id`) | New (`ag_id` v1) |
|---|---|---|
| Crate name | `determin-id` | `ag_id` |
| Display name | — | `Ag^id` |
| BLAKE3 protocol prefix | `b"determin-id:v1:"` (15 B) | `b"agid:v1:"` (8 B) |
| DID URI prefix | `did:agf:` | `did:agid:` |
| Homepage | (n/a, internal) | `https://auriglyph.com/projects/ag_id` |
| Test vectors | regenerated for the new prefix | see [`test-vectors/v1.json`](test-vectors/v1.json) |

The old `determin-id` artefacts were never published to crates.io, never
deployed externally, and have no migration path to `ag_id`. New
implementations MUST NOT accept `did:agf:` URIs as equivalent to
`did:agid:`. See [`SPEC.md` §12](SPEC.md#12-compatibility-with-the-legacy-didagf-form-informative)
for the legacy form's status.

### Added

- `Did::derive(domain, input)` — pure deterministic identifier derivation
  via `BLAKE3(b"agid:v1:" || domain_byte || input)`.
- `Domain` enum with `User`, `Document`, `Session`, `Device`, `Concept`,
  `Custom(u8)`, and the `Opaque` sentinel for parsed values.
- Display formats: `did:agid:<base58>` (≤44 base58 chars) and lowercase
  hex (64 chars), both stack-allocated.
- Parsing: `Did::parse(s)` and `impl FromStr for Did` decode a
  `did:agid:<base58>` string back into a `Did`. The original `Domain`
  is not recoverable from the wire form by design; parsed values carry
  the `Domain::Opaque` sentinel.
- `Did::from_bytes([u8; 32]) -> Did` constructor for raw byte transport.
- `Did::eq_bytes(&other) -> bool` byte-level equality (for comparing
  typed vs. opaque `Did` values).
- Public `DID_PREFIX` constant (`"did:agid:"`).
- Six `Error` variants: `EmptyInput`, `ReservedDomain`, `InvalidUtf8`,
  `MissingPrefix`, `WrongLength`, `InvalidBase58`. `Error` is
  `#[non_exhaustive]`.
- Optional `serde` feature: `Did` serialises as the canonical
  `did:agid:<base58>` string in human-readable formats and as the 32 raw
  bytes in binary formats (`bincode`, `postcard`, `MessagePack`).
  Round-trips through `serde_json` are tested.
- Property tests via `proptest`: 6 invariants × 256 random cases each
  (determinism, domain separation, parse round-trip, hex shape, format
  invariants, byte round-trip).
- Cross-language test-vector artefact at `test-vectors/v1.json` with 10
  canonical `(domain, input, raw_hex, did_string)` tuples; integration
  test (`tests/vectors_json.rs`) asserts JSON ↔ implementation parity.
- `examples/basic.rs` — derive, format, compare, iterate built-in
  domains.
- `examples/parsing.rs` — round-trip a DID through `to_did_string` and
  back via `parse`; demonstrate parse failure cases.
- `SPEC.md` — formal protocol specification suitable for
  re-implementation in another language without reading the Rust source.
- `DESIGN.md` — design rationale, alternatives considered, and prior-art
  comparison (UUID v4/v5, ULID, KSUID, nanoid, did:key).
- `ROADMAP.md` — pre-1.0 stabilisation list, post-1.0 minor additions,
  v2 outlook, ecosystem (cross-language SDKs, W3C registration), and
  explicit anti-roadmap.
- `CONTRIBUTING.md` — what to send, local checks, vector workflow,
  domain-byte allocation procedure, security-disclosure pointer.
- `PUBLISHING.md` — pre-flight, version bump, tag, publish,
  post-publish, and rollback checklists.
- `BENCHMARKS.md` — criterion methodology, full environment block
  (CPU, SIMD flags, rustc version), and reproduction steps.
- `SECURITY.md` — threat model, non-goals, in-scope defects, and
  reporting contact.
- `LICENSE-MIT` and `LICENSE-APACHE` (dual-licensed for public release).
- `[package.metadata.docs.rs]` config so `--all-features` (the `serde`
  impls) render on docs.rs.
- README `Limitations` section covering constant-time, collision
  strength, deletion, domain discipline, truncation, and v1.x stability.
- Security note in `Did` rustdoc: `PartialEq` is not constant-time;
  pointer to constant-time alternatives.
- `no_std` support behind the `std` feature flag (default on).
- Doctest-backed public API with `unsafe_code = "forbid"` and
  `clippy::pedantic + nursery + deny(unwrap_used, todo, panic)`.

### Notes

- Cross-platform stability is enforced both by hardcoded vectors in
  `tests/vectors.rs` / `tests/determinism.rs` (golden anchor hex) and by
  the JSON parity test in `tests/vectors_json.rs`. Any platform that
  disagrees with these bytes is non-conformant.
- This is the first public commit; there are no earlier published
  versions to be compatible with.
