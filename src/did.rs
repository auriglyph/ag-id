use crate::{
    derive::raw,
    domain::Domain,
    encode::{from_base58_to_32, to_base58, to_hex},
    error::Error,
};

/// Canonical DID URI prefix for this method.
pub const DID_PREFIX: &str = "did:agid:";

/// A deterministic identifier.
///
/// Internally stores 32 raw bytes + the domain it was derived from.
/// All display formats (hex, base58, DID string) are computed on the fly
/// from those 32 bytes — no allocation required.
///
/// # Security
///
/// `PartialEq` is derived and performs a byte-by-byte compare. It is **not**
/// constant-time. Do not use `Did` equality to compare secret authenticators
/// or capability tokens — use a constant-time comparator
/// (e.g. [`subtle::ConstantTimeEq`](https://docs.rs/subtle)) instead. See
/// `SECURITY.md` for the full threat model.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Did {
    raw: [u8; 32],
    domain: Domain,
}

impl Did {
    /// Derive a new `Did` from a domain and arbitrary input bytes.
    ///
    /// This is the primary constructor. It is deterministic, side-effect-free,
    /// and produces the same result on every platform.
    ///
    /// ```rust
    /// use ag_id::{Did, Domain};
    ///
    /// let a = Did::derive(Domain::User, b"hello");
    /// let b = Did::derive(Domain::User, b"hello");
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    #[must_use]
    pub fn derive(domain: Domain, input: &[u8]) -> Self {
        Self {
            raw: raw(domain.as_byte(), input),
            domain,
        }
    }

    /// Raw 32-byte representation.
    ///
    /// # Example
    /// ```rust
    /// use ag_id::{Did, Domain};
    /// let id = Did::derive(Domain::User, b"test");
    /// let bytes = id.as_bytes();
    /// assert_eq!(bytes.len(), 32);
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.raw
    }

    /// Domain this identifier belongs to.
    ///
    /// # Example
    /// ```rust
    /// use ag_id::{Did, Domain};
    /// let id = Did::derive(Domain::User, b"test");
    /// assert_eq!(id.domain(), Domain::User);
    /// ```
    #[inline]
    #[must_use]
    pub const fn domain(&self) -> Domain {
        self.domain
    }

    /// Lowercase hex string (64 chars, stack-allocated).
    ///
    /// # Example
    /// ```rust
    /// use ag_id::{Did, Domain};
    /// let id = Did::derive(Domain::User, b"test");
    /// let hex = id.to_hex_array();
    /// assert_eq!(hex.len(), 64);
    /// ```
    #[must_use]
    pub fn to_hex_array(&self) -> [u8; 64] {
        to_hex(&self.raw)
    }

    /// DID string: `did:agid:<base58>` — suitable for W3C DID contexts.
    ///
    /// Length is at most `8 + 44 = 52` characters.
    ///
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "std")]
    /// # {
    /// use ag_id::{Did, Domain};
    /// let id = Did::derive(Domain::User, b"test");
    /// let s = id.to_did_string();
    /// assert!(s.starts_with("did:agid:"));
    /// # }
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_did_string(&self) -> std::string::String {
        let (buf, len) = to_base58(&self.raw);
        let mut s = std::string::String::with_capacity(DID_PREFIX.len() + len);
        s.push_str(DID_PREFIX);
        s.push_str(core::str::from_utf8(&buf[..len]).unwrap_or(""));
        s
    }

    /// Construct a `Did` from raw 32 bytes with `Domain::Opaque`.
    ///
    /// This is the inverse of [`Did::as_bytes`] for transport: when you
    /// have the 32 raw bytes (e.g. from a database column or a binary
    /// protocol) and need a `Did` value, this is the constructor.
    ///
    /// The resulting `Did` has [`Domain::Opaque`] because the original
    /// domain is irrecoverable from the bytes alone. Two `Did`s with the
    /// same raw bytes compare equal regardless of domain — see
    /// [`Did::eq_bytes`].
    ///
    /// # Example
    /// ```rust
    /// use ag_id::{Did, Domain};
    /// let original = Did::derive(Domain::User, b"alice");
    /// let opaque = Did::from_bytes(*original.as_bytes());
    /// assert!(opaque.eq_bytes(&original));
    /// assert_eq!(opaque.domain(), Domain::Opaque);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_bytes(raw: [u8; 32]) -> Self {
        Self {
            raw,
            domain: Domain::Opaque,
        }
    }

    /// Parse a `did:agid:<base58>` string back into a `Did`.
    ///
    /// The original [`Domain`] is **not** recoverable from the serialised
    /// form by design — only the 32 raw hash bytes are. The returned
    /// `Did` therefore has [`Domain::Opaque`]. Use [`Did::eq_bytes`] (or
    /// `==`, which compares both bytes and domain) when matching against
    /// a typed `Did`.
    ///
    /// # Errors
    /// - [`Error::MissingPrefix`] if the string does not start with `did:agid:`.
    /// - [`Error::WrongLength`] if the base58 payload is empty, longer than 44 chars, or does not decode to exactly 32 bytes.
    /// - [`Error::InvalidBase58`] if the payload contains a character outside the Bitcoin base58 alphabet.
    ///
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "std")]
    /// # {
    /// use ag_id::{Did, Domain};
    /// let typed = Did::derive(Domain::User, b"alice");
    /// let s = typed.to_did_string();
    /// let parsed = Did::parse(&s).expect("round-trip");
    /// assert!(parsed.eq_bytes(&typed));
    /// # }
    /// ```
    pub fn parse(s: &str) -> Result<Self, Error> {
        let payload = s.strip_prefix(DID_PREFIX).ok_or(Error::MissingPrefix)?;
        if payload.is_empty() || payload.len() > 44 {
            return Err(Error::WrongLength);
        }
        // Validate ASCII first so payload.as_bytes() is safe to feed.
        if !payload.is_ascii() {
            return Err(Error::InvalidBase58);
        }
        let raw = from_base58_to_32(payload.as_bytes()).ok_or(Error::InvalidBase58)?;
        Ok(Self::from_bytes(raw))
    }

    /// Compare two `Did`s by raw bytes only, ignoring the [`Domain`] field.
    ///
    /// Use this when comparing a parsed [`Domain::Opaque`] `Did` against a
    /// typed one. Note: this is **not** constant-time. See `SECURITY.md`.
    ///
    /// # Example
    /// ```rust
    /// use ag_id::{Did, Domain};
    /// let typed = Did::derive(Domain::User, b"x");
    /// let opaque = Did::from_bytes(*typed.as_bytes());
    /// assert!(typed.eq_bytes(&opaque));
    /// // PartialEq compares both fields, so direct equality fails:
    /// assert_ne!(typed, opaque);
    /// ```
    #[inline]
    #[must_use]
    pub fn eq_bytes(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

#[cfg(feature = "std")]
impl core::str::FromStr for Did {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl core::fmt::Display for Did {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (buf, len) = to_base58(&self.raw);
        write!(
            f,
            "did:agid:{}",
            core::str::from_utf8(&buf[..len]).unwrap_or("")
        )
    }
}

impl core::fmt::Debug for Did {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let hex = to_hex(&self.raw);
        write!(
            f,
            "Did({}/{})",
            self.domain,
            core::str::from_utf8(&hex).unwrap_or("?")
        )
    }
}
