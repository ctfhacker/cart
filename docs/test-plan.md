# CART-RS Test Plan

## Scope & Goals
- Prove the Rust workspace (`cart-core`, `cart-cli`, `cart-benches`) meets charter parity requirements with the Python CaRT v1 reference while honoring ≥10× throughput and bounded memory budgets.
- Guard milestones M0–M3 by coupling functional acceptance tests with reproducible performance checks and a perf ledger grounded in the Performance Mantra.
- Keep tests local-first: every command listed below runs from the repo root and maps directly into CI without bespoke YAML logic.

## Environment Profile
- Reference host: Linux x86_64 (Ubuntu 22.04 LTS), 8 physical cores at ≥3.0 GHz, 32 GiB RAM, NVMe storage pinned to performance CPU governor.
- Rust toolchain: `rustc` nightly from `flake.nix` (freeze commit hash in perf ledger), `cargo` with `CARGO_PROFILE_RELEASE_DEBUG=false`, MSRV TBD but gate tests on `rustc >=1.78`.
- Python parity harness: CPython 3.13.5 (matches `benchmarks/results/python-baseline.json`), `uv`-managed virtualenv seeded via `scripts/uv_setup.sh`.
- Disable background services, fix CPU frequency (`sudo cpupower frequency-set --governor performance`), and record `lscpu`, `numactl --hardware`, and `free -m` before each measurement batch.

## Dataset Matrix
| Dataset | Location | Shape & Distribution | Notes |
| --- | --- | --- | --- |
| `small_1B.bin` | `benchmarks/data/small_1B.bin` | Single byte; pessimistic metadata overhead | Used for smoke and allocation leak checks.
| `pattern_100KB.bin` | `benchmarks/data/pattern_100KB.bin` | Repeating ASCII pattern; compressible | Validates metadata + digest paths.
| `random_1MB.bin` | `benchmarks/data/random_1MB.bin` | Uniform random; incompressible | Surface worst-case throughput.
| `random_10MB.bin` | `benchmarks/data/random_10MB.bin` | Uniform random; incompressible | CLI streaming stressor.
| `zeros_128MiB.bin` | `benchmarks/data/zeros_128MiB.bin` | Highly compressible large payload | Anchor dataset for ≥10× throughput goal.
| `python-baseline.json` | `benchmarks/results/python-baseline.json` | Timing/throughput evidence | Source-of-truth perf ledger entry.
| `generated/*.cart` | `benchmarks/artifacts/` (to be created) | Python-encoded CaRT fixtures mirroring above inputs | Regenerated via `python3 -m cart.cli encode` scripts; track commit + command in ledger.

