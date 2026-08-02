//! Criterion benchmark: measures the Rust `natsort` algorithms on realistic
//! datasets, and *also* measures the reference Python `natsort` on the very
//! same inputs so the reported speedups are measured here, not estimated.
//!
//! Run with the venv active so pyo3 can import `natsort`:
//!
//! ```bash
//! source ../python_src/.venv/bin/activate
//! cargo bench --bench natsort_bench
//! ```

use std::time::Duration;

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use natsort::{NsFlags, natsorted_with, os_sorted, realsorted};
use pyo3::prelude::*;
use pyo3::types::PyList;

/// Build a set of datasets, each `(name, items)`.
fn datasets() -> Vec<(&'static str, Vec<String>)> {
    use fastrand::Rng;
    let mut rng = Rng::new();

    // file names with embedded ints
    let files = (0..20_000)
        .map(|_| {
            let prefix = match rng.u16(0..3) {
                0 => "file",
                1 => "img",
                _ => "item",
            };
            format!("{prefix}{:03}.txt", rng.u16(1..5000))
        })
        .collect::<Vec<_>>();

    // signed floats
    let floats = (0..10_000)
        .map(|i| {
            let sign = if i % 3 == 0 { "-" } else { "" };
            format!("{sign}{}.{}", i / 3, i % 10)
        })
        .collect::<Vec<_>>();

    // nested paths
    let paths = (0..10_000)
        .map(|i| format!("/var/log/service{}/logfile{}.gz", i % 50, i))
        .collect::<Vec<_>>();

    vec![("files", files), ("floats", floats), ("paths", paths)]
}

fn to_refs(data: &[String]) -> Vec<&str> {
    data.iter().map(|s| s.as_str()).collect()
}

// ── Rust-only benches ──────────────────────────────────────────────

fn bench_rust_algorithms(c: &mut Criterion) {
    let flags: &[(&str, NsFlags)] = &[
        ("default", NsFlags::DEFAULT),
        ("real", NsFlags::REAL),
        ("ignorecase", NsFlags::IGNORECASE),
        ("path", NsFlags::PATH),
    ];

    for (dname, data) in datasets() {
        let refs = to_refs(&data);
        for (fname, fl) in flags {
            c.bench_function(&format!("rust/{dname}/natsorted_{fname}"), |b| {
                b.iter_batched(
                    || refs.clone(),
                    |r| black_box(natsorted_with(&r, *fl)),
                    BatchSize::SmallInput,
                )
            });
        }
    }
    for (dname, data) in datasets() {
        let refs = to_refs(&data);
        c.bench_function(&format!("rust/{dname}/realsorted"), |b| {
            b.iter_batched(
                || refs.clone(),
                |r| black_box(realsorted(&r)),
                BatchSize::SmallInput,
            )
        });
        if dname == "paths" {
            c.bench_function("rust/paths/os_sorted", |b| {
                b.iter_batched(
                    || refs.clone(),
                    |r| black_box(os_sorted(&r)),
                    BatchSize::SmallInput,
                )
            });
        }
    }

    // Startup overhead: sort 1 element to measure process init + alloc.
    c.bench_function("rust/startup", |b| {
        b.iter(|| black_box(natsorted_with(&["a"], NsFlags::DEFAULT)))
    });
}

// ── Python·reference bench ─────────────────────────────────────────

/// One pass through Python `natsorted(alg=<flag>)` for a full dataset.
fn python_natsorted(py: Python<'_>, natsort: &Bound<'_, PyAny>, data: &[&str], alg: i64) -> i64 {
    let py_list = PyList::new_bound(py, data);
    let kwargs = pyo3::types::PyDict::new_bound(py);
    kwargs.set_item("alg", alg).unwrap();
    let items: Vec<String> = natsort
        .call_method("natsorted", (py_list,), Some(&kwargs))
        .unwrap()
        .extract()
        .unwrap();
    items.len() as i64
}

fn bench_python_reference(c: &mut Criterion) {
    let data = datasets();

    for (dname, data) in &data {
        let refs = to_refs(data);
        c.bench_function(&format!("python/{dname}/natsorted_default"), |b| {
            b.iter(|| {
                Python::with_gil(|py| {
                    let natsort = py.import_bound("natsort").expect("natsort import");
                    python_natsorted(py, &natsort, &refs, 0)
                })
            })
        });
    }

    // Python's os_sorted has no flag-based natsorted equivalent; benchmark it
    // directly on the nested-path dataset to mirror rust/paths/os_sorted.
    let paths = &data[2].1;
    let refs = to_refs(paths);
    c.bench_function("python/paths/os_sorted", |b| {
        b.iter(|| {
            Python::with_gil(|py| {
                let py_list = PyList::new_bound(py, refs.iter().copied());
                let items: Vec<String> = py
                    .import_bound("natsort")
                    .expect("natsort import")
                    .call_method1("os_sorted", (py_list,))
                    .unwrap()
                    .extract()
                    .unwrap();
                items.len()
            })
        })
    });
}

criterion_group! {
    name = rust_benches;
    config = rust_config();
    targets = bench_rust_algorithms
}
criterion_group! {
    name = python_benches;
    config = python_config();
    targets = bench_python_reference
}
criterion_main!(rust_benches, python_benches);

fn rust_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(400))
        .measurement_time(Duration::from_secs(10))
        .sample_size(100)
}

fn python_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(400))
        .measurement_time(Duration::from_secs(60))
        .sample_size(100)
}
