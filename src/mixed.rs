//! Mixed-type sorting: items that are strings, numbers, or None.
//!
//! The Python library accepts heterogeneous lists like `[10, "2", 3.5, None]` and
//! sorts them with this ordering:
//!
//! ```text
//! None → float(-inf) → int/float → str
//! ```
//!
//! In Rust we model this as an enum so the compiler enforces exhaustiveness.

use core::cmp::Ordering;

use crate::keygen::NatsortKey;
use crate::ns::NsFlags;
use crate::segment::NatsortKeyPart;

/// A single sortable item that may be a string, number, or None.
///
/// Mirrors Python's handling where `None`, `int`, `float`, and `str` can appear
/// together in the same list.  Booleans are treated as their integer values
/// (`False == 0`, `True == 1`).
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A string value (non-numeric).
    Str(String),
    /// An integer value.
    Int(i64),
    /// A floating-point value (may be NaN).
    Float(f64),
    /// Python `None` — always sorts first.
    NoneVal,
}

impl Item {
    /// Create an Item from a string, attempting to parse as a number first.
    ///
    /// Mirrors Python's behavior where `"2"` becomes `Int(2)` and `"apple"`
    /// stays as `Str("apple")`.
    pub fn from_str(s: &str) -> Self {
        if let Ok(n) = s.parse::<i64>() {
            return Self::Int(n);
        }
        if let Ok(f) = s.parse::<f64>() {
            return Self::Float(f);
        }
        Self::Str(s.to_string())
    }

    /// Create an Item from a boolean (Python bool → int mapping).
    pub fn from_bool(b: bool) -> Self {
        Self::Int(if b { 1 } else { 0 })
    }
}

impl Item {
    /// Generate a sort key for the item with the given flags.
    pub(crate) fn key(&self, flags: NsFlags) -> Vec<NatsortKeyPart> {
        match self {
            Self::Str(s) => {
                let kg = NatsortKey::new(flags);
                kg.key(s)
            }
            Self::Int(n) => {
                // Numbers get the sentinel treatment.
                if flags.contains(NsFlags::NANLAST) && *n == i64::MIN {
                    // Treat i64::MIN as a stand-in for -inf (unlikely in practice).
                    vec![
                        NatsortKeyPart::Str("".into()),
                        NatsortKeyPart::Float(f64::NEG_INFINITY),
                        NatsortKeyPart::Str("2".into()),
                    ]
                } else {
                    vec![
                        NatsortKeyPart::Str("".into()),
                        NatsortKeyPart::Int(*n),
                    ]
                }
            }
            Self::Float(f) => {
                if f.is_nan() {
                    let nan_val = if flags.contains(NsFlags::NANLAST) {
                        f64::INFINITY
                    } else {
                        f64::NEG_INFINITY
                    };
                    vec![
                        NatsortKeyPart::Str("".into()),
                        NatsortKeyPart::Float(nan_val),
                        NatsortKeyPart::Str(if flags.contains(NsFlags::NANLAST) {
                            "3"
                        } else {
                            "1"
                        }
                        .into()),
                    ]
                } else {
                    vec![
                        NatsortKeyPart::Str("".into()),
                        NatsortKeyPart::Float(*f),
                    ]
                }
            }
            Self::NoneVal => {
                // None maps to ('', -inf, '2') per Python's parse_number_or_none_factory.
                vec![
                    NatsortKeyPart::Str("".into()),
                    NatsortKeyPart::Float(f64::NEG_INFINITY),
                    NatsortKeyPart::Str("2".into()),
                ]
            }
        }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Item {}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        // Type ranking: None < Number < String.
        let self_rank = type_rank(self);
        let other_rank = type_rank(other);

        match self_rank.cmp(&other_rank) {
            Ordering::Equal => {
                // Same type: compare values.
                value_cmp(self, other)
            }
            ord => ord,
        }
    }
}

/// Returns a rank for type ordering: None(0) < Number(1) < String(2).
fn type_rank(item: &Item) -> u8 {
    match item {
        Item::NoneVal => 0,
        Item::Int(_) | Item::Float(_) => 1,
        Item::Str(_) => 2,
    }
}

