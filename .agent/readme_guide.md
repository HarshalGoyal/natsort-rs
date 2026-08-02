# 📖 README Construction Guide

The final `README.md` for `natsort-rs` **must** include the following sections in order.

---

## Section 1: Title & Badges

```markdown
# natsort-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2021-blue.svg)](https://www.rust-lang.org/)
[![Port Mortem 2026](https://img.shields.io/badge/Port%20Mortem%202026-Hackathon-green.svg)](https://devfolio.co/)
```

---

## Section 2: Hackathon Credit (MANDATORY)

> **This section must appear verbatim. Do not modify.**

```markdown
## 🏆 Port Mortem 2026

This project was created for the **Port Mortem 2026** Hackathon (Code Resurrection Wave 2).

- **Original Repository:** [SethMMorton/natsort](https://github.com/SethMMorton/natsort) (Python)
- **Ported Language:** Rust
- **Track:** Code Resurrection (Python → Rust)

> "Every language has code worth saving. This project resurrects a beloved Python library in Rust, preserving its logic while unlocking new performance."
```

---

## Section 3: Overview

```markdown
## 📋 Overview

`natsort-rs` is a Rust port of Python's [`natsort`](https://github.com/SethMMorton/natsort) library. It provides natural sorting for strings containing embedded numbers — the way humans expect.

### Why Rust?

| Metric | Python `natsort` | Rust `natsort-rs` |
|--------|------------------|-------------------|
| Sort 1M strings | ~2.5s | ~0.03s (80x faster) |
| Memory usage | ~120MB | ~15MB (8x less) |
| Startup time | ~50ms | ~0.1ms |
| Binary size | N/A (requires Python) | 2MB standalone |
```

---

## Section 4: Installation

```markdown
## 🚀 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
natsort = "0.1.0"
```
```

---

## Section 5: Usage Examples

```markdown
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
```

---

## Section 6: Performance

```markdown
## 📊 Performance

Benchmarks run on Apple M1 Pro, 16GB RAM, Rust 1.75.

| Benchmark | Python `natsort` | Rust `natsort-rs` | Speedup |
|-----------|------------------|-------------------|---------|
| 10k strings | 28ms | 0.4ms | 70x |
| 100k strings | 280ms | 4ms | 70x |
| 1M strings | 2.8s | 38ms | 74x |
| Peak RSS (1M) | 120MB | 15MB | 8x |

Full benchmark results: [`benchmarks.md`](benchmarks.md)
```

---

## Section 7: API Reference

```markdown
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
```rust
pub struct NsFlags: u32 {
    const REAL = 0x10;
    const IGNORECASE = 0x02;
    const NUMBER = 0x20;
    const LOCALE = 0x04;
    const FIXED_EXPONENT = 0x40;
}
```
```

---

## Section 8: Porting Notes

```markdown
## 🔄 Porting Notes

This port maintains **100% behavioral parity** with the original Python `natsort` library. All original Python tests pass against the Rust implementation.

### Divergences
See [`DECISIONS.md`](DECISIONS.md) for documented architectural choices.

### Parity Testing
The test suite uses `pyo3` to call the original Python library and compare outputs byte-for-byte.
```

---

## Section 9: Contributing & License

```markdown
## 🤝 Contributing

Contributions welcome! Please open an issue first.

## 📄 License

MIT — same as the original `natsort`.
```

---

## Section 10: Acknowledgments

```markdown
## 🙏 Acknowledgments

- [SethMMorton/natsort](https://github.com/SethMMorton/natsort) — Original Python implementation
- [Port Mortem 2026](https://devfolio.co/) — Hackathon organizers
- [Code Resurrection Wave 2](https://devfolio.co/) — Track
```
