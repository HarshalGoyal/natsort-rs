//! The `NatsortKey` struct: compiled regex + flag-driven key generation.
//!
//! This is the central piece that ties together [`NsFlags`](crate::ns::NsFlags),
//! a pre-compiled [`regex::Regex`], and the segmentation logic from
//! [`segment`](crate::segment).

use regex::Regex;

use crate::ns::NsFlags;
use crate::segment::{insert_sentinels, try_convert_to_number, NatsortKeyPart};

/// A compiled natsort key generator.
///
/// Holds a flags configuration and a pre-compiled regex so that
/// [`key()`](Self::key) can be called repeatedly without recompilation.
#[derive(Debug, Clone)]
pub struct NatsortKey {
    /// Algorithm flags.
    pub flags: NsFlags,
    /// Pre-compiled regex for splitting strings into number/non-number components.
    regex: Regex,
}

impl NatsortKey {
    /// Create a new key generator with the given flags.
    ///
    /// The regex is compiled once based on the flag combination.
    /// Panics if the regex pattern is invalid (should never happen with our patterns).
    pub fn new(flags: NsFlags) -> Self {
        let regex = compile_regex(flags);
        Self { flags, regex }
    }

    /// Generate a sort key for the given string.
    ///
    /// The result is a vector of [`NatsortKeyPart`] elements that implements
    /// [`Ord`](core::cmp::Ord) via element-by-element comparison.  Keys are
    /// guaranteed to always start with a `Str` variant thanks to sentinel
    /// insertion.
    ///
    /// # Examples
    ///
    /// ```
    /// use natsort::{NsFlags, NatsortKey};
    ///
    /// let key_gen = NatsortKey::new(NsFlags::default());
    /// assert_eq!(
    ///     key_gen.key("10a"),
    ///     vec![
    ///         natsort::NatsortKeyPart::Str("".into()),
    ///         natsort::NatsortKeyPart::Int(10),
    ///         natsort::NatsortKeyPart::Str("a".into()),
    ///     ]
    /// );
    /// ```
    pub fn key(&self, input: &str) -> Vec<NatsortKeyPart> {
        let transformed = if self.flags.contains(NsFlags::IGNORECASE) {
            // Apply case-folding before splitting. For ASCII (Phase 1),
            // to_lowercase() is equivalent to Python's str.casefold().
            input.to_lowercase()
        } else {
            input.to_string()
        };
        let parts = self.split_key(&transformed);
        insert_sentinels(parts)
    }

    /// Split an input string into raw segments (before sentinel insertion).
    ///
    /// This applies the regex, filters out empty matches, and converts each
    /// segment to a number when possible.
    pub fn split_key(&self, input: &str) -> Vec<NatsortKeyPart> {
        let mut result = Vec::new();
        let mut last_end = 0;

        for cap in self.regex.captures_iter(input) {
            let mat = cap.get(0).unwrap();
            // Add the text before this match (if any)
            if mat.start() > last_end {
                let before = &input[last_end..mat.start()];
                if !before.is_empty() {
                    result.push(try_convert_to_number(before));
                }
            }
            // Add the matched number
            let matched = &input[mat.start()..mat.end()];
            result.push(try_convert_to_number(matched));
            last_end = mat.end();
        }

        // Add any remaining text after the last match
        if last_end < input.len() {
            let after = &input[last_end..];
            if !after.is_empty() {
                result.push(try_convert_to_number(after));
            }
        }

        result
    }
}

impl Default for NatsortKey {
    fn default() -> Self {
        Self::new(NsFlags::default())
    }
}

/// Compile the regex pattern appropriate for the given flags.
fn compile_regex(flags: NsFlags) -> Regex {
    let pattern = match_pattern(flags);
    Regex::new(pattern).expect("natsort regex pattern must be valid")
}

/// Select the regex pattern string based on algorithm flags.
/// Mirrors Python's `regex_chooser` function.
fn match_pattern(flags: NsFlags) -> &'static str {
    let has_float = flags.contains(NsFlags::FLOAT);

    if has_float {
        let has_signed = flags.contains(NsFlags::SIGNED);
        let has_noexp = flags.contains(NsFlags::NOEXP);

        if has_signed && has_noexp {
            r"([-+]?(?:\d+\.?\d*|\.\d+))"
        } else if has_noexp {
            r"((?:\d+\.?\d*|\.\d+))"
        } else if has_signed {
            r"([-+]?(?:\d+\.?\d*(?:[eE][-+]?\d+)?|\.\d+(?:[eE][-+]?\d+)?))"
        } else {
            r"((?:\d+\.?\d*(?:[eE][-+]?\d+)?|\.\d+(?:[eE][-+]?\d+)?))"
        }
    } else if flags.contains(NsFlags::SIGNED) {
        r"([-+]?\d+)"
    } else {
        r"(\d+)"
    }
}

// ── Module-level convenience ─────────────────────────────────────────

/// Default natsort key generator (uses [`NsFlags::DEFAULT`](NsFlags::DEFAULT)).
pub fn default_key() -> NatsortKey {
    NatsortKey::default()
}

