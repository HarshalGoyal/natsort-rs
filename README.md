# natsort-rs: Rust port of Python's natsort library

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 2024](https://img.shields.io/badge/Rust-2024-blueviolet.svg)](https://www.rust-lang.org/)

## Project Status

**Python reference suite: 344 / 344 pass · Rust `cargo test`: 175 / 175 pass (0 failed, 0 ignored)**

`natsort-rs` is a Rust port of the
[SethMMorton/natsort](https://github.com/SethMMorton/natsort) Python library.
Natural sorting orders strings so that embedded numbers are compared
numerically rather than lexicographically:

```text
file1.txt, file2.txt, file10.txt     (natural order)
file1.txt, file10.txt, file2.txt     (plainly dictionary order)
```

The port is verified against the original Python implementation, including a
differential corpus where Rust and Python must agree byte-for-byte.

## Features

### Core functionality
- **Natural sorting**: `natsorted`, `natsorted_with`
- **Real-number sorting**: `realsorted` (signed floats)
- **Reverse + composite flags**: `natsorted_rev`
- **Path-aware sorting**: `PATH` flag and `os_sorted` for filesystems paths
- **Mixed-type sorting**: `natsorted_mixed` (strings, ints, floats)
- **Byte sorting**: `natsorted_bytes`
- **Recursive descent**: `natsorted_recursive` for nested structures
- **Key factories**: `natsort_keygen`, `NatsortKey`, `NatsortKeyPart`

### Flags
All 14 Python `ns` flags, with bit values identical to Python's `ns_enum.py`:

```
FLOAT 0x001  SIGNED 0x002  NOEXP 0x004        PATH 0x008
LOCALEALPHA 0x010  LOCALENUM 0x020  IGNORECASE 0x040
LOWERCASEFIRST 0x080  GROUPLETTERS 0x100  UNGROUPLETTERS 0x200
NANLAST 0x400  COMPATIBILITYNORMALIZE 0x800  NUMAFTER 0x1000  PRESORT 0x2000
```

Composite flags: `REAL = FLOAT | SIGNED`, `LOCALE = LOCALEALPHA | LOCALENUM`.

### Additional surface
- **CLI tool**: `natsort` binary (`src/main.rs`) with `-i/-r/-f` and stdin
- **Unicode digit support**: `unicode_numbers`
- **fastnumbers-compatible number parsing**: `fastnumbers`

## Implementation notes

The algorithm mirrors Python's exactly: sentinel key generation, no
cross-type comparison, same tuple-position alignment, and the same set of
regex-driven number splits.

Faithful low-level ports worth calling out:
- `src/path.rs::path_splitter` reproduces `natsort.utils.path_splitter`,
  including the subtle CPython `PurePath.suffixes` + `base.replace(...)`
  semantics (double-dot names such as `a..gz`). See
  [DECISIONS.md](DECISIONS.md).

Some deliberate divergences are documented in
[DECISIONS.md](DECISIONS.md) rather than being hidden.

## Testing

The parity suite bridges to the **original Python `natsort`** (via `pyo3`) and
asserts identical output. Python is *not* vendored; the reference checkout is
kept as a sibling directory (`../python_src`) and used only as a test oracle.

### Prerequisites — get the Python reference

```bash
# 1. Clone the original library as a sibling of this crate
git clone https://github.com/SethMMorton/natsort.git ../python_src

# 2. Venv + install natsort and test dependencies
cd ../python_src
python3 -m venv .venv
source .venv/bin/activate
pip install -e . pytest pytest-mock pyo3

# 3. Generate the locales the locale tests require (one-time, if missing)
sudo localedef -i en_US -f UTF-8 en_US.UTF-8
sudo localedef -i de_DE -f UTF-8 de_DE.UTF-8
sudo localedef -i cs_CZ -f UTF-8 cs_CZ.UTF-8
```

### Counts

The Python number (344) is the reference suite from upstream `natsort`. The
Rust suite does **not** re-run that set verbatim; it has its own organization
(unit, parity, integration, doctests). The full `cargo test` run currently
reports **175 run / 175 passed / 0 failed / 0 ignored**:

| Suite | Run | Passed | Failed | Ignored |
|-------|-----|--------|--------|---------|
| `src/lib.rs` unit tests | 102 | 102 | 0 | 0 |
| `tests/broad_parity.rs` (direct py vs Rust via `pyo3`) | 36 | 36 | 0 | 0 |
| `tests/parity.rs` integration | 22 | 22 | 0 | 0 |
| `tests/parity_suite.rs` behavioral differential | 1 | 1 | 0 | 0 |
| doctests | 14 | 14 | 0 | 0 |
| `src/main.rs` unit tests | 0 | 0 | 0 | 0 |
| **Total** | **175** | **175** | **0** | **0** |

Run the summary yourself from `natsort-rs`:

```bash
cargo test 2>&1 | grep -E "test result:"
```

### The "deciding factor": behavioral differential (`parity_suite`)

`tests/parity_suite.rs` replays the **behavioral** tests of the upstream suite
against the Rust implementation and is the parity gate. Its current report:

```text
Python reference suite total           : 344
behavioural module tests               : 69   (natsorted / convenience / os_sorted)
Python-internal-only (no Rust mirror)  : 275
total differential cases compared      : 40
core (non-locale) cases                : 28   matched = 28   mismatched = 0
locale cases (known partial impl)      : 12   matched = 4    divergent = 8
```

- **Core behaviour (non-locale) is 100% parity** — every captured
  `natsorted` / `realsorted` / `os_sorted` call matches Python exactly.
- The 8 divergent cases are all **locale**-flag inputs
  (`LOCALEALPHA` / `LOCALENUM`), a documented partial implementation (see
  [DECISIONS.md](DECISIONS.md)).
- 275 of the 344 reference tests exercise Python's *internals* (regex tables,
  `ns` enum mechanics, transform factories) and have no 1:1 Rust counterpart.

The harness fails loudly if any **core** (non-locale) case diverges, so it is
the honest line for "does Rust match Python".

### Commands (run from `natsort-rs` with the Python venv active)

```bash
# 1. Activate the reference environment
source ../python_src/.venv/bin/activate

# 2. Rust unit + parity + integration + doctests
cargo test

# 3. Broad parity only
cargo test --test parity

# 4. Python reference tests
(cd ../python_src && python -m pytest tests/ -v)

# 5. CLI
cargo run -- -h
```

> The parity harness fails loudly (rather than silently skipping) if `natsort`
> cannot be imported; activate the venv from step 1 before `cargo test`.

## Quick example

```rust
use natsort::natsorted;

fn main() {
    let items = ["item2", "item10", "item1"];
    assert_eq!(natsorted(&items), ["item1", "item2", "item10"]);
}
```

## Documentation

- **`DECISIONS.md`** — every deliberate divergence from Python and the
  reasoning behind it.
- **`benchmarks.md`** — benchmark methodology and the `cargo bench` results.

## Technical details

**Dependencies** (`Cargo.toml`): `bitflags`, `regex`, `unicase`, `thiserror`;
dev: `criterion`, `fastrand`, `pyo3`.

**Architecture**:

- `src/lib.rs` — public API and module wiring
- `src/keygen.rs` — core key-generation algorithm
- `src/segment.rs` — number/string segmentation
- `src/path.rs` — faithful `path_splitter` port (POSIX path components)
- `src/os_sort.rs` — `os_sorted` / `os_sort_key`
- `src/ns.rs` — flag definitions matching Python's `ns_enum.py`
- `src/bytes.rs` / `src/mixed.rs` / `src/recursive.rs` — type-specific sorting
- `src/locale.rs` — locale-aware transforms
- `src/unicode_numbers.rs` / `src/fastnumbers.rs` — number support
- `src/main.rs` — CLI
- `fuzz/fuzz_targets/` — cargo-fuzz targets (differential + plain)
- `benches/` + `benchmarks.md` — criterion benchmarks

## Origin

A Rust port of [SethMMorton/natsort](https://github.com/SethMMorton/natsort).
Goal: behavioral parity with the original. The parity harness drives the suite
against the real Python library.

## License

MIT — same as the original `natsort`.

## Acknowledgments

- [SethMMorton/natsort](https://github.com/SethMMorton/natsort) — the original
  Python library.
- [Port Mortem 2026](https://portmortem.devfolio.co/) — hackathon organizers.