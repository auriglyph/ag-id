use crate::Error;
use core::num::NonZeroU8;

/// Semantic domain accepted by derivation APIs.
///
/// Unlike [`Domain`], this type cannot represent [`Domain::Opaque`], the
/// sentinel used for parsed values. Its custom namespace is also non-zero by
/// construction, so every `DeriveDomain` maps to a valid v1 domain byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DeriveDomain {
    /// Human or machine user identity
    User,
    /// Document, file, or content hash
    Document,
    /// Session or transaction
    Session,
    /// Physical or virtual device
    Device,
    /// Semantic concept
    Concept,
    /// Custom non-zero domain byte you control
    Custom(NonZeroU8),
}

/// Semantic domain — prevents cross-domain ID collisions.
///
/// Same input in different domains → different IDs.
/// This is a security property: a User ID can never equal a Document ID
/// even if both were derived from the same bytes.
///
/// # The `Opaque` variant
///
/// `Domain::Opaque` is a sentinel for `Did` values reconstructed from a
/// `did:agid:` string (see [`crate::Did::parse`]). The original domain that
/// produced the hash is **not** recoverable from the serialized form by
/// design — only the 32 raw bytes are. An `Opaque` `Did` compares equal to
/// any other `Did` (typed or opaque) whose raw bytes match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Domain {
    /// Human or machine user identity
    User,
    /// Document, file, or content hash
    Document,
    /// Session or transaction
    Session,
    /// Physical or virtual device
    Device,
    /// Semantic concept
    Concept,
    /// Custom domain — use a non-zero byte you control
    Custom(u8),
    /// Sentinel for `Did` values parsed from a `did:agid:` string. The
    /// original domain is not recoverable; only the 32 raw hash bytes are.
    Opaque,
}

impl Domain {
    /// Sentinel byte reserved for [`Domain::Opaque`]. Never used to derive
    /// real identifiers (would clash with the no-domain default and
    /// confuse the protocol prefix). Documented for forensic tooling.
    pub const OPAQUE_BYTE: u8 = 0x00;

    /// All built-in (non-`Custom`, non-`Opaque`) domains, in canonical order.
    ///
    /// Useful for testing, CLI listings, and exhaustive iteration.
    /// `Custom(_)` and `Opaque` are intentionally excluded.
    #[must_use]
    pub const fn builtins() -> &'static [Self] {
        &[
            Self::User,
            Self::Document,
            Self::Session,
            Self::Device,
            Self::Concept,
        ]
    }
}

impl DeriveDomain {
    /// Build a custom derivation domain from a raw byte.
    ///
    /// # Errors
    /// Returns [`Error::ReservedDomain`] when `byte == 0x00`, which is reserved
    /// for [`Domain::Opaque`] and must never be used as hash input.
    pub const fn custom(byte: u8) -> Result<Self, Error> {
        match NonZeroU8::new(byte) {
            Some(byte) => Ok(Self::Custom(byte)),
            None => Err(Error::ReservedDomain),
        }
    }

    #[inline]
    pub(crate) const fn as_byte(self) -> u8 {
        match self {
            Self::User => 0x01,
            Self::Document => 0x02,
            Self::Session => 0x03,
            Self::Device => 0x04,
            Self::Concept => 0x05,
            Self::Custom(b) => b.get(),
        }
    }

    /// All built-in derivation domains, in canonical order.
    ///
    /// Useful for testing, CLI listings, and exhaustive iteration.
    /// `Custom(_)` is intentionally excluded.
    #[must_use]
    pub const fn builtins() -> &'static [Self] {
        &[
            Self::User,
            Self::Document,
            Self::Session,
            Self::Device,
            Self::Concept,
        ]
    }
}

impl From<DeriveDomain> for Domain {
    fn from(domain: DeriveDomain) -> Self {
        match domain {
            DeriveDomain::User => Self::User,
            DeriveDomain::Document => Self::Document,
            DeriveDomain::Session => Self::Session,
            DeriveDomain::Device => Self::Device,
            DeriveDomain::Concept => Self::Concept,
            DeriveDomain::Custom(b) => Self::Custom(b.get()),
        }
    }
}

impl TryFrom<Domain> for DeriveDomain {
    type Error = Error;

    fn try_from(domain: Domain) -> Result<Self, Self::Error> {
        match domain {
            Domain::User => Ok(Self::User),
            Domain::Document => Ok(Self::Document),
            Domain::Session => Ok(Self::Session),
            Domain::Device => Ok(Self::Device),
            Domain::Concept => Ok(Self::Concept),
            Domain::Custom(b) => Self::custom(b),
            Domain::Opaque => Err(Error::ReservedDomain),
        }
    }
}

impl core::fmt::Display for Domain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Document => write!(f, "document"),
            Self::Session => write!(f, "session"),
            Self::Device => write!(f, "device"),
            Self::Concept => write!(f, "concept"),
            Self::Custom(b) => write!(f, "custom:{b:02x}"),
            Self::Opaque => write!(f, "opaque"),
        }
    }
}

impl core::fmt::Display for DeriveDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Domain::from(*self).fmt(f)
    }
}
