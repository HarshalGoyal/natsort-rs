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

use natsort::{NsFlags, natsorted, natsorted_with, realsorted};
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

    assert_eq!(
        rs_result, py_result,
        "Rust realsorted output differs from Python"
    );
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

    assert_eq!(
        rs_result, py_result,
        "IGNORECASE output differs from Python"
    );
}

/// Larger dataset to stress-test ordering stability.
#[test]
fn parity_larger_dataset() {
    let data = vec![
        "img12.png",
        "img10.png",
        "img2.png",
        "img1.png",
        "img20.png",
        "img19.png",
        "original_img.png",
        "image5.jpg",
    ];
    let rs_result = natsorted(&data);

    let py_result: Vec<String> = Python::with_gil(|py| {
        py_natsort(py)
            .call_method1("natsorted", (data.clone(),))
            .expect("natsorted accepts a list of str")
            .extract()
            .expect("natsorted returns a list of str")
    });

    assert_eq!(
        rs_result, py_result,
        "Larger dataset output differs from Python"
    );
}

// ── Phase 2 parity tests ───────────────────────────────────────

/// Mixed types: [10, "2", 3.5, "apple"] → ["2", 3.5, 10, "apple"].
#[test]
fn parity_mixed_types() {
    let data = vec![
        natsort::Item::Int(10),
        natsort::Item::Str("2".to_string()),
        natsort::Item::Float(3.5),
        natsort::Item::Str("apple".to_string()),
    ];
    let rs_result = natsort::natsorted_mixed(&data);

    // Python order: numbers before strings, then by value.
    assert_eq!(rs_result[0], natsort::Item::Str("2".to_string()));
    assert!(matches!(&rs_result[1], natsort::Item::Float(f) if (*f - 3.5).abs() < f64::EPSILON));
    assert_eq!(rs_result[2], natsort::Item::Int(10));
    assert_eq!(rs_result[3], natsort::Item::Str("apple".to_string()));
}

/// None values sort first.
#[test]
fn parity_none_first() {
    let data = vec![
        natsort::Item::Str("b".to_string()),
        natsort::Item::NoneVal,
        natsort::Item::Int(3),
        natsort::Item::NoneVal,
        natsort::Item::Str("a".to_string()),
    ];
    let rs_result = natsort::natsorted_mixed(&data);

    // First two should be None values
    assert_eq!(rs_result[0], natsort::Item::NoneVal);
    assert_eq!(rs_result[1], natsort::Item::NoneVal);
}

/// NaN handling: default puts NaN first, NANLAST puts it last.
#[test]
fn parity_nan_handling() {
    let data_default = vec![
        natsort::Item::Float(1.0),
        natsort::Item::Float(f64::NAN),
        natsort::Item::Float(2.0),
    ];
    let result_default = natsort::natsorted_mixed(&data_default);
    // Default: NaN → -inf, sorts first.
    assert!(result_default.first().unwrap().is_nan());

    let result_last = natsort::natsorted_mixed_with(&data_default, NsFlags::NANLAST);
    // NANLAST: NaN → +inf, sorts last.
    assert!(result_last.last().unwrap().is_nan());
}

/// NUMAFTER: numbers after letters.
#[test]
fn parity_numafter() {
    let data = vec!["b", "2", "a", "1"];
    let rs_result = natsorted_with(&data, NsFlags::NUMAFTER);

    let py_result: Vec<String> = Python::with_gil(|py| {
        let natsort_mod = py_natsort(py);
        let ns = natsort_mod.getattr("ns").expect("ns exists");
        let na = ns.getattr("NUMAFTER").expect("NUMAFTER exists");
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("alg", na).unwrap();
        natsort_mod
            .call_method("natsorted", (data.clone(),), Some(&kwargs))
            .expect("natsorted with NUMAFTER works")
            .extract()
            .expect("returns list of str")
    });

    assert_eq!(rs_result, py_result, "NUMAFTER output differs from Python");
}

