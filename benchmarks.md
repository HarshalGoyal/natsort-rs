# Performance Benchmarks

Benchmarks run on WSL2/Ubuntu 22.04, Rust 1.97, Criterion.rs `cargo bench`.
Python reference measured from the same harness (Python natsort 8.4.0 via PyO3).

See [bench-metrics/methodology.md](bench-metrics/methodology.md) for detailed methodology.

## Rust vs Python — Measured Speedups

All times are Criterion median (ms) for 10k–20k items.

| Benchmark | Rust (ms) | Python (ms) | Speedup |
|-----------|-----------|-------------|---------|
| files/natsorted_default | 15.16 | 112.54 | **7.4× faster** |
| paths/natsorted_default | 9.38 | 80.29 | **8.6× faster** |
| floats/natsorted_default | 8.22 | 53.70 | **6.5× faster** |
| paths/os_sorted | 87.92 | 542.38 | **6.2× faster** |

## Additional Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| Startup overhead | ~2 ms | Measured by sorting 1 element (process init + alloc) |
| Peak RSS | measured per run | Reported in `bench-metrics/results.json` |

See `bench-metrics/results.json` for structured data with all benchmarks and metrics.

## Rust Algorithm Breakdown

### natsorted (by dataset × flag)

| Dataset | default | real | ignorecase | path |
|---------|---------|------|------------|------|
| files (20k) | 15.16 ms | 15.89 ms | 14.72 ms | 17.57 ms |
| floats (10k) | 8.22 ms | 7.63 ms | 8.32 ms | 8.06 ms |
| paths (10k) | 9.38 ms | 10.18 ms | 9.69 ms | 18.58 ms |

### realsorted

| Dataset | Time |
|---------|------|
| files (20k) | 15.46 ms |
| floats (10k) | 7.45 ms |
| paths (10k) | 9.87 ms |

### os_sorted

| Dataset | Time |
|---------|------|
| paths (10k) | 87.92 ms |

## Why os_sorted is Slower

os_sorted splits paths into OS components and applies locale-aware comparison
(ICU/CF locale). The per-character normalization and extension handling adds
roughly 10× overhead vs. plain natsorted.

## Methodology

- Criterion.rs: warm-up 0.4s, measurement 10s, sample size 10
- Each dataset is randomly generated once (seeded via fastrand)
- Python benchmarks use PyO3 to call `natsort.natsorted()` / `natsort.os_sorted()`
  from within the same Criterion harness, ensuring identical hardware/thermal
  conditions for fair comparison
- Startup overhead measured by sorting 1 element (isolates process init + alloc)
- Peak RSS measured via `/proc/PID/status` VmRSS polling during bench run

## Running Locally

```bash
source ../python_src/venv/bin/activate
./scripts/bench.sh              # rust only (python if natsort importable)
./scripts/bench.sh --clean      # clear criterion cache for fresh baseline
./scripts/bench.sh --group rust # only rust/
```

Outputs:
- `bench-metrics/bench.log` — full Criterion output
- `bench-metrics/results.json` — structured data (median, RSS, startup, speedup ratios)
