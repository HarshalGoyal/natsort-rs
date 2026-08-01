//! Locale-aware string transformation.
//!
//! Python's natsort supports locale-aware sorting via `ns.LOCALEALPHA` and
//! `ns.LOCALENUM`.  This module provides a cross-platform approximation using
//! `unicase` for Unicode case-insensitive comparison, which covers the most
//! common use cases without requiring platform-specific locale libraries.
//!
//! For full locale support (including thousands separators, decimal points,
//! and platform-specific collation), see the Python implementation in
//! `natsort/compat/locale.py`.

use unicase::UniCase;

use crate::ns::NsFlags;

/// A locale-aware string wrapper that enables case-insensitive comparison.
///
/// When wrapped in a `Vec`, comparing two `LocaleStr` values uses
/// [`unicase`] semantics: `'A' == 'a'`, `'ß' == 'ss'` (where supported), etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocaleStr {
    inner: UniCase<String>,
}

impl LocaleStr {
    /// Create a new locale-aware string from any `AsRef<str>`.
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            inner: UniCase::new(s.into()),
        }
    }

    /// Returns the underlying string.
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl core::ops::Deref for LocaleStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PartialOrd for LocaleStr {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LocaleStr {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

/// Apply locale-aware transformation to a string.
///
/// For `LOCALEALPHA`, this lowercases the string using Unicode case-folding.
/// For `ns.GROUPLETTERS`, doubles all characters with lowercase variants.
/// For `ns.LOWERCASEFIRST`, swaps case (lower→upper, upper→lower).
pub fn locale_transform(input: &str, flags: NsFlags) -> String {
    let has_groupletters = flags.contains(NsFlags::GROUPLETTERS);
    let has_lowercasefirst = flags.contains(NsFlags::LOWERCASEFIRST);
    let has_localealpha = flags.contains(NsFlags::LOCALEALPHA);
    let has_ignorecase = flags.contains(NsFlags::IGNORECASE);

    if has_groupletters {
        // Double all characters, making doubled letters lowercase.
        // "Apple" → "aAppppllee" — lowercase variant FIRST, then original.
        input
            .chars()
            .flat_map(|c| {
                let lower = c.to_lowercase().next().unwrap_or(c);
                vec![lower, c]
            })
            .collect()
    } else if has_lowercasefirst {
        // Swap case: lowercase → uppercase, uppercase → lowercase.
        // "Apple" → "aPPLE"
        input.chars().map(|c| {
            if c.is_lowercase() {
                c.to_uppercase().next().unwrap()
            } else if c.is_uppercase() {
                c.to_lowercase().next().unwrap()
            } else {
                c
            }
        }).collect()
    } else if has_localealpha || has_ignorecase {
        // Use unicase for proper Unicode case folding, then lowercase.
        UniCase::new(input.to_lowercase()).to_string()
    } else {
        input.to_string()
    }
}

/// Compare two strings using locale-aware ordering.
///
/// Returns the same result as comparing their [`LocaleStr`] wrappers.
pub fn locale_cmp(a: &str, b: &str) -> core::cmp::Ordering {
    LocaleStr::new(a).cmp(&LocaleStr::new(b))
}

/// Get the null-string separator used for NUMAFTER mode.
///
/// Python uses `chr(sys.maxunicode) * 20` as the max string for non-locale
/// NUMAFTER, and an empty byte string for locale NUMAFTER.
/// We approximate this with a high-value Unicode repeat.
pub fn numafter_separator() -> &'static str {
    "\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_str_case_insensitive() {
        let a = LocaleStr::new("Apple");
        let b = LocaleStr::new("apple");
        assert_eq!(a, b);
    }

    #[test]
    fn locale_str_ordering() {
        // After case-folding, Banana becomes banana, so apple < banana.
        let a = LocaleStr::new("banana");
        let b = LocaleStr::new("apple");
        assert!(a > b);  // 'banana' > 'apple'
        
        // Same strings (case-insensitive) are equal.
        let c = LocaleStr::new("Apple");
        let d = LocaleStr::new("apple");
        assert_eq!(c, d);
    }

    #[test]
    fn locale_transform_groupletters() {
        assert_eq!(locale_transform("Apple", NsFlags::GROUPLETTERS), "aAppppllee");
    }

    #[test]
    fn locale_transform_lowercasefirst() {
        assert_eq!(locale_transform("Apple", NsFlags::LOWERCASEFIRST), "aPPLE");
    }

    #[test]
    fn locale_transform_localealpha() {
        assert_eq!(locale_transform("Hello WORLD", NsFlags::LOCALEALPHA), "hello world");
    }

    #[test]
    fn locale_transform_ignorecase() {
        assert_eq!(locale_transform("Hello WORLD", NsFlags::IGNORECASE), "hello world");
    }

    #[test]
    fn locale_cmp_works() {
        assert_eq!(locale_cmp("Apple", "apple"), core::cmp::Ordering::Equal);
        assert_eq!(locale_cmp("banana", "apple"), core::cmp::Ordering::Greater);
    }
}