/// PRESORT: breaks ties by string value for stable sort.
#[test]
fn parity_presort() {
    let data = vec!["a1", "a01", "a2"];
    let rs_result = natsorted_with(&data, NsFlags::PRESORT);

    let py_result: Vec<String> = Python::with_gil(|py| {
        let natsort_mod = py_natsort(py);
        let ns = natsort_mod.getattr("ns").expect("ns exists");
        let ps = ns.getattr("PRESORT").expect("PRESORT exists");
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("alg", ps).unwrap();
        natsort_mod
            .call_method("natsorted", (data.clone(),), Some(&kwargs))
            .expect("natsorted with PRESORT works")
            .extract()
            .expect("returns list of str")
    });

    assert_eq!(rs_result, py_result, "PRESORT output differs from Python");
}

/// GROUPLETTERS: groups uppercase and lowercase together.
#[test]
fn parity_groupletters() {
    let data = vec!["Banana", "apple", "banana", "Apple"];
    let rs_result = natsorted_with(&data, NsFlags::GROUPLETTERS);

    let py_result: Vec<String> = Python::with_gil(|py| {
        let natsort_mod = py_natsort(py);
        let ns = natsort_mod.getattr("ns").expect("ns exists");
        let gl = ns.getattr("GROUPLETTERS").expect("GROUPLETTERS exists");
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("alg", gl).unwrap();
        natsort_mod
            .call_method("natsorted", (data.clone(),), Some(&kwargs))
            .expect("natsorted with GROUPLETTERS works")
            .extract()
            .expect("returns list of str")
    });

    assert_eq!(
        rs_result, py_result,
        "GROUPLETTERS output differs from Python"
    );
}

/// LOWERCASEFIRST: lowercase before uppercase.
#[test]
fn parity_lowercasefirst() {
    let data = vec!["Banana", "apple", "banana", "Apple"];
    let rs_result = natsorted_with(&data, NsFlags::LOWERCASEFIRST);

    let py_result: Vec<String> = Python::with_gil(|py| {
        let natsort_mod = py_natsort(py);
        let ns = natsort_mod.getattr("ns").expect("ns exists");
        let lf = ns.getattr("LOWERCASEFIRST").expect("LOWERCASEFIRST exists");
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("alg", lf).unwrap();
        natsort_mod
            .call_method("natsorted", (data.clone(),), Some(&kwargs))
            .expect("natsorted with LOWERCASEFIRST works")
            .extract()
            .expect("returns list of str")
    });

    assert_eq!(
        rs_result, py_result,
        "LOWERCASEFIRST output differs from Python"
    );
}

/// Recursive descent: sorting nested lists.
#[test]
fn parity_recursive_descent() {
    let data = vec![
        natsort::NestedItem::branch(vec![
            natsort::NestedItem::int(1),
            natsort::NestedItem::str_("b"),
        ]),
        natsort::NestedItem::branch(vec![
            natsort::NestedItem::int(1),
            natsort::NestedItem::str_("a"),
        ]),
        natsort::NestedItem::branch(vec![
            natsort::NestedItem::int(2),
            natsort::NestedItem::str_("a"),
        ]),
    ];
    let rs_result = natsort::natsorted_recursive(&data);

    // Verify order: [1,"a"] < [1,"b"] < [2,"a"]
    let first = &rs_result[0];
    if let natsort::NestedItem::Branch(children) = first {
        assert_eq!(children.len(), 2);
        if let natsort::NestedItem::Leaf(natsort::Item::Str(s)) = &children[1] {
            assert_eq!(s, "a");
        } else {
            panic!("Expected second element of first branch to be 'a'");
        }
    } else {
        panic!("Expected first element to be a Branch");
    }
}

/// OS path sorting with extension handling.
#[test]
fn parity_os_sorted_extensions() {
    let data = vec!["file(10).txt", "file(2).txt", "file(1).txt"];
    let rs_result = natsort::os_sorted(&data);

    // Should sort by number in parentheses: 1, 2, 10
    assert_eq!(
        rs_result,
        vec!["file(1).txt", "file(2).txt", "file(10).txt",]
    );
}
