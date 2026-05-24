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

## Linkability across systems

Determinism is the protocol's purpose, and also its main misuse risk.

If two independent systems run `derive(DeriveDomain::User, alice@example.com)`,
they produce **the same** `did:agid:...`. This is the feature: that is how
two parties can refer to the same entity without exchanging keys. It is
also the failure mode: an attacker who observes the public `Did` from one
system can recompute, on their own machine, what input produced it — and
then trivially link the same person across every other system that uses
`ag_id`.

For low-entropy inputs (emails, usernames, phone numbers, account numbers,
IBANs, government IDs) this means: **the `Did` is not a one-way function in
practice**. It is one-way for high-entropy inputs only.

### Recommended pattern for sensitive inputs

If your input has fewer than ~80 bits of entropy and you need
unlinkability across deployments, do not feed it directly into `derive`.
Wrap it under a deployment-local secret first:

```text
secret_input  =  HMAC-BLAKE3(K_deployment, raw_user_input)
did           =  ag_id::derive(domain, secret_input)
```

`K_deployment` is a 32-byte secret held by the deploying system and never
shipped with the data. Different deployments produce different `Did`s for
the same `raw_user_input`, so cross-deployment linkability is broken. This
also prevents recovery of `raw_user_input` from a leaked `Did` without
access to `K_deployment`.

`ag_id` does not provide this wrapper itself: doing so would mean keeping a
secret, which contradicts the crate's zero-state invariant. The wrapper
belongs at the caller layer.

A worked example will be added under `examples/hmac_wrapping.rs` —
tracked as `H-4` in [`docs/CLAIMS_LEDGER.md`](docs/CLAIMS_LEDGER.md).

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
