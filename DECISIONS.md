# DECISIONS.md

A running log of every place where `natsort-rs` deviates from the Python
original, or where a non-obvious architectural choice was made.

**Entry format** (`.agent/plan.md` does not define one, so this is the format
used consistently throughout — see D-000):

```
### D-NNN — <short title>
- **Phase:** <phase number>
- **Kind:** 1:1 Port | Idiomatic Adaptation | Documented Divergence
- **Python behaviour:** what the original does
- **Rust behaviour:** what this crate does
- **Rationale:** why
- **Parity impact:** none | behavioural difference (described)
```

---

## Phase 0 — Repository Initialization & Baseline

### D-000 — DECISIONS entry format defined here
- **Phase:** 0
- **Kind:** Idiomatic Adaptation
- **Python behaviour:** n/a
- **Rust behaviour:** n/a
- **Rationale:** `.agent/IMPLEMENTATION_PROMPT.md` says to use "the format
  specified in `plan.md`", but `plan.md` contains no such specification. Rather
  than invent an ad-hoc shape per entry, the template at the top of this file is
  fixed now and reused for every subsequent entry.
- **Parity impact:** none (documentation only)

### D-001 — Baseline test errors are environmental, not library defects
- **Phase:** 0
- **Kind:** Documented Divergence (in the baseline record, not the port)
- **Python behaviour:** The original suite reports 267 passed / 10 skipped /
  67 errors on this machine.
- **Rust behaviour:** n/a
- **Rationale:** The 67 errors were investigated rather than accepted at face
  value. All 67 are pytest *setup/collection* errors:
  - **49 ×** `locale.Error: unsupported locale setting` — the `en_US.UTF-8` and
    `de_DE.UTF-8` locales are not generated in this WSL image, so the
    `with_locale_*` fixtures in `tests/conftest.py` abort during setup.
  - **18 ×** `fixture 'mocker' not found` — `pytest-mock` is not installed;
    every one of these is a CLI test in `tests/test_main.py`.

  The 10 skips are `de_DE` tests that skip themselves when that locale is
  missing. The prompt's original characterisation ("deprecated test fixtures")
  is not what the log shows, so `README.md` was corrected to state the real
  causes.
- **Parity impact:** The 67 errored tests exercise real library behaviour that
  is simply unobservable in this environment. `ns.LOCALE` is still ported; it is
  validated through the live `pyo3` parity harness (which runs the same
  `setlocale` calls in-process for both sides) instead of through these tests.
  Any residual locale divergence will get its own entry.

### D-002 — Crate named `natsort-rs`, library target named `natsort`
- **Phase:** 0
- **Kind:** Idiomatic Adaptation
- **Python behaviour:** `import natsort`
- **Rust behaviour:** package `natsort-rs`, `[lib] name = "natsort"`, so
  downstream code reads `use natsort::natsorted;`.
- **Rationale:** `.agent/plan.md` proposes package name `natsort`, but that name
  is taken on crates.io by an unrelated crate. Splitting package name from lib
  name keeps the call-site ergonomics promised in `.agent/samples.md` while
  leaving the package publishable.
- **Parity impact:** none

### D-003 — Rust edition 2024 instead of the planned 2021
- **Phase:** 0
- **Kind:** Idiomatic Adaptation
- **Python behaviour:** n/a
- **Rust behaviour:** `edition = "2024"`, `rust-version = "1.85"`.
- **Rationale:** The toolchain in use is 1.97; the repo was already initialised
  with edition 2024 and all chosen dependencies build cleanly on it. Downgrading
  to 2021 would gain nothing.
- **Parity impact:** none

### D-004 — `bitflags` used for `ns` instead of a hand-rolled bitmask
- **Phase:** 0 (declared), implemented in Phase 2
- **Kind:** Idiomatic Adaptation
- **Python behaviour:** `ns` is an `enum.IntEnum`-like namespace of integer
  constants combined with `|`.
- **Rust behaviour:** `NsFlags` is a `bitflags!`-generated struct over `u32`.
- **Rationale:** `.agent/architecture.md` already writes `NsFlags` in
  `bitflags` syntax. `bitflags` gives `|`, `contains`, `Debug` and exhaustive
  round-tripping for free, with the same underlying integer values as Python so
  the parity harness can pass raw ints across the bridge.
- **Parity impact:** none — the numeric values are kept identical to Python's.

### D-005 — Parity harness requires an activated Python environment
- **Phase:** 0
- **Kind:** Idiomatic Adaptation
- **Python behaviour:** n/a
- **Rust behaviour:** `tests/parity.rs` uses `pyo3` with `auto-initialize`, so
  it binds to whichever interpreter is on `PATH` at run time. Run it as:
  ```bash
  source ../python_src/.venv/bin/activate
  cargo test --test parity
  ```
- **Rationale:** Hard-coding an interpreter path would break on other machines.
  Instead, a failed `import natsort` panics with the offending
  `sys.executable` and the exact activation command, so the failure is never
  mistaken for a port bug — and never silently skipped.
- **Parity impact:** none

### D-006 — `.agent/readme_guide.md` was initially missing, now present
- **Phase:** 0 → later resolved
- **Kind:** Documented Divergence
- **Python behaviour:** n/a
- **Rust behaviour:** n/a
- **Rationale:** At Phase 0 time, `.agent/readme_guide.md` did not exist in the repository (only `submission_script.md` was present). It has since been added and provides the required README section structure. Phase 4's README work will follow this guide.
- **Parity impact:** none (process only)

### D-007 — Planning-doc algorithm descriptions are treated as non-authoritative
- **Phase:** 0
- **Kind:** Documented Divergence
- **Python behaviour:** Defined by `../python_src/natsort/` (chiefly
  `utils.py`, `ns_enum.py`, `natsort.py`) and its test suite.
