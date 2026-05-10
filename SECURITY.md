# Security Policy

## Threat model

`ag_id` is a pure function: `(domain, input) → 32 bytes → did:agid:<base58>`.
It has no I/O, no state, no network, no clock, no randomness. The crate's
security surface is therefore narrow and consists of three things:

1. The protocol prefix and domain byte layout (`derive::raw`).
2. The integrity of the underlying BLAKE3 implementation.
3. How callers use the resulting `Did`.

## Non-goals

- **Constant-time comparison.** `Did::eq` derived via `#[derive(PartialEq)]`
  is a byte-by-byte compare and may leak timing. Do not compare secret
  authenticators with it.
- **Confidentiality of `input`.** A `Did` is a deterministic hash of its
  input. Anyone who can guess the input can recompute the `Did`. Do not
  treat a `Did` as a secret derived from a low-entropy input (email,
  username, account number) without an additional secret salt.
- **Forward secrecy or revocation.** The mapping `(domain, input) → Did` is
  permanent. There is no key to rotate. If you need revocable IDs, do not
  use this crate.
- **Authentication.** A `Did` proves nothing about who computed it. It is a
  name, not a credential.

## In scope

We treat the following as security defects:

- Any change to `derive::raw` that breaks the byte layout
  `b"agid:v1:" || domain_byte || input` within a v1.x release.
- Any code path that allocates, panics, calls `unsafe`, or performs I/O.
- Any divergence in output between platforms for the same input.
- Any way to reach the BLAKE3 hasher with a domain byte that is not the
  one the caller supplied.

## Reporting

Report suspected vulnerabilities to <security@auriglyph.com>. Please include
a minimal reproducer and the commit hash you tested against. We aim to
acknowledge within 5 working days.

Public issue trackers are appropriate for non-exploitable correctness or
documentation problems.

## Supported versions

The current `0.x` line is pre-stable. Once `1.0.0` is published, the v1.x
protocol layout will be supported with security fixes for the lifetime of
that major version.
