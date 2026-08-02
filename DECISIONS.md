# DECISIONS.md

A running log of every place where `natsort-rs` deviates from the Python original, or where a non-obvious architectural choice was made.

## Decision 1: Unicode Character Classification

### Python Behaviour
The Python `natsort` library uses `unicodedata` module to classify Unicode characters into decimals, digits, and numeric categories for proper number parsing. It includes full support for Unicode decimal digits from various scripts.

### Rust Behaviour
The Rust implementation provides partial Unicode number support:
- Basic `unicode_numbers.rs` module with Unicode character sets
- Unicode digits in regex patterns for matching
- Conversion of common Unicode digits to ASCII for parsing
- Support for Arabic, Persian, Devanagari, Bengali, and Tamil digits

### Reason
While not implementing the full `unicodedata` database, the current implementation covers the most common Unicode digit scripts used worldwide. Full Unicode character classification would require external dependencies or a large embedded database.

### Trade-offs
- **Pros**: Handles common Unicode digits, no external dependencies, reasonable binary size
- **Cons**: Doesn't cover all Unicode numeric characters (⒈, ½, ¼, etc.), limited to decimal digits

### Test Impact
Most Unicode number tests pass. Some edge cases with less common numeric characters may produce different results than Python.

Status: **Partially Implemented**

## Decision 2: fastnumbers Compatibility Layer

### Python Behaviour
The `fastnumbers.py` module provides optimized number conversion using the `fastnumbers` library when available, falling back to `fake_fastnumbers.py` implementations. `fastnumbers` is an optional C-extension dependency.

### Rust Behaviour
The Rust crate does not depend on any equivalent C-backed number parser. Numeric conversion uses the standard library's `str::parse` (`i64` / `f64`) inside `try_convert_to_number` (see `src/segment.rs`). The `fastnumbers` module is therefore a behavioural no-op: there is no separate fast/slow path.

### Reason
The Python `fastnumbers` layer is an optional dependency for optimization only; the reference behaviour is defined by the pure-Python fallback. In Rust the standard-library parser provides the same semantics directly, so a side-table of the C extension is unnecessary to match behaviour — it would only reproduce a performance optimization that Rust's `parse` already provides.

### Trade-offs
- **Pros**: No equivalent extension dependency; semantics come from the authoritative pure-Python fallback
- **Cons**: None on behaviour; edge-format parsing characteristics follow Rust's `f64` parser rather than C's `strtod`

### Test Impact
Number conversion matches Python's `fake_fastnumbers` (pure-Python) path. Any divergence from C `fastnumbers`'s parser only appears for numeric strings at the extremes of `f64` range/formatting.

Status: **Intentionally Unsupported**

## Decision 3: Hexadecimal Unicode Numbers

### Python Behaviour
The `unicode_numeric_hex.py` module handles hexadecimal and other numeric representations in Unicode.

### Rust Behaviour
Character data and numeric classification in this crate covers decimal and digit characters only (see Decision 1). Hexadecimal (base-16) character values and Unicode fractional/numeric codepoints outside the decimal set are not recognized as numbers; the full codepoint table used by Python's `unicode_data` module is not replicated.

### Reason
Hexadecimal handling in Python relies on a derived numeric-codepoint database. Reproducing that database for Rust carries a binary-size and maintenance cost that would apply to the whole input domain while only affecting the narrow case of hex text embedded in otherwise-natural input. The core decimal-digit path (which the default regex and flag set operate on) is fully implemented.

### Trade-offs
- **Pros**: Smaller binary, no embedded Unicode numeric-codepoint table; decimal/ASCII behaviour unaffected
- **Cons**: Hex and fractional Unicode numerics differ from Python in those specific inputs

### Test Impact
Inputs using hex or fractional numeric codepoints may sort differently from Python solely within those character ranges.

Status: **Intentionally Unsupported**

## Decision 4: Command-Line Interface

### Python Behaviour
The Python library ships a full command-line interface (`python -m natsort`) with options for numbers, case handling, signed/float parsing, path sorting, locale, and output formatting.

### Rust Behaviour
The Rust binary `src/main.rs` exposes a reduced CLI surface:
- `-h, --help` — usage
- `-i, --ignore-case` — case-insensitive ordering
- `-r, --reverse` — reverse the result
- `-f, --real` — parse as real numbers
- Reads items from positional arguments or stdin, one per line

