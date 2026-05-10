#![allow(non_snake_case)]
//! Cross-platform test vectors for `ag_id` (Ag^id).

use ag_id::{Did, Domain};

struct Vector {
    domain: Domain,
    input: &'static [u8],
    /// Expected did:agid:... string.
    expected: &'static str,
}

#[test]
fn did__derive__cross_platform_vectors__match_hardcoded_values() {
    let vectors = [Vector {
        domain: Domain::User,
        input: b"alice@example.com",
        expected: "did:agid:2mDwJhrvWdJsqHAhRTQWpaLgWmnTZxEZJv6hnDmjiYtt",
    }];

    for v in &vectors {
        let id = Did::derive(v.domain, v.input);
        let s = id.to_string();
        assert_eq!(
            s, v.expected,
            "platform mismatch!\n  input: {:?}\n  got:      {}\n  expected: {}",
            v.input, s, v.expected
        );
    }
}

#[test]
fn did__display__format__is_stable_base58() {
    let id = Did::derive(Domain::User, b"format-test");
    let s = id.to_string();
    // DID URI scheme
    assert!(s.starts_with("did:agid:"));
    // Only base58 chars after did:agid:
    let payload = &s["did:agid:".len()..];
    for c in payload.chars() {
        assert!(
            "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c),
            "non-base58 char: {c}"
        );
    }
}
