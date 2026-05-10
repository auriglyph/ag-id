# Ag^id — Roadmap

This is a public roadmap for `Ag^id` (Rust crate `ag_id`). Items are grouped by realistic horizon. None of the items below promise a date; they describe order of intent.

---

## v0.x → 1.0 (the stabilisation line)

Goal: turn the v1 protocol into something callers can build on without fearing a future break.

- [x] Wire format frozen: `BLAKE3(b"agid:v1:" || domain_byte || input)`.
- [x] Display format frozen: `did:agid:<base58btc>` ≤ 44 chars.
- [x] `Domain` byte assignments frozen for `User=0x01`, `Document=0x02`, `Session=0x03`, `Device=0x04`, `Concept=0x05`, `Opaque=0x00` (sentinel).
- [x] Cross-platform test vectors in `test-vectors/v1.json` with a Rust integration test asserting parity.
- [x] `Did::parse` round-trip (`did:agid:` string → `Did`) and `FromStr` impl.
- [x] Property tests (`proptest`) for determinism, domain separation, and string round-trips.
- [x] Optional `serde` feature with both human-readable (string) and binary (bytes) representations.
- [x] `examples/basic.rs` and `examples/parsing.rs`.
- [x] `SPEC.md`, `DESIGN.md`, `SECURITY.md`, `BENCHMARKS.md`, `CONTRIBUTING.md`.
- [ ] Public GitHub repository at `github.com/auriglyph/ag-id`.
- [ ] Initial `cargo publish` to crates.io as `0.1.0`.
- [ ] CI: GitHub Actions running `cargo fmt --check && cargo clippy -- -D warnings && cargo test --all-features && cargo build --no-default-features` on every PR.
- [ ] `docs.rs` build verified with `--all-features` so the `serde` impls render.
- [ ] At least one outside reviewer reads SPEC.md and confirms the bytes can be re-implemented in another language from the spec alone (no need to read the Rust source).
- [ ] Promote to `1.0.0` once the above lands and one minor release cycle passes without a wire-format defect.

---

## Post-1.0 (semver-minor additions)

These extend the API without breaking the v1 wire format. Each is scoped to be a single PR.

- **Constant-time equality.** Built-in `Did::ct_eq(&self, other) -> Choice` gated on a new `subtle` feature flag. Cleaner ergonomics than pointing callers at an external crate.
- **Salted derivation.** A typed `Did::derive_keyed(domain, key, input)` that uses BLAKE3's keyed mode. Lets callers derive IDs that an outsider cannot pre-compute. Wire format unchanged for callers of `derive`.
- **Additional `Domain` variants.** Candidates: `Event`, `Capability`, `Schema`. Each must claim a fresh byte in `0x06..=0xFF` and once published cannot be reassigned.
- **`AsRef<[u8]>` ergonomic input.** `Did::derive(domain, input)` accepts anything with `AsRef<[u8]>` (string slices, `Vec<u8>`, byte arrays) without callers writing `.as_bytes()` everywhere.
- **`hex` feature flag.** Make `to_hex_array` opt-in for embedded callers who only want base58.
- **`zeroize` feature flag.** Optional `zeroize::Zeroize` impl for callers who treat `Did` as derived secret material.
- **DID document resolution.** A pure-function `resolve(did) -> DidDocument` returning a static document (since `did:agid` is a self-resolving method). Makes interop with W3C resolver libraries seamless.

---

## v2 outlook (non-binding)

The v2 line is reserved for genuine wire-format changes. We do not expect to need it for the foreseeable future, but the migration shape is documented now so callers can plan:

- New protocol prefix: `b"agid:v2:"`.
- New URI prefix: `did:agid2:` (or a fresh method registered with W3C).
- Triggered only by a credible attack on BLAKE3 (currently unimaginable) or by a community decision to move to a successor primitive.
- Implementations are encouraged to ship both `v1` and `v2` derivation in the same library, behind separate functions. They MUST NOT silently produce v2 outputs from APIs that previously produced v1.

---

## Ecosystem

Items that are not Rust-crate work but are necessary for `Ag^id` to be useful beyond a single language.

- **Cross-language reference implementations.**
  - TypeScript / WASM (browser, Node, Deno, Bun).
  - Python (CPython + PyPy, with `blake3` PyPI package).
  - Go.
  - Swift / Objective-C bindings for iOS / macOS.
  - Each must ingest `test-vectors/v1.json` and pass it as part of CI.
- **W3C DID Method registration.** Submit `did:agid` to the [W3C DID Method registry](https://www.w3.org/TR/did-spec-registries/). Requires a published method spec (we have `SPEC.md`) and at least one production deployment.
- **JSON-LD context.** Publish `https://auriglyph.com/projects/ag_id/contexts/v1` so DID documents emitted by `did:agid` resolve cleanly in JSON-LD tooling.
- **Verification CLI.** A small standalone binary `did-agid-verify` that takes a `(domain, input, expected_did)` triple from stdin and exits 0/1. Useful for shell scripts and integration tests in non-Rust pipelines.
- **`docs.rs` documentation polish.** Ensure all public items have examples, the `Domain` enum table renders, and the `serde` impls show up.

---

## Anti-roadmap

Things we have explicitly decided NOT to do, with reasons:

- **A central registry of `Custom` byte allocations.** Decentralisation is the point — collisions in `Custom(b)` are a property of the caller, not us. Document your byte choices in your own README.
- **Async wrapping.** The function is microseconds; async overhead would dwarf the work. Callers can wrap if they want.
- **Truncation.** Showing fewer than the full 32 bytes of meaning is a footgun (collision-grindable). We will never expose a "short ID" API.
- **Random fallback.** Some libraries fall back to randomness if input is empty or "weak". `Ag^id` does the boring thing: empty input is well-defined and produces a stable hash like any other input.
- **Multiple wire formats per protocol version.** One canonical form per version (v1 → `did:agid:<base58>`). Multibase-style "encoder of the day" undermines determinism.
- **`did:agf:` legacy compatibility.** The pre-rename internal version of this protocol used `did:agf:` and a different protocol prefix. No external clients depended on it, so we do not carry it forward.
