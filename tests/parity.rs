//! Parity harness: bridges to the original Python `natsort` via `pyo3`.
//!
//! Every feature ported in later phases gets a test here that runs the *same*
//! input through Python's `natsort` and through this crate, then asserts the
//! outputs are identical. This file is the executable specification.
//!
//! # Requirements
//!
//! The Python interpreter that `pyo3` links against must have `natsort`
//! installed. In this workspace that is the sibling virtualenv:
//!
//! ```bash
//! source ../python_src/.venv/bin/activate
//! cargo test --test parity
//! ```
//!
//! When the interpreter cannot import `natsort`, the tests fail loudly with a
//! actionable message rather than silently passing.

use natsort::{natsorted, natsorted_with, realsorted, NsFlags};
use pyo3::prelude::*;

/// Imports the Python `natsort` module, with a diagnostic on failure.
fn py_natsort(py: Python<'_>) -> Bound<'_, PyModule> {
    match py.import_bound("natsort") {
        Ok(module) => module,
        Err(err) => {
            let sys = py.import_bound("sys").expect("sys is always importable");
            let executable: String = sys
                .getattr("executable")
                .and_then(|e| e.extract())
                .unwrap_or_else(|_| "<unknown>".to_string());
            panic!(
                "failed to import the Python `natsort` module.\n\
                 interpreter: {executable}\n\
                 hint: `source ../python_src/.venv/bin/activate` (with natsort installed via `pip install -e .`) before running `cargo test`.\n\
                 original error: {err}"
            );
        }
    }
}

/// Phase 0 smoke test: the bridge can reach the reference implementation.
#[test]
fn test_pyimport_works() {
    Python::with_gil(|py| {
        let natsort = py_natsort(py);
        let version: String = natsort
            .getattr("__version__")
            .expect("natsort exposes __version__")
            .extract()
            .expect("__version__ is a string");
        println!("reference natsort version: {version}");
        assert!(
            !version.is_empty(),
            "reference natsort reported an empty version"
        );
    });
}

/// Sanity-check that the reference implementation behaves as the spec describes,
/// so a broken bridge is distinguishable from a broken port.
#[test]
fn test_reference_natsorted_is_sane() {
    let input = vec!["file10.txt", "file2.txt", "file1.txt"];
    let expected = vec!["file1.txt", "file2.txt", "file10.txt"];

    let actual: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (input.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(actual, expected);
}

// ── Phase 1 parity tests ─────────────────────────────────────────────

/// Basic integer sorting: ["4", "8", "2", "10", "3"] → ["2", "3", "4", "8", "10"].
#[test]
fn parity_basic_integers() {
    let data = vec!["4", "8", "2", "10", "3"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// File-style names with embedded numbers.
#[test]
fn parity_file_names() {
    let data = vec!["file10.txt", "file2.txt", "file1.txt"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// Numbers sort before text.
#[test]
fn parity_numbers_before_text() {
    let data = vec!["b", "2", "a", "1"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// Mixed alphanumeric strings like "file10.5.txt".
#[test]
fn parity_mixed_alphanumeric() {
    let data = vec!["file10.5.txt", "file2.3.txt", "file1.10.txt"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// Empty string handling.
#[test]
fn parity_empty_and_single() {
    let data = vec!["", "a", "1"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// Negative numbers without ns.SIGNED: "-" is treated as text.
#[test]
fn parity_negative_without_signed() {
    let data = vec!["-5", "3", "-1", "10"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// Scientific notation without FLOAT flag: "1e10" splits into "1", "e", "10".
#[test]
fn parity_scientific_notation_no_float() {
    let data = vec!["1e10", "1e2", "100"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// Decimal numbers without FLOAT flag: "1.5" splits into "1", ".", "5".
#[test]
fn parity_decimals_no_float() {
    let data = vec!["1.5", "1.10", "1.2"];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust output differs from Python");
}

/// Realsorted (ns.REAL): signed floats sorted numerically.
#[test]
fn parity_realsorted() {
    let data = vec!["1.5", "-3.2", "10.0", "+2.1"];
    let rs_result = realsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        let natsort = py_natsort(py);
        natsort
            .call_method1("realsorted", (data.clone(),))
            .expect("realsorted accepts a list of str")
            .extract()
            .expect("realsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Rust realsorted output differs from Python");
}

/// Case-insensitive sorting.
#[test]
fn parity_ignorecase() {
    let data = vec!["Banana", "apple", "Cherry", "banana", "Apple"];
    let rs_result = natsorted_with(&data, NsFlags::IGNORECASE);

    let py_result: Vec<String> = Python::with_gil(|py| {
        let natsort = py_natsort(py);
        let ns = natsort.getattr("ns").expect("ns enum exists");
        let ic = ns.getattr("IGNORECASE").expect("IGNORECASE exists");
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("alg", ic).unwrap();
        natsort
            .call_method("natsorted", (data.clone(),), Some(&kwargs))
            .expect("natsorted with IGNORECASE works")
            .extract()
            .expect("returns list of str")
    });

    assert_eq!(rs_result, py_result, "IGNORECASE output differs from Python");
}

/// Larger dataset to stress-test ordering stability.
#[test]
fn parity_larger_dataset() {
    let data = vec![
        "img12.png", "img10.png", "img2.png", "img1.png", "img20.png",
        "img19.png", "original_img.png", "image5.jpg",
    ];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(rs_result, py_result, "Larger dataset output differs from Python");
}
