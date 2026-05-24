#!/usr/bin/env python3
"""Independent Python witness for Ag^id v1 conformance.

Reads test-vectors/v1.json and verifies that every positive vector is
reproducible by an independent BLAKE3 implementation, and that every negative
case would be rejected by a conforming parser.

Depends only on:
    - the third-party `blake3` package (pip install blake3)
    - the Python standard library

Does not read or depend on any Rust source.

Usage:
    python3 agid_v1_witness.py [path/to/v1.json]

Exit codes:
    0  all vectors verified and all negative cases rejected
    1  one or more mismatches
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    from blake3 import blake3
except ImportError:
    print("error: this witness requires the `blake3` package.", file=sys.stderr)
    print("       install with: pip install blake3", file=sys.stderr)
    sys.exit(2)


# Constants — must match SPEC.md §4 and §6 exactly.
PREFIX = b"agid:v1:"
DID_URI_PREFIX = "did:agid:"
BASE58_ALPHABET = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
BASE58_ALPHABET_SET = set(BASE58_ALPHABET.decode("ascii"))
BASE58_DECODE_TABLE = {c: i for i, c in enumerate(BASE58_ALPHABET.decode("ascii"))}

# Canonical built-in domain byte assignments per SPEC.md §3. These are frozen
# for the v1.x line; mismatches in a test vector indicate a corrupted file.
BUILTIN_DOMAIN_BYTES: dict[str, int] = {
    "User": 0x01,
    "Document": 0x02,
    "Session": 0x03,
    "Device": 0x04,
    "Concept": 0x05,
}


class ReservedDomainError(ValueError):
    """Raised when a derivation is attempted with the reserved domain byte 0x00."""


def base58_encode_32(raw: bytes) -> str:
    """Encode exactly 32 bytes as base58btc with leading-zero counting."""
    if len(raw) != 32:
        raise ValueError("expected exactly 32 bytes")
    n = int.from_bytes(raw, "big")
    out: list[bytes] = []
    while n > 0:
        n, rem = divmod(n, 58)
        out.append(BASE58_ALPHABET[rem : rem + 1])
    # Leading 0x00 bytes become leading '1' chars.
    leading_zero_bytes = 0
    for b in raw:
        if b == 0:
            leading_zero_bytes += 1
        else:
            break
    encoded = b"1" * leading_zero_bytes + b"".join(reversed(out))
    return encoded.decode("ascii")


def base58_decode_to_32(payload: str) -> bytes | None:
    """Decode a base58btc string to exactly 32 bytes, mirroring src/encode.rs::from_base58_to_32.

    Returns None if any character is outside the alphabet, the decoded value
    would not fit in 32 bytes (overflow), or the leading-zero count of the
    decoding does not match the leading-`1` count of the encoding.
    """
    if not payload or len(payload) > 44:
        return None

    # Count leading '1' chars — each represents a leading 0x00 byte.
    leading_ones = 0
    for ch in payload:
        if ch == "1":
            leading_ones += 1
        else:
            break
    if leading_ones > 32:
        return None

    bytes_buf = bytearray(32)
    for ch in payload:
        digit = BASE58_DECODE_TABLE.get(ch)
        if digit is None:
            return None
        carry = digit
        # Multiply existing value by 58 and add carry, big-endian.
        for i in range(31, -1, -1):
            carry += bytes_buf[i] * 58
            bytes_buf[i] = carry & 0xFF
            carry >>= 8
        if carry != 0:
            return None  # overflow — does not fit in 32 bytes

    # Leading-zero alignment: decoded value must have at least as many leading
    # 0x00 bytes as the encoded form has leading '1' chars.
    actual_leading_zeros = 0
    for b in bytes_buf:
        if b == 0:
            actual_leading_zeros += 1
        else:
            break
    if actual_leading_zeros < leading_ones:
        return None

    return bytes(bytes_buf)


def derive_raw(domain_byte: int, payload: bytes) -> bytes:
    """The single hash call defined in SPEC.md §5.

    Raises ReservedDomainError if `domain_byte` is 0x00 — that byte is reserved
    for `Domain::Opaque` and must never be used as hash input. This mirrors
    `ag_id::Did::try_derive` returning `Error::ReservedDomain`.
    """
    if domain_byte == 0x00:
        raise ReservedDomainError("domain_byte 0x00 is reserved (SPEC.md §3)")
    return blake3(PREFIX + bytes([domain_byte]) + payload).digest()


def domain_byte_for(name: str, byte_str: str) -> int:
    """Resolve the domain byte for a test vector, asserting that the JSON's
    `domain_byte` field matches the canonical value for built-in domains.

    Built-in domains have fixed byte assignments per SPEC.md §3; the JSON-provided
    `domain_byte` is treated as a redundant declaration that the witness verifies
    rather than ignores. A corrupted vector like
    `{"domain":"User","domain_byte":"0x09"}` will be rejected here.
    """
    trimmed = byte_str.lower().removeprefix("0x")
    declared = int(trimmed, 16)
    if name in BUILTIN_DOMAIN_BYTES:
        canonical = BUILTIN_DOMAIN_BYTES[name]
        if declared != canonical:
            raise ValueError(
                f"built-in domain {name!r} must have byte 0x{canonical:02x}; "
                f"JSON declared 0x{declared:02x}"
            )
        return canonical
    if name == "Custom":
        return declared
    raise ValueError(f"unknown domain in JSON: {name!r}")


def input_bytes_for(vector: dict) -> bytes:
    if vector["name"] == "long_input_1024_zeros":
        return b"\x00" * 1024
    return vector["input_utf8"].encode("utf-8")


def check_positive_vector(vector: dict) -> tuple[bool, str]:
    name = vector["name"]
    try:
        dom_byte = domain_byte_for(vector["domain"], vector["domain_byte"])
    except ValueError as err:
        return False, f"  domain_byte sanity: {err}"
    payload = input_bytes_for(vector)

    actual_raw = derive_raw(dom_byte, payload)
    actual_hex = actual_raw.hex()
    if actual_hex != vector["raw_hex"]:
        return False, f"  raw_hex mismatch: got {actual_hex}, expected {vector['raw_hex']}"

    actual_did = DID_URI_PREFIX + base58_encode_32(actual_raw)
    if actual_did != vector["did_string"]:
        return False, f"  did_string mismatch: got {actual_did}, expected {vector['did_string']}"

    return True, ""


def parse_shape_error(s: str) -> str | None:
    """Return the expected error name for the given parse input, or None if it
    would parse successfully. Mirrors `ag_id::Did::parse` step-for-step,
    including the base58 decode + leading-zero alignment check.
    """
    if not s.startswith(DID_URI_PREFIX):
        return "MissingPrefix"
    payload = s[len(DID_URI_PREFIX):]
    if len(payload) == 0 or len(payload) > 44:
        return "WrongLength"
    if not payload.isascii():
        return "InvalidBase58"
    if not all(c in BASE58_ALPHABET_SET for c in payload):
        return "InvalidBase58"
    # Actually attempt the base58 → 32-byte decode. The Rust reference
    # returns InvalidBase58 for overflow or leading-zero misalignment.
    if base58_decode_to_32(payload) is None:
        return "InvalidBase58"
    return None  # would parse


def check_negative_parse(case: dict) -> tuple[bool, str]:
    expected = case["expected_error"]
    actual = parse_shape_error(case["input_string"])
    if actual is None:
        return False, f"  expected {expected}; case would have parsed successfully"
    if actual != expected:
        return False, f"  expected {expected}; would fire {actual}"
    return True, ""


def check_negative_derivation(case: dict) -> tuple[bool, str]:
    expected = case["expected_error"]
    if expected != "ReservedDomain":
        return False, f"  unrecognised derivation rejection: {expected}"
    declared = int(case["domain_byte"], 16)
    if declared != 0x00:
        return False, "  ReservedDomain must correspond to domain_byte 0x00"
    # Actually attempt the derivation and confirm the rejection fires.
    try:
        derive_raw(declared, b"witness-derivation-rejection")
    except ReservedDomainError:
        return True, ""
    return False, "  derive_raw(0x00, ...) did not raise ReservedDomainError"


def main() -> int:
    json_path = Path(sys.argv[1]) if len(sys.argv) > 1 else (
        Path(__file__).resolve().parents[3] / "test-vectors" / "v1.json"
    )
    if not json_path.is_file():
        print(f"error: cannot find {json_path}", file=sys.stderr)
        return 2

    with json_path.open(encoding="utf-8") as f:
        data = json.load(f)

    if data.get("spec_version") != "1":
        print(f"error: spec_version != '1' (got {data.get('spec_version')!r})", file=sys.stderr)
        return 2
    if data.get("protocol", {}).get("prefix") != "agid:v1:":
        print("error: protocol.prefix != 'agid:v1:'", file=sys.stderr)
        return 2
    if data.get("protocol", {}).get("uri_prefix") != "did:agid:":
        print("error: protocol.uri_prefix != 'did:agid:'", file=sys.stderr)
        return 2

    failures: list[str] = []

    positive = data.get("vectors", [])
    print(f"checking {len(positive)} positive vectors")
    for v in positive:
        ok, msg = check_positive_vector(v)
        status = "OK" if ok else "FAIL"
        print(f"  [{status}] {v['name']}")
        if not ok:
            failures.append(f"positive {v['name']}: {msg}")

    negative = data.get("negative_cases", {})
    deriv = negative.get("derivation_rejections", [])
    parse = negative.get("parse_rejections", [])
    print(f"checking {len(deriv)} derivation rejections")
    for c in deriv:
        ok, msg = check_negative_derivation(c)
        status = "OK" if ok else "FAIL"
        print(f"  [{status}] {c['name']}")
        if not ok:
            failures.append(f"derivation {c['name']}: {msg}")

    print(f"checking {len(parse)} parse rejections")
    for c in parse:
        ok, msg = check_negative_parse(c)
        status = "OK" if ok else "FAIL"
        print(f"  [{status}] {c['name']}")
        if not ok:
            failures.append(f"parse {c['name']}: {msg}")

    print()
    if failures:
        print(f"FAIL: {len(failures)} witness assertion(s) failed:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print(
        f"PASS: {len(positive)} positive + "
        f"{len(deriv)} derivation rejections + "
        f"{len(parse)} parse rejections all match the Rust reference contract."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
