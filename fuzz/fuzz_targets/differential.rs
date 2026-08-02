#![no_main]

//! Differential fuzz target: feed the same input to Rust `natsort` and to the
//! original Python `natsort` (via pyo3) and assert identical output.
//!
//! This catches behavioral-parity regressions, not just panics.
//!
//! # Requirements
//!
//! The Python interpreter must have `natsort` importable, e.g. the sibling
//! virtualenv: `source ../python_src/.venv/bin/activate` then
//! `cargo fuzz run differential`. If Python/natsort is unavailable the target
//! aborts loudly rather than producing false parity results.

use libfuzzer_sys::fuzz_target;
use pyo3::prelude::*;

use natsort::{natsorted_with, NsFlags};

// Flag sets that exercise distinct paths and map to comparable Python ns
// constants. bitflags prevents opaque or-constants in `const`, so build at runtime.
fn diff_flags() -> Vec<(NsFlags, &'static str)> {
    vec![
        (NsFlags::INT, "DEFAULT"),
        (NsFlags::REAL, "REAL"),
        (NsFlags::IGNORECASE, "IGNORECASE"),
        (NsFlags::PATH, "PATH"),
    ]
}

/// Returns `true` if any item contains a codepoint in Python's non-decimal
/// `digit` / `numeric` sets, which Rust deliberately treats as text.
///
/// Queries `natsort.unicode_numbers` so the skip list stays in sync with the
/// Python version being compared against.
fn has_divergent_unicode(py: Python<'_>, items: &[String]) -> bool {
    use std::collections::HashSet;

    let Ok(module) = py.import_bound("natsort.unicode_numbers") else {
        return false;
    };
    let mut divergent: HashSet<char> = HashSet::new();
    for name in ["digits_no_decimals", "numeric_no_decimals"] {
        let Ok(chars) = module.getattr(name).and_then(|c| c.extract::<String>()) else {
            continue;
        };
        divergent.extend(chars.chars());
    }
    items.iter().any(|s| s.chars().any(|ch| divergent.contains(&ch)))
}

fn rust_order(items: &[String], flags: NsFlags) -> Vec<String> {
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    natsorted_with(&refs, flags)
}

fn python_order(py: Python<'_>, items: &[String], alg: &str) -> PyResult<Vec<String>> {
    let module = py.import_bound("natsort")?;
    let ns = module.getattr("ns")?;
    let encoded = ns.getattr(alg)?;
    let kwargs = pyo3::types::PyDict::new_bound(py);
    kwargs.set_item("alg", encoded)?;
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let out = module.call_method("natsorted", (refs,), Some(&kwargs))?;
    out.extract::<Vec<String>>()
}

fuzz_target!(|data: &[u8]| {
    let items: Vec<String> = data
        .split(|&b| b == b',')
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect();

    Python::with_gil(|py| {
        // Known, deliberate parity divergence (see DECISIONS.md §1): Rust
        // treats non-decimal `digit`/`numeric` codepoints (superscripts,
        // circled digits, `½`, …) as text, but Python classifies them as
        // numbers. Skip any input containing such a character so the target
        // hunts for genuine regressions instead of aborting on the documented
        // gap.
        if has_divergent_unicode(py, &items) {
            return;
        }

        for &(flags, alg) in &diff_flags() {
            let rust = rust_order(&items, flags);
            let python = python_order(py, &items, alg)
                .unwrap_or_else(|err| {
                    eprintln!(
                        "Python natsort unavailable: {err}\n\
                         hint: `source ../python_src/.venv/bin/activate` before `cargo fuzz run differential`"
                    );
                    std::process::abort();
                });
            if rust != python {
                eprintln!(
                    "PARITY MISMATCH\n  alg:    {alg}\n  items:  {:?}\n  rust:   {:?}\n  python: {python:?}",
                    items, rust
                );
                std::process::abort();
            }
        }
    });
});