## Acceptance Tests
| ID | Milestone | Goal | Inputs / Dataset | Command (repo root) | Expected Signal | Budget / Gate |
| --- | --- | --- | --- | --- | --- | --- |
| AT-M0-01 | M0 | Workspace compiles & tooling aligned | N/A | `cargo check --workspace` | No failures, no unexpected warnings | Completes ≤90 s on reference host.
| AT-M0-02 | M0 | Stub smoke tests wired | N/A | `cargo test --workspace --lib -- --nocapture` | All stub tests pass; logs show `CartError::Unimplemented` for unimplemented paths | Completes ≤120 s.
| AT-M0-03 | M0 | Python parity harness callable | `cart-py/` | `python3 -m unittest cart-py/unittests/test_cart.py` | Test suite passes | Runtime ≤180 s; failures block milestone.
| AT-M1-01 | M1 | Pack/unpack parity vs Python | `random_1MB.bin`, `generated/random_1MB.cart` | `cargo test -p cart-core parity::round_trip_random_1mb -- --exact` | `assert_eq!` on bytes and metadata | Zero mismatched bytes; diff >0 bytes blocks merge.
| AT-M1-02 | M1 | Metadata peek matches Python | `generated/random_10MB.cart` | `cargo test -p cart-core parity::metadata_view -- --exact` | Header/footer JSON identical | Byte-for-byte JSON equality; allocate ≤128 KiB heap.
| AT-M1-03 | M1 | `is_cart` detection fast path | `generated/*.cart`, synthetic noise | `cargo test -p cart-core detection::is_cart_matrix` | True for CaRT, false for noise | 0 false positives/negatives across matrix.
| AT-M2-01 | M2 | Allocation ceiling enforced | `zeros_128MiB.bin` | `cargo test -p cart-core perf::encode_allocation_guard -- --exact --nocapture` (uses `jemalloc_ctl` or `malloc_stats`) | Peak heap <128 KiB beyond buffer pool | Regression if limit exceeded.
| AT-M2-02 | M2 | Perf ledger serialization stable | `python-baseline.json` | `cargo test -p cart-benches ledger::round_trip` | Ledger entry re-serializes identically | 0 diff in JSON canonical format; ensures reproducibility.
| AT-M3-01 | M3 | CLI encode/decode parity | `random_10MB.bin`, `zeros_128MiB.bin` | `target/release/cart-cli encode --input benchmarks/data/random_10MB.bin --output /tmp/random_10MB.cart --format json --perf-log /tmp/run.json && python3 -m cart.cli decode ...` | CLI output matches Python decode; exit code 0 | Round-trip diff ≤0 bytes; command completes ≤60 s.
| AT-M3-02 | M3 | CLI piping and RC4 key override | `random_1MB.bin` | `cat benchmarks/data/random_1MB.bin | target/release/cart-cli encode --input - --key test-key --output - | target/release/cart-cli decode --input - --output /tmp/out.bin` | Pipeline succeeded; `diff` reports no mismatch; logs redact key | Throughput ≥200 MiB/s; memory stable.
| AT-M3-03 | M3 | Docs doctests compile | N/A | `cargo test --doc` | All documentation examples compile | Must succeed before release tagging.

## Performance Checks
| ID | Scenario | Dataset / Fixture | Metric | Budget / Threshold | Measurement Method |
| --- | --- | --- | --- | --- | --- |
| PC-ENC-128 | Encode streaming zeros | `zeros_128MiB.bin` | Throughput (GiB/s), wall time, allocations | ≥1.5 GiB/s; wall time ≤0.082 s; heap ≤128 KiB | `cargo bench -p cart-benches encode_zeros -- --sample-size 30 --warm-up-time 5` with `criterion`, record wall/CPU, allocations via `dh-atlas` hook.
| PC-DEC-128 | Decode streaming zeros | Python-generated `.cart` | Throughput, wall time | ≥5.5 GiB/s; wall time ≤0.042 s | Same as above with decode bench; reuse buffers; measure via `criterion` custom collector.
| PC-ENC-RAND-10M | Encode incompressible random | `random_10MB.bin` | Throughput, CPU utilization | ≥400 MiB/s; CPU ≤250% (async threads) | `cargo bench -p cart-benches encode_random -- --sample-size 50 --warm-up 10`, monitor `perf stat -e task-clock,cycles,cache-misses`.
| PC-META-SCAN | Metadata-only read | `generated/random_10MB.cart` | Latency per call | ≤0.016 ms median; p99 ≤0.050 ms | `cargo bench ... metadata_view`, 200 samples, ensure buffers reused; measure via `criterion` + `hdrhistogram` export.
| PC-IS-CART | Magic detection | 64-byte slices | Latency, misclassification rate | ≤5 µs median; 0 misclassifications across 10k samples | Microbenchmark using `criterion::BenchmarkGroup`; 5 warm-up runs.
| PC-CLI-PIPE | CLI encode+decode streaming STDIN | `random_10MB.bin` piped | End-to-end throughput, max RSS | ≥300 MiB/s; RSS ≤256 MiB; CPU steady | `/usr/bin/time -v cargo run --release -p cart-cli -- encode ...` piped; run 5 iterations, discard warm-up, average remaining.
| PC-REG-LEDGER | Regression delta vs Python | `python-baseline.json` + latest Rust metrics | % delta throughput, memory variance | Encode/Decode delta within ±5%; memory delta within ±10% | `scripts/compare_perf.py --baseline benchmarks/results/python-baseline.json --candidate benchmarks/results/rust-latest.json` (to be added); record results in ledger.

