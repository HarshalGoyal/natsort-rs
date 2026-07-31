\# 🧠 Understanding `natsort`: The Algorithm



\## 1. The Core Problem

Standard lexicographical sorting fails on strings containing numbers:

```python

sorted(\["file10.txt", "file2.txt", "file1.txt"])

\# Output: \['file1.txt', 'file10.txt', 'file2.txt']  ❌ Wrong!

```

Humans expect `1` to come before `10`. `natsort` fixes this by treating embedded numbers as numeric values, not character sequences.



\## 2. The Core Mechanism: Segmentation

`natsort` splits every string into a list of alternating \*\*text\*\* and \*\*number\*\* segments.



\### Example: `"file10.5.txt"`

1\. Scan left-to-right.

2\. `"file"` → Text segment (`Str("file")`)

3\. `"10"` → Integer segment (`Int(10)`)

4\. `"."` → Text segment (`Str(".")`)

5\. `"5"` → Float segment (`Float(5.0)`)

6\. `".txt"` → Text segment (`Str(".txt")`)



\*\*Result:\*\* `\[Str("file"), Int(10), Str("."), Float(5.0), Str(".txt")]`



\### Regex Pattern Used

```regex

(\[+-]?\\d\*\\.?\\d+(?:\[eE]\[+-]?\\d+)?|\\d+|\[^\\d]+)

```

\* Matches signed/unsigned integers, floats (with optional scientific notation), and non-digit chunks.

\* Captures everything, preserving order.



\## 3. The Comparison Logic

Once strings are segmented, comparison is \*\*element-by-element\*\*:



1\. Compare `Segment` A vs `Segment` B.

2\. If types match (`Int` vs `Int`), compare numerically.

3\. If types differ (`Int` vs `Str`), Rust/Python has a defined type ordering. In `natsort`, `Int` < `Float` < `Str`.

4\. If all compared elements are equal, the shorter list sorts first.

5\. If lengths differ, the first mismatch determines order.



\*\*Example Comparison:\*\*

`"file2.txt"` vs `"file10.txt"`

\* `Str("file")` == `Str("file")` → Tie

\* `Int(2)` < `Int(10)` → \*\*`"file2.txt"` wins\*\* ✅



\## 4. The `ns` Flags (Bitmask Configuration)

`natsort` uses a bitmask enum to modify behavior. These map directly to Rust flags.



| Flag | Hex | Behavior |

|------|-----|----------|

| `ns.REAL` | `0x10` | Parse signed floats (`-3.0`, `+5.10`). Without it, negative signs are treated as text. |

| `ns.IGNORECASE` | `0x02` | Lowercase strings before comparison (`"B"` == `"b"`). |

| `ns.NUMBER` | `0x20` | Force all text segments to be parsed as numbers. Fallback to `0.0` on failure. |

| `ns.LOCALE` | `0x04` | Use locale-aware collation (e.g., `ñ` sorts near `n`). |

| `ns.FIXED\_EXPONENT` | `0x40` | Align floating point exponents for fair comparison (`1e2` vs `100`). |



\## 5. The Key Generator Pattern (`natsort\_keygen`)

Python's `sorted()` accepts a `key` function. `natsort` exposes `natsort\_keygen()` to create a reusable key function with custom flags.



\*\*Python:\*\*

```python

key\_func = natsort.natsort\_keygen(ns.IGNORECASE | ns.REAL)

sorted(data, key=key\_func)

```



\*\*Rust Adaptation:\*\*

Rust doesn't have first-class closures with captured state in the same way. Instead, we return a struct:

```rust

let key\_gen = NatsortKey::new(NsFlags::IGNORECASE | NsFlags::REAL);

let sorted: Vec<\_> = data.iter()

&#x20;   .sorted\_by\_key(|s| key\_gen.key(s))

&#x20;   .cloned()

&#x20;   .collect();

```



\## 6. Advanced Features



\### 6.1 Mixed Types

`natsort` handles lists containing strings, ints, floats, and `None`.

\* `None` sorts first.

\* Numbers sort before strings.

\* Custom `Ord` implementation handles cross-type coercion.



\### 6.2 Recursive Descent

Handles nested lists: `\[\[1, "a"], \[2, "b"], \[1, "b"]]`.

\* Compares element-by-element.

\* Recursively applies natural sort to nested lists.

\* Shorter lists sort first if prefixes match.



\### 6.3 OS Sorting (`os\_sorted`)

Mimics file explorer behavior:

\* Splits paths by directory separators (`/` or `\\`).

\* Applies natural sort to each path component.

\* Handles case-insensitivity and locale rules specific to OS conventions.



\## 7. Edge Cases \& Gotchas

1\. \*\*Empty Strings:\*\* `""` splits to `\[]`. Sorts first.

2\. \*\*Unicode Combining Marks:\*\* `cafe\\u0301` (é) may split unexpectedly. `natsort` relies on Unicode normalization in locale mode.

3\. \*\*Scientific Notation:\*\* `1e10` vs `10000000000`. `ns.REAL` handles parsing; `ns.FIXED\_EXPONENT` aligns them.

4\. \*\*Locale Collation:\*\* `ñ` vs `n`. Requires `unicase` or platform-specific string transformation.

5\. \*\*Mixed Alphanumeric:\*\* `file10.5.txt` → `Int(10)`, `Float(5.0)`. Regex must capture decimals correctly.



\## 8. Validation Strategy

\* \*\*Golden Standard:\*\* The original Python `natsort` tests are the spec.

\* \*\*Parity Test:\*\* Run identical inputs through Python and Rust. Outputs must be byte-identical.

\* \*\*Performance:\*\* Rust should be 10-100x faster due to zero-alloc string handling and compiled regex.



\*\*Reference Implementation:\*\* `SethMMorton/natsort` on GitHub.

\*\*Algorithm Complexity:\*\* O(N log N) for sorting, O(L) per string for segmentation (L = string length).