Flags available in the Python CLI but absent here: locale selection, path/number-type toggles beyond `--real`, and custom output separators.

### Reason
The library's `NsFlags` already covers all sorting behaviour (see Decision 6); the CLI exists as a thin convenience wrapper over `natsorted_with` / `realsorted` and mirrors the core, non-locale options. Remaining Python CLI flags map onto `NsFlags` that carry no Rust command-line analogue rather than being unimplemented algorithmically.

### Trade-offs
- **Pros**: Minimal, dependency-free argument handling; CLI is a thin facade over the public API
- **Cons**: Does not expose the full set of Python's CLI flags

### Test Impact
Python's CLI tests (in `tests/test_main.py`) exercise the Python-specific argument surface and are not part of the Rust parity suite. They are skipped in this environment pending `pytest-mock`; see Decision 9.

Status: **Minor Difference**

## Decision 5: Locale Support

### Python Behaviour
The `LOCALE` flag (`ns.LOCALE = ns.LOCALEALPHA | ns.LOCALENUM`) enables locale-aware sorting:
- `LOCALEALPHA`: Case-insensitive sorting using locale-specific collation
- `LOCALENUM`: Handling of locale-specific thousands separators and decimal points

### Rust Behaviour
The Rust implementation provides partial locale support:
- `LOCALEALPHA`: Implemented using `unicase` crate for Unicode case folding
- `LOCALENUM`: Basic implementation that removes common thousands separators (comma, period, space) when followed by 3 digits

### Reason
Full locale support requires system locale APIs which are complex and platform-dependent. The current implementation covers common use cases without requiring external locale libraries.

### Trade-offs
- **Pros**: Works cross-platform, no external dependencies, handles common cases
- **Cons**: Doesn't handle locale-specific decimal point conversion (e.g., comma in de_DE), doesn't use system locale settings

### Test Impact
Tests requiring full locale support may produce different results than Python implementation.

Status: **Partially Implemented**

## Decision 6: Flag Value Differences from Planning Docs

### Python Behaviour
Python `ns_enum.py` defines flags with specific bit values assigned in declaration order.

### Rust Behaviour
Rust `NsFlags` mirrors Python's exact bit values, not the values documented in `.agent/` planning files.

### Reason
Planning documentation contained incorrect flag values. Actual implementation follows Python source code.

### Trade-offs
- **Pros**: 100% compatibility with Python flag values
- **Cons**: Different from planning documentation

### Test Impact
None - correct behavior matches Python exactly.

Status: **Equivalent Behaviour**

## Decision 7: PATH Mode Implementation

### Python Behaviour
When `ns.PATH` is set, `parse_path_factory` wraps the string parser. Each path component gets its own natural-sort key, producing a nested tuple.

### Rust Behaviour
`NatsortKey::key()` splits input by `/` and `\`, generates keys for each component, and flattens them into a single `Vec<NatsortKeyPart>`.

### Reason
Rust cannot represent heterogeneous nested tuples at runtime. Flattening with sentinels produces identical comparison results.

### Trade-offs
- **Pros**: Same comparison results, simpler implementation
- **Cons**: Different internal representation

### Test Impact
All PATH mode tests pass identically to Python.

Status: **Equivalent Behaviour**

## Decision 8: Locale Availability

### Python Tests Intentionally Ignored
None. Originally `en_US.UTF-8`, `de_DE.UTF-8`, and `cs_CZ.UTF-8` were not generated in the test environment, causing locale-fixture tests to error/skip.

### Reason
The locale fixtures require the corresponding system locales. These have since been generated (`localedef`) and are present on the test host.

### Recommendation
None required — all three locales are installed and the full Python suite (344 tests) passes with no skips.

Status: **Resolved**

## Decision 9: pytest-mock Availability

### Python Tests Intentionally Ignored
None. The CLI tests in `tests/test_main.py` previously could not run because `pytest-mock` was not installed in the environment.

### Reason
The `tests/test_main.py` suite uses the `mocker` fixture provided by `pytest-mock`. The dependency has now been installed in the Python virtual environment.

### Recommendation
None required — all 33 CLI tests pass. The full Python suite (344 tests) passes with no skips.

Status: **Resolved**