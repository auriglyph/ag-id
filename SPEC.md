# Ag^id — Protocol Specification (v1)

**Status:** Draft for stabilisation. Frozen as of v0.1.x of the Rust reference
implementation (`ag_id` crate).
**Audience:** Authors of conforming implementations in other languages, integrators
verifying interoperability, and reviewers auditing the byte-level contract.

This document specifies the bytes that flow through an `Ag^id` derivation. It is the
source of truth that `test-vectors/v1.json` verifies. Any implementation that
produces the same byte sequences from the same inputs is conformant; any that does
not is not.

---

## 1. Notation

- `||` denotes byte concatenation.
- `0xNN` is a single byte with the given hex value.
- All multi-byte literals are UTF-8 encoded unless explicitly stated.
- `BLAKE3-256(x)` is the standard 32-byte BLAKE3 hash of `x` with no key, no
  context, no derive-key parameters. Reference:
  <https://github.com/BLAKE3-team/BLAKE3>.

---

## 2. Inputs

A derivation takes:

| Field | Type | Constraints |
|---|---|---|
| `domain_byte` | `u8` | Value from §3 (or any non-`0x00` byte for custom). |
| `input` | byte sequence | Any length ≥ 0 bytes. Empty is allowed and well-defined. |

There is no salt, no clock, no random, no keying material, no environment, and no
I/O. The function is pure.

---

## 3. Domain bytes

| Symbolic name | Byte | Meaning |
|---|---|---|
| `User` | `0x01` | Human or machine identity. |
| `Document` | `0x02` | File, document, content. |
| `Session` | `0x03` | Session, transaction, request. |
| `Device` | `0x04` | Physical or virtual device. |
| `Concept` | `0x05` | Semantic concept or anchor. |
| `Custom(b)` | any non-`0x00` `b` | Caller-allocated namespace. |
| `Opaque` (reserved) | `0x00` | Sentinel for parsed values. **Never used as input** to the hash. Implementations MUST reject derivations where `domain_byte = 0x00`. |

The bytes `0x06..=0xFF` (excluding `0x00`) are reserved for future built-in domains
or for caller use via `Custom`. Implementations MUST NOT silently remap one byte
to another.

---

## 4. The protocol prefix

```
PREFIX := b"agid:v1:"   (8 bytes, ASCII)
```

Hex: `61 67 69 64 3a 76 31 3a`.

The prefix is part of the v1 contract. Changing it produces a different v2
protocol. Implementations MUST feed exactly these 8 bytes, in exactly this order,
before the domain byte.

---

## 5. The hash

```
raw[0..32] := BLAKE3-256( PREFIX || domain_byte || input )
```

That is the **only** hash call in a conformant derivation. There is no length
prefix on `input`, no Merkle tree wrapper, no domain-separated keying. The 8-byte
prefix and the 1-byte domain are sufficient domain separation.

Note: BLAKE3 itself is keyed/derive-key-aware, but `Ag^id` v1 uses the *unkeyed*
mode only (the default `Hasher::new()` in the reference). Conforming
implementations MUST use the unkeyed `BLAKE3-256` mode.

---

## 6. The DID-URI string form

```
did_string := "did:agid:" || base58btc( raw )
```

- `"did:agid:"` is 9 ASCII bytes: `64 69 64 3a 61 67 69 64 3a`.
- `base58btc` is the Bitcoin base58 alphabet (no `0`, `O`, `I`, `l`):

```
123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
```

- Encoding follows the standard "leading-zero counting" rule: each leading `0x00`
  byte in `raw` is encoded as a leading `1` in the output.
- The output payload (after `did:agid:`) is between 1 and 44 characters. For
  uniformly distributed 32-byte inputs the typical length is ~43–44 characters.

The full `did_string` URI conforms to [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986)
generic-URI syntax with method `agid`.

---

## 7. Hex form (optional)

```
hex_string := lowercase_hex( raw )
```

- 64 ASCII characters in `[0-9a-f]`.
- Useful for debugging and for tooling that does not handle base58.
- Implementations MAY expose this; the canonical wire form is the `did:agid:`
  string.

