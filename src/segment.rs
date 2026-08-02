//! Core segmentation types and the sentinel-based ordering mechanism.
//!
//! The Python natsort algorithm produces tuples that **always** start with a
//! string, then alternate between numbers and strings.  This is achieved by a
//! *sentinel* — an empty `''` string — that is prepended when the first
//! component is numeric, and inserted between adjacent numeric components.
//!
//! Rust cannot mix types inside a tuple for comparison (comparing `str` to
//! `i64` is a compile error).  Instead we produce a single enum type whose
//! [`Ord`] impl reproduces exactly the same element-by-element comparison
//! semantics that Python's mixed-type tuple would give:
//!
//! ```text
//! key("10a") == ('', 10, 'a')      key("a10") == ('a', 10)
//! key("5")   == ('', 5)            key("a")   == ('a',)
//! ```
//!
//! The critical invariant from [DECISIONS.md §D-007(c)](../DECISIONS.md#d-007c):
//! numbers sort before letters because `'' < 'a'`, not through any cross-type
//! ranking.  Between adjacent numbers, `''` is also inserted so the alternation
//! never breaks.

use core::cmp::Ordering;

/// A single element of a natsort key.
///
/// Produced by [`split_key`](fn@crate::keygen::NatsortKey::split_key).
/// Two keys are compared element-by-element using [`Ord`].
#[derive(Debug, Clone, PartialEq)]
pub enum NatsortKeyPart {
    /// A text fragment from the input string.
    Str(String),
    /// An integer fragment parsed from digits in the input.
    Int(i64),
    /// A floating-point fragment parsed from digits/dots/exponents in the input.
    Float(f64),
}

// ----- Ord implementation -----------------------------------------

impl PartialOrd for NatsortKeyPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for NatsortKeyPart {}

impl Ord for NatsortKeyPart {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Same type: compare values directly.
            (Self::Str(a), Self::Str(b)) => a.cmp(b),
            (Self::Int(a), Self::Int(b)) => a.cmp(b),
            (Self::Float(a), Self::Float(b)) => a.total_cmp(b),

            // Numbers always sort before strings.
            (Self::Int(_), Self::Str(_)) | (Self::Float(_), Self::Str(_)) => Ordering::Less,
            (Self::Str(_), Self::Int(_)) | (Self::Str(_), Self::Float(_)) => Ordering::Greater,

            // Int vs Float: compare as f64.
            (Self::Int(a), Self::Float(b)) => (*a as f64).total_cmp(b),
            (Self::Float(a), Self::Int(b)) => a.total_cmp(&(*b as f64)),
        }
    }
}

// ------------- Sentinel insertion ---------------------------------------

/// Insert empty-string sentinels into a sequence of parts so that:
///
/// 1. If the first part is numeric, prepend `Str("")`.
/// 2. Between every pair of adjacent numeric parts, insert `Str("")`.
///
/// This guarantees the output always starts with a string and alternates
/// str / num / str / num … which is the exact invariant produced by
/// Python's `sep_inserter` + final wrapping.
///
/// See [DECISIONS.md §D-007(c)](../DECISIONS.md#d-007c).
pub fn insert_sentinels(parts: Vec<NatsortKeyPart>) -> Vec<NatsortKeyPart> {
    if parts.is_empty() {
        return parts;
    }

    let mut result = Vec::with_capacity(parts.len() * 2);

    // Check if first part is numeric → prepend sentinel.
    let mut prev_is_num = is_numeric(&parts[0]);
    if prev_is_num {
        result.push(NatsortKeyPart::Str(String::new()));
    }
    result.push(parts[0].clone());

    for part in parts.iter().skip(1) {
        let is_num = is_numeric(part);
        if prev_is_num && is_num {
            result.push(NatsortKeyPart::Str(String::new()));
        }
        result.push(part.clone());
        prev_is_num = is_num;
    }

    result
}

/// Returns `true` if the given part is numeric (Int or Float).
fn is_numeric(part: &NatsortKeyPart) -> bool {
    matches!(part, NatsortKeyPart::Int(_) | NatsortKeyPart::Float(_))
}

// ----- Component transformation -------------------------------------

