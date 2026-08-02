# DECISIONS.md

A running log of every place where `natsort-rs` deviates from the Python
original, or where a non-obvious architectural choice was made.

Each entry records the **Python behaviour**, the **Rust behaviour**, the
**reason** for the choice, the resulting **trade-offs**, the **test impact**,
and a **status** (`Equivalent Behaviour`, `Partially Implemented`,
`Intentionally Unsupported`, `Minor Difference`, or `Resolved`).

---

## Decision 1: Unicode Character Classification

### Python Behaviour
The Python `natsort` library uses `unicodedata` to classify characters into
decimal / digit / numeric categories for number parsing, giving full Unicode
digit coverage across every script.

### Rust behaviour
Partial Unicode number support in `src/unicode_numbers.rs`:
- a character set used inside the regex patterns;
- conversion of the most common decimal-digit scripts to ASCII for parsing;
- coverage for Arabic, Persian, Devanagari, Bengali, and Tamil digits.

### Reason
Replicating the whole `unicodedata` database would add a large embedded table
for the sake of a narrow class of inputs. Covering the most widespread digit
scripts keeps the crate dependency-free with a reasonable binary size.

### Trade-offs
- **Pros**: No external dependency, small binary, common scripts handled.
- **Cons**: Uncommon numeric codepoints (⒈ ½ ¼ …) and non-decimal numerics are
  not recognized.

### Test impact
Most Unicode-number tests pass. Uncommon numeric codepoints may sort
differently from Python.

### Status: Partially Implemented

---

## Decision 2: `fastnumbers` Compatibility Layer

### Python behaviour
`natsort/compat/fastnumbers.py` uses the optional C extension `fastnumbers`
for fast number conversion, falling back to the pure-Python
`fake_fastnumbers`. Semantics for sorting come from the pure-Python fallback.

### Rust behaviour
Numeric conversion uses the standard library's `str::parse::<f64>` /
`str::parse::<i64>` (see `src/segment.rs`). There is no separate fast path;
there is nothing to mirror a C-backed extension.

### Reason
In Python the `fastnumbers` layer is purely an optimization; the authoritative
behaviour is the pure-Python fallback. Rust's `parse` already provides that
semantics directly, so an equivalent side-table would only reproduce a speed
optimisation that the standard library already offers.

### Trade-offs
- **Pros**: No extension dependency; behaviour matches the authoritative
  pure-Python path.
- **Cons**: Edge-format parsing follows Rust's `f64` parser rather than C
  `strtod` for strings at the extremes of the float range.

### Status: Intentionally Unsupported

---

## Decision 3: Hexadecimal Unicode Numbers

### Python behaviour
`natsort/unicode_numeric_hex.py` derives hexadecimal (base-16) values and other
numeric representations from the Unicode codepoint database.

### Rust behaviour
Only decimal and digit characters are classified as numbers (see
Decision 1). Hex and fractional Unicode codepoints outside the decimal set are
treated as ordinary text.

### Reason
Hex handling depends on a derived numeric-codepoint table. Reproducing it
carries a binary-size and maintenance cost across the whole input domain for a
narrow set of hex-in-text inputs, while the default decimal path is unaffected.

### Status: Intentionally Unsupported

---

## Decision 4: Command-Line Interface

### Python behaviour
The library ships a full CLI (`python -m natsort`) with options for number
type, case, signedness, path parsing, locale, and output formatting.

### Rust behaviour
`src/main.rs` exposes a reduced CLI surface:

- `-h, --help` usage
- `-i, --ignore-case` case-insensitive ordering
- `-r, --reverse` reverse the result
- `-f, --real` parse as real (float) numbers
- items from positional arguments or stdin, one per line

Locale selection and advanced output-format options are not carried over.

### Reason
The library `NsFlags` already covers all sorting behaviour (see
Decision 6); the CLI is a thin facade over `natsorted_with` / `realsorted`.
Missing Python CLI flags map to `NsFlags` that have no Rust CLI analogue
rather than being unimplemented algorithmically.

### Status: Minor Difference

---

## Decision 5: Locale Support

### Python behaviour
`ns.LOCALE = ns.LOCALEALPHA | ns.LOCALENUM` enables locale-aware sorting:
collation ordering (`LOCALEALPHA`) and locale decimal/thousands handling
(`LOCALENUM`).

### Rust behaviour
Partial locale support:
- `LOCALEALPHA` via the `unicase` crate (Unicode case-folding);
- `LOCALENUM` removes common thousands separators when followed by three
  digits.

### Reason
Full locale collation relies on platform locale APIs. This approach is
cross-platform, dependency-light, and covers the common cases without binding
to the host locale.

