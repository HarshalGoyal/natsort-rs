#!/usr/bin/env bash
# Benchmark the natsort-rs algorithms (Rust + Python reference) and summarize.
#
# Usage:
#   ./scripts/bench.sh                 # run all Rust benches (skips python if natsort is missing)
#   ./scripts/bench.sh --group rust    # only the rust/ benchmarks
#   ./scripts/bench.sh --group python  # only the python/ reference benchmarks
#   ./scripts/bench.sh --clean         # clear target/criterion before running
#
# The Python reference group needs `natsort` importable (venv active), e.g.:
#   source ../python_src/.venv/bin/activate
#
# Full output goes to bench-metrics/bench.log (overwritten); summary +
# Rust-vs-Python table are appended to it. bench-metrics/results.json is
# generated with structured data.

set -euo pipefail

cd "$(dirname "$0")/.."

LOG=bench-metrics/bench.log
RESULTS_JSON=bench-metrics/results.json

ARG_GROUP=""
for arg in "$@"; do
    case "$arg" in
        --clean)
            echo "Removing target/criterion for a clean baseline..." >&2
            rm -rf target/criterion
            ;;
        --group=*) ARG_GROUP="${arg#*=}" ;;
        --group)   echo "--group requires a value: rust|python" >&2; exit 1 ;;
        *)         echo "Unknown option: $arg" >&2; exit 1 ;;
    esac
done

case "$ARG_GROUP" in
    rust|python|"") ;;
    *) echo "Unknown group: $ARG_GROUP (expected rust|python)" >&2; exit 1 ;;
esac

# The python group panics if `natsort` isn't importable. Detect first so we
# never abort the whole run mid-way.
group="$ARG_GROUP"
if [ "$group" != "rust" ] && ! python3 -c "import natsort" >/dev/null 2>&1; then
    if [ "$group" = "python" ]; then
        echo "ERROR: python 'natsort' not importable (venv active?)." >&2
        exit 1
    fi
    echo "WARNING: python 'natsort' not importable (venv active?) -> rust only." >&2
    group="rust"
fi

ARGS=(--warm-up-time 0.4 --measurement-time 10 --sample-size 10)
GROUP_FILTER=""
case "$group" in
    rust)   GROUP_FILTER="rust" ;;
    python) GROUP_FILTER="python" ;;
esac
[ -n "$GROUP_FILTER" ] && ARGS+=("$GROUP_FILTER")

echo "Running: cargo bench --bench natsort_bench ${ARGS[*]}" >&2

# ── Run cargo bench in background, poll RSS ──────────────────────────
MAX_RSS_KB=0

cargo bench --bench natsort_bench -- "${ARGS[@]}" >"$LOG.raw" 2>&1 &
BENCH_PID=$!

while kill -0 "$BENCH_PID" 2>/dev/null; do
    if [ -f "/proc/$BENCH_PID/status" ]; then
        rss=$(awk '/^VmRSS:/{print $2}' "/proc/$BENCH_PID/status" 2>/dev/null || echo 0)
        if [ "${rss:-0}" -gt "${MAX_RSS_KB:-0}" ]; then
            MAX_RSS_KB=$rss
        fi
    fi
    sleep 0.2
done
wait "$BENCH_PID" || true

# ── Parse output: summary table ─────────────────────────────────────
tr '\r' '\n' < "$LOG.raw" | tee "$LOG" | awk -v rss="$MAX_RSS_KB" '
function median(s,    a,t,i,k,num){
    a=s; sub(/^.*\[/,"",a); sub(/\].*$/,"",a)
    gsub(/[[:space:]]+/," ",a); sub(/^ /,"",a)
    k=0; n=split(a,t," ")
    for(i=1;i<=n;i++) if(t[i] ~ /^[0-9]+(\.[0-9]+)?$/) num[++k]=t[i]+0+0
    return num[2]
}

