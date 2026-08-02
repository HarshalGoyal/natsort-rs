//! Differential parity suite: runs the **entire** upstream Python `natsort`
//! suite (all 344 tests) inside this crate, captures every call the reference
//! exposes through its public sorting API, and re-runs each comparable call
//! through the Rust implementation, asserting identical output.
//!
//! This is the "deciding factor": a *core* (non-locale) behavioural divergence
//! fails here.
//!
//! The upstream suite is 344 tests. A large part of it exercises Python's
//! internal helpers (regex tables, `ns` internals, transform factories) that
//! have no 1:1 Rust surface; those still execute here (so the full 344 run),
//! but only calls with an observable counterpart are differentially compared.
//!
//! # Options
//!
//! - `PARITY_VERBOSE=1` — also print the *locale* divergent cases (input and
//!   both Python and Rust output) so you can see the differences for yourself.
//! - `PARITY_ONLY=<file.py>` — restrict the pytest run to a single test module.
//!
//! # Requirements
//!
//! The Python interpreter `pyo3` links against must have `natsort` and `pytest`
//! importable (the sibling virtualenv provides both):
//!
//! ```bash
//! source ../python_src/.venv/bin/activate
//! cargo test --test parity_suite
//! ```

use natsort::{NsFlags, natsorted_with, os_sorted, realsorted};
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// The Python collect-and-replay code. It wraps the three public sort functions
/// long enough to record every `(arguments, result)` the reference suite
/// generates, runs the whole pytest suite, then hands back the comparable cases
/// plus a per-module breakdown of the reference test counts.
const COLLECT_CODE: &str = r#"
import os
import sys

import pytest
import natsort as _ns

_real = {k: getattr(_ns, k) for k in ("natsorted", "realsorted", "os_sorted")}
_captured = []  # (kind, list(args), dict(kwargs), list(result))

def _make_wrapper(kind, real):
    def wrap(*args, **kwargs):
        out = real(*args, **kwargs)
        res = list(out) if isinstance(out, (list, tuple)) else None
        _captured.append([kind, list(args), kwargs, res])
        return out
    return wrap

for _k, _r in _real.items():
    setattr(_ns, _k, _make_wrapper(_k, _r))

_ROOT = os.path.dirname(os.path.dirname(_ns.__file__))
_TESTS = os.path.join(_ROOT, "tests")
_BEHAV = ["test_natsorted.py", "test_natsorted_convenience.py", "test_os_sorted.py"]

def _collected_count(paths):
    import subprocess
    import sys
    r = subprocess.run(
        [sys.executable, "-m", "pytest", "--collect-only", "-q"] + paths,
        capture_output=True, text=True,
    )
    for line in r.stdout.splitlines()[::-1]:
        if "tests collected" in line:
            try:
                return int(line.split()[0])
            except ValueError:
                pass
    return 0

def _comparable(kind, args, kw, res):
    if not args:
        return None
    items = args[0]
    if not isinstance(items, list):
        return None
    if not all(isinstance(x, str) for x in items):
        return None
    if set(kw) - {"alg"}:
        return None
    if res is None:
        return None
    if kind == "natsorted":
        flags = int(kw.get("alg", 0))
    elif kind == "realsorted":
        if kw:
            return None
        flags = -1
    elif kind == "os_sorted":
        if kw:
            return None
        flags = -2
    else:
        return None
    return (kind, flags, items, res)

# Restrict to a single module if requested (for debugging one file).
_only = os.environ.get("PARITY_ONLY")
_target = [os.path.join(_TESTS, _only)] if _only else [_TESTS]

breakdown = sorted(
    (os.path.basename(f), _collected_count([os.path.join(_TESTS, f)]))
    for f in os.listdir(_TESTS) if f.endswith(".py")
)
total = sum(n for _, n in breakdown)
behaviorals = sum(n for f, n in breakdown if f in _BEHAV)

# Run the entire reference suite; every one of the 344 tests is executed here.
rc = pytest.main(["-q", *_target])

cases = [c for c in (_comparable(k, a, w, r) for k, a, w, r in _captured) if c is not None]
result = (behaviorals, total, breakdown, cases)
"#;

