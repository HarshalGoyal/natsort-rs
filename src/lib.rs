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
//! Phase 2: mixed types, recursive descent, additional flags (NANLAST, PRESORT,
//! NUMAFTER, LOWERCASEFIRST, GROUPLETTERS), locale-aware string transform,
//! and OS path sorting.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use rayon::prelude::*;

pub mod bytes;
pub mod error;
pub mod keygen;
pub mod locale;
pub mod mixed;
pub mod ns;
pub mod os_sort;
pub mod path;
pub mod recursive;
pub mod segment;
pub mod unicode_numbers;

pub use bytes::{decode_bytes, decode_bytes_ascii, natsorted_bytes, natsorted_bytes_ignorecase};
pub use error::{Error, Result};
pub use keygen::NatsortKey;
pub use locale::LocaleStr;
pub use mixed::{Item, natsorted_mixed, natsorted_mixed_with};
pub use ns::NsFlags;
pub use os_sort::{os_sorted, os_sort_key, os_sort_keygen};
pub use recursive::{NestedItem, natsorted_recursive, natsorted_recursive_with};
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
    // decorate-sort-undecorate: compute each key exactly once (in parallel)
    // instead of re-deriving it on every comparator call (O(n log n)).
    let mut decorated: Vec<(Vec<NatsortKeyPart>, usize, &str)> = items
        .par_iter()
        .enumerate()
        .map(|(i, item)| (key_gen.key(item), i, *item))
        .collect();
    decorated.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    decorated
        .into_iter()
        .map(|(_, _, item)| item.to_string())
        .collect()
}

/// Sort a slice of string-like items with custom flags.
///
/// Supports all [`NsFlags`](crate::ns::NsFlags) including:
/// - `IGNORECASE` — case-insensitive comparison
/// - `GROUPLETTERS` — group uppercase/lowercase together (doubles chars)
/// - `LOWERCASEFIRST` — swap case so lowercase sorts first
/// - `NUMAFTER` — numbers after letters
/// - `PRESORT` — break ties by string value for stable sort
/// - `LOCALEALPHA` — Unicode case-folding via unicase
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
    // Apply PRESORT: pre-sort by string value to establish tiebreaker order.
    let indexed: Vec<(usize, &str)> = if flags.contains(NsFlags::PRESORT) {
        let mut indexed_items: Vec<(usize, &str)> =
            items.iter().enumerate().map(|(i, &item)| (i, item)).collect();
        indexed_items.sort_by(|&(_, a), &(_, b)| a.cmp(b));
        // Re-index so that position in presorted array becomes the tiebreaker.
        indexed_items
            .into_iter()
            .enumerate()
            .map(|(new_idx, (_, item))| (new_idx, item))
            .collect()
    } else {
        items.iter().enumerate().map(|(i, &item)| (i, item)).collect()
    };

    let key_gen = NatsortKey::new(flags);
    // decorate-sort-undecorate using the pre-sorted index as the tiebreaker.
    // Keys are computed once (in parallel), not per comparison.
    let mut decorated: Vec<(Vec<NatsortKeyPart>, usize, &str)> = indexed
        .into_par_iter()
        .map(|(idx, item)| (key_gen.key(item), idx, item))
        .collect();
    decorated.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    decorated
        .into_iter()
        .map(|(_, _, item)| item.to_string())
        .collect()
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
    // decorate-sort-undecorate, comparing keys in reverse; equal keys keep
    // their original relative order (stable) exactly like the old sort_by.
    let mut decorated: Vec<(Vec<NatsortKeyPart>, usize, &str)> = items
        .par_iter()
        .enumerate()
        .map(|(i, item)| (key_gen.key(item), i, *item))
        .collect();
    decorated.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    decorated
        .into_iter()
        .map(|(_, _, item)| item.to_string())
        .collect()
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
    fn natsorted_with_groupletters() {
        let data = vec!["Banana", "apple", "banana", "Apple"];
        let result = natsorted_with(&data, NsFlags::GROUPLETTERS);
        assert_eq!(result, vec!["Apple", "apple", "Banana", "banana"]);
    }

    #[test]
    fn natsorted_with_lowercasefirst() {
        let data = vec!["Banana", "apple", "banana", "Apple"];
        let result = natsorted_with(&data, NsFlags::LOWERCASEFIRST);
        assert_eq!(result, vec!["apple", "banana", "Apple", "Banana"]);
    }

    #[test]
    fn natsorted_with_numafter() {
        let data = vec!["b", "2", "a", "1"];
        let result = natsorted_with(&data, NsFlags::NUMAFTER);
        assert_eq!(result, vec!["a", "b", "1", "2"]);
    }

    #[test]
    fn natsorted_with_presort() {
        // 'a1' and 'a01' have the same natural key ('a', 1).
        // With PRESORT, tie-break by original string: 'a01' < 'a1'.
        let data = vec!["a1", "a01", "a2"];
        let result = natsorted_with(&data, NsFlags::PRESORT);
        assert_eq!(result, vec!["a01", "a1", "a2"]);
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

    #[test]
    fn os_sorted_basic() {
        let data = vec!["/dir/file10.txt", "/dir/file2.txt", "/dir/file1.txt"];
        let result = os_sorted(&data);
        assert_eq!(result, vec![
            "/dir/file1.txt",
            "/dir/file2.txt",
            "/dir/file10.txt",
        ]);
    }

    #[test]
    fn os_sorted_extension_handling() {
        let data = vec!["file(10).txt", "file(2).txt", "file(1).txt"];
        let result = os_sorted(&data);
        assert_eq!(result, vec![
            "file(1).txt",
            "file(2).txt",
            "file(10).txt",
        ]);
    }

    #[test]
    fn mixed_basic() {
        let data = vec![
            Item::Int(10),
            Item::from_str("2"),
            Item::Float(3.5),
            Item::Str("apple".to_string()),
        ];
        let result = natsorted_mixed(&data);
        assert_eq!(
            result,
            vec![
                Item::Int(2),
                Item::Float(3.5),
                Item::Int(10),
                Item::Str("apple".to_string()),
            ]
        );
    }

    #[test]
    fn mixed_none_first() {
        let data = vec![
            Item::Str("b".to_string()),
            Item::NoneVal,
            Item::Int(3),
            Item::NoneVal,
            Item::Str("a".to_string()),
        ];
        let result = natsorted_mixed(&data);
        assert_eq!(
            result,
            vec![
                Item::NoneVal,
                Item::NoneVal,
                Item::Int(3),
                Item::Str("a".to_string()),
                Item::Str("b".to_string()),
            ]
        );
    }

    #[test]
    fn recursive_basic() {
        let data = vec![
            NestedItem::branch(vec![NestedItem::int(1), NestedItem::str_("b")]),
            NestedItem::branch(vec![NestedItem::int(1), NestedItem::str_("a")]),
            NestedItem::branch(vec![NestedItem::int(2), NestedItem::str_("a")]),
        ];
        let result = natsorted_recursive(&data);
        assert_eq!(
            result,
            vec![
                NestedItem::branch(vec![NestedItem::int(1), NestedItem::str_("a")]),
                NestedItem::branch(vec![NestedItem::int(1), NestedItem::str_("b")]),
                NestedItem::branch(vec![NestedItem::int(2), NestedItem::str_("a")]),
            ]
        );
    }
}
