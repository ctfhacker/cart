# CART-RS Architecture

## Goals & Constraints
- Deliver a Rust crate and CLI that are byte-identical with the Python CaRT v1 reference while hitting ≥10× throughput on representative encode/decode workloads and keeping peak memory bounded to <128 KiB beyond stream buffers.
- Stay within safe Rust (no `unsafe`), reuse existing CaRT v1 on-disk format, and support Linux x86_64 first.
- Provide ergonomic sync APIs and a CLI for pipelines, plus observability hooks grounded in the Performance Mantra (measure first, make cost visible, avoid hidden allocations).
- Integrate with existing Python fixtures, benchmarks, and decision logs documented under `docs/` and `benchmarks/`.

## Option Exploration
- **Option A – Single Crate + Binary Target**: One Cargo package with `lib` and `bin`, shared modules for codec, metadata, CLI parsing. Low setup cost, but mixes CLI dependencies into the library build, complicating embedding and testing.
- **Option B – Workspace (`cart-core`, `cart-cli`, `cart-benches`)**: Core library isolated from CLI, optional bench crate mirrors Python suite, shared test fixtures via dev-dependency. Enables clean API surface, faster library builds, and dedicated perf tooling. Slightly more scaffolding up front.
- **Option C – Thin Rust Wrapper Over Python via FFI**: Use PyO3 or C-ABI to call Python implementation. Minimal Rust coding, but fails performance goals, adds runtime dependency on Python, and complicates distribution.
- **Decision**: Adopt **Option B**. It balances modularity and performance by separating concerns, keeps CLI optional for embedders, and gives a home for perf/interop tooling without entangling production APIs with binary requirements.

## Architecture Overview
```
+---------------------------+      +------------------+
| cart-cli (bin crate)      |----->| cart-core (lib)  |
| - clap-based CLI          |      | - codec modules  |
| - logging/metrics init    |<-----| - format & RC4   |
| - file I/O orchestration  |      | - metadata utils |
+---------------------------+      | - trait-based IO |
             |                     +------------------+
             v                             |
     +----------------+                    v
     | bench harness  | --------> +------------------+
     | (criterion +   |           | fixtures & tests |
     | python parity) |           +------------------+
     +----------------+                   
```
- Workspace root owns shared config (MSRV, lint, formatting) and fixture assets.
- `cart-core` exposes streaming encode/decode, metadata peek, and detection APIs over `Read`/`Write` traits.
- `cart-cli` wires parse/emit, uses `cart-core` engines, and adds validation/round-trip modes required by charter.
- Benchmark harness consumes `cart-core` to run per-scenario metrics against JSON baselines in `benchmarks/`.

## Simplicity Guardrail
- Never future-proof: implement only the behaviors required by the current milestone. Stubs are allowed solely for features we are actively landing next; remove or avoid placeholders for speculative work. When direction changes, revisit the design instead of layering abstractions in advance.

## Component Responsibilities & Trust Boundaries
- `cart-core::format`: Parse and emit mandatory header/footer structs, validate magic/version, expose serde-compatible optional metadata JSON handling. Trust boundary: raw byte streams.
- `cart-core::crypto`: Wrap ARC4 implementation (e.g. `rc4` crate) with key derivation/validation; no plaintext keys leave the crate. Trust boundary: key material.
- `cart-core::codec`: Pack/unpack pipelines using `Read`/`Write`; current decode path reads the encrypted payload into memory before inflating with zlib, with buffer reuse planned in the next iteration. Trust boundary: plaintext payloads.
- `cart-core::metadata`: Fast path to inspect optional header/footer without full decode; uses `Peeker` abstraction over `Read + Seek` or buffered slices.
- `cart-core::detect`: Lightweight function to confirm CaRT magic without allocating; accepts `Read` or `[u8; N]` slice via generics.
- `cart-cli`: Clap command graph (`encode`, `decode`, `inspect`, `verify`), file system interactions, progress reporting, structured logs/metrics emission. Trust boundary: user inputs, filesystem paths.
- Bench harness: Criterion-based micro/meso benchmarks, JSON exporter to compare to Python baseline, nightly perf guardrails.

## Interfaces & Data Contracts
- Library entry points (sync first, async later if justified):
  - `fn encode<R: Read, W: Write>(input: R, output: W, opts: EncodeOptions) -> Result<EncodeReport>`
  - `fn decode<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<DecodeReport>` (reads the encrypted payload into memory today; streaming decode with reuse pools is tracked for M1).
  - `fn metadata<R: Read + Seek>(input: &mut R) -> Result<MetadataView>`
  - `fn is_cart<R: Read>(input: &mut R) -> Result<bool>`
  - `fn round_trip<R: Read + Seek>(input: &mut R, writer: W, opts: RoundTripOptions) -> Result<RoundTripReport>` (CLI-only helper).
