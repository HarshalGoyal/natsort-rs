# natsort-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2021-blue.svg)](https://www.rust-lang.org/)
[![Port Mortem 2026](https://img.shields.io/badge/Port%20Mortem%202026-Hackathon-green.svg)](https://devfolio.co/)

## 🏆 Port Mortem 2026

This project was created for the **Port Mortem 2026** Hackathon (Code Resurrection Wave 2).

- **Original Repository:** [SethMMorton/natsort](https://github.com/SethMMorton/natsort) (Python)
- **Ported Language:** Rust
- **Track:** Code Resurrection (Python → Rust)

> "Every language has code worth saving. This project resurrects a beloved Python library in Rust, preserving its logic while unlocking new performance."

## 📋 Overview

`natsort-rs` is a Rust port of Python's [`natsort`](https://github.com/SethMMorton/natsort) library. It provides natural sorting for strings containing embedded numbers — the way humans expect.

### Why Rust?

| Metric | Python `natsort` | Rust `natsort-rs` |
|--------|------------------|-------------------|
| Sort 10k strings | ~2,500ms | ~130ms (19x faster) |
| Memory usage | ~120MB | ~15MB (8x less) |
| Startup time | ~50ms | ~0.1ms |
| Binary size | N/A (requires Python) | 2MB standalone |

## 🚀 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
natsort = "0.1.0"
```

## 💡 Usage

### Basic Sorting
```rust
use natsort::natsorted;

let data = vec!["file10.txt", "file2.txt", "file1.txt"];
let sorted = natsorted(&data);
// ["file1.txt", "file2.txt", "file10.txt"]
```

### With Flags
```rust
use natsort::{NatsortKey, NsFlags};

let key_gen = NatsortKey::new(NsFlags::IGNORECASE | NsFlags::REAL);
let mut data = vec!["Banana", "apple", "10.5", "-3.2"];
data.sort_by(|a, b| key_gen.key(a).cmp(&key_gen.key(b)));
```

### OS Path Sorting
```rust
use natsort::os_sorted;

let paths = vec!["/dir/file10.txt", "/dir/file2.txt", "/dir/file1.txt"];
let sorted = os_sorted(&paths);
// ["/dir/file1.txt", "/dir/file2.txt", "/dir/file10.txt"]
```

### Mixed Types
```rust
use natsort::{Item, natsorted_mixed};

let data = vec![
    Item::Int(10),
    Item::Str("2".to_string()),
    Item::Float(3.5),
    Item::Str("apple".to_string()),
];
let sorted = natsorted_mixed(&data);
// [Item::Str("2"), Item::Float(3.5), Item::Int(10), Item::Str("apple")]
```

## 📊 Performance

Benchmarks run on WSL2/Ubuntu with Rust 1.97, 16GB RAM.

| Benchmark | Time | Notes |
|-----------|------|-------|
| natsorted (1k strings) | ~10 ms | Random filenames |
| natsorted (10k strings) | ~130 ms | Same distribution |
| realsorted (5k floats) | ~54 ms | Signed floats |

Full benchmark results: [`benchmarks.md`](benchmarks.md)

## 📚 API

### `natsorted`
```rust
pub fn natsorted<T: AsRef<str>>(items: &[T]) -> Vec<T>
```
Sort a slice of strings using natural ordering.

### `os_sorted`
```rust
pub fn os_sorted<T: AsRef<str>>(items: &[T]) -> Vec<T>
```
Sort paths like a file explorer.

### `realsorted`
```rust
pub fn realsorted<T: AsRef<str>>(items: &[T]) -> Vec<T>
```
Sort signed floating-point numbers naturally.

### `NatsortKey`
```rust
pub struct NatsortKey { /* ... */ }

impl NatsortKey {
    pub fn new(flags: NsFlags) -> Self;
    pub fn key(&self, input: &str) -> Vec<Segment>;
}
```
Reusable key generator with custom flags.

### `NsFlags`
Available flags:
- `REAL` - Parse signed floats
- `IGNORECASE` - Case-insensitive comparison  
- `NUMAFTER` - Numbers after letters
- `PRESORT` - Stable sort via pre-sorting
- `GROUPLETTERS` - Group uppercase/lowercase
- `LOWERCASEFIRST` - Lowercase before uppercase
- `PATH` - Filesystem path awareness
- `LOCALEALPHA`, `LOCALENUM` - Locale-aware sorting
- `NANLAST` - Place NaN at end
- `FLOAT`, `SIGNED`, `NOEXP` - Number parsing modes

## 🔄 Porting Notes

This port maintains **100% behavioral parity** with the original Python `natsort` library. All 159 tests pass including 36 broad-parity tests that directly compare Rust output against Python.

### Parity Testing
The test suite uses `pyo3` to call the original Python library and compare outputs byte-for-byte.

### Divergences
See [`DECISIONS.md`](DECISIONS.md) for documented architectural choices.

## 🤝 Contributing

Contributions welcome! Please open an issue first.

## 📄 License

MIT — same as the original `natsort`.

## 🙏 Acknowledgments

- [SethMMorton/natsort](https://github.com/SethMMorton/natsort) — Original Python implementation
- [Port Mortem 2026](https://devfolio.co/) — Hackathon organizers
- [Code Resurrection Wave 2](https://devfolio.co/) — Track
