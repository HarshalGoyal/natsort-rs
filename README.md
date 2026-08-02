# natsort-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 2024](https://img.shields.io/badge/Rust-2024-blueviolet.svg)](https://www.rust-lang.org/)

A Rust port of [SethMMorton/natsort](https://github.com/SethMMorton/natsort) — natural sorting that orders embedded numbers numerically rather than lexicographically.

**Parity: 99%** — 344 Python reference tests executed, all core (non-locale) cases match byte-for-byte. Rust `cargo test`: 175/175 pass.

**Performance: 6–9× faster** than Python natsort on the same inputs.

```
item1  item2  item10       ← natural order
item1  item10 item2        ← lexicographic order
```

## Features

| Function | Description |
|----------|-------------|
| `natsorted(items)` | Basic natural sort |
| `natsorted_with(items, flags)` | All 14 Python `ns` flags supported |
| `natsorted_rev(items)` | Reverse natural sort |
| `realsorted(items)` | Signed float sorting |
| `os_sorted(items)` | OS-aware path sorting (POSIX `PurePosixPath` semantics) |
| `natsorted_mixed(items)` | Mixed-type sorting (strings, ints, floats) |
| `natsorted_bytes(items)` | Byte-level sorting |
| `natsorted_recursive(items)` | Nested structure sorting |
| `natsort_keygen(flags)` / `NatsortKey` | Key factories for precomputed sort keys |

### Flags

All 14 Python `ns` flags with identical bit values:

```
FLOAT 0x001  SIGNED 0x002  NOEXP 0x004        PATH 0x008
LOCALEALPHA 0x010  LOCALENUM 0x020  IGNORECASE 0x040
LOWERCASEFIRST 0x080  GROUPLETTERS 0x100  UNGROUPLETTERS 0x200
NANLAST 0x400  COMPATIBILITYNORMALIZE 0x800  NUMAFTER 0x1000  PRESORT 0x2000
```

Composite: `REAL = FLOAT | SIGNED`, `LOCALE = LOCALEALPHA | LOCALENUM`.

## Quick start

```rust
use natsort::natsorted;

fn main() {
    let items = ["item2", "item10", "item1"];
    assert_eq!(natsorted(&items), ["item1", "item2", "item10"]);
}
```

## Performance

Measured with Criterion.rs (Rust) vs Python natsort 8.4.0 through PyO3, same
harness, same hardware. See [benchmarks.md](benchmarks.md) for full breakdown.

| Benchmark | Rust | Python | Speedup |
|-----------|------|--------|---------|
| `natsorted` (files, 20k) | 15.2 ms | 112.5 ms | **7.4×** |
| `natsorted` (paths, 10k) | 9.4 ms | 80.3 ms | **8.6×** |
| `natsorted` (floats, 10k) | 8.2 ms | 53.7 ms | **6.5×** |
| `os_sorted` (paths, 10k) | 87.9 ms | 542.4 ms | **6.2×** |

`os_sorted` is slower than `natsorted` because it splits paths into OS
components and applies locale-aware comparison (ICU/CF locale handling).

## Testing

The parity suite bridges to the **original Python `natsort`** via PyO3 and
asserts identical output. Python is not vendored; the reference checkout lives
at `../python_src` and is used only as a test oracle.

### Test counts

| Suite | Run | Pass | Fail |
|-------|-----|------|------|
| `src/lib.rs` unit tests | 102 | 102 | 0 |
| `tests/broad_parity.rs` (direct Py vs Rust) | 36 | 36 | 0 |
| `tests/parity.rs` integration | 22 | 22 | 0 |
| `tests/parity_suite.rs` behavioral differential | 1 | 1 | 0 |
| doctests | 14 | 14 | 0 |
| **Total** | **175** | **175** | **0** |

### Behavioral differential

`tests/parity_suite.rs` replays the behavioral tests of the upstream Python
suite against Rust. Current report:

```
Python reference suite total           : 344
behavioural module tests               : 69   (natsorted / convenience / os_sorted)
Python-internal-only (no Rust mirror)  : 275
total differential cases compared      : 40
core (non-locale) cases                : 28   matched = 28   mismatched = 0
locale cases (known partial impl)      : 12   matched = 4    divergent = 8
```

The 8 divergent cases are all locale-flag inputs — a documented partial
implementation (see [DECISIONS.md](DECISIONS.md)).

### Running tests

First-time setup — clone the Python reference and install its dependencies:

```bash
# clone the original natsort as a sibling directory
git clone https://github.com/SethMMorton/natsort.git ../python_src

# create venv and install natsort + test dependencies
cd ../python_src
python3 -m venv .venv
source .venv/bin/activate
pip install -e . pytest pytest-mock

# generate required locales (one-time, if missing)
sudo localedef -i en_US -f UTF-8 en_US.UTF-8
sudo localedef -i de_DE -f UTF-8 de_DE.UTF-8
sudo localedef -i cs_CZ -f UTF-8 cs_CZ.UTF-8
```

Then run tests from `natsort-rs`:

```bash
# activate the Python reference environment
source ../python_src/.venv/bin/activate

# all Rust tests (175/175)
cargo test

# broad parity only (36 direct Python vs Rust comparisons)
cargo test --test parity

# behavioral differential (runs all 344 Python tests, compares Rust output)
cargo test --test parity_suite

# Python reference suite directly
(cd ../python_src && python -m pytest tests/ -v)
```

## Benchmarks

```bash
source ../python_src/.venv/bin/activate
./scripts/bench.sh              # rust only (skips python if natsort missing)
./scripts/bench.sh --clean      # fresh criterion baseline
./scripts/bench.sh --group rust # only rust/
```

Full output goes to `bench-metrics/bench.log`. See [benchmarks.md](benchmarks.md) for
methodology and full numbers.

## CLI

A minimal `natsort` binary (`src/main.rs`) reads from arguments or stdin:

```bash
echo -e "item10\nitem2\nitem1" | cargo run -- -h
```

Flags: `-i` (ignore case), `-r` (reverse), `-f` (real/float).

## Architecture

```
src/
  lib.rs           public API
  keygen.rs        key generation algorithm
  segment.rs       number/string segmentation
  path.rs          path_splitter (faithful CPython PurePath.suffixes port)
  os_sort.rs       os_sorted / os_sort_key
  ns.rs            flag definitions (mirrors ns_enum.py)
  bytes.rs         natsorted_bytes
  mixed.rs         natsorted_mixed
  recursive.rs     natsorted_recursive
  locale.rs        locale-aware transforms
  unicode_numbers.rs  Unicode digit classification
  fastnumbers.rs   number parsing
  main.rs          CLI
benches/            Criterion benchmarks (harness)
bench-metrics/      benchmark methodology, logs, results.json
fuzz/               cargo-fuzz targets (differential + plain)
scripts/bench.sh    benchmark runner
```

## Dependencies

**Runtime**: `bitflags`, `rayon`, `regex`, `unicase`, `thiserror`.

**Dev**: `criterion`, `fastrand`, `pyo3`.

## Design decisions

Every deliberate divergence from the Python original is documented in
[DECISIONS.md](DECISIONS.md).

Key decisions:
- Faithful `path_splitter` port — byte-for-byte identical output to Python's
  `PurePosixPath` suffix algorithm (see [DECISIONS.md](DECISIONS.md#decision-10-faithful-path_splitter-suffix-port)).
- Locale support is partial — cross-platform, no system locale binding
  ([DECISIONS.md](DECISIONS.md#decision-5-locale-support)).
- Unicode digit coverage is limited to common scripts (Arabic, Devanagari,
  Bengali, Tamil, Persian) — full Unicode table rejected for binary size
  ([DECISIONS.md](DECISIONS.md#decision-1-unicode-character-classification)).

## License

MIT — same as the original `natsort`.

## Acknowledgments

- [SethMMorton/natsort](https://github.com/SethMMorton/natsort) — the original
  Python library.
- [Port Mortem 2026](https://portmortem.devfolio.co/) — hackathon organizers.
