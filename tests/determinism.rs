#![allow(non_snake_case)]
//! Determinism tests for `ag_id` (Ag^id).

use ag_id::{derive, Domain};

#[test]
fn did__derive__same_input__same_output() {
    let a = derive(Domain::User, b"test@example.com");
    let b = derive(Domain::User, b"test@example.com");
    assert_eq!(a, b);
}

#[test]
fn did__derive__different_input__different_output() {
    let a = derive(Domain::User, b"alice");
    let b = derive(Domain::User, b"bob");
    assert_ne!(a, b);
}

#[test]
fn did__derive__different_domains__different_ids() {
    let user = derive(Domain::User, b"same-input");
    let doc = derive(Domain::Document, b"same-input");
    let sess = derive(Domain::Session, b"same-input");
    let dev = derive(Domain::Device, b"same-input");
    // All four must be distinct
    assert_ne!(user, doc);
    assert_ne!(user, sess);
    assert_ne!(user, dev);
    assert_ne!(doc, sess);
    assert_ne!(doc, dev);
    assert_ne!(sess, dev);
}

#[test]
fn did__derive__empty_input__is_valid() {
    let id = derive(Domain::User, b"");
    assert_eq!(id.as_bytes().len(), 32);
}

#[test]
fn did__derive__large_input__is_valid() {
    let big = vec![0xABu8; 1_000_000];
    let id = derive(Domain::Document, &big);
    assert_eq!(id.as_bytes().len(), 32);
}

#[test]
fn did__derive__unicode_input__is_stable() {
    let a = derive(Domain::User, "Михаил".as_bytes());
    let b = derive(Domain::User, "Михаил".as_bytes());
    assert_eq!(a, b);
}

#[test]
fn did__display__starts_with_did_agid__is_true() {
    let id = derive(Domain::User, b"test");
    let s = id.to_string();
    assert!(s.starts_with("did:agid:"), "got: {s}");
}

#[test]
fn did__as_bytes__length__is_32() {
    let id = derive(Domain::Concept, b"semantic-anchor");
    assert_eq!(id.as_bytes().len(), 32);
}

#[test]
fn did__derive__custom_domain__works() {
    let a = derive(Domain::Custom(0xFF), b"input");
    let b = derive(Domain::Custom(0xFF), b"input");
    assert_eq!(a, b);
    // Must differ from standard domains
    assert_ne!(a, derive(Domain::User, b"input"));
}

#[test]
fn did__clone__equals_original__is_true() {
    let id = derive(Domain::Session, b"session-token");
    let cloned = id.clone();
    assert_eq!(id, cloned);
}

/// Golden vector — MUST NEVER CHANGE between versions.
/// If this test fails after a refactor, you broke the protocol.
///
/// Hex value below is pinned to the canonical hash for the v1 protocol
/// (`b"agid:v1:"` || `Domain::User` || `b"agid:golden:v1"`). It is also
/// mirrored in `test-vectors/v1.json` under `golden_v1_anchor`.
#[test]
fn did__derive__golden_input__matches_hardcoded_hex() {
    let id = derive(Domain::User, b"agid:golden:v1");
    let hex = id.to_hex_array();
    let hex_str = std::str::from_utf8(&hex).expect("invalid hex utf8");
    assert_eq!(
        hex_str,
        "7c62f564159188295c2eb1b4fa3b67edb81c21b07b53d21b040f1a03340de849"
    );
}
