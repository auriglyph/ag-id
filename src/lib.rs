#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

//! # `Ag^id` (crate `ag_id`)
//!
//! The same input always produces the same identifier. On every platform. Forever.
//!
//! ```rust
//! use ag_id::{Did, Domain};
//!
//! let id = Did::derive(Domain::User, b"architect@auriglyph.com");
//! let id2 = Did::derive(Domain::User, b"architect@auriglyph.com");
//! assert_eq!(id, id2); // always
//! # #[cfg(feature = "std")]
//! assert!(id.to_did_string().starts_with("did:agid:"));
//! ```

mod derive;
mod did;
mod domain;
mod encode;
#[cfg(feature = "serde")]
mod serde_impl;

/// Error types for deterministic identifier operations.
pub mod error;

pub use did::{Did, DID_PREFIX};
pub use domain::Domain;
pub use error::Error;

/// Derive a deterministic identifier from a domain and input bytes.
///
/// # Example
/// ```rust
/// use ag_id::{derive, Domain};
/// let id = derive(Domain::User, b"hello");
/// ```
#[must_use]
pub fn derive(domain: Domain, input: &[u8]) -> Did {
    Did::derive(domain, input)
}

/// Derive from a string slice.
///
/// # Example
/// ```rust
/// use ag_id::{derive_str, Domain};
/// let id = derive_str(Domain::User, "hello");
/// ```
#[must_use]
pub fn derive_str(domain: Domain, input: &str) -> Did {
    Did::derive(domain, input.as_bytes())
}
