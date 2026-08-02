#!/usr/bin/env python3
"""Merge per-benchmark p99 (ms) into bench-metrics/results.json.

Reads Criterion's raw samples from target/criterion/<bench>/new/sample.json
and injects rust_p99_ms / python_p99_ms into each benchmark entry.

Usage: bench_p99.py <results.json> <criterion_dir>
"""

import json
import os
import sys


def p99_from_sample(path):
    data = json.load(open(path))
    iters = data["iters"]
    times = data["times"]
    per = []
    for i in range(len(iters)):
        if i == 0:
            n, t = iters[0], times[0]
        else:
            n, t = iters[i] - iters[i - 1], times[i] - times[i - 1]
        if n > 0:
            per.append(t / n)
    per.sort()
    n = len(per)
    if n == 0:
        return None
    idx = 0.99 * (n - 1)
    lo = int(idx)
    hi = min(lo + 1, n - 1)
    frac = idx - lo
    return per[lo] * (1 - frac) + per[hi] * frac


def dir_to_id(name, prefix):
    rest = name[len(prefix):]
    if "_" in rest:
        dataset, _, alg = rest.partition("_")
        return f"{dataset}/{alg}"
    return rest


def main():
    results_path, criterion_dir = sys.argv[1], sys.argv[2]
    with open(results_path) as f:
        results = json.load(f)

    p99 = {}
    for entry in sorted(os.listdir(criterion_dir)):
        sample = os.path.join(criterion_dir, entry, "new", "sample.json")
        if not os.path.isfile(sample):
            continue
        if entry.startswith("rust_"):
            v = p99_from_sample(sample)
            p99.setdefault(dir_to_id(entry, "rust_"), {})["rust_p99_ms"] = v / 1e6 if v else None
        elif entry.startswith("python_"):
            v = p99_from_sample(sample)
            p99.setdefault(dir_to_id(entry, "python_"), {})["python_p99_ms"] = v / 1e6 if v else None

    for bench_id, entry in results.get("benchmarks", {}).items():
        if bench_id in p99:
            entry.update(p99[bench_id])

    if "startup" in p99 and "rust_p99_ms" in p99["startup"]:
        results["startup_p99_ms"] = p99["startup"]["rust_p99_ms"]

    with open(results_path, "w") as f:
        json.dump(results, f, indent=2)
        f.write("\n")


if __name__ == "__main__":
    main()
