//! Optional `serde` integration. Enabled via the `serde` feature.
//!
//! In human-readable formats (JSON, YAML, TOML) a `Did` serialises as its
//! canonical `did:agid:<base58>` string. In binary formats (`bincode`,
//! `postcard`, `MessagePack`) it serialises as the 32 raw bytes — both
//! significantly cheaper to ser/de and preserves byte-identity.
//!
//! In either format the original [`crate::Domain`] is **not** preserved —
//! the wire format is the 32-byte hash only. Deserialised values come back
//! as [`crate::Domain::Opaque`]. See `SECURITY.md` for the rationale.

use crate::{Did, Error};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Did {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        if ser.is_human_readable() {
            ser.collect_str(self)
        } else {
            ser.serialize_bytes(self.as_bytes())
        }
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> de::Visitor<'de> for V {
            type Value = Did;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a did:agid:<base58> string or a 32-byte array")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> Result<Did, E> {
                Did::parse(s).map_err(|err: Error| E::custom(err))
            }
            fn visit_bytes<E: de::Error>(self, b: &[u8]) -> Result<Did, E> {
                let arr: [u8; 32] = b
                    .try_into()
                    .map_err(|_| E::invalid_length(b.len(), &"32 bytes"))?;
                Ok(Did::from_bytes(arr))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Did, A::Error> {
                let mut buf = [0u8; 32];
                for (i, slot) in buf.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(i, &"32 bytes"))?;
                }
                if seq.next_element::<u8>()?.is_some() {
                    return Err(de::Error::invalid_length(33, &"32 bytes"));
                }
                Ok(Did::from_bytes(buf))
            }
        }
        if de.is_human_readable() {
            de.deserialize_str(V)
        } else {
            de.deserialize_bytes(V)
        }
    }
}
