//! Error types for `natsort-rs`.
//!
//! The Python library raises `ValueError`/`TypeError` for a small number of
//! invalid-configuration cases. Those become typed variants here so that no
//! library code needs to panic.

use thiserror::Error;

/// Convenience alias for results produced by this crate.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors produced by `natsort-rs`.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A combination of [`crate::ns`] flags that the algorithm cannot honour.
    ///
    /// Mirrors the Python `ValueError` raised by `natsort_keygen` when, for
    /// example, `ns.LOCALE` is combined with an incompatible option.
    #[error("invalid combination of ns flags: {0}")]
    InvalidFlags(String),

    /// Input bytes were not valid UTF-8 and could not be decoded.
    ///
    /// Python operates on `bytes` directly; Rust needs a decoding step, so
    /// undecodable input is surfaced instead of silently replaced.
    #[error("input was not valid UTF-8 at byte {position}")]
    InvalidUtf8 {
        /// Byte offset of the first invalid sequence.
        position: usize,
    },
}