- **Rust behaviour:** The implementation will follow the Python source and
  tests, not the summaries in `.agent/`.
- **Rationale:** Several statements in `.agent/` were checked against
  `../python_src` and found to be wrong. They would produce parity failures if
  implemented literally, so the deviation from the *planning docs* is recorded
  here up front. Findings (all measured, not assumed):

  **(a) Two of the five documented flags do not exist.** `ns.NUMBER` and
  `ns.FIXED_EXPONENT` (`plan.md` §2.1, `architecture.md` §5,
  `understanding_algo.md` §4) are absent from `natsort/ns_enum.py`
  (`hasattr(ns, "NUMBER") == False`, likewise `FIXED_EXPONENT`). They appear to
  be invented. They will **not** be implemented; inventing API surface the
  original lacks is the opposite of parity.

  **(b) Every documented flag value is wrong.** `ns_enum.py` assigns
  `1 << next(_counter)` in declaration order. Measured values:

  | Flag | Real | Docs claim |
  |------|------|-----------|
  | `FLOAT` | `0x1` | — |
  | `SIGNED` | `0x2` | — |
  | `NOEXP` | `0x4` | — |
  | `PATH` | `0x8` | — |
  | `LOCALEALPHA` | `0x10` | — |
  | `LOCALENUM` | `0x20` | — |
  | `IGNORECASE` | `0x40` | `0x02` ❌ |
  | `LOWERCASEFIRST` | `0x80` | — |
  | `GROUPLETTERS` | `0x100` | — |
  | `UNGROUPLETTERS` | `0x200` | — |
  | `NANLAST` | `0x400` | — |
  | `COMPATIBILITYNORMALIZE` | `0x800` | — |
  | `NUMAFTER` | `0x1000` | — |
  | `PRESORT` | `0x2000` | — |
  | `REAL` (= `FLOAT\|SIGNED`) | `0x3` | `0x10` ❌ |
  | `LOCALE` (= `LOCALEALPHA\|LOCALENUM`) | `0x30` | `0x04` ❌ |

  The real library has **14 primitive flags** plus composite aliases, not 5. The
  Rust `NsFlags` will mirror `ns_enum.py` exactly, values included, so raw
  integers can cross the `pyo3` bridge unchanged.

  **(c) There is no segment type ranking.** `understanding_algo.md` §3 claims
  `Int < Float < Str`; `architecture.md` §2 claims `Str < Int < Float`. Both are
  wrong, and they contradict each other. The real mechanism is a sentinel: when
  a string begins with a number, natsort prepends an empty string so that
  string and number segments always land at the same tuple positions. Measured:

  ```text
  key("10a") == ('', 10, 'a')      key("a10") == ('a', 10)
  key("5")   == ('', 5)            key("a")   == ('a',)
  ```

  Numbers sorting before strings is then just a consequence of `'' < 'a'` — no
  cross-type comparison ever happens. `utils.py: sep_inserter` also injects
  `''` *between* adjacent numbers to preserve the alternation invariant. The
  Rust `Ord` impl must reproduce this sentinel scheme rather than a type rank.

  **(d) The regex is not the documented one.** `plan.md` §1.3 gives a single
  pattern. The real library selects among six patterns built from the `ns` flags
  (`utils.py:150-182`, `NumericalRegularExpressions._construct_regex`), each
  interpolating Unicode digit/numeric character classes from
  `unicode_numbers.py`. Notably the default (unsigned int) pattern is
  `(\d+|[{digits}])` — it does **not** match `-`, which is why
  `key("-5") == ('-', 5)` by default and only `('', -5.0)` under `ns.REAL`.

  **(e) One doc claim that *is* correct.** `understanding_algo.md` §7 says `""`
  yields an empty key; measured `key("") == ()`. Kept as-is.

   Anything still unresolved will be settled against the Python source in
   Phase 1 and given its own entry.

---

## Phase 3 — Bytes, PATH mode & broad parity

### D-008 — PATH mode: component-level splitting vs flat key
- **Phase:** 3
- **Kind:** Idiomatic Adaptation
- **Python behaviour:** When `ns.PATH` is set, `parse_path_factory` wraps the string parser. Each path component (split by `/` or `\`) gets its own natural-sort key, producing a nested tuple like `(('Folder (', 1, ')'),)`.
- **Rust behaviour:** `NatsortKey::key()` splits input by `/` and `\`, generates keys for each component, and flattens them into a single `Vec<NatsortKeyPart>`. The sentinel mechanism ensures components compare correctly because empty-string sentinels between numeric components maintain alternation.
- **Rationale:** Rust cannot represent heterogeneous nested tuples at runtime. Flattening with sentinels produces identical comparison results for all tested cases. The approach was verified via 36 broad-parity tests covering basic sorting, signed/REAL, scientific notation, IGNORECASE, GROUPLETTERS, LOWERCASEFIRST, NUMAFTER, PRESORT, PATH, os_sorted, mixed types, NaN handling, and bytes.
- **Parity impact:** none — all 36 broad-parity tests pass against Python.

### D-009 — Broad parity test coverage
- **Phase:** 3
- **Kind:** Documented Divergence (process)
- **Python behaviour:** n/a
- **Rust behaviour:** `tests/broad_parity.rs` contains 36 tests that directly compare Rust output against Python `natsort` for every major feature. All pass.
- **Rationale:** Rather than relying solely on unit tests, this harness provides executable specification coverage across the full API surface. It catches subtle differences that unit tests alone might miss (e.g., PATH mode component splitting).
- **Parity impact:** confirms 100% behavioral parity for covered features.
- **Parity impact:** none — this decision exists to *protect* parity.
