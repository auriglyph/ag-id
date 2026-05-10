#![allow(clippy::unwrap_used)]
//! Minimal example: derive identifiers, compare them, format them.
//!
//! Run with:
//!   cargo run --example basic

use ag_id::{Did, Domain};

fn main() {
    // Derive an identifier from a (domain, input) pair.
    let alice = Did::derive(Domain::User, b"alice@example.com");
    println!("alice (user)     = {alice}");
    println!("  raw bytes      = {:?}", alice.as_bytes());
    println!(
        "  hex            = {}",
        core::str::from_utf8(&alice.to_hex_array()).unwrap()
    );

    // Same input → same identifier.
    let alice_again = Did::derive(Domain::User, b"alice@example.com");
    assert_eq!(alice, alice_again);
    println!("\nDeterminism check: same input → same id ✓");

    // Different domain → different identifier (security property).
    let alice_doc = Did::derive(Domain::Document, b"alice@example.com");
    assert_ne!(alice, alice_doc);
    println!("Domain separation: User('alice@…') ≠ Document('alice@…') ✓");
    println!("  alice (doc)    = {alice_doc}");

    // Custom domains let you carve out your own namespace.
    let custom = Did::derive(Domain::Custom(0x42), b"my-namespace-thing");
    println!("\ncustom(0x42)     = {custom}");

    // Iterate built-in domains.
    println!("\nbuilt-in domains:");
    for d in Domain::builtins() {
        println!(
            "  {d:10}  byte=0x{:02x}",
            Did::derive(*d, b"x").as_bytes()[0]
        );
    }
}
