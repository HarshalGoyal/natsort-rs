//! Bytes handling: sorting `Vec<u8>` inputs.
//!
//! Python's natsort accepts byte strings like `[b"a10", b"a2"]` and sorts them
//! lexicographically (no number splitting within bytes).  The `as_utf8()` or
//! `as_ascii()` decoders convert bytes to str first, enabling natural sort on
//! the decoded content.
//!
//! This module provides `natsorted_bytes()` for raw byte comparison and a
//! decoder function that mirrors Python's `decoder("utf-8")`.

/// Sort a slice of byte slices using lexicographic ordering.
///
/// Bytes are compared directly without number splitting — this mirrors
/// Python's behavior where `natsorted([b"a10", b"a2"])` returns
/// `[b"a10", b"a2"]` because bytes are treated as opaque sequences.
///
/// # Examples
///
/// ```
/// use natsort::natsorted_bytes;
///
/// let data = vec![b"z".as_slice(), b"a".as_slice()];
/// let sorted = natsorted_bytes(&data);
/// assert_eq!(sorted, vec![b"a".as_slice(), b"z".as_slice()]);
/// ```
pub fn natsorted_bytes(items: &[&[u8]]) -> Vec<Vec<u8>> {
    let mut indexed: Vec<_> = items.iter().enumerate().collect();
    indexed.sort_by_key(|&(_, &a)| a);
    indexed.into_iter().map(|(_, item)| item.to_vec()).collect()
}

/// Sort a slice of byte slices with case-insensitive ordering.
///
/// Equivalent to applying `ns.IGNORECASE` before sorting bytes.
///
/// # Examples
///
/// ```
/// use natsort::natsorted_bytes_ignorecase;
///
/// let data = vec![b"B".as_slice(), b"a".as_slice()];
/// let sorted = natsorted_bytes_ignorecase(&data);
/// assert_eq!(sorted, vec![b"a".as_slice(), b"B".as_slice()]);
/// ```
pub fn natsorted_bytes_ignorecase(items: &[&[u8]]) -> Vec<Vec<u8>> {
    let mut indexed: Vec<_> = items.iter().enumerate().collect();
    indexed.sort_by(|&(_, &a), &(_, &b)| {
        let a_lower: Vec<u8> = a.iter().map(|c| c.to_ascii_lowercase()).collect();
        let b_lower: Vec<u8> = b.iter().map(|c| c.to_ascii_lowercase()).collect();
        a_lower.cmp(&b_lower)
    });
    indexed.into_iter().map(|(_, item)| item.to_vec()).collect()
}

/// Decode bytes to UTF-8 string, or return the original value.
///
/// This mirrors Python's `natsort.decoder("utf-8")` which is commonly used
/// as a key function: `natsorted(data, key=natsort.as_utf8)`.
///
/// Returns `Ok(String)` if the bytes are valid UTF-8, or `Err(Vec<u8>)`
/// containing the original bytes if decoding fails.
///
/// # Examples
///
/// ```
/// use natsort::decode_bytes;
///
/// assert_eq!(decode_bytes(b"hello"), Ok("hello".to_string()));
/// assert!(decode_bytes(&[0xff, 0xfe]).is_err());
/// ```
pub fn decode_bytes(bytes: &[u8]) -> Result<String, Vec<u8>> {
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|_| bytes.to_vec())
}

/// Decode bytes to ASCII string, or return the original value.
///
/// Mirrors Python's `natsort.as_ascii()`.  Non-ASCII bytes are replaced
/// with `?` (question mark), matching the ASCII codec behavior.
///
/// # Examples
///
/// ```
/// use natsort::decode_bytes_ascii;
///
/// assert_eq!(decode_bytes_ascii(b"hello"), "hello".to_string());
/// // Non-ASCII bytes become '?'
/// assert_eq!(decode_bytes_ascii(&[65, 255, 97]), "A?a".to_string());
/// ```
pub fn decode_bytes_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| if b.is_ascii() { b as char } else { '?' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_basic() {
        let data = vec![b"z".as_slice(), b"a".as_slice(), b"m".as_slice()];
        let result = natsorted_bytes(&data);
        assert_eq!(
            result,
            vec![b"a".as_slice(), b"m".as_slice(), b"z".as_slice()]
        );
    }

    #[test]
    fn bytes_empty() {
        let data: Vec<&[u8]> = vec![];
        let result = natsorted_bytes(&data);
        assert!(result.is_empty());
    }

    #[test]
    fn bytes_ignorecase() {
        let data = vec![b"B".as_slice(), b"a".as_slice(), b"A".as_slice()];
        let result = natsorted_bytes_ignorecase(&data);
        assert_eq!(
            result,
            vec![b"a".as_slice(), b"A".as_slice(), b"B".as_slice()]
        );
    }

    #[test]
    fn decode_bytes_valid_utf8() {
        assert_eq!(decode_bytes(b"hello"), Ok("hello".to_string()));
        assert_eq!(decode_bytes("café".as_bytes()), Ok("café".to_string()));
    }

    #[test]
    fn decode_bytes_invalid() {
        assert!(decode_bytes(&[0xff, 0xfe]).is_err());
    }

    #[test]
    fn decode_bytes_ascii_replacement() {
        assert_eq!(decode_bytes_ascii(b"hello"), "hello");
        assert_eq!(decode_bytes_ascii(&[65, 255, 97]), "A?a");
    }
}