/// Generate a sort key using the default flags.
pub fn default_sort_key(input: &str) -> Vec<NatsortKeyPart> {
    default_key().key(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_key_10a() {
        let key_gen = NatsortKey::default();
        let result = key_gen.key("10a");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Int(10),
                NatsortKeyPart::Str("a".into()),
            ]
        );
    }

    #[test]
    fn default_key_a10() {
        let key_gen = NatsortKey::default();
        let result = key_gen.key("a10");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("a".into()),
                NatsortKeyPart::Int(10),
            ]
        );
    }

    #[test]
    fn default_key_empty() {
        let key_gen = NatsortKey::default();
        assert!(key_gen.key("").is_empty());
    }

    #[test]
    fn default_key_minus_five() {
        let key_gen = NatsortKey::default();
        let result = key_gen.key("-5");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("-".into()),
                NatsortKeyPart::Int(5),
            ]
        );
    }

    #[test]
    fn default_key_1e10() {
        let key_gen = NatsortKey::default();
        let result = key_gen.key("1e10");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Int(1),
                NatsortKeyPart::Str("e".into()),
                NatsortKeyPart::Int(10),
            ]
        );
    }

    #[test]
    fn default_key_1_5() {
        let key_gen = NatsortKey::default();
        let result = key_gen.key("1.5");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Int(1),
                NatsortKeyPart::Str(".".into()),
                NatsortKeyPart::Int(5),
            ]
        );
    }

    #[test]
    fn default_key_file10_5_txt() {
        let key_gen = NatsortKey::default();
        let result = key_gen.key("file10.5.txt");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("file".into()),
                NatsortKeyPart::Int(10),
                NatsortKeyPart::Str(".".into()),
                NatsortKeyPart::Int(5),
                NatsortKeyPart::Str(".txt".into()),
            ]
        );
    }

    #[test]
    fn signed_key_minus_five() {
        let key_gen = NatsortKey::new(NsFlags::SIGNED);
        let result = key_gen.key("-5");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Int(-5),
            ]
        );
    }

    #[test]
    fn float_key_1_5() {
        let key_gen = NatsortKey::new(NsFlags::FLOAT);
        let result = key_gen.key("1.5");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Float(1.5),
            ]
        );
    }

    #[test]
    fn float_key_1e10() {
        let key_gen = NatsortKey::new(NsFlags::FLOAT);
        let result = key_gen.key("1e10");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Float(1e10),
            ]
        );
    }

    #[test]
    fn real_key_minus_3_2() {
        let key_gen = NatsortKey::new(NsFlags::REAL);
        let result = key_gen.key("-3.2");
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Float(-3.2),
            ]
        );
    }

    #[test]
    fn sorting_order_basic() {
        let key_gen = NatsortKey::default();
        let items = vec!["4", "8", "2", "10", "3"];
        let keys: Vec<_> = items.iter().map(|s| key_gen.key(s)).collect();

        let mut sorted_indices: Vec<usize> = (0..keys.len()).collect();
        sorted_indices.sort_by(|&a, &b| keys[a].cmp(&keys[b]));

        let sorted: Vec<&str> = sorted_indices.iter().map(|&i| items[i]).collect();
        assert_eq!(sorted, vec!["2", "3", "4", "8", "10"]);
    }

    #[test]
    fn sorting_order_files() {
        let key_gen = NatsortKey::default();
        let items = vec!["file10.txt", "file2.txt", "file1.txt"];
        let keys: Vec<_> = items.iter().map(|s| key_gen.key(s)).collect();

        let mut sorted_indices: Vec<usize> = (0..keys.len()).collect();
        sorted_indices.sort_by(|&a, &b| keys[a].cmp(&keys[b]));

        let sorted: Vec<&str> = sorted_indices.iter().map(|&i| items[i]).collect();
        assert_eq!(sorted, vec!["file1.txt", "file2.txt", "file10.txt"]);
    }

    #[test]
    fn sorting_numbers_before_text() {
        let key_gen = NatsortKey::default();
        let items = vec!["b", "2", "a", "1"];
        let keys: Vec<_> = items.iter().map(|s| key_gen.key(s)).collect();

        let mut sorted_indices: Vec<usize> = (0..keys.len()).collect();
        sorted_indices.sort_by(|&a, &b| keys[a].cmp(&keys[b]));

        let sorted: Vec<&str> = sorted_indices.iter().map(|&i| items[i]).collect();
        assert_eq!(sorted, vec!["1", "2", "a", "b"]);
    }

    #[test]
    fn sorting_with_sentinel() {
        // "5" should sort before "a" because ('', 5) < ('a',)
        let key_gen = NatsortKey::default();
        let items = vec!["a", "5"];
        let keys: Vec<_> = items.iter().map(|s| key_gen.key(s)).collect();

        let mut sorted_indices: Vec<usize> = (0..keys.len()).collect();
        sorted_indices.sort_by(|&a, &b| keys[a].cmp(&keys[b]));

        let sorted: Vec<&str> = sorted_indices.iter().map(|&i| items[i]).collect();
        assert_eq!(sorted, vec!["5", "a"]);
    }
}
