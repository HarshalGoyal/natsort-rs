//! Fastnumbers compatibility for natsort-rs
//!
//! This module provides compatibility with Python's fastnumbers library behavior
//! for numeric parsing.

use std::str::FromStr;

/// Parse a string as a number with fastnumbers-like behavior
pub fn fast_float(s: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // This is a simplified version - real fastnumbers has more complex logic
    // For now, we'll just use standard parsing
    Ok(s.parse::<f64>()?)
}

/// Parse a string as an integer with fastnumbers-like behavior
pub fn fast_int(s: &str) -> Result<i64, Box<dyn std::error::Error>> {
    // This is a simplified version - real fastnumbers has more complex logic
    // For now, we'll just use standard parsing
    Ok(s.parse::<i64>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_float() {
        assert_eq!(fast_float("1.5").unwrap(), 1.5);
        assert_eq!(fast_float("-3.2").unwrap(), -3.2);
    }

    #[test]
    fn test_fast_int() {
        assert_eq!(fast_int("10").unwrap(), 10);
        assert_eq!(fast_int("-5").unwrap(), -5);
    }
}