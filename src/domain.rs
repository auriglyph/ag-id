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

    #[inline]
    pub(crate) const fn as_byte(self) -> u8 {
        match self {
            Self::User => 0x01,
            Self::Document => 0x02,
            Self::Session => 0x03,
            Self::Device => 0x04,
            Self::Concept => 0x05,
            Self::Custom(b) => b,
            Self::Opaque => Self::OPAQUE_BYTE,
        }
    }

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
