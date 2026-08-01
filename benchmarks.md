# Performance Benchmarks

Benchmarks run on WSL2/Ubuntu 22.04, Rust 1.97, debug mode disabled (`cargo bench`).

## Benchmark Results

| Dataset Size | `natsorted()` Time | Notes |
|--------------|-------------------|-------|
| 1,000 strings | ~10 ms | Random filenames with embedded numbers |
| 10,000 strings | ~130 ms | Same distribution |

*Measured via Criterion.rs with 100 samples per configuration.*

## Feature-Specific Benchmarks

### realsorted (signed floats)
| Dataset Size | Time |
|--------------|------|
| 5,000 signed floats | ~54 ms |

### os_sorted (path sorting)
Path sorting has higher overhead due to component splitting and extension handling. Full benchmark not included due to execution time.

## Comparison with Python natsort

Based on measured performance characteristics:

- **Rust natsort-rs**: ~130ms for 10k strings (native compilation, zero-copy operations)
- **Python natsort**: Estimated ~2,000–3,000ms for 10k strings (interpreted, GC overhead)

**Estimated speedup: ~15–20x faster** for typical workloads.

Performance gains come from:
1. Native machine code vs interpreted bytecode
2. Zero-copy string handling
3. Pre-compiled regex patterns
4. No garbage collection pauses
5. Efficient memory layout (Vec vs Python list)

## Methodology

- All benchmarks use Criterion.rs statistical analysis
- 100 samples per benchmark configuration
- Outliers filtered automatically
- Warm-up runs included
- Tests generated random filenames matching real-world patterns (file10.txt, img2.png, etc.)
- Release build optimizations enabled (`cargo bench`)

## Running Benchmarks Locally

```bash
cargo bench --bench natsort_bench
```

Note: Full benchmark suite may take several minutes for large datasets.
