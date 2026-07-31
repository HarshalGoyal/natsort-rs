# 🚀 Natsort Port Execution Plan (Python → Rust)

## 📋 Project Overview
**Target:** Port `SethMMorton/natsort` (Python) to idiomatic Rust.
**Goal:** 100% behavioral parity with the original Python library. All original Python tests must pass when run against the Rust implementation.
**Scope:** Standard band (~2k-3k LOC). Pure logic, no external I/O.
**Timeline:** 72 hours (phased execution).

---

## 🛠 Phase 1: Environment & Core Splitter (Hours 0–12)

### 1.1 Repository Setup
```bash
git clone https://github.com/SethMMorton/natsort.git python_src
cd python_src
python -m pytest tests/ -v  # ✅ Baseline: Must pass
```

```bash
cargo new natsort_rs --lib
cd natsort_rs
```

### 1.2 `Cargo.toml` Dependencies
```toml
[package]
name = "natsort"
version = "0.1.0"
edition = "2021"

[dependencies]
regex = "1.10"
unicase = "2.7"
serde = { version = "1.0", features = ["derive"] }
criterion = "0.5" # For benchmarks

[dev-dependencies]
pyo3 = { version = "0.22", features = ["auto-initialize"] }
```

### 1.3 Core Algorithm: `Segment` & `split_key`
**Task:** Implement the fundamental string segmentation logic.
**File:** `src/lib.rs`

