//! Broad parity tests: comprehensive comparison against Python natsort.
//!
//! These tests cover the majority of test cases from the original Python
//! test suite (`../python_src/tests/test_natsorted.py`, etc.).

use natsort::{
    Item, NsFlags, decode_bytes, decode_bytes_ascii, natsorted_bytes, natsorted_bytes_ignorecase,
};
use pyo3::prelude::*;

/// Call Python's `natsort.natsorted()` and extract as Vec<String>.
fn py_natsorted(py: Python<'_>, data: &[&str]) -> Vec<String> {
    let ns = py.import_bound("natsort").unwrap();
    let py_list = pyo3::types::PyList::new_bound(py, data);
    ns.call_method1("natsorted", (py_list,))
        .unwrap()
        .extract::<Vec<String>>()
        .unwrap()
}

/// Call Python's `natsort.natsorted(alg=...)` and extract as Vec<String>.
fn py_natsorted_alg(py: Python<'_>, data: &[&str], flag_name: &str) -> Vec<String> {
    let ns_mod = py.import_bound("natsort").unwrap();
    let ns_enum = ns_mod.getattr("ns").unwrap();
    let flag_val = ns_enum.getattr(flag_name).unwrap();
    let kw = pyo3::types::PyDict::new_bound(py);
    kw.set_item("alg", flag_val).unwrap();
    let py_list = pyo3::types::PyList::new_bound(py, data);
    ns_mod
        .call_method("natsorted", (py_list,), Some(&kw))
        .unwrap()
        .extract::<Vec<String>>()
        .unwrap()
}

// ── Basic sorting ────────────────────────────────────────────

