//! Wrapper pattern for low-entropy identifiers (emails, usernames, phone numbers).
//!
//! Run with:
//!   cargo run --example `hmac_wrapping`

use ag_id::{DeriveDomain, Did};

/// Caller-layer secret wrapper for sensitive low-entropy inputs.
///
/// This keeps `ag_id` itself stateless while breaking cross-deployment
/// linkability for public identifiers.
fn derive_private_user_did(deployment_key: &[u8; 32], raw_user_input: &str) -> Did {
    let wrapped = blake3::keyed_hash(deployment_key, raw_user_input.as_bytes());
    Did::derive(DeriveDomain::User, wrapped.as_bytes())
}

fn main() {
    // Example only: in production, load from a secure secret store / HSM.
    let key_a = [0x11; 32];
    let key_b = [0x22; 32];
    let raw = "alice@example.com";

    let id_a_1 = derive_private_user_did(&key_a, raw);
    let id_a_2 = derive_private_user_did(&key_a, raw);
    let id_b = derive_private_user_did(&key_b, raw);

    assert_eq!(id_a_1, id_a_2);
    assert_ne!(id_a_1, id_b);

    println!("same key, same input => same id: {id_a_1}");
    println!("different key, same input => different id: {id_b}");
}