- Options structs clarify allocation budgets (buffer reuse flags, digest selection, ARC4 key override) and versioning via `#[non_exhaustive]` to allow extension.
- Data contract: Mandatory header `<4s, u16, u64, [u8;16], u64>` (magic, version, reserved, ARC4 key, optional header len); optional header/footer serialized as canonical JSON (sorted keys, compact separators) and RC4-encrypted using the negotiated key. Footer ends with mandatory trailer `<4s, u64, u64, u64>` (magic, reserved, opt footer len, payload crc/digest len).
- Error types: `CartError` enum covering I/O, format, cryptography, digest mismatch, version mismatch; implements `std::error::Error`.
- CLI contracts: JSON/text output for `inspect`, exit codes (0 success, 2 format error, 3 verification mismatch), deterministic stdout/stderr separation for scripting.

## Performance & Resource Budgets
- Throughput targets (per 128 MiB scenario, derived from `benchmarks/results/python-baseline.json`):
  - Encode ≥1.5 GiB/s (≤0.082 s per 128 MiB zeros) under release build on target hardware.
  - Decode ≥5.5 GiB/s (≤0.042 s per 128 MiB zeros) leveraging streaming reads.
  - Metadata-only inspect ≤0.02 ms; `is_cart` ≤5 µs.
- Allocation budget: reuse a single 64 KiB block buffer for compression/decompression; heap usage stays <128 KiB beyond stream buffers; avoid per-chunk allocations in hot path.
- CPU budget: keep encode pipeline single-threaded but ready for SIMD (e.g. `crc32fast`) if profiling proves bottlenecks; wall clock improvements measured via Criterion with warm cache.
- Observability: record wall time, CPU time, bytes processed, and allocation counts via `cart-perf` counters in CLI verbose mode; emit JSON to extend perf ledger.

## Security, Privacy, and Compliance
- ARC4 key handling stored in stack-owned `[u8;16]`, zeroized after use via `zeroize` crate.
- Optional metadata and footer JSON validated for size (<64 KiB) and UTF-8; reject oversized inputs to avoid DoS.
- No new cryptography; document legacy RC4 posture in CLI help and README.
- Secrets (keys) never logged; debug traces gate behind explicit `--trace` flag that redacts sensitive fields.
- Source of truth for digests matches Python defaults (MD5/SHA1/SHA256/length) with pluggable digester trait for extensibility.

## Observability & Operations
- Structured logs via `tracing` with CLI `--log-format` (`text`/`json`); default info-level with timing spans for encode/decode.
- Metrics adapter trait in `cart-core` (no-op by default) so embedders can wire Prometheus or OpenTelemetry counters without extra deps.
- Bench harness writes metrics to `benchmarks/results/` with timestamped snapshots; CI diff warns on regression beyond ±5%.
- Error telemetry surfaces digests and offsets to aid debugging; include optional hex dumps with size guard.

## Sequencing & Rollout Plan
1. Scaffold Cargo workspace, `cart-core` skeleton with header/footer parsing and CLI stub; integrate Python fixtures as integration tests.
1. Implement decode pipeline (easier to validate) with metadata inspection; confirm byte parity using fixtures.
1. Add encode with digest computation, buffer reuse, and optional metadata; cross-verify round-trips vs Python.
1. Build CLI commands incrementally (`inspect`, `decode`, `encode`, `verify`), wiring structured logging and exit codes.
1. Introduce benchmark harness, capture initial Rust baseline, set up perf regression guard.
1. Harden observability, document APIs, finalize MSRV + release checklist.

## Risks, Assumptions, and Mitigations
- **ARC4 crate performance/maintenance**: evaluate candidates (`rc4`, `arc4`) early; keep abstraction to swap if needed.
- **JSON canonicalization drift**: enforce sorted keys in encoder, re-use serde tooling with deterministic serializer; add regression tests.
- **Benchmark parity**: assumption that Python fixtures stay stable; mitigate by vendoring necessary fixtures and version pinning in CI.
- **Large file streaming**: ensure `Read` implementations that do not support `Seek` are handled; provide fallback requiring temporary storage with explicit opt-in.
- **MSRV alignment**: pending decision (#Q1); block release until confirmed.

## Assumptions & Open Questions
- MSRV and memory ceiling pending stakeholder input (`docs/open-questions.md#L6`).
- ARC4 key management policy assumed static; confirm if rotation or override modes required by operators.
- Distribution strategy for CLI (crate vs prebuilt binaries) still TBD; track in decision log update.