/// Attempt to convert a regex-captured string segment into a number.
///
/// Tries `i64` first, then `f64`, then returns the original string.
/// Empty strings are returned as-is (they should have been filtered out
/// before calling this, but we handle it defensively).
/// Convert Unicode digits to ASCII digits for parsing
fn unicode_digit_to_ascii(c: char) -> Option<char> {
    // Check if it's a Unicode decimal digit
    if c.is_ascii_digit() {
        Some(c)
    } else {
        // Map common Unicode digits to ASCII
        match c {
            '٠' => Some('0'), // Arabic-Indic digit zero
            '١' => Some('1'),
            '٢' => Some('2'),
            '٣' => Some('3'),
            '٤' => Some('4'),
            '٥' => Some('5'),
            '٦' => Some('6'),
            '٧' => Some('7'),
            '٨' => Some('8'),
            '٩' => Some('9'),
            '۰' => Some('0'), // Persian digit zero
            '۱' => Some('1'),
            '۲' => Some('2'),
            '۳' => Some('3'),
            '۴' => Some('4'),
            '۵' => Some('5'),
            '۶' => Some('6'),
            '۷' => Some('7'),
            '۸' => Some('8'),
            '۹' => Some('9'),
            '०' => Some('0'), // Devanagari digit zero
            '१' => Some('1'),
            '२' => Some('2'),
            '३' => Some('3'),
            '४' => Some('4'),
            '५' => Some('5'),
            '६' => Some('6'),
            '७' => Some('7'),
            '८' => Some('8'),
            '९' => Some('9'),
            '০' => Some('0'), // Bengali digit zero
            '১' => Some('1'),
            '২' => Some('2'),
            '৩' => Some('3'),
            '৪' => Some('4'),
            '৫' => Some('5'),
            '৬' => Some('6'),
            '৭' => Some('7'),
            '৮' => Some('8'),
            '৯' => Some('9'),
            '௦' => Some('0'), // Tamil digit zero
            '௧' => Some('1'),
            '௨' => Some('2'),
            '௩' => Some('3'),
            '௪' => Some('4'),
            '௫' => Some('5'),
            '௬' => Some('6'),
            '௭' => Some('7'),
            '௮' => Some('8'),
            '௯' => Some('9'),
            _ => None,
        }
    }
}

/// Convert a string with Unicode digits to ASCII for parsing
fn convert_unicode_to_ascii(s: &str) -> String {
    s.chars()
        .map(|c| unicode_digit_to_ascii(c).unwrap_or(c))
        .collect()
}

/// Convert a parsed string segment into a number when possible, else keep it as a [`NatsortKeyPart::Str`].
///
/// Non-ASCII digits are first normalized to ASCII, then the string is tried as
/// an `i64` and then an `f64`. If neither parses it is returned unchanged as a
/// string segment.
pub fn try_convert_to_number(s: &str) -> NatsortKeyPart {
    if s.is_empty() {
        return NatsortKeyPart::Str(s.to_string());
    }

    // Convert Unicode digits to ASCII for parsing
    let ascii_s = convert_unicode_to_ascii(s);
    
    // Fast path: check if it looks like a number.
    let first_char = ascii_s.chars().next().unwrap();
    if !first_char.is_ascii_digit() && first_char != '+' && first_char != '-' && first_char != '.' {
        return NatsortKeyPart::Str(s.to_string());
    }

    // Try int first (covers cases like "10", "+5", "-3").
    if let Ok(n) = ascii_s.parse::<i64>() {
        return NatsortKeyPart::Int(n);
    }

    // Try float (covers cases like "1.5", "1e10", ".5", "+2.3E-4").
    if let Ok(f) = ascii_s.parse::<f64>() {
        return NatsortKeyPart::Float(f);
    }

    NatsortKeyPart::Str(s.to_string())
}

/// Convert a numeric value to its sort-key representation with NaN handling.
///
/// Python's `parse_number_or_none_factory` maps NaN/None to special tuples:
/// - Default: `('', -inf, '1')` for NaN, `('', -inf, '2')` for None
/// - NANLAST: `('', +inf, '3')` for NaN, `('', +inf, '1')` for NaN replacement
pub fn convert_number_with_nan(val: f64, nanlast: bool) -> Vec<NatsortKeyPart> {
    if val.is_nan() {
        let nan_val = if nanlast { f64::INFINITY } else { f64::NEG_INFINITY };
        vec![
            NatsortKeyPart::Str("".into()),
            NatsortKeyPart::Float(nan_val),
            NatsortKeyPart::Str(if nanlast { "3" } else { "1" }.into()),
        ]
    } else {
        vec![NatsortKeyPart::Str("".into()), NatsortKeyPart::Float(val)]
    }
}

