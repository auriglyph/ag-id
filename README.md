# Ag^id

**The same input always produces the same identifier. On every platform. Stable across the v1.x line.**

For input `(DeriveDomain::User, b"alice@example.com")`:

```text
did:agid:2mDwJhrvWdJsqHAhRTQWpaLgWmnTZxEZJv6hnDmjiYtt  ← Linux x86-64
did:agid:2mDwJhrvWdJsqHAhRTQWpaLgWmnTZxEZJv6hnDmjiYtt  ← macOS ARM
did:agid:2mDwJhrvWdJsqHAhRTQWpaLgWmnTZxEZJv6hnDmjiYtt  ← Raspberry Pi
did:agid:2mDwJhrvWdJsqHAhRTQWpaLgWmnTZxEZJv6hnDmjiYtt  ← WASM in browser
```

No database. No coordination. No randomness.

---

## Why

UUID v4 is random. That's fine until two nodes need to agree on the same identifier
for the same entity — without talking to each other. Ag^id solves this with a
single BLAKE3 hash: same input → same ID, always.

This matters for:
- **Distributed systems** — deterministic entity IDs across nodes
- **Content addressing** — stable document IDs that survive moves/renames
- **Replayable AI pipelines** — entity references that survive serialisation
- **Edge/offline apps** — generate IDs without a server

---

## Usage

```toml
[dependencies]
ag_id = "0.1"
```

```rust
use ag_id::{DeriveDomain, Did};

// Derive an identifier
let id = Did::derive(DeriveDomain::User, b"alice@example.com");
println!("{}", id); // did:agid:2mDwJhrvWdJsqHAhRTQWpaLgWmnTZxEZJv6hnDmjiYtt

// Same input, same result — every time, everywhere
let id2 = Did::derive(DeriveDomain::User, b"alice@example.com");
assert_eq!(id, id2);

// Different domains → different IDs (even with same input)
let doc_id = Did::derive(DeriveDomain::Document, b"alice@example.com");
assert_ne!(id, doc_id);

// Round-trip through the wire form
let s = id.to_did_string();
let parsed: Did = s.parse().expect("did:agid:<base58>");
assert!(parsed.eq_bytes(&id));
```

`serde` integration is opt-in via the `serde` feature flag. With it enabled,
`Did` serialises as a `"did:agid:<base58>"` string in JSON/YAML/TOML and as 32
raw bytes in binary formats (`bincode`, `postcard`, `MessagePack`).

For a runnable end-to-end walkthrough see [`examples/basic.rs`](examples/basic.rs)
and [`examples/parsing.rs`](examples/parsing.rs).

---

## Domains

| Domain | Byte | Use case |
|--------|------|----------|
| `User` | `0x01` | Human or machine identity |
| `Document` | `0x02` | File, document, content |
| `Session` | `0x03` | Session or transaction |
| `Device` | `0x04` | Physical or virtual device |
| `Concept` | `0x05` | Semantic concept |
| `Custom(u8)` | any | Your namespace |

Domain separation is a security property: a `User` ID can never equal a `Document` ID
even if both were derived from the same bytes.

---

## How it works

```text
BLAKE3("agid:v1:" || domain_byte || input)  →  32 bytes  →  did:agid:<base58>
```

That's it. No state. No clock. No random. Pure function.

---

## Features

