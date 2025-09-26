# Performance Measurement Plan

This plan defines how we measure throughput, latency, and memory for the Rust implementation versus the Python CaRT reference. It captures budgets, tooling, and reporting steps so every run produces comparable data.

## Scope & Goals
- Validate that Rust encode, decode, metadata, and detection APIs meet the charter targets (≥10× speedup, <128 KiB extra heap beyond streaming buffers).
- Produce repeatable measurements that can be reproduced locally and in Continuous Integration (CI) using the same commands.
- Record results in a shared ledger so trends stay visible and regressions are caught quickly.

## Baseline Evidence
- Python reference results live at `benchmarks/results/python-baseline.json`. Regenerate with `python3 benchmarks/python_baseline.py --iters 3 --profile unittests --out benchmarks/results/python-baseline.json` after running `scripts/uv_setup.sh` when dependencies drift.
- For implementation parity, compare against the installed reference module under `.venv/lib64/python3.11/site-packages/cart/` (or the equivalent path for your Python version).

## Budgets
| Scenario | Metric | Budget | Notes |
| --- | --- | --- | --- |
| Encode 128 MiB zeros | Throughput | ≥1.5 GiB/s | Measure wall time after warm-up, release build with `--bench` profile. |
| Decode 128 MiB zeros | Throughput | ≥5.5 GiB/s | Single-threaded decode; current implementation buffers payload in memory with reuse plan tracked for M1. |
| Metadata peek | Latency | ≤0.02 ms | Use warmed file handles and reuse buffers. |
| `is_cart` detection | Latency | ≤5 µs | Use fixed-size slice inputs. |
| Any scenario | Peak heap beyond streaming buffer | <128 KiB | Track via `valgrind --tool=dhat` or `jeprof` on stable input. |
| Encode/Decode | Allocation count | No per-chunk heap allocs | Profiler should show buffer reuse; note exceptions explicitly. |

All numbers assume Linux x86_64 release builds compiled with `cargo build --profile release`. If baseline hardware differs, capture CPU model in the ledger entry.

## Tooling & Commands
- **Rust benchmarks:** Use Criterion via `cargo bench` once `cart-benches` wires real harnesses. Until then, dry-run the command to ensure the binary builds.
- **Python baseline:** `python3 benchmarks/python_baseline.py --profile unittests` (see Baseline Evidence).
- **Throughput measurements:** Prefer Criterion’s throughput API. For ad-hoc runs, wrap encode/decode calls in `hyperfine --warmup 3 --runs 10`.
- **CPU profiling:** `perf stat -d -- cargo bench --bench <name>` for aggregate counters; `perf record` for hotspot analysis.
- **Memory/allocation profiling:** `valgrind --tool=dhat target/release/<bench>` or `jemalloc`’s `MALLOC_CONF=prof:true` with `jeprof`. Document command, sample size, and peak figures.
- **Allocation guard:** For quick checks, build with `RUSTFLAGS="-Z print-type-sizes"` on nightly or use `cargo instruments --template allocations` (macOS fallback) when Linux tooling is unavailable.

## Measurement Procedure
1. Build release artifacts: `cargo build --workspace --profile release`.
2. Warm caches with one throwaway run of each benchmark or CLI invocation.
3. Execute the benchmark command three times (or Criterion’s default resampling) to stabilize variance. Record mean, median, standard deviation when available.
4. Capture environment metadata (CPU model, core count, governor state). Include the output of `scripts/uv_setup.sh --print-env` when applicable.
5. For allocation checks, run `valgrind --tool=dhat <command>` or `MALLOC_CONF="prof:true" <command>` and note peak bytes plus live allocations.
6. Update the ledger (see below) with raw numbers, units, tool versions, and links to full reports (`perf.data`, Criterion output, profiler HTML files).
7. Compare against the latest Python baseline. Document deltas and flag regressions >5% in throughput or >10% in memory.

## Result Ledger Template
Record each measurement snapshot in `docs/performance-ledger.md` (create if missing) using the following table:

```
| Date | Scenario | Tool | Build Flags | Bytes | Mean Time (s) | Throughput (MiB/s) | Peak Heap (KiB) | Delta vs Python | Notes |
| ---- | -------- | ---- | ----------- | ----- | ------------- | ------------------ | --------------- | --------------- | ----- |
```

Attach additional detail (e.g., Criterion’s `benchmark.json`) under `benchmarks/results/` and reference it in the Notes column.

## Reporting & Review Cadence
- Add new ledger entries after significant changes (feature landing, optimization, dependency bump).
- Summarize metrics in milestone review meetings and capture approvals or follow-up actions in `docs/decision-log.md`.
- Raise an issue if budgets are exceeded; include profiler evidence and a remediation proposal.

## Risks & Mitigations
- **Environment drift:** Pin tool versions via `scripts/uv_setup.sh` and Nix flake. Capture tool versions in each ledger entry.
- **Variance between hosts:** Record CPU topology and rerun on the reference machine before declaring regressions.
- **Profiler overhead:** For dhat or valgrind, note that timings are distorted; collect separate wall-clock numbers without instrumentation.
- **Data set skew:** Reuse the shared fixtures and document any custom datasets alongside their provenance.

## Next Steps
- Wire Criterion benchmarks in `cart-benches` once encode/decode land.
- Automate ledger updates in CI after nightly runs, guarded to avoid flaky noise.
- Add Service Level Objective (SLO) thresholds only after real-world workloads are agreed upon; avoid speculative targets.
