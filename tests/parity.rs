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
