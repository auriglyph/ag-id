#![allow(clippy::doc_markdown)]
//! Print canonical vectors to stdout. Run via:
//!   cargo test --test vector_export -- --nocapture print_vectors
//!
//! The output is consumed by `test-vectors/v1.json` and any re-implementation
//! in another language.

use ag_id::{DeriveDomain, Did};

fn hex(bytes: &[u8; 32]) -> String {
    use core::fmt::Write as _;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[derive(Clone, Copy)]
struct V {
    name: &'static str,
    domain: DeriveDomain,
    input: &'static [u8],
}

const VECTORS: &[V] = &[
    V {
        name: "user_alice_at_example",
        domain: DeriveDomain::User,
        input: b"alice@example.com",
    },
    V {
        name: "user_architect_at_auriglyph",
        domain: DeriveDomain::User,
        input: b"architect@auriglyph.com",
    },
    V {
        name: "golden_v1_anchor",
        domain: DeriveDomain::User,
        input: b"agid:golden:v1",
    },
    V {
        name: "empty_input_user",
        domain: DeriveDomain::User,
        input: b"",
    },
    V {
        name: "unicode_cyrillic",
        domain: DeriveDomain::User,
        // UTF-8 bytes for "Михаил"
        input: "Михаил".as_bytes(),
    },
    V {
        name: "domain_separation_user",
        domain: DeriveDomain::User,
        input: b"same-input",
    },
    V {
        name: "domain_separation_document",
        domain: DeriveDomain::Document,
        input: b"same-input",
    },
    V {
        name: "domain_separation_session",
        domain: DeriveDomain::Session,
        input: b"same-input",
    },
    V {
        name: "custom_domain_0xff",
        domain: DeriveDomain::Custom(core::num::NonZeroU8::MAX),
        input: b"some-input",
    },
    V {
        name: "long_input_1024_zeros",
        domain: DeriveDomain::Document,
        input: &[0u8; 1024],
    },
];

#[test]
fn print_vectors() {
    println!();
    println!("{:30}  {:8}  raw_hex                                                            did_string", "name", "domain");
    println!("{:-<150}", "");
    for v in VECTORS {
        let did = Did::derive(v.domain, v.input);
        let raw = hex(did.as_bytes());
        let s = did.to_did_string();
        let dom = format!("{:?}", v.domain);
        println!("{:30}  {dom:8}  {raw}  {s}", v.name);
    }
}