/// Compare two items of the same type by value.
fn value_cmp(a: &Item, b: &Item) -> Ordering {
    match (a, b) {
        (Item::Int(x), Item::Int(y)) => x.cmp(y),
        (Item::Float(x), Item::Float(y)) => x.total_cmp(y),
        (Item::Str(x), Item::Str(y)) => x.cmp(y),
        (Item::NoneVal, Item::NoneVal) => Ordering::Equal,
        // Cross-type within same rank: int vs float.
        (Item::Int(x), Item::Float(y)) => (*x as f64).total_cmp(y),
        (Item::Float(x), Item::Int(y)) => x.total_cmp(&(*y as f64)),
        _ => Ordering::Equal,
    }
}

// ── Public API ─────────────────────────────────────────────────

/// Sort a slice of [`Item`] values naturally.
///
/// Uses the default algorithm.  Returns a new `Vec<Item>` sorted so that
/// `None` comes first, then numbers, then strings.
///
/// # Examples
///
/// ```
/// use natsort::{Item, natsorted_mixed};
///
/// let data = vec![
///     Item::Int(10),
///     Item::from_str("2"),
///     Item::Float(3.5),
///     Item::Str("apple".to_string()),
/// ];
/// let sorted = natsorted_mixed(&data);
/// assert_eq!(sorted, vec![
///     Item::Int(2),
///     Item::Float(3.5),
///     Item::Int(10),
///     Item::Str("apple".to_string()),
/// ]);
/// ```
pub fn natsorted_mixed(items: &[Item]) -> Vec<Item> {
    natsorted_mixed_with_impl(items, NsFlags::default())
}

/// Sort a slice of [`Item`] values with custom flags.
///
/// Supports [`NsFlags::NANLAST`](crate::ns::NsFlags::NANLAST) for NaN handling
/// among float values.
///
/// # Examples
///
/// ```
/// use natsort::{Item, natsorted_mixed_with, NsFlags};
///
/// let data = vec![
///     Item::Float(1.0),
///     Item::Float(f64::NAN),
///     Item::Float(2.0),
/// ];
/// let sorted = natsorted_mixed_with(&data, NsFlags::NANLAST);
/// // NaN moves to the end.
/// assert!(sorted.last().unwrap().is_nan());
/// ```
pub fn natsorted_mixed_with(items: &[Item], flags: NsFlags) -> Vec<Item> {
    natsorted_mixed_with_impl(items, flags)
}

fn natsorted_mixed_with_impl(items: &[Item], flags: NsFlags) -> Vec<Item> {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| {
        let ka = a.key(flags.clone());
        let kb = b.key(flags.clone());
        ka.cmp(&kb)
    });
    sorted
}

/// Check if an item is NaN (only meaningful for floats).
impl Item {
    /// Returns `true` if this is a NaN float value.
    pub fn is_nan(&self) -> bool {
        matches!(self, Item::Float(f) if f.is_nan())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn mixed_nan_default() {
        let data = vec![
            Item::Float(1.0),
            Item::Float(f64::NAN),
            Item::Float(2.0),
        ];
        let result = natsorted_mixed(&data);
        // Default: NaN → -inf, so it sorts first.
        assert!(result.first().unwrap().is_nan());
    }

    #[test]
    fn mixed_nan_last() {
        let data = vec![
            Item::Float(1.0),
            Item::Float(f64::NAN),
            Item::Float(2.0),
        ];
        let result = natsorted_mixed_with(&data, NsFlags::NANLAST);
        // NANLAST: NaN → +inf, so it sorts last.
        assert!(result.last().unwrap().is_nan());
    }

    #[test]
    fn mixed_bool_equivalent() {
        // False == 0, True == 1 in Python.
        let data = vec![
            Item::Int(1),
            Item::Int(0),
            Item::Str("a".to_string()),
        ];
        let result = natsorted_mixed(&data);
        assert_eq!(
            result,
            vec![
                Item::Int(0),
                Item::Int(1),
                Item::Str("a".to_string()),
            ]
        );
    }

    #[test]
    fn mixed_float_vs_int() {
        let data = vec![
            Item::Float(3.5),
            Item::Int(3),
            Item::Int(4),
        ];
        let result = natsorted_mixed(&data);
        assert_eq!(
            result,
            vec![
                Item::Int(3),
                Item::Float(3.5),
                Item::Int(4),
            ]
        );
    }
}
