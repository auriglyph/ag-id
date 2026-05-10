//! Round-trip example: produce a `did:agid:` string, transmit it, parse it back.
//!
//! Run with:
//!   cargo run --example parsing

use ag_id::{Did, Domain};

fn main() {
    let typed = Did::derive(Domain::User, b"alice@example.com");
    let s = typed.to_did_string();
    println!("produced:  {s}");

    // Parse it back. Domain is irrecoverable — we get Domain::Opaque.
    let parsed: Did = s.parse().expect("valid did:agid string");
    println!("parsed:    {parsed}  domain={}", parsed.domain());

    // Byte equality: parsed and typed agree.
    assert!(typed.eq_bytes(&parsed));
    println!("eq_bytes:  ✓");

    // PartialEq compares both bytes AND domain, so direct == differs:
    assert_ne!(typed, parsed);
    println!("PartialEq is stricter: typed ≠ opaque (by design)");

    // Demonstrate parse failures.
    println!("\nparse failures:");
    for s in [
        "did:other:abc",
        "did:agid:",
        "did:agid:O0Il",
        "did:agid:thisIsTooLongForA32ByteHashEncodedInBase58000000",
    ] {
        match s.parse::<Did>() {
            Ok(_) => println!("  {s:>50}  unexpectedly parsed"),
            Err(e) => println!("  {s:>50}  → {e}"),
        }
    }
}
