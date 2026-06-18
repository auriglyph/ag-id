# Claims Ledger

Every public claim made by `ag_id` (Ag^id), with its evidence, current
status, and the residual risk if the claim turns out to be wrong.

This file is **adversarial** by design: its job is to help us avoid
overclaiming. For an identifier protocol intended for ERP-class buyers,
overclaiming is the single most expensive mistake possible — one broken
claim collapses trust in the entire protocol.

**Status legend:** `verified` · `partially verified` · `unverified` · `aspirational`

**Audit date:** 2026-05-24. Re-run before every minor release.

---

| ID | Claim | Where stated | Evidence | Status | Residual risk if wrong |
|---|---|---|---|---|---|
| A-1 | "0 unsafe in production code" | SECURITY.md §In scope | `grep -rn unsafe src/` → 0 hits as of 2026-05-24; `src/lib.rs` has `#![forbid(unsafe_code)]` | **verified** | Drift risk now limited to explicit policy changes; accidental introduction is compile-time blocked. |
| A-2 | "Same input always produces the same identifier. On every platform." | README headline | `tests/determinism.rs` (7 tests); BLAKE3 is a deterministic algorithm; no platform-specific code paths in `src/` | **verified within tested platforms** | Only x86_64 Linux is currently exercised in CI. Other platforms rely on BLAKE3's documented determinism, not local CI evidence. Cross-platform CI matrix would close this. |
| A-3 | "Stable across the v1.x line" | README headline | `PREFIX = b"agid:v1:"` constant in `src/derive.rs:8`; protocol prefix encoded in every hash input; semver discipline noted in README §6 | **partially verified** | No automated check enforces that PREFIX cannot change. Manual code review only. A future PR could change the prefix and break every existing identifier. Fix: add a SPEC freeze test that asserts the byte content of `PREFIX`. |
| A-4 | "42 / 42 tests passing" (implicit by absence of failures) | (not stated in README; included here for accuracy) | `cargo test --release` 2026-05-24: 7+13+6+0+1+2+1+12 = 42 passed across unit + integration + doctest suites | **verified** | None today. Once a public CI badge is added, replace with live signal. |
| A-5 | "No I/O. No file paths. No network. No clock. No randomness." | SECURITY.md §Threat model + DESIGN.md §196 | `grep -rn 'std::fs\|std::net\|std::process\|Instant::\|rand::' src/` → 0 hits as of 2026-05-24; only dependency is `blake3` (no I/O) plus optional `serde` | **verified** | None today. Same drift risk as A-1 — no compile-time enforcement. |
| A-6 | "BLAKE3-256, unkeyed mode" | README §6, SPEC.md | `derive::raw` calls `blake3::hash` (the unkeyed entry point); `Cargo.toml` pins `blake3 = "1.5"` | **verified** | If a future change introduces `blake3::keyed_hash` or `derive_key`, the claim becomes wrong. Fix: cite `blake3::hash` in a SPEC freeze test. |
| A-7 | "Hash input layout: `b\"agid:v1:\" \|\| domain_byte \|\| input`" | README §6, SPEC.md | `src/derive.rs:8-30` — `PREFIX` + single domain byte + raw user input fed into hasher | **verified** | A reordering or extra byte would silently break every existing identifier. Fix: this is what SPEC freeze tests are for. The `test-vectors/v1.json` corpus catches it indirectly but only for the specific inputs in the corpus. |
| A-8 | "Domain separation is a security property: a User ID can never equal a Document ID" | README §Domains | Each `DeriveDomain` variant maps to a distinct `u8` tag (`domain.rs`), tag is hashed before user input, BLAKE3 collisions are not currently practical | **verified, modulo BLAKE3 assumptions** | "Never" relies on BLAKE3 collision resistance. If BLAKE3-256 collision resistance is broken in the future, this claim weakens to "computationally infeasible". |
| A-9 | "W3C DID URI ABNF-conformant string `did:agid:<base58>`" | README §6 | The wire form matches `did:method-name:method-specific-id` ABNF from DID Core 1.0 §3.1 | **verified syntactically, unverified socially** | Syntactic conformance is established by inspection of `Did::to_did_string`. The `did:agid` method is **not registered** in the W3C DID Method Registry. Claim is correct as "ABNF-conformant", but readers may incorrectly infer "registered method". Fix: README wording should say "ABNF-conformant; method registration not pursued at this stage." |
| A-10 | "no randomness" | SECURITY.md §Threat model | `derive` is `fn(domain, input) -> [u8; 32]` — no `OsRng`, no `Instant`, no entropy source touched | **verified** | None today. |
| A-11 | "no panics on valid inputs" | (implicit in API design; not yet stated) | `Did::derive` accepts any `&[u8]` of any length — BLAKE3 has no length limit; `Did::parse` returns `Result`, never panics | **partially verified** | The phrase is not yet in customer-facing docs. If stated, must be qualified: `unwrap_*` methods on the public API (if any) can still panic by contract. Audit `src/did.rs` for any `unwrap()` / `expect()` / panic-bearing methods before stating this claim publicly. |
| A-12 | "Constant-time comparison" (NEGATIVE CLAIM — explicitly not provided) | SECURITY.md §Non-goals | `Did::eq` is derived `PartialEq` (byte compare); explicitly documented as non-constant-time | **verified as a negative** | Risk is in callers misusing `Did::eq` to compare secret-derived identifiers and leaking inputs via timing. Mitigation: explicit non-goal in SECURITY.md (already present). |
| A-13 | "Confidentiality of `input`" (NEGATIVE CLAIM — explicitly not provided) | SECURITY.md §Non-goals | A `Did` is a deterministic BLAKE3 hash of `(domain, input)`. With no salt and a fast hash, dictionary attacks against low-entropy inputs are feasible. | **verified as a negative** | If a downstream user feeds emails/usernames/phone numbers into `derive` and treats the resulting `Did` as opaque, an attacker with the public `Did` can recover the input by guessing. SECURITY.md and [`examples/hmac_wrapping.rs`](../examples/hmac_wrapping.rs) document the mitigation: pre-hash sensitive inputs under a private deployment key before calling `derive`. |
| A-14 | "Forward secrecy / revocation" (NEGATIVE CLAIM — explicitly not provided) | SECURITY.md §Non-goals | Permanent function; no key to rotate; no separable "revoked" state | **verified as a negative** | Inappropriate use in revocable-identity contexts (sessions, credentials) would force product redesign on first incident. Already documented. |
| A-15 | "`did:agid` is a DID method" | DESIGN.md, conformance/, SPEC.md | The method conforms to DID Core 1.0 §3 (ABNF), §5 (identifier syntax). The method is **not** registered in the W3C DID Specification Registries. | **partially verified** | Buyers familiar with W3C process may expect registry presence. Mitigation: explicit "method registration not pursued at this stage" in README §6 and DESIGN.md. |
| A-16 | "Conformance witnesses in two languages prove cross-implementation determinism" | conformance/README.md, conformance/witnesses/python/agid_v1_witness.py | The Python witness exists (`agid_v1_witness.py`) and is exercised by `.github/workflows/ci.yml` against `test-vectors/v1.json`; one bit of divergence would break CI | **verified for v1.json corpus** | Determinism is verified only for the inputs in `v1.json`. Random adversarial inputs are exercised by `tests/properties.rs` (proptest) but only against the Rust implementation, not cross-language. Plan: add a property test that runs the Python witness in subprocess against random Rust inputs. |
| A-17 | "Cross-platform bit-identical output" | (implicit in README headline; same hash on Linux/macOS/RPi/WASM) | The README shows four lines of identical output across platforms — but those are **claims**, not CI evidence. Locally, only x86_64 Linux is reproducible today. | **partially verified** | The header image of the README is asserting platform results that no current CI job validates. Risk: a buyer running on AArch64 / Windows ARM finds a divergence that nobody saw. Fix: CI matrix across `x86_64-linux`, `aarch64-apple-darwin`, `x86_64-pc-windows-gnu`, and `wasm32-unknown-unknown` running the same `test-vectors/v1.json` corpus. |
| A-18 | "Production-ready" / "production" status | NOT CURRENTLY CLAIMED | (the README does not state "production". It says "pre-stable 0.x line" in SECURITY.md §Supported versions) | **n/a — claim is correctly absent** | Reputation risk only arises if a future README change introduces "production-ready" without external audit + cross-platform CI. This row exists to prevent future drift. |
| A-19 | "Dependencies: minimal" | README §6 | Production deps: `blake3` (1) + optional `serde` (1) = 2 prod deps. Dev-deps add ~125 (criterion, proptest, serde_json + transitive). | **verified for production tree** | Dev-deps are an attack surface for the development environment (xz-utils lesson). `cargo audit` + `cargo deny` are not yet configured. Tracked under separate hardening item. |
| A-20 | "Output length: 32 bytes binary / 44–48 chars base58" | SPEC.md | BLAKE3 default output is 32 bytes; base58 encoding of 32 bytes is 44-46 chars (variable) | **verified** | Variable-length base58 output may surprise integrators expecting fixed-width identifiers. Already documented in SPEC.md. |