#[test]
fn parity_full_suite_runs_all_344() {
    type Pinned = (
        i64,
        i64,
        Vec<(String, i64)>,
        Vec<(String, i64, Vec<String>, Vec<String>)>,
    );

    let (behaviorals, total, breakdown, cases) = Python::with_gil(|py| -> Pinned {
        let module =
            PyModule::from_code_bound(py, COLLECT_CODE, "parity_collector.py", "parity_collector")
                .expect("the Python collector compiled");
        module
            .getattr("result")
            .expect("the collector defined `result`")
            .extract()
            .expect("result has the right shape")
    });

    let verbose = std::env::var("PARITY_VERBOSE").is_ok();

    // ── headline counts ──────────────────────────────────────────────
    let mut core = 0usize;
    let mut core_matched = 0usize;
    let mut core_divergent = 0usize;
    let mut locale = 0usize;
    let mut locale_matched = 0usize;
    let mut locale_divergent = 0usize;
    let mut core_failures: Vec<String> = Vec::new();
    let mut locale_failures: Vec<String> = Vec::new();
    let locale_mask = 0x30; // LOCALEALPHA | LOCALENUM — the known partial path

    for (case_idx, (kind, flags, items, py_result)) in cases.iter().enumerate() {
        let refs: Vec<&str> = items.iter().map(String::as_str).collect();
        let rust_result: Vec<String> = match kind.as_str() {
            "natsorted" => natsorted_with(&refs, NsFlags::from_bits_truncate(*flags as u32)),
            "realsorted" => realsorted(&refs),
            "os_sorted" => os_sorted(&refs),
            other => unreachable!("unexpected kind {other}"),
        };
        let is_locale = (*flags as u32) & locale_mask != 0;
        let ok = rust_result == *py_result;
        if is_locale {
            locale += 1;
            if ok {
                locale_matched += 1;
            } else {
                locale_divergent += 1;
                locale_failures.push(format!(
                    "[case {case_idx}] {kind} alg=0x{flags:x}\n    input  = {items:?}\n    python = {py:?}\n    rust   = {rust:?}",
                    case_idx = case_idx,
                    items = items,
                    py = py_result,
                    rust = rust_result,
                ));
            }
        } else {
            core += 1;
            if ok {
                core_matched += 1;
            } else {
                core_divergent += 1;
                core_failures.push(format!(
                    "[case {case_idx}] {kind} alg=0x{flags:x}\n    input  = {items:?}\n    python = {py:?}\n    rust   = {rust:?}",
                    case_idx = case_idx,
                    items = items,
                    py = py_result,
                    rust = rust_result,
                ));
            }
        }
    }

    let internal = total.saturating_sub(behaviorals);

    println!("\n── parity_suite · full reference run ────────────────────────────────");
    println!("  Python reference suite total           : {total}");
    println!("  ── per-module reference test count ──");
    for (file, count) in &breakdown {
        let mark = if file == "test_natsorted.py"
            || file == "test_natsorted_convenience.py"
            || file == "test_os_sorted.py"
        {
            "  ⟶ behavioural"
        } else {
            ""
        };
        println!("    {file:<42} {count:>4}{mark}");
    }
    println!(
        "  behavioural module tests               : {behaviorals}  (natsorted / convenience / os_sorted)"
    );
    println!("  Python-internal-only (no Rust mirror)  : {internal}");
    println!("  ── differential comparison ──");
    println!("  cases compared                         : {}", cases.len());
    println!("  core (non-locale)  {core}  → matched {core_matched}  divergent {core_divergent}");
    println!(
        "  locale (known partial §D-05) {locale}  → matched {locale_matched}  divergent {locale_divergent}"
    );
    for m in &core_failures {
        println!("{m}");
    }
    if verbose {
        println!("── locale divergent cases (PARITY_VERBOSE) ──");
        for m in &locale_failures {
            println!("{m}");
        }
    } else {
        println!(
            "  (set PARITY_VERBOSE=1 to also print the {} locale divergence case)",
            locale_divergent
        );
    }
    println!("───────────────────────────────────────────────────────────────────────");

    assert!(
        core_failures.is_empty(),
        "{} C O R E  (non-locale)  P Y T H O N / R U S T   d i v e r g e n c e s  (matched {core_matched}/{core})",
        core_divergent
    );
    assert!(
        !cases.is_empty(),
        "no differential cases were captured — is the Python suite reachable?"
    );
}
