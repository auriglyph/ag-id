# Ag^id — Design Rationale

This document explains *why* `Ag^id` (Rust crate `ag_id`) makes the choices it does. It is not a specification (see [SPEC.md](SPEC.md) for that). It is meant to be read by reviewers, contributors, and anyone deciding whether to adopt the library.

---

## 1. The problem

Two services need to refer to the same entity using the same identifier, without coordinating. Examples:

- A web app and an offline mobile client both need to address "the user with email `alice@example.com`" — and they must agree on the identifier without a central server.
- Replayable AI pipelines need stable references to entities so the same record can be re-derived after serialisation, days later, on different hardware.
- Content-addressed storage needs identifiers that survive moves and renames.
- Edge devices generating telemetry need to mint IDs without round-tripping to a registry.

UUID v4 (random) does not solve this — two nodes generating UUIDs for the same entity get different UUIDs. UUID v5 (namespace-based, SHA-1) does, but it is locked in 1990s cryptography and 128-bit output.

---

## 2. Design choices and the alternatives

### Hash function: BLAKE3 (not SHA-256, not SHA-3, not SHA-1)

| Candidate | Reason rejected |
|---|---|
| **SHA-1** (UUID v5) | Cryptographically broken (collision attacks since 2017). |
| **MD5** (UUID v3) | Cryptographically dead. |
| **SHA-256** | Solid, but ~3× slower than BLAKE3 on commodity hardware and no parallelism. |
| **SHA-3 (Keccak)** | Excellent security but 5–10× slower than BLAKE3 and no SIMD speedup on most hardware. |
| **BLAKE2s/2b** | BLAKE3's predecessor; BLAKE3 is strictly faster with the same security model. |
| **xxh3 / wyhash / FNV** | Non-cryptographic — collisions are easy to grind. Unsuitable for adversarial domains. |
| **SipHash** | Keyed-only; no defined unkeyed deterministic mode. |

BLAKE3 wins on three axes: security model (PRF assumption, formal analysis), speed (multi-GiB/s with SIMD), and zero-copy streaming. It is also `no_std`-friendly with no allocator dependency.

### Output length: 32 bytes (not 16, not 20, not 64)

| Length | Reason rejected or chosen |
|---|---|
| 16 bytes (UUID) | 2⁻⁶⁴ collision probability after 2³² IDs — birthday bound is too close for global namespaces. |
| 20 bytes (SHA-1) | Better than 16 but still legacy-shaped. |
| **32 bytes** | 2⁻¹²⁸ collision probability after 2⁶⁴ IDs. Comfortable headroom for any realistic system, and it matches BLAKE3's natural output. |
| 64 bytes | Wasteful; encoded URI gets unwieldy. |

256 bits is the sweet spot for cryptographic identifiers in the post-2020 era. It matches the security level of secp256k1, Curve25519, and the rest of the modern crypto stack.

### Display encoding: base58 (not base32, not base64, not hex)

| Encoding | Reason |
|---|---|
| **hex** | Available, but doubles the URI length (64 vs ~44 characters). Kept as `to_hex_array()` for tooling. |
| **base32** | Case-insensitive, widely supported, but ~52 chars for 32 bytes. |
| **base64 / base64url** | 44 chars, but `+/-_=` cause copy-paste hazards in URLs and command lines. |
| **base58 (Bitcoin)** | ≤44 chars, no visually ambiguous characters (`0` vs `O`, `1` vs `l` vs `I`), copy-pastes cleanly, ecosystem familiar from Bitcoin/Monero/Solana. **Chosen.** |
| **multibase (varied)** | Lets each producer pick their own encoding. We want one wire form, not a menu. |

### Domain separation: typed enum + protocol prefix

A naked hash of `input` is dangerous: if two systems hash different *kinds* of things (a user email, a document URL) and one user's email happens to equal another system's URL, their IDs collide. Worse: an attacker can construct such collisions on demand to forge cross-system references.

`Ag^id` defends against this with two layers:

1. An 8-byte protocol prefix `b"agid:v1:"` ensures no other hash-of-bytes scheme accidentally collides with us.
2. A typed `Domain` enum encoded as a single byte ensures `User`, `Document`, `Session`, etc. live in different sub-spaces.

This is the same idea as RFC 9106 (Argon2) personalization or NIST SP 800-185 (cSHAKE) customization, applied to the simpler case of a deterministic hash.

### Domain as a single byte (not a UUID, not a string)

UUID v5 forces callers to allocate a namespace UUID for each kind of entity. This is awkward — what UUID do you use for "documents"? `Ag^id` uses a single typed byte so the answer is "0x02 — see the table." Future-proofing: 256 byte values gives ample room; we have used 5 of them.

### URI prefix: `did:agid:` (not `dnid:`, not `urn:agid:`, not just bare base58)

Two communities matter:

- **W3C DID** — the `did:method:` URI scheme is the most widely adopted way to express decentralised identifiers on the web. Conforming to it makes `Ag^id` legible to JSON-LD tooling, verifiable-credential libraries, and DID resolvers without modification.
- **URN registry** — `urn:agid:` would also work but is less idiomatic for new identifier schemes in 2026.

`agid` stands for AuriGlyph IDentifier (the project family is AuriGlyph; this crate is the identifier primitive). The full method spec for `did:agid:` is in this document and `SPEC.md`; W3C registration is on the roadmap (see `ROADMAP.md`).

### `Domain::Opaque` and parsed `Did`s

A subtle decision: parsing `did:agid:<base58>` recovers the 32 raw bytes but **not** the original domain. We had three options:

1. Encode domain in the wire form (e.g. `did:agid:U:<base58>` for User). Rejected: breaks compatibility with the W3C DID syntax which wants a single base58/base64 payload, and it makes the wire form longer.
2. Refuse to parse — only let users construct `Did` via `derive`. Rejected: serialisation round-trips are essential.
3. Parse to a `Did` with a sentinel `Domain::Opaque` value and document that parsed values are byte-identifiable but domain-anonymous. **Chosen.**

This is the right trade-off: the *byte identity* is what callers actually need from a serialised ID. Domain context is recovered from the surrounding code (the database column, the JSON field name), not from the bytes.

Derivation APIs accept `DeriveDomain`, not `Domain`, so the `Opaque` sentinel and
the reserved byte `0x00` cannot enter the v1 hash input path.

### Allocation strategy: zero on the hot path

`Did::derive` does not allocate. `to_hex_array` returns a stack `[u8; 64]`. `to_base58` returns a stack `([u8; 44], usize)`. Only `to_did_string` and the optional `serde` impl allocate, and only because they return `String` for ergonomics.

This makes the crate suitable for `no_std` embedded contexts and for high-throughput pipelines where allocator pressure matters.

### `unsafe_code = "forbid"`

There is no use of `unsafe` in `Ag^id`. The hash dependency (BLAKE3) uses `unsafe` for SIMD intrinsics, which is appropriate. Our 350-line crate has no business with `unsafe`.

### Strict lints (clippy::pedantic + nursery + deny unwrap/panic/todo)

Anyone reviewing the source will see exactly what the code does, with no `unwrap()` traps, no `todo!()` placeholders, no panic paths reachable from public APIs, and no documented-but-not-implemented surface. The integration tests legitimately panic on bad fixtures and are scoped-allowed.

---

## 3. Comparison to prior art

| Property | UUID v4 | UUID v5 | ULID | KSUID | nanoid | did:key | **Ag^id** |
|---|---|---|---|---|---|---|---|
| Deterministic from input | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ (from key) | **✓** |
| Modern hash | n/a | ✗ (SHA-1) | n/a | n/a | n/a | ✓ (multihash) | **✓ (BLAKE3)** |
| Bits of collision resistance | 122 (random) | ~80 (SHA-1 weakened) | 128 | 128 | configurable | ≥128 | **256** |
| Cross-platform stable | ✓ | ✓ | mostly | ✓ | ✓ | ✓ | **✓** |
| `no_std` | ✓ | ✓ | ✓ | varies | ✓ | varies | **✓** |
| W3C DID compatible | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | **✓** |
| Type-safe domain separation | ✗ | namespace-UUID | ✗ | ✗ | ✗ | n/a (key-based) | **✓** |
| Side-channel friendly equality | n/a | ✗ | ✗ | ✗ | ✗ | varies | **opt-in via `subtle`** |
| Format readable | medium | medium | sortable | sortable | random | URL-shaped | **URL-shaped, no ambig chars** |

The honest summary: **`Ag^id` is what UUID v5 would have been if designed in 2026.** Modern hash, modern output length, type-safe domain separation, no_std-clean, W3C DID syntax, with an explicit cross-platform contract.

It is not a revolutionary new primitive. It is a careful re-statement of an old primitive with the corners filed off, the cryptography updated, and the wire format chosen to be legible to current tooling. The "this should have existed already" reaction is the point.

---

## 4. Why "an event"

Whether `Ag^id` ends up an event in the world depends on three things, none of which the code itself can decide:

1. **Adoption.** Distributed-systems and AI-pipeline communities have to encounter the problem and reach for it. Outside our control.
2. **DID method registration.** Becoming a registered W3C DID method (`did:agid`) makes it interoperable with verifiable credentials, JSON-LD, and resolver infrastructure for free. This is a multi-month process.
3. **Multi-language reach.** A single Rust crate is a tool; the same library implemented (or even just verified against the same test vectors) in TypeScript, Python, Go, and Swift is an ecosystem. The `test-vectors/v1.json` artefact is the seed.

The code in this repository is the part we control. It is small, scrutable, well-tested, and protocol-frozen. The rest is what `ROADMAP.md` is for.

---

## 5. What this crate intentionally does NOT do

To stay small and trustworthy:

- No I/O. No file paths. No network.
- No async. The function is microsecond-scale; an async wrapper is the caller's problem if they want one.
- No database. No registry. No central authority. Anyone can derive the same ID by knowing the inputs.
- No randomness. There is no `OsRng`, no clock, no nonce.
- No revocation. A `Did` exists forever once derived; if the input changes, that's a different `Did`.
- No truncation. We never display fewer than the full 32 bytes of meaning.

If you need any of these, layer them on top — but they are not in the trust boundary of this crate.

---

## 6. Open design questions

Genuinely open, listed here for transparency:

- **Salted variant.** Should we expose a keyed-BLAKE3 mode for callers who need to derive IDs that are not predictable to outsiders? Currently they can do this themselves by salting `input`, but a typed API would be safer. Tracked as a v1.x candidate addition (would not change v1 outputs for existing users).
- **`hex` feature flag.** Currently `to_hex_array` is always built. Some embedded callers want only base58. Could be feature-gated.
- **Constant-time equality.** Should we expose a built-in `ct_eq` method (depending on `subtle`) instead of pointing callers at the external crate? Trade-off: an extra dep vs. a clearer API.

These are deferred to v1.x minor releases pending real user feedback.