**Implementation Specs:**
1. Define an enum for segments:
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub enum Segment {
       Str(String),
       Int(i64),
       Float(f64),
   }
   ```
2. Implement `split_key(input: &str) -> Vec<Segment>`:
   * Use `regex` to split on boundaries between non-digits and digits.
   * Pattern: `r"([+-]?\d*\.?\d+(?:[eE][+-]?\d+)?|\d+|[^\d]+)"`
   * Parse matched digits into `Int` or `Float` (check for `.` or `e/E`).
   * Keep non-digits as `Str`.
   * **Crucial:** Preserve empty segments if the split produces them.

3. Implement `Ord` for `Segment` to enable comparison:
   * `Int` < `Float` < `Str` (or match Python's exact type coercion rules).
   * Numeric values compare mathematically.
   * String values compare lexicographically (case-sensitive by default).

### 1.4 Basic Sort Function
**Task:** Implement `natsorted(items: &[String]) -> Vec<String>`.
* Map each item through `split_key`.
* Use `sort_by` on the mapped segments.
* Return sorted original strings.
* **Deliverable:** `cargo test` passes basic unit tests you write.

---

## 🏁 Phase 2: The `ns` Flags & Key Generator (Hours 12–24)

### 2.1 The `ns` Bitmask Enum
**Task:** Port the `ns` (natsort settings) flags.
**File:** `src/ns.rs`

**Flags to Implement:**
* `ns.REAL` (0x10): Parse signed floats (`-3.0`, `+5.10`).
* `ns.IGNORECASE` (0x02): Lowercase strings before comparison.
* `ns.NUMBER` (0x20): Force all string segments to be treated as numbers (fallback to 0.0 on parse failure).
* `ns.LOCALE` (0x04): Use locale-aware string comparison (use `unicase::Unicode` or `strxfrm` equivalent).
* `ns.FIXED_EXPONENT` (0x40): Align floating point exponents for comparison.

### 2.2 The Key Generator (`natsort_keygen`)
**Task:** Port the factory function that returns a reusable key function.
**File:** `src/keygen.rs`

**Implementation Specs:**
1. Define a `NatsortKey` struct that holds the `ns` flags and configuration.
2. Implement `impl NatsortKey { pub fn generate(&self, input: &str) -> Vec<Segment> }`.
3. Expose `pub fn natsort_keygen(flags: NsFlags) -> NatsortKey`.
4. **Rust Adaptation:** Instead of returning a Python callable, return a struct with a `.key(&str)` method that can be passed to iterators.

### 2.3 Integration
* Wire `natsorted()` to use `natsort_keygen()`.
* Ensure flags modify the segmentation/comparison logic correctly.
* **Deliverable:** `natsorted(["10", "2", "-3"], ns.REAL)` works correctly.

---

## 🧠 Phase 3: Advanced Features (Hours 24–36)

### 3.1 Mixed Type Support
**Task:** Handle `Vec<dyn Any>` or heterogeneous collections.
**File:** `src/mixed.rs`

**Implementation Specs:**
1. Define a generic `Item` enum:
   ```rust
   pub enum Item { Str(String), Int(i64), Float(f64), Bool(bool), None }
   ```
2. Implement `split_key` to handle `Item` variants.
3. Ensure `Ord` handles cross-type comparison (e.g., `Int` vs `Float`, `None` sorts first).

### 3.2 Recursive Descent (`recursive_sort`)
**Task:** Handle lists of lists (e.g., `[[1, "a"], [2, "b"]]`).
**File:** `src/recursive.rs`

**Implementation Specs:**
1. Define `pub enum NestedItem { Leaf(Item), Branch(Vec<NestedItem>) }`.
2. Implement `split_key` to recursively process `Branch` variants.
3. Comparison: Compare element-by-element. If lengths differ, shorter list sorts first.

### 3.3 OS Sorting (`os_sorted`)
**Task:** Sort paths like a file explorer (Windows Explorer / Finder).
**File:** `src/os_sort.rs`

**Implementation Specs:**
1. Normalize paths using `std::path::Path`.
2. Split paths by directory separators.
3. Apply natural sort to each component.
4. Handle case-insensitivity and locale rules specific to OS sorting.

### 3.4 Bytes & Encoding
**Task:** Handle `Vec<u8>` inputs.
**File:** `src/bytes.rs`

**Implementation Specs:**
1. Decode `Vec<u8>` to UTF-8 strings.
2. If decoding fails, handle gracefully (skip, fallback, or error based on Python behavior).
3. Apply standard `natsort` logic to decoded strings.

---

## 🧪 Phase 4: Validation & Parity (Hours 36–48)

### 4.1 Python Bridge Script
**Task:** Create a validation harness to compare Rust vs Python outputs.
**File:** `tests/parity.rs`

**Implementation Specs:**
1. Use `pyo3` to import the original `natsort` Python module.
2. Define test cases from `python_src/tests/test_natsort.py`.
3. For each test case:
   * Run Python: `py_natsort(input)`
   * Run Rust: `natsort::natsorted(&input)`
   * Assert `rust_output == py_output` (byte-identical).
4. **Deliverable:** 100% parity. Zero differences.

### 4.2 Edge Case Hardening
**Task:** Fix failures from Phase 4.1.
**Common Gotchas:**
* Empty strings `""`
* Unicode combining marks (e.g., `e\u0301`)
* Locale-specific collation (e.g., `ñ` vs `n`)
* Scientific notation (`1e10` vs `10000000000`)
* Mixed alphanumeric strings (`file10.5.txt`)

---

## 🚀 Phase 5: Polish & Submission (Hours 48–72)

### 5.1 Performance Benchmarking
**Task:** Prove Rust is faster.
**File:** `benches/natsort_bench.rs`

**Implementation Specs:**
1. Use `criterion` to benchmark `natsorted()` against 10k, 100k, 1M strings.
2. Compare against Python `natsort` baseline.
3. Highlight in README: "X50 faster than Python equivalent."

### 5.2 API Polish & Documentation
**Task:** Make it idiomatic Rust.
**File:** `src/lib.rs`

**Implementation Specs:**
1. Expose clean public API:
   ```rust
   pub fn natsorted<T: AsRef<str>>(items: &[T]) -> Vec<T>
   pub fn os_sorted<T: AsRef<str>>(items: &[T]) -> Vec<T>
   pub struct NatsortKey { pub fn key(&self, input: &str) -> Vec<Segment> }
   ```
2. Add `#[derive(Debug, Clone, PartialEq, Eq)]` to all public types.
3. Write doc comments with examples.

### 5.3 Devfolio Submission
**Task:** Create hackathon project.
**Checklist:**
* [ ] Clone original repo & run tests
* [ ] Rust implementation passes parity tests
* [ ] Benchmark shows performance win
* [ ] Clean, idiomatic Rust code
* [ ] README explains the port & architecture
* [ ] Submit to Devfolio

---

## 📦 File Structure Target
```
natsort_rs/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API
│   ├── segment.rs      # Split logic & Ord impl
│   ├── ns.rs           # Flags enum
│   ├── keygen.rs       # Key generator
│   ├── mixed.rs        # Type handling
│   ├── recursive.rs    # Nested lists
│   └── os_sort.rs      # OS path sorting
├── tests/
│   └── parity.rs       # Python bridge validation
└── benches/
    └── natsort_bench.rs
```

**Execute Phase 1.**