## Measurement Procedure
- Warm-up: discard first 5 runs or first 5 s (whichever longer) for Criterion benches; for CLI sweeps, perform one dry run before timed iterations.
- Sampling: Criterion default confidence level (95%) with sample size ≥30 for large datasets; collect median, mean, std-dev, and 99th percentile.
- Environment pinning: run under `taskset --cpu-list 0-7` to keep workloads on physical cores; set `MALLOC_CONF="prof:true"` when tracking allocations; disable Turbo Boost if variance >3%.
- Variability control: ensure no Python parity process runs concurrently during Rust measurements; monitor temps via `sensors` and abort if CPU exceeds 85°C.
- Recording: append each significant run to `benchmarks/results/rust-ledger.jsonl` with {commit, scenario, dataset, toolchain, metrics, notes}; cross-link with decision log entries.

## Instrumentation Plan
- Use `tracing` spans (`encode`, `decode`, `metadata`) with fields for bytes processed, buffer reuse hits, and allocation counts; log to stdout (CLI) and JSON file (bench harness).
- Integrate `metered` or custom allocation counters behind `cfg(feature = "alloc-stats")`; expose `CartAllocReport` in bench crate for reuse.
- Criterion benches export CSV + JSON to `target/criterion/**`; copy summaries into perf ledger via `scripts/summarize_criterion.py` (to be added) to avoid manual transcription.
- For CLI runs, wrap commands with `/usr/bin/time -v` and `perf stat -x ,` to capture wall, CPU, faults, cache misses; attach outputs to ledger entry.
- Maintain before/after diff by storing Python-produced `.cart` artifacts alongside Rust outputs and comparing via `sha256sum`; keep hashed evidence in `benchmarks/artifacts/checksums/`.

## Regression Policy
- Any acceptance test failure blocks merge; no waivers without decision log entry referencing mitigation date.
- Performance checks enforce gates: encode/decode regressions >5% or memory/latency breaches trigger `perf-regression` label and require rollback or approved mitigation plan.
- Perf ledger is authoritative: CI must compare latest run against last known good; if ledger missing entry for commit, merge blocked until recorded.
- Budget adjustments need stakeholder approval and documented rationale referencing charter metrics; update this plan and decision log simultaneously.

## Example Skeletons
```rust
// Unit (cart-core/src/lib.rs tests)
#[test]
fn encode_returns_unimplemented_for_stub() {
    let err = cart_core::encode(&[][..], &mut Vec::new(), Default::default()).unwrap_err();
    assert!(matches!(err, CartError::Unimplemented));
}

// Integration parity test (tests/parity.rs)
#[test]
fn round_trip_random_1mb_matches_python() {
    let input = std::fs::read("benchmarks/data/random_1MB.bin").unwrap();
    let py_cart = std::fs::read("benchmarks/artifacts/random_1MB.cart").unwrap();
    let rust_cart = encode_with_rust(&input);
    assert_eq!(py_cart, rust_cart);
    let decoded = decode_with_rust(&rust_cart);
    assert_eq!(input, decoded);
}

// Criterion microbenchmark (benches/encode.rs)
fn encode_zeros(c: &mut Criterion) {
    let data = std::fs::read("benchmarks/data/zeros_128MiB.bin").unwrap();
    c.bench_function("encode_zeros", |b| {
        b.iter(|| encode_with_reuse(&data));
    });
}

// End-to-end CLI sweep (docs/example.sh)
# Warm run
cargo run --release -p cart-cli -- encode --input benchmarks/data/random_10MB.bin --output /tmp/out.cart
# Timed runs
/usr/bin/time -v bash -c 'for i in {1..5}; do \
  cat benchmarks/data/random_10MB.bin | target/release/cart-cli encode --input - --output /tmp/out.cart && \
  target/release/cart-cli decode --input /tmp/out.cart --output /tmp/out.bin; done'
```
