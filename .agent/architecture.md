\# 🏗 Architecture: `natsort-rs`



\## 1. Module Map



```

natsort\_rs/

├── Cargo.toml

├── src/

│   ├── lib.rs          # Public API re-exports

│   ├── segment.rs      # Segment enum + split\_key + Ord

│   ├── ns.rs           # NsFlags bitmask enum

│   ├── keygen.rs       # NatsortKey struct + key generator

│   ├── mixed.rs        # Item enum for mixed-type support

│   ├── recursive.rs    # NestedItem for recursive descent

│   ├── os\_sort.rs      # OS path sorting

│   └── bytes.rs        # Vec<u8> handling

├── tests/

│   └── parity.rs       # Python bridge validation

└── benches/

&#x20;   └── natsort\_bench.rs

```



\## 2. Data Flow



```

Input: \&\[T] where T: AsRef<str>

&#x20;        │

&#x20;        ▼

┌─────────────────────────────────────────────┐

│  natsorted(items)                           │

│    1. For each item: split\_key(item)        │

│    2. Collect (original, key) pairs         │

│    3. Sort by key using Ord                 │

│    4. Return original strings in order      │

└─────────────────────────────────────────────┘

&#x20;        │

&#x20;        ▼

┌─────────────────────────────────────────────┐

│  split\_key(input: \&str) -> Vec<Segment>     │

│    1. Apply regex to tokenize               │

│    2. Parse digits → Int/Float              │

│    3. Keep non-digits → Str                 │

│    4. Apply NsFlags (case, locale, etc.)    │

└─────────────────────────────────────────────┘

&#x20;        │

&#x20;        ▼

┌─────────────────────────────────────────────┐

│  Segment Ord                                │

│    Str(String) < Int(i64) < Float(f64)      │

│    Same type: compare values                │

│    Different type: compare by type ranking  │

└─────────────────────────────────────────────┘

```



\## 3. Parity Strategy



| Approach | When to Use |

|----------|-------------|

| \*\*1:1 Port\*\* | Algorithm maps directly to Rust. Most of natsort. |

| \*\*Idiomatic Adaptation\*\* | Python pattern doesn't translate (e.g., `key=` closures → struct-based keygen). |

| \*\*Documented Divergence\*\* | Feature requires platform-specific behavior (e.g., locale collation). Log in `DECISIONS.md`. |



\### Parity Testing Flow

```

Python Input → natsort.natsorted() → Expected Output

&#x20;                                        │

Rust Input   → natsort::natsorted() → Actual Output

&#x20;                                        │

&#x20;                             Assert Equal (byte-identical)

```



\## 4. Key Design Decisions



| Decision | Choice | Rationale |

|----------|--------|-----------|

| Error handling | `thiserror` enums | Idiomatic, composable, no `unwrap()` in library code |

| Regex | `regex` crate | Mature, fast, PCRE-compatible |

| Locale support | `unicase` crate | Cross-platform Unicode case folding |

| Benchmarks | `criterion` crate | Statistical rigor, HTML reports |

| Parity bridge | `pyo3` (dev-dependency) | Direct Python interop for testing |



\## 5. Public API



```rust

// Simple API

pub fn natsorted<T: AsRef<str>>(items: \&\[T]) -> Vec<T>;

pub fn os\_sorted<T: AsRef<str>>(items: \&\[T]) -> Vec<T>;



// Key generator API

pub struct NatsortKey { /\* ... \*/ }

impl NatsortKey {

&#x20;   pub fn new(flags: NsFlags) -> Self;

&#x20;   pub fn key(\&self, input: \&str) -> Vec<Segment>;

}



// Flags

pub struct NsFlags: u32 {

&#x20;   const REAL = 0x10;

&#x20;   const IGNORECASE = 0x02;

&#x20;   const NUMBER = 0x20;

&#x20;   const LOCALE = 0x04;

&#x20;   const FIXED\_EXPONENT = 0x40;

}

```



