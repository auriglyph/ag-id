//! Property tests — invariants that must hold for arbitrary inputs.

use ag_id::{DeriveDomain, Did, Domain};
use proptest::prelude::*;

fn arb_domain() -> impl Strategy<Value = DeriveDomain> {
    prop_oneof![
        Just(DeriveDomain::User),
        Just(DeriveDomain::Document),
        Just(DeriveDomain::Session),
        Just(DeriveDomain::Device),
        Just(DeriveDomain::Concept),
        // Custom: any non-zero byte (0x00 is the Opaque sentinel).
        (1u8..=255).prop_map(|b| DeriveDomain::custom(b).expect("non-zero domain")),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// `derive` is deterministic for any input.
    #[test]
    fn derive_is_deterministic(
        domain in arb_domain(),
        input in any::<Vec<u8>>(),
    ) {
        let a = Did::derive(domain, &input);
        let b = Did::derive(domain, &input);
        prop_assert_eq!(a, b);
    }

    /// Different domains with the same input never collide.
    /// (Probabilistic — relies on BLAKE3 collision resistance, but for
    /// any concrete pair of distinct (domain_byte, input) the probability
    /// of collision is ~2⁻²⁵⁶.)
    #[test]
    fn different_domains_differ(
        input in any::<Vec<u8>>(),
    ) {
        let a = Did::derive(DeriveDomain::User, &input);
        let b = Did::derive(DeriveDomain::Document, &input);
        prop_assert_ne!(a.as_bytes(), b.as_bytes());
    }

    /// `to_did_string` ↔ `parse` is a perfect round-trip on the byte level.
    #[test]
    fn parse_roundtrips_string(
        domain in arb_domain(),
        input in any::<Vec<u8>>(),
    ) {
        let typed = Did::derive(domain, &input);
        let s = typed.to_did_string();
        let parsed = Did::parse(&s).expect("round-trip");
        prop_assert!(typed.eq_bytes(&parsed));
        prop_assert_eq!(parsed.domain(), Domain::Opaque);
    }

    /// `from_bytes` ↔ `as_bytes` is a perfect round-trip.
    #[test]
    fn from_bytes_roundtrip(
        bytes in any::<[u8; 32]>(),
    ) {
        let did = Did::from_bytes(bytes);
        prop_assert_eq!(*did.as_bytes(), bytes);
    }

    /// Hex output is always 64 lowercase hex characters.
    #[test]
    fn hex_is_64_lowercase(
        domain in arb_domain(),
        input in any::<Vec<u8>>(),
    ) {
        let did = Did::derive(domain, &input);
        let hex = did.to_hex_array();
        prop_assert_eq!(hex.len(), 64);
        for &c in &hex {
            prop_assert!(matches!(c, b'0'..=b'9' | b'a'..=b'f'));
        }
    }

    /// DID string always starts with `did:agid:` and uses the base58 alphabet.
    #[test]
    fn did_string_format_invariants(
        domain in arb_domain(),
        input in any::<Vec<u8>>(),
    ) {
        let s = Did::derive(domain, &input).to_did_string();
        prop_assert!(s.starts_with("did:agid:"));
        let payload = &s["did:agid:".len()..];
        prop_assert!(!payload.is_empty());
        prop_assert!(payload.len() <= 44);
        for c in payload.chars() {
            prop_assert!(
                "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c),
                "non-base58 char: {}",
                c
            );
        }
    }
}
