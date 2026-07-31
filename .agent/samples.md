# 📝 Code Samples: Python → Rust

## 1. Basic Sorting

### Python
```python
import natsort

data = ["file10.txt", "file2.txt", "file1.txt"]
sorted_data = natsort.natsorted(data)
# ['file1.txt', 'file2.txt', 'file10.txt']
```

### Rust
```rust
use natsort::natsorted;

let data = vec!["file10.txt", "file2.txt", "file1.txt"];
let sorted_data = natsorted(&data);
// ["file1.txt", "file2.txt", "file10.txt"]
```

## 2. Key Generator with Flags

### Python
```python
import natsort

data = ["Banana", "apple", "cherry"]
key_func = natsort.natsort_keygen(natsort.ns.IGNORECASE)
sorted_data = sorted(data, key=key_func)
# ['apple', 'Banana', 'cherry']
```

### Rust
```rust
use natsort::{NatsortKey, NsFlags};

let data = vec!["Banana", "apple", "cherry"];
let key_gen = NatsortKey::new(NsFlags::IGNORECASE);
let mut sorted_data = data.clone();
sorted_data.sort_by(|a, b| key_gen.key(a).cmp(&key_gen.key(b)));
// ["apple", "Banana", "cherry"]
```

## 3. Real Numbers (Signed Floats)

### Python
```python
import natsort

data = ["1.5", "-3.2", "10.0", "+2.1"]
sorted_data = natsort.natsorted(data, key=natsort.natsort_keygen(natsort.ns.REAL))
# ['-3.2', '+2.1', '1.5', '10.0']
```

### Rust
```rust
use natsort::{natsorted, NatsortKey, NsFlags};

let data = vec!["1.5", "-3.2", "10.0", "+2.1"];
let key_gen = NatsortKey::new(NsFlags::REAL);
let mut sorted_data = data.clone();
sorted_data.sort_by(|a, b| key_gen.key(a).cmp(&key_gen.key(b)));
// ["-3.2", "+2.1", "1.5", "10.0"]
```

## 4. Mixed Types

### Python
```python
import natsort

data = [10, "2", 3.5, "apple"]
sorted_data = natsort.natsorted(data)
# [2, 3.5, 10, 'apple']
```

### Rust
```rust
use natsort::{Item, natsorted_mixed};

let data = vec![
    Item::Int(10),
    Item::Str("2".to_string()),
    Item::Float(3.5),
    Item::Str("apple".to_string()),
];
let sorted_data = natsorted_mixed(&data);
// [Item::Str("2"), Item::Float(3.5), Item::Int(10), Item::Str("apple")]
```

## 5. OS Path Sorting

### Python
```python
import natsort

paths = ["/dir/file10.txt", "/dir/file2.txt", "/dir/file1.txt"]
sorted_paths = natsort.os_sorted(paths)
# ['/dir/file1.txt', '/dir/file2.txt', '/dir/file10.txt']
```

### Rust
```rust
use natsort::os_sorted;

let paths = vec![
    "/dir/file10.txt",
    "/dir/file2.txt",
    "/dir/file1.txt",
];
let sorted_paths = os_sorted(&paths);
// ["/dir/file1.txt", "/dir/file2.txt", "/dir/file10.txt"]
```

## 6. Unit Test Pattern

### Python (from original test suite)
```python
def test_natsort_integers():
    data = ["4", "8", "2", "10", "3"]
    result = natsort.natsorted(data)
    assert result == ["2", "3", "4", "8", "10"]
```

### Rust
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natsort_integers() {
        let data = vec!["4", "8", "2", "10", "3"];
        let result = natsorted(&data);
        assert_eq!(result, vec!["2", "3", "4", "8", "10"]);
    }
}
```

## 7. Parity Test Pattern

```rust
#[cfg(test)]
mod parity {
    use pyo3::prelude::*;

    #[test]
    fn parity_natsorted_integers() {
        let data = vec!["4", "8", "2", "10", "3"];

        // Python output
        let py_result: Vec<String> = Python::with_gil(|py| {
            let natsort = py.import("natsort").unwrap();
            let sorted = natsort.call_method1("natsorted", (data.clone(),)).unwrap();
            sorted.extract().unwrap()
        });

        // Rust output
        let rs_result = natsorted(&data);

        assert_eq!(py_result, rs_result);
    }
}
```
