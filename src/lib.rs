//! `natsort-rs` — a Rust port of the Python [`natsort`] library.
//!
//! Natural sorting orders strings the way a human expects when they contain
//! embedded numbers:
//!
//! ```text
//! lexicographic: ["file1.txt", "file10.txt", "file2.txt"]
//! natural:       ["file1.txt", "file2.txt",  "file10.txt"]
//! ```
//!
//! The port targets 100% behavioural parity with the original Python library.
//! Every feature is validated against the real Python implementation through a
//! `pyo3` bridge in `tests/parity.rs`.
//!
//! [`natsort`]: https://github.com/SethMMorton/natsort
//!
//! # Status
//!
//! Phase 1: core algorithm with basic flags (INT, SIGNED, FLOAT, REAL).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod keygen;
pub mod ns;
pub mod segment;

pub use error::{Error, Result};
pub use keygen::NatsortKey;
pub use ns::NsFlags;
pub use segment::NatsortKeyPart;

/// Sort a slice of string-like items using natural ordering.
///
/// Uses the default algorithm (`ns.DEFAULT`).  Returns a new `Vec<String>` with the
/// items sorted so that embedded numbers are compared numerically rather than
/// lexicographically.
///
/// # Examples
///
/// ```
/// use natsort::natsorted;
///
/// let data = vec!["file10.txt", "file2.txt", "file1.txt"];
/// let sorted = natsorted(&data);
/// assert_eq!(sorted, vec!["file1.txt", "file2.txt", "file10.txt"]);
/// ```
pub fn natsorted(items: &[&str]) -> Vec<String> {
    let key_gen = NatsortKey::default();
    let mut indexed: Vec<_> = items.iter().enumerate().collect();
    indexed.sort_by(|&(_, &a), &(_, &b)| {
        key_gen.key(a).cmp(&key_gen.key(b))
    });
    indexed.into_iter().map(|(_, item)| item.to_string()).collect()
}

/// Sort a slice of string-like items with custom flags.
///
/// # Examples
///
/// ```
/// use natsort::{natsorted_with, NsFlags};
///
/// let data = vec!["Banana", "apple", "Cherry"];
/// let sorted = natsorted_with(&data, NsFlags::IGNORECASE);
/// assert_eq!(sorted, vec!["apple", "Banana", "Cherry"]);
/// ```
pub fn natsorted_with(items: &[&str], flags: NsFlags) -> Vec<String> {
    let key_gen = NatsortKey::new(flags);
    let mut indexed: Vec<_> = items.iter().enumerate().collect();
    indexed.sort_by(|&(_, &a), &(_, &b)| {
        key_gen.key(a).cmp(&key_gen.key(b))
    });
    indexed.into_iter().map(|(_, item)| item.to_string()).collect()
}

/// Reverse the sort order: largest elements first.
///
/// Equivalent to calling [`natsorted`] and then reversing, but avoids an
/// extra allocation by sorting with reversed comparison directly.
///
/// ```
/// use natsort::natsorted_rev;
///
/// let data = vec!["file1.txt", "file2.txt", "file10.txt"];
/// let sorted = natsorted_rev(&data);
/// assert_eq!(sorted, vec!["file10.txt", "file2.txt", "file1.txt"]);
/// ```
pub fn natsorted_rev(items: &[&str]) -> Vec<String> {
    let key_gen = NatsortKey::default();
    let mut indexed: Vec<_> = items.iter().enumerate().collect();
    indexed.sort_by(|&(_, &a), &(_, &b)| {
        key_gen.key(b).cmp(&key_gen.key(a))
    });
    indexed.into_iter().map(|(_, item)| item.to_string()).collect()
}

/// Sort a slice of string-like items using real-number parsing (signed floats).
///
/// This is equivalent to calling [`natsorted_with`] with `ns::REAL`, which is
/// useful when sorting strings like `"-3.2"`, `"+2.1"`, `"1.5"`.
///
/// ```
/// use natsort::realsorted;
///
/// let data = vec!["1.5", "-3.2", "10.0", "+2.1"];
/// let sorted = realsorted(&data);
/// assert_eq!(sorted, vec!["-3.2", "1.5", "+2.1", "10.0"]);
/// ```
pub fn realsorted(items: &[&str]) -> Vec<String> {
    natsorted_with(items, NsFlags::REAL)
}

/// Create a reusable sorting key function with the given flags.
///
/// Returns a [`NatsortKey`] that can be applied to multiple inputs.  This is
/// the Rust idiomatic equivalent of Python's `natsort_keygen`.
///
/// ```
/// use natsort::{natsort_keygen, NsFlags};
///
/// let key_gen = natsort_keygen(NsFlags::IGNORECASE);
/// let key_a = key_gen.key("Apple");
/// let key_b = key_gen.key("banana");
/// assert!(key_a < key_b);
/// ```
pub fn natsort_keygen(alg: NsFlags) -> NatsortKey {
    NatsortKey::new(alg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natsorted_basic_integers() {
        let data = vec!["4", "8", "2", "10", "3"];
        let result = natsorted(&data);
        assert_eq!(result, vec!["2", "3", "4", "8", "10"]);
    }

    #[test]
    fn natsorted_files() {
        let data = vec!["file10.txt", "file2.txt", "file1.txt"];
        let result = natsorted(&data);
        assert_eq!(result, vec!["file1.txt", "file2.txt", "file10.txt"]);
    }

    #[test]
    fn natsorted_numbers_before_text() {
        let data = vec!["b", "2", "a", "1"];
        let result = natsorted(&data);
        assert_eq!(result, vec!["1", "2", "a", "b"]);
    }

    #[test]
    fn natsorted_empty() {
        let data: Vec<&str> = vec![];
        let result = natsorted(&data);
        assert!(result.is_empty());
    }

    #[test]
    fn natsorted_single() {
        let data = vec!["hello"];
        let result = natsorted(&data);
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn natsorted_already_sorted() {
        let data = vec!["1", "2", "3"];
        let result = natsorted(&data);
        assert_eq!(result, vec!["1", "2", "3"]);
    }

    #[test]
    fn natsorted_reverse() {
        let data = vec!["file1.txt", "file2.txt", "file10.txt"];
        let result = natsorted_rev(&data);
        assert_eq!(result, vec!["file10.txt", "file2.txt", "file1.txt"]);
    }

    #[test]
    fn natsorted_with_ignorecase() {
        let data = vec!["Banana", "apple", "Cherry"];
        let result = natsorted_with(&data, NsFlags::IGNORECASE);
        assert_eq!(result, vec!["apple", "Banana", "Cherry"]);
    }

    #[test]
    fn realsorted_basic() {
        let data = vec!["1.5", "-3.2", "10.0", "+2.1"];
        let result = realsorted(&data);
        assert_eq!(result, vec!["-3.2", "1.5", "+2.1", "10.0"]);
    }

    #[test]
    fn natsort_keygen_reusable() {
        let key_gen = natsort_keygen(NsFlags::IGNORECASE);
        let key_a = key_gen.key("Apple");
        let key_b = key_gen.key("banana");
        assert!(key_a < key_b);
    }
}
