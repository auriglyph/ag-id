/// Protocol namespace — pinned for the v1.x line of the `Ag^id` protocol.
///
/// These exact 9 bytes (`agid:v1:`) are fed into BLAKE3 before the domain
/// byte and the caller-supplied input. Changing them invalidates every
/// previously derived identifier, which is why they are part of the
/// stability contract. A future v2 protocol may move to `b"agid:v2:"` or a
/// new hash function.
const PREFIX: &[u8] = b"agid:v1:";

/// Derive 32 raw bytes from domain + input.
///
/// Layout fed to BLAKE3:
///   PREFIX || `domain_byte` || input
///
/// This is the only hash call in the entire library.
/// Everything else is encoding.
#[inline]
pub fn raw(domain_byte: u8, input: &[u8]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(PREFIX);
    h.update(&[domain_byte]);
    h.update(input);
    *h.finalize().as_bytes()
}
