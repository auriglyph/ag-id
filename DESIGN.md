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

`agid` stands for "agnostic identifier" — the DID method name for this protocol. The full method spec for `did:agid:` is in this document and `SPEC.md`; W3C registration is on the roadmap (see `ROADMAP.md`).

### `Domain::Opaque` and parsed `Did`s

A subtle decision: parsing `did:agid:<base58>` recovers the 32 raw bytes but **not** the original domain. We had three options:

1. Encode domain in the wire form (e.g. `did:agid:U:<base58>` for User). Rejected: breaks compatibility with the W3C DID syntax which wants a single base58/base64 payload, and it makes the wire form longer.
2. Refuse to parse — only let users construct `Did` via `derive`. Rejected: serialisation round-trips are essential.
3. Parse to a `Did` with a sentinel `Domain::Opaque` value and document that parsed values are byte-identifiable but domain-anonymous. **Chosen.**

This is the right trade-off: the *byte identity* is what callers actually need from a serialised ID. Domain context is recovered from the surrounding code (the database column, the JSON field name), not from the bytes.

Derivation APIs accept `DeriveDomain`, not `Domain`, so the `Opaque` sentinel and
the reserved byte `0x00` cannot enter the v1 hash input path.

### Why `DeriveDomain` and `Domain` are separate types

The crate exposes two enums that look similar at first glance. The split is
deliberate:

- `DeriveDomain` is the **input type** for `Did::derive`. It cannot represent
  `Opaque`, and its `Custom(NonZeroU8)` rejects `0x00` at the type level
  (`NonZeroU8::new(0)` returns `None`, so `DeriveDomain::custom(0)` returns
  `Err(Error::ReservedDomain)` without reaching the hash). This means the v1
  hash input path is **structurally incapable** of feeding the reserved byte
  to BLAKE3.
- `Domain` is the **observation type** attached to a `Did` value. It includes
  the same five built-ins plus `Custom(u8)` and `Opaque`. `Opaque` exists
  because parsing `did:agid:<base58>` must produce a `Did` whose original
  derivation domain is no longer known.

A single enum cannot serve both roles without either weakening the type-system
guarantee on the derive path (`Custom(u8)` including `0`) or amputating the
parsed-value state (no `Opaque`). The two types are the cheapest way to keep
both promises simultaneously.

### What `Domain::Opaque` actually means

`Domain::Opaque` is the type-level encoding of "this `Did` was reconstructed
from a wire form — the original derivation domain is unrecoverable." It maps
to the reserved byte `0x00`, which the protocol forbids as a derivation input
(see SPEC.md §3). The mapping is consistent across three layers:

- **Type layer.** `Opaque` is a sentinel `Domain` variant.
- **Byte layer.** Byte `0x00` is reserved.
- **Protocol layer.** No `(domain_byte, input) → raw` derivation may target
  `domain_byte = 0x00`.

This tri-layer invariant is the design's way of saying "we have a
type-system-visible state for unresolved provenance, and that state cannot
collide with any derivation."

### Reserved byte `0x00`

Reserving a single byte from the start of the domain space costs one slot
(255 remain) and buys two properties at once:

- A sentinel for the unresolved-domain state.
- A failure-safe value for a future v2 protocol that might want to indicate
  "this is not a v1 derivation" without ambiguity.

Other deterministic-identifier schemes that did not reserve such a byte have
had to reintroduce it later as a magic value. Doing it on day one is cheaper
than doing it after adoption.

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

### Rejected alternatives at the design level

These were considered as the project's overall shape and rejected. Listed here
so the rejection is visible:

- **A database sequence with a UUID-style identifier.** Requires a central
  authority. Two offline parties cannot agree on the same identifier. Defeats
  the determinism premise.
- **A full identity platform** (key management, authentication, attestations,
  revocation lists). Ag^id is a name primitive. Adding an authentication or
  attestation layer would force the trust boundary to grow far beyond a pure
  hash function, which is the property that makes Ag^id auditable in a single
  reading.
- **A random short-form identifier with a server-side mapping** (e.g.
  shortener-style). Reintroduces the central authority and a database. The
  generated short form is not derivable from the inputs.
- **Multibase / multihash encoding.** Lets each producer pick the encoding.
  The wire form ceases to be canonical and "this is the v1 form" becomes
  "this is one of several v1 forms," which defeats deterministic
  identifier-equality across sender and receiver.
- **A blockchain-anchored identifier.** Requires a chain. Adds resolution
  latency. Couples the lifecycle of the identifier to the operational
  lifecycle of the chain. None of these costs buy anything for a pure
  deterministic-name use case.

---

## 6. What is NOT yet implemented

The crate at v0.1.x ships the deterministic-derivation primitive and the
`did:agid:` URI form. The ecosystem layer that a complete W3C DID method
requires is **not in this crate today**. Specifically:

- **No DID resolver.** The current code has `Did::parse` (URI → bytes) but no
  function returning a structured `DidDocument`. Resolution to a DID Document
  is tracked in [`ROADMAP.md`](ROADMAP.md) under "Post-1.0".
- **No DID Document type.** There is no `DidDocument` struct, no JSON-LD
  serialisation of the resolved form, no service or verificationMethod
  arrays.
- **No JSON-LD context.** The URL `https://auriglyph.com/projects/ag_id/contexts/v1`
  is referenced from ROADMAP but is not yet hosted.
- **No registered W3C DID method.** The `agid` method name has not been
  submitted to the [W3C DID Method registry](https://www.w3.org/TR/did-spec-registries/).
  Submission is on the roadmap; it is a multi-month process.
- **No verifiable-credentials integration.** Ag^id is a name primitive, not a
  credential. Use it as the subject identifier of a credential issued by some
  other system; do not use it to authenticate anything.
- **No verification CLI.** A standalone `agid-verify` binary that re-derives
  from `(domain, input, expected_did)` triples is tracked in ROADMAP.

Honest framing of these gaps belongs in the project's claims. See README's
"What this crate is and is not" section.

---

## 6. Open design questions

Genuinely open, listed here for transparency:

- **Salted variant.** Should we expose a keyed-BLAKE3 mode for callers who need to derive IDs that are not predictable to outsiders? Currently they can do this themselves by salting `input`, but a typed API would be safer. Tracked as a v1.x candidate addition (would not change v1 outputs for existing users).
- **`hex` feature flag.** Currently `to_hex_array` is always built. Some embedded callers want only base58. Could be feature-gated.
- **Constant-time equality.** Should we expose a built-in `ct_eq` method (depending on `subtle`) instead of pointing callers at the external crate? Trade-off: an extra dep vs. a clearer API.

These are deferred to v1.x minor releases pending real user feedback.
