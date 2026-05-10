//! `serde` integration tests. Only built when `--features serde` is enabled.

#![cfg(feature = "serde")]

use ag_id::{Did, Domain};

#[test]
fn json_roundtrip() {
    let original = Did::derive(Domain::User, b"alice@example.com");
    let s = serde_json::to_string(&original).expect("serialize");
    // JSON: a string starting with did:agid:
    assert!(s.starts_with("\"did:agid:"));
    let back: Did = serde_json::from_str(&s).expect("deserialize");
    assert!(original.eq_bytes(&back));
    // Domain becomes Opaque after a round-trip.
    assert_eq!(back.domain(), Domain::Opaque);
}

#[test]
fn json_in_struct() {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Record {
        id: Did,
        name: String,
    }
    let r = Record {
        id: Did::derive(Domain::User, b"alice@example.com"),
        name: "alice".into(),
    };
    let s = serde_json::to_string(&r).expect("serialize");
    let parsed: Record = serde_json::from_str(&s).expect("deserialize");
    assert!(parsed.id.eq_bytes(&r.id));
    assert_eq!(parsed.name, "alice");
}

#[test]
fn json_rejects_garbage() {
    let bad = "\"did:agid:O0Il\"";
    let r: Result<Did, _> = serde_json::from_str(bad);
    assert!(r.is_err(), "must reject invalid base58");

    let bad = "\"plain text\"";
    let r: Result<Did, _> = serde_json::from_str(bad);
    assert!(r.is_err(), "must reject missing prefix");
}