---

## 8. Parsing (`did:agid:` → 32 bytes)

A conformant parser:

1. Verifies the input begins with `"did:agid:"`. If not → reject.
2. Takes the trailing payload. If empty or longer than 44 ASCII characters →
   reject.
3. Verifies every payload character is in the base58btc alphabet → if not, reject.
4. Decodes the payload to exactly 32 raw bytes. If the decoded value would not
   fit in 32 bytes, or the leading-zero count of the decoding does not match the
   leading-`1` count of the encoding → reject.
5. Returns the 32 bytes.

The parser does **not** recover the original `domain_byte`. The original domain
is not present in the wire form. Implementations that wrap the result in a typed
value SHOULD use a sentinel domain (the Rust reference uses `Domain::Opaque`) and
document that parsed values are byte-identifiable but domain-anonymous.

---

## 9. Determinism contract

Two implementations on any two platforms MUST produce byte-identical outputs
(`raw`, `did_string`, `hex_string`) for the same `(domain_byte, input)` pair,
given the same protocol version (v1). This includes:

- Endianness must not affect output.
- BLAKE3 SIMD path selection must not affect output (BLAKE3's specification
  already guarantees this; `Ag^id` does not introduce any platform-conditional
  code paths).
- Locale, time zone, and process state must not affect output.

Conforming implementations are expected to ship the test vectors from
`test-vectors/v1.json` as part of their CI.

---

## 10. Stability commitments (v1.x)

The following are part of the **major-version** contract. Changing any of them
in a v1.x release is a defect:

- The 8-byte `PREFIX` value (`b"agid:v1:"`).
- The byte assignments `User=0x01`, `Document=0x02`, `Session=0x03`, `Device=0x04`,
  `Concept=0x05`.
- The reservation of `0x00` for `Opaque`.
- The base58 alphabet (Bitcoin variant, exact ordering).
- The `did:agid:` URI prefix.
- The 32-byte raw output length.

The following are **minor-version** changes (do not break v1.x clients):

- Adding new built-in domain symbols. Their byte assignments must be in
  `0x06..=0xFF` and once published cannot be changed.
- Adding new APIs, examples, lints, or documentation.

Bug fixes that do not change the `(domain_byte, input) → raw` mapping are
**patch** changes.

---

## 11. Hash agility — v2 outlook (non-normative)

If BLAKE3 is ever weakened to the point that this protocol must move, the
migration path is:

- A new prefix: `b"agid:v2:"`.
- A new URI prefix: `did:agid2:` (or a fresh method registered with W3C).
- Implementations MAY expose both v1 and v2 derivation in the same library,
  behind separate functions. They MUST NOT silently produce v2 outputs from
  APIs that previously produced v1.

This document does not specify a v2 hash; it only commits to the migration shape
so callers can plan ahead.

---


## 12. Security considerations

See [`SECURITY.md`](SECURITY.md) for the full threat model. Salient points:

- `Ag^id` is a **name**, not a credential. It authenticates nothing.
- Equality (`PartialEq`) on `Did` is **not** constant-time. Use a constant-time
  comparator for capability-token comparisons.
- The mapping is permanent. There is no rotation, no revocation. If `input` is
  sensitive (e.g. an email address), the derived `Did` is a stable pseudonym for
  it forever.
- Truncated displays (showing fewer than the full 32 bytes / 44 base58
  characters) are not safe for uniqueness checks.

---

## 13. Test vectors

See [`test-vectors/v1.json`](test-vectors/v1.json) for canonical inputs and
outputs. The Rust reference implementation has an integration test
(`tests/vectors_json.rs`) that asserts the JSON file matches the live
implementation; conforming re-implementations should ship an equivalent test.

---

## 14. References

- BLAKE3 specification: <https://github.com/BLAKE3-team/BLAKE3-specs>
- Base58 (Bitcoin variant): <https://en.bitcoin.it/wiki/Base58Check_encoding>
- W3C DID Core 1.0: <https://www.w3.org/TR/did-core/>
- RFC 3986 (URI Generic Syntax): <https://www.rfc-editor.org/rfc/rfc3986>