/^(rust|python)\// {
    benches++
    side = ($0 ~ /^rust\//) ? "R" : "P"
    curid=$0
    sub(/^(rust|python)\//,"",curid); sub(/[[:space:]]*time:.*$/,"",curid); sub(/[[:space:]]+$/,"",curid)
    curs=side
    print
    if (index($0,"time:")>0) { m=median($0); if(side=="R") tr[curid]=m; else tp[curid]=m }
    next
}

/^[[:space:]]*time:/ {
    print
    if (curid!="") { m=median($0); if(curs=="R") tr[curid]=m; else tp[curid]=m }
    next
}

/^[[:space:]]*change:/ { print; next }

/Performance has improved/          { improved++; print; next }
/No change in performance detected/ { unchanged++; print; next }
/Performance has regressed/         { regressed++; next }
/Change within noise threshold/     { noise++; print; next }
/Warning:/                          { warnings++; print; next }
/Found [0-9]+ outliers/             { outliers++; print; next }

END {
    # ── Rust vs Python table ──
    printf "\n================ Rust vs Python ================\n"
    printf "%-30s %-12s %-12s %s\n", "benchmark", "Rust (ms)", "Python (ms)", "ratio"
    for (k in tp) {
        if (!(k in tr)) continue
        r=tr[k]; p=tp[k]; ratio=r/p
        label = (ratio>=1) ? sprintf("Rust %.2fx slower", ratio) \
                           : sprintf("Rust %.2fx faster", p/r)
        printf "%-30s %-12s %-12s %s\n", k, sprintf("%.2f",r), sprintf("%.2f",p), label
    }

    # ── Startup ──
    startup = (tr["startup"] > 0) ? tr["startup"] : 0

    # ── Summary ──
    printf "\n================ Benchmark Summary ================\n"
    printf "Benchmarks           : %d\n", benches
    printf "Improved             : %d\n", improved
    printf "No change            : %d\n", unchanged
    printf "Noise threshold      : %d\n", noise
    printf "Regressed            : %d\n", regressed
    printf "Warnings             : %d\n", warnings
    printf "Outlier reports      : %d\n", outliers
    printf "Peak RSS             : %d KiB\n", rss
    printf "Startup overhead     : %.2f ms\n", startup
    printf "Full log saved to    : '"$LOG"'\n"
    printf "===================================================\n"
}' | tee -a "$LOG"

# ── Generate bench-metrics/results.json ─────────────────────────────
mkdir -p bench-metrics

awk -v rss="$MAX_RSS_KB" '
function median(s,    a,t,i,k,num){
    a=s; sub(/^.*\[/,"",a); sub(/\].*$/,"",a)
    gsub(/[[:space:]]+/," ",a); sub(/^ /,"",a)
    k=0; n=split(a,t," ")
    for(i=1;i<=n;i++) if(t[i] ~ /^[0-9]+(\.[0-9]+)?$/) num[++k]=t[i]+0+0
    return num[2]
}

/^(rust|python)\// {
    curid=$0
    sub(/^(rust|python)\//,"",curid); sub(/[[:space:]]*time:.*$/,"",curid); sub(/[[:space:]]+$/,"",curid)
    curs = ($0 ~ /^rust\//) ? "R" : "P"
    if (index($0,"time:")>0) {
        m=median($0)
        if (curs=="R") tr[curid]=m; else tp[curid]=m
    }
    next
}
/^[[:space:]]*time:/ {
    if (curid!="") { m=median($0); if(curs=="R") tr[curid]=m; else tp[curid]=m }
    next
}
END {
    startup = (tr["startup"] > 0) ? tr["startup"] : 0
    printf "{\n"
    printf "  \"methodology\": \"Criterion.rs 0.5.1, warm-up 0.4s, measurement 10s, sample-size 10\",\n"
    printf "  \"rss_kb\": %d,\n", rss
    printf "  \"startup_ms\": %.2f,\n", startup
    printf "  \"benchmarks\": {\n"
    first = 1
    # Rust medians
    for (k in tr) {
        if (k == "startup") continue
        if (!first) printf ",\n"
        first = 0
        if (k in tp) {
            r=tr[k]; p=tp[k]; ratio=p/r
            printf "    \"%s\": {\"rust_ms\": %.2f, \"python_ms\": %.2f, \"speedup\": %.2f}", k, r, p, ratio
        } else {
            printf "    \"%s\": {\"rust_ms\": %.2f}", k, tr[k]
        }
    }
    # Python-only (no Rust pair)
    for (k in tp) {
        if (k in tr) continue
        if (!first) printf ",\n"
        first = 0
        printf "    \"%s\": {\"python_ms\": %.2f}", k, tp[k]
    }
    printf "\n  }\n}\n"
}
' <(tr '\r' '\n' < "$LOG.raw") > "$RESULTS_JSON"

rm -f "$LOG.raw"
echo "" >&2
echo "Results written to $RESULTS_JSON" >&2
