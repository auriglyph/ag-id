/// Error types for deterministic identifier operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Input is empty — at least one byte required.
    EmptyInput,
    /// Domain byte 0x00 is reserved.
    ReservedDomain,
    /// String is not valid UTF-8.
    InvalidUtf8,
    /// String does not start with the required `did:agid:` prefix.
    MissingPrefix,
    /// Base58 payload is the wrong length (must be 1..=44 characters that
    /// decode to exactly 32 bytes).
    WrongLength,
    /// Base58 payload contains a character outside the alphabet
    /// (`123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`).
    InvalidBase58,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "ag_id: input must not be empty"),
            Self::ReservedDomain => write!(f, "ag_id: domain byte 0x00 is reserved"),
            Self::InvalidUtf8 => write!(f, "ag_id: input is not valid UTF-8"),
            Self::MissingPrefix => write!(f, "ag_id: missing 'did:agid:' prefix"),
            Self::WrongLength => write!(f, "ag_id: base58 payload has wrong length"),
            Self::InvalidBase58 => write!(f, "ag_id: invalid character in base58 payload"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