---

## Standing rules

1. **Every PR that touches a row's evidence column must update the row.** The row is the contract; the code is the implementation.
2. **No new claim enters customer-facing docs until it has a row here.** README / SECURITY.md / DESIGN.md changes must reference the row ID they support.
3. **Status `verified` requires reproducible evidence**, not "looks right". A grep output, a test name, a hash, or a CI link.
4. **Negative claims (A-12, A-13, A-14) are also tracked.** They are statements about what `ag_id` is *not*, and they are equally load-bearing for trust.

## Pending hardening items (referenced from rows above)

| Item | Closes rows | Description |
|---|---|---|
| H-1 | A-1, A-5 | **Completed 2026-05-26.** `#![forbid(unsafe_code)]` added to `src/lib.rs`; compile-time enforcement active. |
| H-2 | A-3, A-6, A-7 | SPEC freeze test: a unit test in `tests/spec_freeze.rs` that asserts the byte content of `derive::PREFIX`, the BLAKE3 entry point used, and the exact input layout. |
| H-3 | A-2, A-17 | Cross-platform CI matrix. At minimum: `ubuntu-latest`, `macos-latest`, `windows-latest` running `cargo test` + `test-vectors/v1.json`. |
| H-4 | A-13 | **Completed 2026-05-26.** Added `examples/hmac_wrapping.rs` demonstrating deployment-key wrapping for low-entropy inputs. |
| H-5 | A-16 | Cross-language property test: random Rust input → fed to Python witness via subprocess → assert equal output. |
| H-6 | A-19 | Wire `cargo audit` + `cargo deny` into CI. Block CVE-bearing dev-deps too. |
| H-7 | A-9, A-15 | README + DESIGN.md wording: "ABNF-conformant; method registration in the W3C registry is not pursued at this stage." |