### Trade-offs
- **Pros**: Cross-platform; no extra dependencies.
- **Cons**: No locale-specific decimal-point conversion and no dependence on
  the system locale selection.

### Status: Partially Implemented

### Measured impact (`tests/parity_suite.rs`)
The behavioral differential harness captures the 8 suite cases that diverge;
all 8 are `LOCALE`-flag inputs (`LOCALEALPHA`/`LOCALENUM`), confirming the
locale handling is the sole remaining visible gap. Core (non-locale)
behavioral cases match Python 100%.

---

## Decision 6: Flag Values Match Python's `ns_enum.py`

### Python behaviour
`natsort/ns_enum.py` assigns flag bits in declaration order
(`FLOAT 0x1 … PRESORT 0x2000`).

### Rust behaviour
`NsFlags` mirrors the authoritative bit values from `ns_enum.py`, notably NOT
the (incorrect) values from early planning notes.

### Reason
Correctness against the Python original is the goal; planning notes contained
outdated values.

### Trade-offs
- **Pros**: 100% bit-for-bit compatibility with Python.
- **Cons**: Diverges from the earlier planning documentation (and DECISIONS
  notes). The planning doc was itself updated.

### Status: Equivalent Behaviour

---

## Decision 7: PATH Mode Implementation

### Python behaviour
With `ns.PATH`, `parse_path_factory` wraps the string parser; each path
component (from `natsort.utils.path_splitter`) gets its own key, producing a
nested tuple.

### Rust behaviour
`src/path.rs::path_splitter` is a faithful port of the CPython
`PurePosixPath` semantics underlying `natsort.utils.path_splitter` and is
treated strictly as POSIX:
- `/` and exactly `//` roots are kept as a component; `.` and empty segments
  are dropped; `..` is preserved;
- the base's extensions are split off using the CPython `PurePath.suffixes`
  algorithm, then removed with `base = base.replace("".join(suffixes), "")`
  (a substring replace, not a trimmed cut — this matters for repeated and
  double-dot names such as `x.gz.tar.gz` → `x.gz`, `a..gz` → `a . .gz`);
- an empty or `.` path collapses to the single component `"."`.

`NatsortKey::key()` then keys each component and flattens the result into a
single `Vec<NatsortKeyPart>` (runtime tuples of unlike types cannot be
represented in Rust).

### Trade-offs
- **Pros**: Identical comparison results; validated against the actual Python
  implementation (a 400-case differential corpus matches exactly).
- **Cons**: Different internal representation (flat rather than nested).

### Status: Equivalent Behaviour

---

## Decision 8: Locale Availability

Previously the `en_US.UTF-8`, `de_DE.UTF-8`, and `cs_CZ.UTF-8` locale-fixture
tests could not run because those locales were not generated on the host. They
have since been created with `localedef`, and the full Python suite passes with
no skips.

### Status: Resolved

---

## Decision 9: `pytest-mock` Availability

The Python CLI tests (`tests/test_main.py`) use the `mocker` fixture from
`pytest-mock`. `pytest-mock` is now installed and all CLI tests pass, so those
tests are back in the parity count.

### Status: Resolved

---

## Decision 10: Faithful `path_splitter` Suffix Port

### Python behaviour
`natsort.utils.path_splitter` derives extensions not by a naive
"last dot" split but by computing `PurePath(base).suffixes` and then doing
`base = base.replace("".join(suffixes), "")`:

```python
# CPython PurePath.suffixes
name = s
if name.endswith('.'):
    suffixes = []
else:
    name = name.lstrip('.')
    suffixes = ['.' + p for p in name.split('.')[1:]]
```

### Rust behaviour
`src/path.rs` reproduces this exactly, including two easy-to-miss subtleties:

1. `suffixes` keeps **empty** pieces, so `a..gz` → suffixes `['.', '.gz']`,
   stripping `..gz` from the base and yielding `['a', '.', '.gz']`.
2. `str::replace` removes **all** occurrences of the joined suffix string, not
   just a suffix at the end. For a body like `x.gz.tar.gz` this yields the
   stem `x.gz` (`['x.gz', '.tar', '.gz']`), which a simple right-to-left cut
   would get wrong.

The stop conditions (a suffix beginning with a digit — the `\.\d` match —
more than two suffixes, or a suffix longer than 5 chars) are applied while
walking the reversed suffix list, mirroring the loop order in the Python
source.

### Trade-offs
- **Pros**: Byte-for-byte identical output to Python across the POSIX corpus.
- **Cons**: `PurePosixPath` semantics assume `/`-separated paths; backslashes
  are treated as literals. The existing `os_sort` family handles backslash
  splitting separately.

### Status: Equivalent Behaviour