- `no_std` compatible (default: `std` feature enabled)
- Zero heap allocations in the hot path
- BLAKE3-256 hash; `Hasher::new()` (unkeyed) mode only
- Wire form is a W3C DID URI ABNF–conformant string (`did:agid:<base58>`); a
  W3C DID method registration, resolver, and DID Document layer are tracked
  on the roadmap but are not implemented in this crate today (see
  [`DESIGN.md` §6](DESIGN.md#6-what-is-not-yet-implemented))
- Cross-platform by construction (verified by independent Python witness in
  [`conformance/`](conformance/))

---

## Performance

Indicative numbers from `cargo bench --bench throughput` on a Ryzen 9 7900X
(BLAKE3 with AVX-512). Reproduce on your hardware before quoting — see
[`BENCHMARKS.md`](BENCHMARKS.md) for full methodology, environment, and a
larger result table.

```text
derive/16     time: [67.2 ns]   throughput: [227.0 MiB/s]
derive/1024   time: [833.4 ns]  throughput: [1.14 GiB/s]
derive/65536  time: [9.96 µs]   throughput: [6.13 GiB/s]
```

---

## What this crate is and is not

**This crate is:**

- A deterministic identifier primitive: pure function `(domain, input) → 32 bytes`.
- A canonical wire form: `did:agid:<base58>`, conforming to the W3C DID URI ABNF.
- A domain-separated derivation scheme: same input in different `DeriveDomain`s
  produces different identifiers, enforced by the protocol prefix and the
  1-byte domain in the hash input.
- A portable, protocol-style spec with cross-language test vectors in
  [`test-vectors/v1.json`](test-vectors/v1.json) and an independent Python
  witness in [`conformance/`](conformance/).

**This crate is not (yet):**

- A registered W3C DID method. The `agid` name has not been submitted to the
  [W3C DID Method registry](https://www.w3.org/TR/did-spec-registries/);
  submission is on the [roadmap](ROADMAP.md).
- A DID resolver. There is no function returning a structured `DidDocument`
  from a `did:agid:` URI in this crate today.
- A full DID Document layer. No JSON-LD context, no `service` arrays, no
  `verificationMethod` arrays.
- An authentication, credentials, signing, or attestation system. `Did` is a
  name, not a credential — it proves nothing about who computed it. Do not
  use `==` between `Did` values as a proof of control or possession.
- An encryption scheme. Ag^id does not encrypt anything. The only security
  property is collision resistance from BLAKE3-256 and domain separation
  from the 8-byte prefix plus 1-byte domain.

---

## Limitations

Read these before using `Ag^id` in security-sensitive contexts.

- **Not constant-time.** `Did::eq` is a byte-by-byte compare. Do **not** use it
  to compare secret authenticators or capability tokens — use a constant-time
  comparator (e.g. `subtle::ConstantTimeEq`) instead.
- **Collision strength is 256-bit.** Cryptographically strong against random
  collisions, but identifier length (≤44 base58 chars) is the user-visible
  surface — not security strength.
- **No deletion, no rotation.** A `Did` is a pure function of `(domain, input)`.
  If `input` is sensitive (e.g. an email address), the resulting `Did` is a
  stable pseudonym for that input forever. There is no server-side
  invalidation. Hash inputs you control, not raw PII, when this matters.
- **Custom domain separation is enforced by discipline.**
  `DeriveDomain::custom(b)` lets callers pick any non-zero byte; if two systems
  pick the same byte for different semantics, their IDs collide by design.
  Reserve `0x01..=0x05` for the built-in variants and document your
  custom-byte allocations.
- **Not for adversarial uniqueness.** A determined attacker can grind inputs
  until two map to the same prefix substring. The full 32-byte ID resists
  this, but **truncated** displays do not. Never truncate a `Did` for
  uniqueness checks.
- **Stability contract is v1.x only.** The protocol prefix
  (`b"agid:v1:"`) and the `Domain` byte assignments are part of the
  semver-major contract. A `v2` would intentionally produce different IDs.

---

## Licence

Licensed under either of:

- MIT licence ([`LICENSE-MIT`](LICENSE-MIT))
- Apache Licence, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))

at your option.

© 2026 Mikhail Kostan / `AuriGlyph`.

---

## Further reading

- [`SPEC.md`](SPEC.md) — formal protocol specification (re-implementation reference).
- [`DESIGN.md`](DESIGN.md) — design rationale and prior-art comparison.
- [`ROADMAP.md`](ROADMAP.md) — pre-1.0 stabilisation, post-1.0 plans, v2 outlook.
- [`SECURITY.md`](SECURITY.md) — threat model and disclosure policy.
- [`BENCHMARKS.md`](BENCHMARKS.md) — methodology + environment block.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to send patches.
- [`test-vectors/v1.json`](test-vectors/v1.json) — canonical cross-language vectors (positive + negative cases).
- [`conformance/README.md`](conformance/README.md) — conformance suite overview and how to add a new-language witness.

---

*Built by [AuriGlyph](https://auriglyph.com). Project home: <https://auriglyph.com/projects/ag_id>.*
