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
/// For `ns.LOCALENUM`, removes thousands separators and converts decimal points.
pub fn locale_transform(input: &str, flags: NsFlags) -> String {
    let mut result = input.to_string();

    // Apply transformations in order (matching Python's chain_functions)

    // GROUPLETTERS transformation
    if flags.contains(NsFlags::GROUPLETTERS) {
        // Double all characters, making doubled letters lowercase.
        // "Apple" → "aAppppllee" — lowercase variant FIRST, then original.
        result = result
            .chars()
            .flat_map(|c| {
                let lower = c.to_lowercase().next().unwrap_or(c);
                vec![lower, c]
            })
            .collect();
    }

    // LOWERCASEFIRST transformation
    if flags.contains(NsFlags::LOWERCASEFIRST) {
        // Swap case: lowercase → uppercase, uppercase → lowercase.
        // "Apple" → "aPPLE"
        result = result
            .chars()
            .map(|c| {
                if c.is_lowercase() {
                    c.to_uppercase().next().unwrap()
                } else if c.is_uppercase() {
                    c.to_lowercase().next().unwrap()
                } else {
                    c
                }
            })
            .collect();
    }

    // LOCALEALPHA or IGNORECASE transformation
    if flags.contains(NsFlags::LOCALEALPHA) || flags.contains(NsFlags::IGNORECASE) {
        // Use unicase for proper Unicode case folding, then lowercase.
        result = UniCase::new(result.to_lowercase()).to_string();
    }

    // LOCALENUM transformation
    if flags.contains(NsFlags::LOCALENUM) {
        result = handle_localenum(&result);
    }

    result
}

/// Handle LOCALENUM transformation: remove thousands separators and convert decimal points.
/// This is a simplified implementation that handles the most common cases.
fn handle_localenum(input: &str) -> String {
    // Simplified implementation that mimics Python's logic:
    // 1. Remove thousands separators (comma, period, space, non-breaking space)
    // 2. Only remove if followed by exactly 3 digits
    // 3. Don't remove if it's actually a decimal point

    // Note: This doesn't handle locale-specific decimal points (e.g., comma in de_DE)
    // For full locale support, we would need to know the current locale's
    // thousands separator and decimal point.

    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Check if this character could be a thousands separator
        if c == ',' || c == '.' || c == ' ' || c == '\u{00A0}' {
            // Check if it's a valid thousands separator:
            // 1. Must have at least 1 digit before
            // 2. Must have exactly 3 digits after
            // 3. Those 3 digits must be followed by a non-digit or end of string

            let mut is_thousands_sep = false;

            // Check for digit before
            if i > 0 && chars[i - 1].is_ascii_digit() {
                // Check for 3 digits after
                if i + 3 < chars.len() {
                    let d1 = chars[i + 1].is_ascii_digit();
                    let d2 = chars[i + 2].is_ascii_digit();
                    let d3 = chars[i + 3].is_ascii_digit();

                    if d1 && d2 && d3 {
                        // Check what comes after the 3 digits
                        if i + 4 == chars.len() || !chars[i + 4].is_ascii_digit() {
                            // Valid thousands separator pattern
                            is_thousands_sep = true;
                        }
                    }
                }
            }

            if is_thousands_sep {
                // Skip this thousands separator
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

/// Compare two strings using locale-aware ordering.
///
/// Returns the same result as comparing their [`LocaleStr`] wrappers.
pub fn locale_cmp(a: &str, b: &str) -> core::cmp::Ordering {
    LocaleStr::new(a).cmp(&LocaleStr::new(b))
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
        assert!(a > b); // 'banana' > 'apple'

        // Same strings (case-insensitive) are equal.
        let c = LocaleStr::new("Apple");
        let d = LocaleStr::new("apple");
        assert_eq!(c, d);
    }

    #[test]
    fn locale_transform_groupletters() {
        assert_eq!(
            locale_transform("Apple", NsFlags::GROUPLETTERS),
            "aAppppllee"
        );
    }

    #[test]
    fn locale_transform_lowercasefirst() {
        assert_eq!(locale_transform("Apple", NsFlags::LOWERCASEFIRST), "aPPLE");
    }

    #[test]
    fn locale_transform_localealpha() {
        assert_eq!(
            locale_transform("Hello WORLD", NsFlags::LOCALEALPHA),
            "hello world"
        );
    }

    #[test]
    fn locale_transform_ignorecase() {
        assert_eq!(
            locale_transform("Hello WORLD", NsFlags::IGNORECASE),
            "hello world"
        );
    }

    #[test]
    fn locale_transform_localenum_thousands() {
        // Test thousands separator removal
        let result1 = locale_transform("a5,467", NsFlags::LOCALENUM);
        println!("locale_transform(\"a5,467\", LOCALENUM) = \"{}\"", result1);
        assert_eq!(result1, "a5467");

        let result2 = locale_transform("a12,543,642", NsFlags::LOCALENUM);
        println!(
            "locale_transform(\"a12,543,642\", LOCALENUM) = \"{}\"",
            result2
        );
        assert_eq!(result2, "a12543642");

        // Should NOT remove comma not followed by 3 digits
        assert_eq!(locale_transform("a5,6", NsFlags::LOCALENUM), "a5,6");
        assert_eq!(locale_transform("a5,67", NsFlags::LOCALENUM), "a5,67");

        // Test with LOCALE flag (includes LOCALENUM)
        let result3 = locale_transform("a5,467", NsFlags::LOCALE);
        println!("locale_transform(\"a5,467\", LOCALE) = \"{}\"", result3);
        assert_eq!(result3, "a5467");
    }
}
