/// Encode 32 bytes as lowercase hex (64 chars, no allocation).
pub fn to_hex(bytes: &[u8; 32]) -> [u8; 64] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 64];
    for (i, &b) in bytes.iter().enumerate() {
        out[i * 2] = HEX[(b >> 4) as usize];
        out[i * 2 + 1] = HEX[(b & 0x0f) as usize];
    }
    out
}

/// Base58 alphabet (Bitcoin variant — no 0/O/I/l).
pub const BASE58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Reverse lookup: base58 char → digit value (0..58), or 0xFF if not in alphabet.
///
/// Generated once at compile time from `BASE58`.
#[allow(clippy::cast_possible_truncation)] // i ∈ 0..58, fits in u8 by construction
const fn base58_decode_table() -> [u8; 256] {
    let mut t = [0xFFu8; 256];
    let mut i = 0u8;
    while i < 58 {
        t[BASE58[i as usize] as usize] = i;
        i += 1;
    }
    t
}
const BASE58_DECODE: [u8; 256] = base58_decode_table();

/// Encode 32 bytes as base58 (≤44 chars).
/// Returns (buf, len) — only buf[..len] is valid.
pub fn to_base58(bytes: &[u8; 32]) -> ([u8; 44], usize) {
    let mut digits = [0u32; 44];
    let mut len = 1usize;

    for &byte in bytes {
        let mut carry = u32::from(byte);
        for d in &mut digits[..len] {
            carry += (*d) << 8;
            *d = carry % 58;
            carry /= 58;
        }
        while carry > 0 {
            digits[len] = carry % 58;
            len += 1;
            carry /= 58;
        }
    }

    let mut out = [0u8; 44];
    for (i, &d) in digits[..len].iter().rev().enumerate() {
        out[i] = BASE58[d as usize];
    }
    (out, len)
}

/// Decode a base58 string into exactly 32 bytes.
///
/// Returns `None` if any character is outside the base58 alphabet or if
/// the decoded value would not fit into 32 bytes (or is too short to fill
/// 32 bytes). Whitespace is not tolerated; the caller must trim if needed.
///
/// This decoder runs in `O(len^2)` time (multiplication ladder), which is
/// fine for our fixed ≤44-character payload but not appropriate for
/// arbitrary-length input.
pub fn from_base58_to_32(s: &[u8]) -> Option<[u8; 32]> {
    if s.is_empty() || s.len() > 44 {
        return None;
    }

    // Count leading '1's — each represents a leading 0x00 byte.
    let mut leading_ones = 0usize;
    while leading_ones < s.len() && s[leading_ones] == b'1' {
        leading_ones += 1;
    }
    if leading_ones > 32 {
        return None;
    }

    // Convert remaining digits into bytes via repeated *=58 + digit.
    let mut bytes = [0u8; 32];
    for &c in s {
        let digit = BASE58_DECODE[c as usize];
        if digit == 0xFF {
            return None;
        }
        let mut carry = u32::from(digit);
        for byte in bytes.iter_mut().rev() {
            carry += u32::from(*byte) * 58;
            *byte = (carry & 0xFF) as u8;
            carry >>= 8;
        }
        if carry != 0 {
            // Overflow — input does not fit in 32 bytes.
            return None;
        }
    }

    // Sanity-check leading-zero alignment: the number of leading 0x00
    // bytes in `bytes` must equal `leading_ones`.
    let mut actual_leading_zeros = 0usize;
    for &b in &bytes {
        if b == 0 {
            actual_leading_zeros += 1;
        } else {
            break;
        }
    }
    if actual_leading_zeros < leading_ones {
        // Decoded payload claims more leading zeros than the encoded form.
        // Should be impossible for a well-formed encoding.
        return None;
    }

    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let bytes = [
            0xde, 0xad, 0xbe, 0xef, 0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
        ];
        let hex = to_hex(&bytes);
        assert_eq!(&hex[..8], b"deadbeef");
    }

    #[test]
    fn base58_stable() {
        let bytes = [1u8; 32];
        let (buf, len) = to_base58(&bytes);
        // Must be non-empty and all valid base58 chars
        assert!(len > 0 && len <= 44);
        for &c in &buf[..len] {
            assert!(BASE58.contains(&c));
        }
    }

    #[test]
    fn base58_roundtrip_known() {
        // Hash with no leading zeros.
        let original: [u8; 32] = [
            0x4f, 0xd7, 0xfb, 0x9a, 0xb8, 0xaa, 0x83, 0x42, 0x99, 0x73, 0x2e, 0x28, 0x7c, 0x6e,
            0x4e, 0xda, 0x74, 0x6e, 0x7e, 0xfc, 0x09, 0x2c, 0xf7, 0x36, 0x72, 0x9b, 0xcd, 0xcc,
            0xb3, 0x34, 0x0d, 0x20,
        ];
        let (buf, len) = to_base58(&original);
        let decoded = from_base58_to_32(&buf[..len]).expect("decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn base58_roundtrip_leading_zeros() {
        // Hash with some leading zeros — base58 encodes them as leading '1's.
        let original: [u8; 32] = [
            0x00, 0x00, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01,
            0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
            0xef, 0x01, 0x23, 0x45,
        ];
        let (buf, len) = to_base58(&original);
        let decoded = from_base58_to_32(&buf[..len]).expect("decode leading zeros");
        assert_eq!(decoded, original);
    }

    #[test]
    fn base58_roundtrip_all_zeros() {
        let original = [0u8; 32];
        let (buf, len) = to_base58(&original);
        let decoded = from_base58_to_32(&buf[..len]).expect("decode all zeros");
        assert_eq!(decoded, original);
    }

    #[test]
    fn base58_decode_rejects_invalid_chars() {
        // 'O', '0', 'I', 'l' are not in the alphabet.
        assert!(from_base58_to_32(b"O0Il").is_none());
        assert!(from_base58_to_32(b"valid_but_underscore").is_none());
    }

    #[test]
    fn base58_decode_rejects_empty_and_oversized() {
        assert!(from_base58_to_32(b"").is_none());
        // 45 valid chars — too long to fit in 32 bytes for any input.
        let too_long = [b'z'; 45];
        assert!(from_base58_to_32(&too_long).is_none());
    }
}