/// The null-string separator used for NUMAFTER mode.
///
/// Mirrors Python's `chr(sys.maxunicode) * 20`.
pub const NUMAFTER_SEPARATOR: &str = "\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}\u{10ffff}";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_part_ord_same_type() {
        assert!(NatsortKeyPart::Int(10) > NatsortKeyPart::Int(5));
        assert!(NatsortKeyPart::Float(2.5) > NatsortKeyPart::Float(1.5));
        assert!(NatsortKeyPart::Str("b".into()) > NatsortKeyPart::Str("a".into()));
    }

    #[test]
    fn key_part_ord_numbers_before_strings() {
        assert!(NatsortKeyPart::Int(5) < NatsortKeyPart::Str("a".into()));
        assert!(NatsortKeyPart::Float(1.0) < NatsortKeyPart::Str("a".into()));
        assert!(NatsortKeyPart::Int(5) < NatsortKeyPart::Str("".into()));
    }

    #[test]
    fn key_part_ord_sentinel_works() {
        // '' < 'a' means numeric-first sorts before letter-first
        assert!(NatsortKeyPart::Str("".into()) < NatsortKeyPart::Str("a".into()));
    }

    #[test]
    fn insert_sentinels_first_is_numeric() {
        let parts = vec![
            NatsortKeyPart::Int(10),
            NatsortKeyPart::Str("a".into()),
        ];
        let result = insert_sentinels(parts);
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
    fn insert_sentinels_adjacent_numbers() {
        let parts = vec![
            NatsortKeyPart::Int(10),
            NatsortKeyPart::Int(5),
        ];
        let result = insert_sentinels(parts);
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Int(10),
                NatsortKeyPart::Str("".into()),
                NatsortKeyPart::Int(5),
            ]
        );
    }

    #[test]
    fn insert_sentinels_no_change_for_text_first() {
        let parts = vec![
            NatsortKeyPart::Str("file".into()),
            NatsortKeyPart::Int(10),
            NatsortKeyPart::Str(".".to_string()),
            NatsortKeyPart::Int(5),
            NatsortKeyPart::Str(".txt".into()),
        ];
        let result = insert_sentinels(parts.clone());
        assert_eq!(result, parts);
    }

    #[test]
    fn try_convert_int() {
        assert_eq!(try_convert_to_number("10"), NatsortKeyPart::Int(10));
        assert_eq!(try_convert_to_number("-5"), NatsortKeyPart::Int(-5));
        assert_eq!(try_convert_to_number("+3"), NatsortKeyPart::Int(3));
    }

    #[test]
    fn try_convert_float() {
        assert_eq!(try_convert_to_number("1.5"), NatsortKeyPart::Float(1.5));
        assert_eq!(try_convert_to_number("1e10"), NatsortKeyPart::Float(1e10));
        assert_eq!(try_convert_to_number(".5"), NatsortKeyPart::Float(0.5));
    }

    #[test]
    fn try_convert_string() {
        assert_eq!(try_convert_to_number("abc"), NatsortKeyPart::Str("abc".into()));
        assert_eq!(try_convert_to_number("."), NatsortKeyPart::Str(".".into()));
        assert_eq!(try_convert_to_number(""), NatsortKeyPart::Str("".into()));
    }

    #[test]
    fn full_key_10a() {
        // Simulate: split "10a" → ["10", "a"] → convert → insert_sentinels
        let raw_parts = vec![
            try_convert_to_number("10"),
            try_convert_to_number("a"),
        ];
        let result = insert_sentinels(raw_parts);
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
    fn full_key_a10() {
        let raw_parts = vec![
            try_convert_to_number("a"),
            try_convert_to_number("10"),
        ];
        let result = insert_sentinels(raw_parts);
        assert_eq!(
            result,
            vec![
                NatsortKeyPart::Str("a".into()),
                NatsortKeyPart::Int(10),
            ]
        );
    }

    #[test]
    fn full_key_file10_5_txt() {
        // Without FLOAT flag, "5" parses as Int(5), not Float(5.0)
        let raw_parts = vec![
            try_convert_to_number("file"),
            try_convert_to_number("10"),
            try_convert_to_number("."),
            try_convert_to_number("5"),
            try_convert_to_number(".txt"),
        ];
        let result = insert_sentinels(raw_parts);
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
}