#[test]
fn bp_basic_integers() {
    let data = vec!["4", "8", "2", "10", "3"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_basic_strings() {
    let data = vec!["b", "a", "c"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_numbers_before_text() {
    let data = vec!["b", "2", "a", "1"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_empty_string() {
    let data = vec!["", "a", "1"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_single_element() {
    let data = vec!["hello"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_already_sorted() {
    let data = vec!["1", "2", "3"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_reverse_sort() {
    let data = vec!["file1.txt", "file2.txt", "file10.txt"];
    let rs = natsort::natsorted_rev(&data);
    let mut py = Python::with_gil(|g| py_natsorted(g, &data));
    py.reverse();
    assert_eq!(rs, py);
}

// ── File-style names ─────────────────────────────────────────

#[test]
fn bp_file_names() {
    let data = vec!["file10.txt", "file2.txt", "file1.txt"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_mixed_alphanumeric() {
    let data = vec!["file10.5.txt", "file2.3.txt", "file1.10.txt"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_image_files() {
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
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_numbers_then_letters() {
    let data = vec!["1a", "2a", "10a"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

// ── Signed / REAL mode ───────────────────────────────────────

#[test]
fn bp_signed_minus_five() {
    // Without SIGNED: "-" is text, "5" is number
    let data = vec!["-5", "3", "-1", "10"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_signed_mode() {
    let data = vec!["-5", "+3", "10"];
    let rs = natsort::natsorted_with(&data, NsFlags::SIGNED);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "SIGNED"));
    assert_eq!(rs, py);
}

#[test]
fn bp_realsorted() {
    let data = vec!["1.5", "-3.2", "10.0", "+2.1"];
    let rs = natsort::realsorted(&data);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "REAL"));
    assert_eq!(rs, py);
}

#[test]
fn bp_real_mode_decimals() {
    let data = vec!["1.5", "1.10", "1.2", "2.0"];
    let rs = natsort::natsorted_with(&data, NsFlags::REAL);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "REAL"));
    assert_eq!(rs, py);
}

// ── Scientific notation ──────────────────────────────────────

#[test]
fn bp_sci_notation_no_float() {
    let data = vec!["1e10", "1e2", "100"];
    assert_eq!(
        natsort::natsorted(&data),
        Python::with_gil(|g| py_natsorted(g, &data))
    );
}

#[test]
fn bp_sci_notation_with_float() {
    let data = vec!["1e10", "1e2", "100"];
    let rs = natsort::natsorted_with(&data, NsFlags::FLOAT);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "FLOAT"));
    assert_eq!(rs, py);
}

// ── IGNORECASE ───────────────────────────────────────────────

#[test]
fn bp_ignorecase_basic() {
    let data = vec!["Banana", "apple", "Cherry", "banana", "Apple"];
    let rs = natsort::natsorted_with(&data, NsFlags::IGNORECASE);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "IGNORECASE"));
    assert_eq!(rs, py);
}

// ── GROUPLETTERS ─────────────────────────────────────────────

#[test]
fn bp_groupletters() {
    let data = vec!["Banana", "apple", "banana", "Apple"];
    let rs = natsort::natsorted_with(&data, NsFlags::GROUPLETTERS);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "GROUPLETTERS"));
    assert_eq!(rs, py);
}

// ── LOWERCASEFIRST ───────────────────────────────────────────

#[test]
fn bp_lowercasefirst() {
    let data = vec!["Banana", "apple", "banana", "Apple"];
    let rs = natsort::natsorted_with(&data, NsFlags::LOWERCASEFIRST);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "LOWERCASEFIRST"));
    assert_eq!(rs, py);
}

// ── NUMAFTER ─────────────────────────────────────────────────

#[test]
fn bp_numafter_basic() {
    let data = vec!["b", "2", "a", "1"];
    let rs = natsort::natsorted_with(&data, NsFlags::NUMAFTER);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "NUMAFTER"));
    assert_eq!(rs, py);
}

// ── PRESORT ──────────────────────────────────────────────────

#[test]
fn bp_presort_tiebreak() {
    let data = vec!["a1", "a01", "a2"];
    let rs = natsort::natsorted_with(&data, NsFlags::PRESORT);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "PRESORT"));
    assert_eq!(rs, py);
}

// ── PATH mode ────────────────────────────────────────────────

#[test]
fn bp_path_mode() {
    let data = vec!["Folder/", "Folder (1)/", "Folder (10)/"];
    let rs = natsort::natsorted_with(&data, NsFlags::PATH);
    let py = Python::with_gil(|g| py_natsorted_alg(g, &data, "PATH"));
    assert_eq!(rs, py);
}

// ── OS sorted ────────────────────────────────────────────────

#[test]
fn bp_os_sorted_basic() {
    let data = vec!["/dir/file10.txt", "/dir/file2.txt", "/dir/file1.txt"];
    let rs = natsort::os_sorted(&data);
    assert_eq!(
        rs,
        vec!["/dir/file1.txt", "/dir/file2.txt", "/dir/file10.txt"]
    );
}

#[test]
fn bp_os_sorted_extensions() {
    let data = vec!["file(10).txt", "file(2).txt", "file(1).txt"];
    let rs = natsort::os_sorted(&data);
    assert_eq!(rs, vec!["file(1).txt", "file(2).txt", "file(10).txt"]);
}

#[test]
fn bp_os_sorted_multiple_ext() {
    let data = vec!["file.tar.gz", "file2.tar.gz"];
    let rs = natsort::os_sorted(&data);
    assert_eq!(rs, vec!["file.tar.gz", "file2.tar.gz"]);
}

// ── Mixed types ──────────────────────────────────────────────

#[test]
fn bp_mixed_types() {
    let data = vec![
        Item::Int(10),
        Item::parse_item("2"),
        Item::Float(3.5),
        Item::Str("apple".to_string()),
    ];
    let rs = natsort::natsorted_mixed(&data);
    assert_eq!(
        rs,
        vec![
            Item::Int(2),
            Item::Float(3.5),
            Item::Int(10),
            Item::Str("apple".to_string()),
        ]
    );
}

#[test]
fn bp_mixed_none_first() {
    let data = vec![
        Item::Str("b".to_string()),
        Item::NoneVal,
        Item::Int(3),
        Item::NoneVal,
        Item::Str("a".to_string()),
    ];
    let rs = natsort::natsorted_mixed(&data);
    assert_eq!(
        rs,
        vec![
            Item::NoneVal,
            Item::NoneVal,
            Item::Int(3),
            Item::Str("a".to_string()),
            Item::Str("b".to_string()),
        ]
    );
}

#[test]
fn bp_mixed_nan_default() {
    let data = vec![Item::Float(1.0), Item::Float(f64::NAN), Item::Float(2.0)];
    let rs = natsort::natsorted_mixed(&data);
    assert!(rs.first().unwrap().is_nan());
}

#[test]
fn bp_mixed_nan_last() {
    let data = vec![Item::Float(1.0), Item::Float(f64::NAN), Item::Float(2.0)];
    let rs = natsort::natsorted_mixed_with(&data, NsFlags::NANLAST);
    assert!(rs.last().unwrap().is_nan());
}

#[test]
fn bp_mixed_bool_equivalent() {
    let data = vec![Item::Int(1), Item::Int(0), Item::Str("a".to_string())];
    let rs = natsort::natsorted_mixed(&data);
    assert_eq!(
        rs,
        vec![Item::Int(0), Item::Int(1), Item::Str("a".to_string())]
    );
}

// ── Bytes ────────────────────────────────────────────────────

#[test]
fn bp_bytes_basic() {
    let data = vec![b"z".as_slice(), b"a".as_slice(), b"m".as_slice()];
    let rs = natsorted_bytes(&data);
    assert_eq!(rs, vec![b"a".to_vec(), b"m".to_vec(), b"z".to_vec()]);
}

#[test]
fn bp_bytes_ignorecase() {
    let data = vec![b"B".as_slice(), b"a".as_slice(), b"A".as_slice()];
    let rs = natsorted_bytes_ignorecase(&data);
    assert_eq!(rs, vec![b"a".to_vec(), b"A".to_vec(), b"B".to_vec()]);
}

#[test]
fn bp_decode_bytes_valid() {
    assert_eq!(decode_bytes(b"hello"), Ok("hello".to_string()));
}

#[test]
fn bp_decode_bytes_invalid() {
    assert!(decode_bytes(&[0xff, 0xfe]).is_err());
}

#[test]
fn bp_decode_bytes_ascii_replacement() {
    assert_eq!(decode_bytes_ascii(&[65, 255, 97]), "A?a");
}
