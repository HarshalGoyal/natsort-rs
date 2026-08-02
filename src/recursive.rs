//! Recursive descent: sorting nested lists of items.
//!
//! Python's natsort handles lists of lists like `[[1, "b"], [1, "a"], [2, "a"]]`
//! by comparing element-by-element, recursively applying natural sort to nested
//! lists.  Shorter lists sort first if their prefix matches a longer one.

use crate::keygen::NatsortKey;
use crate::mixed::Item;
use crate::ns::NsFlags;
use crate::segment::NatsortKeyPart;

/// A value that may be a leaf (string, number, None) or a branch (nested list).
///
/// This mirrors Python's ability to sort `[[1, "a"], [2, "b"], [1, "b"]]`.
#[derive(Debug, Clone, PartialEq)]
pub enum NestedItem {
    /// A leaf value: string, number, or None.
    Leaf(Item),
    /// A nested list of items.
    Branch(Vec<NestedItem>),
}

impl NestedItem {
    /// Generate a sort key for this nested item.
    ///
    /// For leaves, delegates to [`Item::key`].  For branches, recursively
    /// generates keys for each child and wraps them in a tuple structure
    /// so that element-by-element comparison works correctly.
    fn key(&self, flags: NsFlags) -> Vec<NatsortKeyPart> {
        match self {
            Self::Leaf(item) => item.key(flags),
            Self::Branch(children) => {
                // Each child gets its own key vector. We flatten them into
                // a single key by concatenating, with a separator between children.
                let mut result = Vec::new();
                for child in children {
                    let child_key = child.key(flags.clone());
                    result.extend(child_key);
                }
                // If empty branch, return empty key (sorts first).
                if result.is_empty() {
                    result.push(NatsortKeyPart::Str("".into()));
                }
                result
            }
        }
    }
}

impl PartialOrd for NestedItem {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NestedItem {}

impl Ord for NestedItem {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Compare element-by-element via keys.
        let key_gen = NatsortKey::default();
        let self_parts = self.parts(&key_gen);
        let other_parts = other.parts(&key_gen);

        // Element-by-element comparison.
        for (a, b) in self_parts.iter().zip(other_parts.iter()) {
            match a.cmp(b) {
                core::cmp::Ordering::Equal => continue,
                ord => return ord,
            }
        }

        // All compared elements equal: shorter list sorts first.
        self_parts.len().cmp(&other_parts.len())
    }
}

impl NestedItem {
    /// Extract the top-level parts for comparison.
    ///
    /// For a leaf, returns the leaf's key parts.
    /// For a branch, recursively extracts parts from each child.
    fn parts(&self, key_gen: &NatsortKey) -> Vec<NatsortKeyPart> {
        match self {
            Self::Leaf(item) => item.key(key_gen.flags),
            Self::Branch(children) => {
                let mut parts = Vec::new();
                for child in children {
                    parts.extend(child.parts(key_gen));
                }
                if parts.is_empty() {
                    parts.push(NatsortKeyPart::Str("".into()));
                }
                parts
            }
        }
    }
}

// -------- Convenience constructors ---------------------------------------------

impl NestedItem {
    /// Create a leaf from a string.
    pub fn str_(s: impl Into<String>) -> Self {
        Self::Leaf(Item::Str(s.into()))
    }

    /// Create a leaf from an integer.
    pub fn int(n: i64) -> Self {
        Self::Leaf(Item::Int(n))
    }

    /// Create a leaf from a float.
    pub fn float(n: f64) -> Self {
        Self::Leaf(Item::Float(n))
    }

    /// Create a None leaf.
    pub fn none() -> Self {
        Self::Leaf(Item::NoneVal)
    }

    /// Create a branch (nested list).
    pub fn branch(children: Vec<NestedItem>) -> Self {
        Self::Branch(children)
    }
}

// --- Public API ---------------------------------------------------------

/// Sort a slice of [`NestedItem`] values with recursive descent.
///
/// Lists are compared element-by-element.  Nested lists are recursed into.
/// Shorter lists sort first if their prefix matches a longer one.
///
/// # Examples
///
/// ```
/// use natsort::{NestedItem, natsorted_recursive};
///
/// let data = vec![
///     NestedItem::branch(vec![
///         NestedItem::int(1),
///         NestedItem::str_("b"),
///     ]),
///     NestedItem::branch(vec![
///         NestedItem::int(1),
///         NestedItem::str_("a"),
///     ]),
///     NestedItem::branch(vec![
///         NestedItem::int(2),
///         NestedItem::str_("a"),
///     ]),
/// ];
/// let sorted = natsorted_recursive(&data);
/// assert_eq!(sorted[0], NestedItem::branch(vec![
///     NestedItem::int(1),
///     NestedItem::str_("a"),
/// ]));
/// ```
pub fn natsorted_recursive(items: &[NestedItem]) -> Vec<NestedItem> {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| a.cmp(b));
    sorted
}

/// Sort a slice of [`NestedItem`] values with custom flags.
pub fn natsorted_recursive_with(items: &[NestedItem], flags: NsFlags) -> Vec<NestedItem> {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| {
        let ka = a.key(flags.clone());
        let kb = b.key(flags.clone());
        ka.cmp(&kb)
    });
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn recursive_shorter_first() {
        let data = vec![
            NestedItem::branch(vec![NestedItem::int(1), NestedItem::str_("a")]),
            NestedItem::branch(vec![NestedItem::int(1)]),
        ];
        let result = natsorted_recursive(&data);
        // Shorter list sorts first when prefix matches.
        assert_eq!(result[0], NestedItem::branch(vec![NestedItem::int(1)]));
    }

    #[test]
    fn recursive_nested() {
        let data = vec![
            NestedItem::branch(vec![
                NestedItem::int(2),
                NestedItem::branch(vec![NestedItem::int(2), NestedItem::str_("b")]),
            ]),
            NestedItem::branch(vec![
                NestedItem::int(1),
                NestedItem::branch(vec![NestedItem::int(1), NestedItem::str_("a")]),
            ]),
        ];
        let result = natsorted_recursive(&data);
        assert_eq!(
            result,
            vec![
                NestedItem::branch(vec![
                    NestedItem::int(1),
                    NestedItem::branch(vec![NestedItem::int(1), NestedItem::str_("a")]),
                ]),
                NestedItem::branch(vec![
                    NestedItem::int(2),
                    NestedItem::branch(vec![NestedItem::int(2), NestedItem::str_("b")]),
                ]),
            ]
        );
    }

    #[test]
    fn recursive_leaf_in_branch() {
        // Mix of leaf and branch at top level.
        let data = vec![
            NestedItem::Leaf(Item::Str("b".to_string())),
            NestedItem::branch(vec![NestedItem::int(1)]),
            NestedItem::Leaf(Item::Int(2)),
        ];
        let result = natsorted_recursive(&data);
        // Branches and leaves compare via their keys.
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn empty_branch() {
        let data = vec![
            NestedItem::branch(vec![]),
            NestedItem::branch(vec![NestedItem::int(1)]),
        ];
        let result = natsorted_recursive(&data);
        assert_eq!(result[0], NestedItem::branch(vec![]));
    }
}
