#![allow(clippy::panic, clippy::unwrap_used)]
//! Verify that `test-vectors/v1.json` matches the live implementation.
//!
//! Re-implementations in other languages should ingest this JSON and run
//! the equivalent assertion. If this test fails, either the JSON is wrong
//! or the implementation changed in a way that breaks the v1.x contract.

use ag_id::{DeriveDomain, Did};

fn parse_domain(name: &str, domain_byte_str: &str) -> DeriveDomain {
    match name {
        "User" => DeriveDomain::User,
        "Document" => DeriveDomain::Document,
        "Session" => DeriveDomain::Session,
        "Device" => DeriveDomain::Device,
        "Concept" => DeriveDomain::Concept,
        "Custom" => {
            let trimmed = domain_byte_str
                .trim_start_matches("0x")
                .trim_start_matches("0X");
            let byte = u8::from_str_radix(trimmed, 16)
                .unwrap_or_else(|_| panic!("bad domain_byte: {domain_byte_str}"));
            DeriveDomain::custom(byte)
                .unwrap_or_else(|_| panic!("bad domain_byte: {domain_byte_str}"))
        }
        other => panic!("unknown domain in JSON: {other}"),
    }
}

fn hex_string(bytes: &[u8; 32]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[test]
fn vectors_v1_json_matches_implementation() {
    let raw = include_str!("../test-vectors/v1.json");
    let v: serde_json::Value = serde_json::from_str(raw).expect("valid JSON");

    let arr = v["vectors"].as_array().expect("vectors is an array");
    assert!(!arr.is_empty(), "no test vectors");

    for entry in arr {
        let name = entry["name"].as_str().unwrap_or("?");
        let dom_name = entry["domain"].as_str().expect("domain field");
        let dom_byte = entry["domain_byte"].as_str().expect("domain_byte field");
        let domain = parse_domain(dom_name, dom_byte);

        // Build input bytes. For long_input_1024_zeros we synthesise.
        let input: Vec<u8> = match name {
            "long_input_1024_zeros" => vec![0u8; 1024],
            _ => entry["input_utf8"]
                .as_str()
                .expect("input_utf8")
                .as_bytes()
                .to_vec(),
        };

        let did = Did::derive(domain, &input);
        let actual_hex = hex_string(did.as_bytes());
        let actual_did = did.to_did_string();

        let expected_hex = entry["raw_hex"].as_str().expect("raw_hex");
        let expected_did = entry["did_string"].as_str().expect("did_string");

        assert_eq!(
            actual_hex, expected_hex,
            "vector '{name}' raw_hex mismatch: got {actual_hex}, expected {expected_hex}"
        );
        assert_eq!(
            actual_did, expected_did,
            "vector '{name}' did_string mismatch: got {actual_did}, expected {expected_did}"
        );

        // Round-trip through parse for good measure.
        let parsed = Did::parse(&actual_did).expect("round-trip");
        assert!(parsed.eq_bytes(&did), "vector '{name}' parse round-trip");
    }
}
