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
//! Phase 0: scaffolding. The public API lands in Phase 1.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;

pub use error::{Error, Result};
