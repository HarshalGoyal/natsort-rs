# Benchmark Methodology

## Tool

Criterion.rs 0.5.1 (Rust) via `cargo bench --bench natsort_bench`.

Python reference benchmarks run through the same Criterion harness via PyO3,
ensuring identical hardware/thermal conditions.

## Parameters

| Parameter | Value |
|-----------|-------|
| Warm-up time | 0.4 s |
| Measurement time | 10 s |
| Sample size | 10 |
| Confidence level | 95% (default) |

## Metrics

### Median (ms)

Central tendency of measured time across samples. Reported as
`[lower_bound, median, upper_bound]` by Criterion with 95% CI.

### p99 (99th percentile)

Extracted from Criterion's raw sample files (`target/criterion/*/new/sample.json`).
With n=10 samples, p99 approximates the worst-case measurement. Useful for
understanding tail latency characteristics.

### RSS (resident set size)

Peak memory usage measured via `/usr/bin/time -v` wrapping the entire
`cargo bench` process. Reported in KiB. Represents the high-water mark
across all benchmarks in a single run — not per-benchmark.

### Startup time

Measured by the `rust/startup` benchmark, which sorts a single element.
This isolates process startup, library initialization, and allocator setup
from actual sort work. The median of this benchmark IS the startup overhead.

## Datasets

| Dataset | Size | Description |
|---------|------|-------------|
| files | 20,000 | Random filenames with embedded integers (file001.txt, img234.png, item007.txt) |
| floats | 10,000 | Signed float strings (-12.3, 45.6, 0.1) |
| paths | 10,000 | Nested POSIX paths (/var/log/service42/logfile9999.gz) |

Generated once via `fastrand` (seeded RNG), reused across runs.

## Out of scope

- **Throughput-only benchmarks**: We report latency (ms per sort), not ops/sec.
- **Multi-threaded scaling**: All benchmarks are single-threaded. Rayon is used
  internally for key generation but each Criterion iteration is synchronous.

## Reproducing

```bash
cd natsort-rs
source ../python_src/venv/bin/activate
./scripts/bench.sh --clean
```

Results written to `bench.log` (full output) and `bench/results.json` (structured).
