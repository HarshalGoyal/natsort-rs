# Benchmark Methodology

## Tool

Criterion.rs 0.5.1 (Rust) via `cargo bench --bench natsort_bench`.

Python reference benchmarks run through the same Criterion harness via PyO3,
ensuring identical hardware/thermal conditions. The Python group uses a
60s measurement window (vs 10s for Rust) so the slower reference benches
(e.g. `os_sorted` ~540 ms/iter) can still collect the full 100 samples.

## Parameters

| Parameter | Value |
|-----------|-------|
| Warm-up time | 0.4 s |
| Measurement time | 10 s (rust) / 60 s (python) |
| Sample size | 100 |
| Confidence level | 95% (default) |

## Metrics

### Median (ms)

Central tendency of measured time across samples. Reported as
`[lower_bound, median, upper_bound]` by Criterion with 95% CI.

### p99 (99th percentile)

Extracted from Criterion's raw sample files (`target/criterion/*/new/sample.json`),
computed per-iteration from cumulative `times`/`iters`, then linearly
interpolated at the 99th percentile. Reported per benchmark as
`rust_p99_ms` / `python_p99_ms`, and top-level as `startup_p99_ms`.
With n=100 samples (Criterion default), p99 is a statistically meaningful
tail-latency estimate; lower sample counts (e.g. 10) would collapse p99
onto the observed max.

### RSS (resident set size)

Peak memory usage measured by polling `/proc/<pid>/status` (VmRSS) while the
`cargo bench` process runs in the background. Reported in KiB as `rss_kb`.
Represents the high-water mark across all benchmarks in a single run — not
per-benchmark.

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

Results written to `bench-metrics/bench.log` (full output) and
`bench-metrics/results.json` (structured, incl. p99).
