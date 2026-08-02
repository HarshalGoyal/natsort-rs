//! `parity_diff` — run an arbitrary list of strings through **both** the
//! reference Python `natsort` and this Rust port, and print the results side
//! by side so you can see exactly where they differ.
//!
//! ```bash
//! # (needs the venv with `natsort` installed — see README)
//! source ../python_src/.venv/bin/activate
//!
//! cargo run --release --example parity_diff -- file10.txt file2.txt file1.txt
//! cargo run --release --example parity_diff --alg LOCALE -- Apple apple banana
//! echo -e "a2\na10\na1" | cargo run --release --example parity_diff --alg REAL --
//! ```
//!
//! `--alg` accepts any `natsort.ns` member name (DEFAULT, INT, FLOAT, SIGNED,
//! NOEXP, REAL, PATH, IGNORECASE, LOCALE, ...). With no positional items,
//! input is read from stdin (one item per line).

use natsort::{natsorted_with, NsFlags};
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::io::{self, BufRead};

struct Args {
    alg: String,
    items: Vec<String>,
}

fn parse_args() -> Args {
    let mut alg = "DEFAULT".to_string();
    let mut items = Vec::new();
    let mut av = std::env::args().skip(1);
    let mut rest_is_items = false;
    while let Some(a) = av.next() {
        if rest_is_items {
            items.push(a);
        } else if a == "--" {
            rest_is_items = true;
        } else if a == "--alg" {
            alg = av.next().unwrap_or_else(|| "DEFAULT".to_string());
        } else {
            items.push(a);
        }
    }

    if items.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            if !line.trim().is_empty() {
                items.push(line);
            }
        }
    }
    Args { alg, items }
}

fn main() {
    let args = parse_args();
    if args.items.is_empty() {
        eprintln!("no input items given (pass arguments or pipe stdin)");
        std::process::exit(2);
    }

    let (flag_val, python_result, rust_result) = Python::with_gil(|py| {
        let natsort = py.import_bound("natsort").expect("natsort importable");
        let ns_enum = natsort.getattr("ns").expect("natsort.ns");
        let flag_val: i64 = ns_enum
            .getattr(args.alg.as_str())
            .unwrap_or_else(|_| panic!("unknown ns flag: {:?}", args.alg))
            .extract()
            .expect("ns value is an int");

        let py_list = PyList::new_bound(py, &args.items);
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("alg", flag_val).unwrap();
        let python_result: Vec<String> = natsort
            .call_method("natsorted", (py_list,), Some(&kwargs))
            .expect("python natsorted")
            .extract()
            .expect("list of str");

        let refs: Vec<&str> = args.items.iter().map(String::as_str).collect();
        let rust_result = natsorted_with(&refs, NsFlags::from_bits_truncate(flag_val as u32));

        (flag_val, python_result, rust_result)
    });

    println!("algorithm: natsorted(alg=ns.{})  (0x{flag_val:x})", args.alg);
    println!("input :");
    for s in &args.items {
        println!("   {s:?}");
    }
    println!("python:");
    for s in &python_result {
        println!("   {s:?}");
    }
    println!("rust  :");
    for s in &rust_result {
        println!("   {s:?}");
    }

    if python_result == rust_result {
        println!("\nMATCH — Python and Rust agree on all {} items.\n", args.items.len());
        std::process::exit(0);
    } else {
        println!("\nDIFF — {n} orderings diverge (see above).", n = args.items.len());
        std::process::exit(1);
    }
}