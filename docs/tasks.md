# CART-RS Tasks

## Dependency Plan
- Introduce `clap` (MIT OR Apache-2.0) for CLI argument parsing; owner: CLI maintainer; pin to 4.x series via `cargo update -p clap --precise`; rollback by isolating CLI behind a feature flag and reverting to manual `std::env::args` parsing.
- Introduce `serde` and `serde_json` (MIT OR Apache-2.0) for metadata serialization; owner: core maintainer; pin to compatible 1.0.x releases; rollback by swapping to manual JSON encoding with a documented fixture contract.
- Introduce `rc4` (MIT OR Apache-2.0) for ARC4 cipher; owner: crypto lead; pin to an exact version; rollback path is an internal reference implementation gated behind the same API.
- Introduce `crc32fast` and `digest` (MIT OR Apache-2.0) for digests; owner: performance lead; pin exact versions; rollback uses Rust stdlib hashing plus slower fallback helpers.
- Introduce `criterion` (Apache-2.0) as a dev-dependency for benchmarks only; owner: performance lead; pin exact version; rollback by falling back to Python-only benchmarking while retaining the ledger format.

## Dependency Map
| Task | Depends On | Shared Risks |
| --- | --- | --- |
| CART-M0-01 | docs/architecture.md, docs/roadmap.md (M0) | Toolchain drift, missing MSRV decision |
| CART-M0-02 | CART-M0-01, benchmarks/results/python-baseline.json | Unconfirmed budgets, stakeholder alignment |
| CART-M0-03 | CART-M0-01, cart-py fixtures | Python environment availability |
| CART-M1-01 | CART-M0-01, CART-M0-03, docs/roadmap.md (M1) | ARC4 correctness, fixture coverage |
| CART-M1-02 | CART-M1-01 | Digest parity, buffer reuse correctness |
| CART-M1-03 | CART-M0-02, CART-M1-01 | Metadata corner cases, measurement tooling drift |
| CART-M2-01 | CART-M1-02, CART-M1-03 | Profiling variance, dev host differences |
| CART-M2-02 | CART-M2-01 | Regression risk from optimizations |
| CART-M2-03 | CART-M2-01 | Allocation tracking blind spots |
| CART-M3-01 | CART-M1-02, CART-M1-03 | CLI UX ambiguity, file-system variance |
| CART-M3-02 | CART-M3-01 | Docs drift, observability gaps |
| CART-M3-03 | CART-M3-02, docs/decision-log.md | Release infra readiness, dependency licenses |

## Task List

- **Completion Check:** When a task changes code or tests, run `./ci/lint.sh` and ensure it passes before marking the task complete.
- **Docs Upkeep:** Run `promptfall/prompts/docsync.md` and follow its checklist before closing a task so documentation stays current.
- **No Future Proofing:** Deliver only what the task currently requires. Stubs exist solely for immediately planned work; remove speculative placeholders instead of guessing at future design needs.

### CART-M0-01 – Scaffold Rust workspace and align CI
- **Milestone:** M0 – Workspace Bootstrapped & Measurement Guardrails
- **Goal:** Establish a Cargo workspace with `cart-core`, `cart-cli`, and `cart-benches` crates and ensure `ci/lint.sh` and local commands target the workspace successfully.
- **Rationale:** Satisfies roadmap M0 exit criteria and architecture decision for a multi-crate workspace, enabling focused feature work.
- **Scope:** Create workspace manifests, stub modules returning `CartError::Unimplemented`, wire `cart-cli` main with placeholder subcommands, update `ci/lint.sh` to call `cargo fmt`, `cargo clippy --workspace --all-targets`, and `cargo test`.
- **Out of Scope:** Implementing encode/decode behavior, shipping binaries, or altering benchmark datasets.
- **Dependencies:** docs/architecture.md, docs/roadmap.md (M0), existing CI scripts.
- **Acceptance Hints:** `cargo check` and `ci/lint.sh` succeed without warnings; workspace README snippet documents crates and local-first commands; no `unsafe` or unused dependencies introduced; release build completes under 120 s on reference dev host; placeholder code avoids heap allocations and returns structured errors for unimplemented paths.
- **Performance Budget:** Stub binaries allocate <16 KiB beyond stack; CLI placeholder reads args without cloning; future hot paths marked with `TODO(perf)` comments for traceability.
- **Simplicity Budget:** Keep crates dependency-free except artillery listed in dependency plan; use borrow-based function signatures even for stubs; avoid trait objects or dynamic dispatch in placeholders.
- **Testing Expectations:** Add a smoke unit test per crate confirming stubs return `CartError::Unimplemented`; ensure `cargo test --workspace` passes.
- **Review Focus:** Workspace layout mirrors architecture, CI script accuracy, absence of hidden allocations or premature abstractions.
- **Handoff Notes:** Document crate layout in docs/architecture.md appendix and link from README.
- **Traceability:** docs/roadmap.md (M0), docs/charter.md (§Scope, §Constraints).
- **Effort / Risk:** Effort S (~1 day); Risk Low (toolchain drift until MSRV decided).

### CART-M0-02 – Publish performance measurement plan and budgets
- **Milestone:** M0 – Workspace Bootstrapped & Measurement Guardrails
- **Goal:** Author a measurement plan capturing throughput, latency, memory, and allocation budgets plus tooling steps for Rust vs Python comparisons.
- **Rationale:** Roadmap M0 exit criteria require a documented measurement plan tied to benchmarks and performance mantra.
- **Scope:** Create `docs/performance-plan.md` detailing workloads, metrics, sample sizes, warm-up rules, and reporting cadence; reference `benchmarks/results/python-baseline.json`; update decision log with plan approval entry.
- **Out of Scope:** Implementing actual Rust benchmarks or optimizing code.
- **Dependencies:** CART-M0-01 workspace skeleton, Python baseline data, docs/charter.md success metrics.
- **Acceptance Hints:** Plan defines encode (≥1.5 GiB/s), decode (≥5.5 GiB/s), metadata (<0.02 ms), and detection (<5 µs) budgets; memory ceiling documented as <128 KiB beyond streaming buffer; plan specifies tooling (`criterion`, `perf`, heap profiler) and local-first scripts; document includes ledger template for tracking runs; language remains accessible (spell out Service Level Objectives).
- **Performance Budget:** Measurement scripts must stream data in ≤64 KiB chunks to avoid unnecessary allocations; plan mandates verifying allocation counts via `valgrind` or `dhat` on representative runs.
- **Simplicity Budget:** Prefer plain Markdown tables and existing scripts; avoid introducing new automation frameworks; provide single command (`cargo bench` wrapper) developers can run locally.
- **Testing Expectations:** Add doc lint or link check via CI to ensure plan stays valid; run dry-run command to confirm scripts resolve paths.
- **Review Focus:** Completeness of budgets, clarity of instructions, alignment with performance mantra.
- **Handoff Notes:** Cross-link plan from docs/roadmap.md M0 checkpoints and README contributor section.
- **Traceability:** docs/roadmap.md (M0 exit criteria), docs/charter.md (§Success Metrics, §Performance Mantra Alignment).
- **Effort / Risk:** Effort S (~1 day); Risk Medium (awaiting stakeholder confirmation of budgets).

### CART-M0-03 – Build cross-language parity smoke harness
- **Milestone:** M0 – Workspace Bootstrapped & Measurement Guardrails
- **Goal:** Provide a reusable harness that invokes `cart-py` fixtures and future Rust APIs to assert byte parity for encode/decode and metadata scenarios.
- **Rationale:** Required for roadmap M0 exit criteria and ensures local-first smoke checks before deeper parity coverage.
- **Scope:** Add `tests/parity_smoke.rs` (ignored until implementations land) calling shared helper library; create `scripts/run_parity_smoke.sh` to seed fixtures via Python; ensure harness streams data in 64 KiB chunks and stores expected outputs under `tests/fixtures/`.
- **Out of Scope:** Implementing Rust encode/decode logic or exhaustive parity coverage; CI wiring beyond documenting invocation.
- **Dependencies:** CART-M0-01, cart-py vendor directory, Python runtime availability.
- **Acceptance Hints:** Harness validates fixture availability at runtime; `cargo test --test parity_smoke -- --ignored` completes under 30 s with Python reference; file operations reuse buffers without cloning payloads; documented instructions specify environment variables for Python module path; script exits non-zero if parity mismatches once Rust code lands.
- **Performance Budget:** Harness memory watermark <64 MiB even on large fixtures; Python round trips executed once per test to limit runtime; streaming helper avoids buffering more than 2 blocks concurrently.
- **Simplicity Budget:** Use standard library pipes and `tempfile`; avoid additional crates beyond dependency plan; helpers return borrowed slices where possible.
- **Testing Expectations:** Add CI job documentation for optional parity smoke (manual trigger); include unit tests for fixture discovery logic.
- **Review Focus:** Deterministic fixture handling, streaming strategy, avoid flaky timing dependencies.
- **Handoff Notes:** Document usage in docs/performance-plan.md and README developer workflow.
- **Traceability:** docs/roadmap.md (M0 exit criteria), docs/context-report.md (§Repository Components), docs/charter.md (§Compatibility).
- **Effort / Risk:** Effort M (~2 days); Risk Medium (Python tooling or path assumptions).

### CART-M1-01 – Implement streaming decode parity in cart-core
- **Milestone:** M1 – Minimal Encode/Decode Parity
- **Goal:** Deliver the Rust decode pipeline with streaming buffer reuse, header/footer parsing, and Python parity for representative fixtures.
- **Rationale:** Core capability for charter parity requirement and roadmap M1 exit criteria.
- **Scope:** Implement header/footer parsing, ARC4 decrypt streaming, digest validation, `CartError` types, and `decode<R: Read, W: Write>` API with buffer reuse.
- **Out of Scope:** Encode pipeline, CLI wiring, benchmarking harness updates beyond decode coverage.
- **Dependencies:** CART-M0-01, CART-M0-03, docs/architecture.md component design.
- **Acceptance Hints:** Cross-language parity tests pass for small (≤1 MiB) and large (≥128 MiB) fixtures; decode throughput ≥5.5 GiB/s (≤0.042 s per 128 MiB zeros) recorded in ledger; peak heap allocation outside streaming buffer remains <128 KiB verified via allocator instrumentation; API surfaces borrow-based views for metadata; error paths documented and unit tested.
- **Performance Budget:** Loop hoists invariants, reuses single 64 KiB buffer, avoids per-chunk allocations, and ensures branch predictability by checking hot-path flags first.
- **Simplicity Budget:** Maintain one decode state struct, no dynamic dispatch, functions under 120 lines, follow straightforward control flow with documented invariants.
- **Testing Expectations:** Add property tests for header/footer parser, integration tests via parity harness for decode-only flow, and failure-case tests (bad magic, digest mismatch).
- **Review Focus:** Buffer reuse correctness, ARC4 usage, error handling clarity, and adherence to performance mantra.
- **Handoff Notes:** Update docs/performance-plan.md ledger with decode metrics and docs/decision-log.md with API readiness note.
- **Traceability:** docs/roadmap.md (M1 exit criteria), docs/charter.md (§Success Metrics), docs/architecture.md (§Component Responsibilities).
- **Effort / Risk:** Effort L (~1.5 weeks); Risk Medium (RC4 edge cases, fixture fidelity).

### CART-M1-02 – Implement encode and round-trip parity in cart-core
- **Milestone:** M1 – Minimal Encode/Decode Parity
- **Goal:** Provide streaming encode pipeline, digests, optional metadata handling, and round-trip verification utilities with Python parity.
- **Rationale:** Completes charter requirement for encode parity and supports CLI verification workflows.
- **Scope:** Implement `encode<R: Read, W: Write>`, digest selection, metadata serialization, ARC4 encryption, optional footer writing, and round-trip helper returning structured report.
- **Out of Scope:** CLI wiring, advanced optimizations, metadata schema changes beyond parity needs.
- **Dependencies:** CART-M1-01 decode work, dependency plan crates.
- **Acceptance Hints:** Encode parity tests pass for same fixtures; encode throughput ≥1.5 GiB/s on 128 MiB zeros, captured in ledger; memory footprint remains <128 KiB beyond streaming buffer; metadata JSON canonicalization matches Python (sorted keys, compact separators) within ±0 byte diff; round-trip helper logs digest and size stats without extra allocations.
- **Performance Budget:** Single allocation for buffer reuse, no Vec reallocation in hot loop, digest computation uses `crc32fast`/`digest` once per block; avoid string allocations by reusing serializer buffers.
- **Simplicity Budget:** Public APIs return structs with borrowed views; avoid macros or hidden branching; maintain straightforward error propagation via `?` operator.
- **Testing Expectations:** Extend parity harness for encode and round-trip; add regression test covering optional metadata with non-ASCII keys; verify digest permutations.
- **Review Focus:** Metadata canonicalization, buffer lifetime correctness, digest coverage, absence of unnecessary copies.
- **Handoff Notes:** Update docs/performance-plan.md with encode metrics and docs/open-questions.md if new assumptions discovered.
- **Traceability:** docs/roadmap.md (M1 exit criteria), docs/charter.md (§Scope, §Success Metrics).
- **Effort / Risk:** Effort L (~1.5 weeks); Risk High (metadata determinism, digest mismatches).

### CART-M1-03 – Deliver metadata inspection utilities and record baseline metrics
- **Milestone:** M1 – Minimal Encode/Decode Parity
- **Goal:** Implement `metadata` and `is_cart` helpers plus capture initial Rust baseline metrics documented in the performance ledger.
- **Rationale:** Required for roadmap exit criteria around metadata parity and establishing first Rust benchmarks.
- **Scope:** Implement metadata peek returning view structs, detection helper optimized for magic check, Criterion benches for encode/decode/metadata/is_cart, and populate ledger with Rust vs Python results.
- **Out of Scope:** Further optimizations beyond baseline, CLI wiring of metadata output.
- **Dependencies:** CART-M1-01, CART-M1-02, CART-M0-02 measurement plan.
- **Acceptance Hints:** Metadata helper completes in ≤0.02 ms, `is_cart` in ≤5 µs (benchmarked with stable inputs); baseline ledger includes wall time, throughput, memory stats for all scenarios; helper APIs borrow slices without copying; benches run via `cargo bench` and export JSON matching plan format.
- **Performance Budget:** No heap allocations in metadata path beyond 1 KiB temporary buffer; detection reads ≤32 bytes before returning; benches warm cache and control variance (<5%).
- **Simplicity Budget:** Avoid exposing mutable views; keep structs ≤5 fields with `#[non_exhaustive]`; bench harness script limited to Criterion defaults.
- **Testing Expectations:** Unit tests for metadata edge cases (missing footer, truncated header); parity harness verifies metadata fields; CI job ensures benches compile (even if skipped by default).
- **Review Focus:** Zero-copy behavior, benchmark repeatability, ledger completeness.
- **Handoff Notes:** Store bench output under benchmarks/results/ with timestamp and link in docs/performance-plan.md.
- **Traceability:** docs/roadmap.md (M1 exit criteria), docs/charter.md (§Success Metrics), docs/performance-plan.md.
- **Effort / Risk:** Effort M (~1 week); Risk Medium (benchmark variance, metadata edge cases).

### CART-M2-01 – Wire profiling instrumentation and perf observability
- **Milestone:** M2 – Throughput & Memory Gains Delivered
- **Goal:** Add profiling hooks, allocation counters, and observability instrumentation to guide optimization work.
- **Rationale:** Roadmap M2 entrance requires profiling data to inform performance improvements.
- **Scope:** Integrate optional `tracing` spans, allocation counters via feature flag, Criterion benchmark annotations, and scripts to run `perf`/`heaptrack` with documented commands.
- **Out of Scope:** Structural optimizations or API redesigns.
- **Dependencies:** CART-M1-02, CART-M1-03, docs/performance-plan.md tooling guidance.
- **Acceptance Hints:** Profiling scripts capture CPU, allocations, cache misses for encode/decode; instrumentation overhead <2% when disabled; `--perf-log` CLI flag emits JSON with bytes processed, duration, allocations; documentation updated with usage instructions.
- **Performance Budget:** Default builds exclude instrumentation cost; enabling profiling keeps allocation overhead <64 KiB for logs; ensures instrumentation runs in O(1) per block.
- **Simplicity Budget:** Instrumentation hidden behind feature flags; no global mutable state; use lightweight `tracing` macros without custom subscribers in core.
- **Testing Expectations:** Add integration test verifying `--perf-log` JSON schema; ensure instrumentation features compile via `cargo check --all-features`.
- **Review Focus:** Conditional compilation boundaries, low overhead instrumentation, doc clarity.
- **Handoff Notes:** Append profiling commands to docs/performance-plan.md and commit initial profiling captures to ledger.
- **Traceability:** docs/roadmap.md (M2 entrance), docs/charter.md (§Performance Mantra Alignment).
- **Effort / Risk:** Effort M (~1 week); Risk Medium (profiling tooling parity across dev environments).

### CART-M2-02 – Achieve ≥10× throughput target with evidence
- **Milestone:** M2 – Throughput & Memory Gains Delivered
- **Goal:** Optimize encode/decode paths to meet or exceed ≥10× Python throughput while maintaining correctness.
- **Rationale:** Core success metric from charter and roadmap M2 exit criteria.
- **Scope:** Apply evidence-backed optimizations (buffer sizing, SIMD digests, loop hoisting), update benchmarks, and document before/after metrics in ledger.
- **Out of Scope:** Memory optimization beyond throughput target (handled separately), CLI changes.
- **Dependencies:** CART-M2-01 profiling data, CART-M1-02 encode implementation.
- **Acceptance Hints:** Benchmarks show ≥10× Python throughput across all datasets with variance <5%; regression tests and parity suite remain green; ledger entry details optimization steps and impact; any new `unsafe` justified with benchmark evidence (otherwise avoid).
- **Performance Budget:** Hot loops branch predictable; no extra heap allocations introduced; CPU utilization measured and recorded; maintain streaming block size at 64 KiB unless evidence suggests alternative.
- **Simplicity Budget:** Prefer algorithmic changes over micro-optimizations; document rationale inline; avoid template explosion or specialization without need.
- **Testing Expectations:** Rerun parity harness post-optimization; add regression benchmarks to guard improved throughput; include perf regression threshold in CI (fail if <9.5×).
- **Review Focus:** Evidence for each optimization, correctness guardrails, readability of optimized code.
- **Handoff Notes:** Update docs/decision-log.md with optimization decisions and docs/performance-plan.md ledger.
- **Traceability:** docs/roadmap.md (M2 exit criteria), docs/charter.md (§Success Metrics).
- **Effort / Risk:** Effort M (~1 week); Risk High (risk of regressions, hardware variance).

### CART-M2-03 – Enforce memory and allocation guardrails
- **Milestone:** M2 – Throughput & Memory Gains Delivered
- **Goal:** Validate and enforce memory usage stays within documented budgets under stress scenarios.
- **Rationale:** Completes roadmap M2 exit criteria for bounded memory and supports simplicity mandate.
- **Scope:** Add stress tests (huge files, metadata extremes), instrument allocation counters, add CI checks that fail on allocation regressions, and document mitigation plan if thresholds exceeded.
- **Out of Scope:** Throughput optimizations unless needed to fix memory regressions.
- **Dependencies:** CART-M2-01 instrumentation, CART-M1-03 baseline metrics.
- **Acceptance Hints:** Allocation tracking shows <128 KiB overhead beyond stream buffers across scenarios; stress tests run within 10% of expected throughput; CI guard rails compare allocations vs baseline and alert on >5% increase; documentation states remediation steps.
- **Performance Budget:** Ensure stress harness streams data without copying; memory guard measured with `dhat` or `jemalloc` stats; detection of leaks within 1 KiB accuracy.
- **Simplicity Budget:** Prefer RAII buffer pools with borrow semantics; avoid global allocators; keep stress harness scripts simple shell or Rust integration tests.
- **Testing Expectations:** Add integration test for large-file decode verifying memory watermark; update benches to record allocation counts; include doc on running memory checks locally.
- **Review Focus:** Accuracy of allocation measurement, clarity of guardrails, minimal instrumentation overhead.
- **Handoff Notes:** Record baseline memory report in docs/performance-plan.md and link in decision log.
- **Traceability:** docs/roadmap.md (M2 exit criteria), docs/charter.md (§Constraints & Boundaries).
- **Effort / Risk:** Effort M (~1 week); Risk Medium (measurement tooling flakiness).

### CART-M3-01 – Ship CLI workflows with streaming guarantees
- **Milestone:** M3 – CLI Productization & Release Readiness
- **Goal:** Implement CLI subcommands (`encode`, `decode`, `inspect`, `verify`) that wrap `cart-core` APIs with streaming I/O and parity tests.
- **Rationale:** Roadmap M3 outcome requires CLI parity and round-trip verification support.
- **Scope:** Wire `clap` command graph, map flags to options, implement structured logging, ensure file I/O streams with minimal buffering, and integrate parity harness into CLI tests.
- **Out of Scope:** Distribution automation, documentation (covered elsewhere), asynchronous APIs.
- **Dependencies:** CART-M1-02 core features, CART-M1-03 metadata utilities.
- **Acceptance Hints:** CLI commands process 128 MiB fixture within 5% overhead vs library APIs; `inspect` prints metadata JSON deterministically; `verify` command reuses library round-trip report; CLI streaming uses buffer reuse to keep allocated memory <128 KiB; parity tests confirm CLI outputs byte-identical to Python CLI for sample fixtures.
- **Performance Budget:** Avoid reading entire files into memory; commands operate in O(n) with single pass; `inspect` uses metadata peek to avoid full decode.
- **Simplicity Budget:** Keep clap definitions declarative; avoid custom derive macros beyond clap; structure module ≤3 layers deep; reuse library error types.
- **Testing Expectations:** Add CLI integration tests using `assert_cmd`; extend parity harness to compare CLI vs Python outputs; include help/usage snapshot tests.
- **Review Focus:** Flag handling, streaming adherence, determinism of output formatting.
- **Handoff Notes:** Link CLI usage examples in README and docs/performance-plan.md for measurement reproduction.
- **Traceability:** docs/roadmap.md (M3 exit criteria), docs/charter.md (§Scope, §Usability).
- **Effort / Risk:** Effort M (~1 week); Risk Medium (UX ambiguity, filesystem variance).

### CART-M3-02 – Document CLI UX, observability, and supportability
- **Milestone:** M3 – CLI Productization & Release Readiness
- **Goal:** Produce documentation, logging guidance, and troubleshooting aids aligned with charter usability metrics.
- **Rationale:** Roadmap M3 exit criteria call for docs and onboarding materials.
- **Scope:** Update README and `docs/` with CLI examples, observability knobs, error codes, and security posture note; ensure telemetry integrates with perf ledger; add `--log-format` options and doc coverage.
- **Out of Scope:** Release automation, code optimizations, user analytics.
- **Dependencies:** CART-M3-01, docs/performance-plan.md, docs/open-questions.md (security posture).
- **Acceptance Hints:** Docs show end-to-end examples for all commands, including piping via stdin/stdout; logging section details `--log-format` and `--perf-log`; security posture note references RC4 legacy status; docs reviewed by CLI stakeholders; documentation build or lint passes; updates stay within 120-character width.
- **Performance Budget:** Ensure documented usage encourages streaming (e.g., `--input -` guidance) and warns about large file memory usage; examples show zero-copy paths.
- **Simplicity Budget:** Use straightforward language, avoid jargon; code samples minimal and rely on public APIs.
- **Testing Expectations:** Add doc tests for code snippets; run `cargo test --doc` to ensure examples compile.
- **Review Focus:** Accuracy of instructions, completeness of logging/observability guidance, accessibility of language.
- **Handoff Notes:** Reference docs in decision log and onboarding materials; notify stakeholders of updates.
- **Traceability:** docs/roadmap.md (M3 exit criteria), docs/charter.md (§Usability, §Performance Mantra Alignment).
- **Effort / Risk:** Effort S (~3 days); Risk Low (doc drift if CLI evolves).

### CART-M3-03 – Establish release packaging and regression gates
- **Milestone:** M3 – CLI Productization & Release Readiness
- **Goal:** Create release automation, packaging scripts, and verification gates for publishing the crate and CLI binaries.
- **Rationale:** Final roadmap milestone demands distribution strategy and deterministic outputs.
- **Scope:** Add release script (GitHub Actions or local `scripts/release.sh`) reusing local-first commands, produce reproducible artifacts, update release checklist covering determinism and security messaging, and ensure parity suite runs in pipeline.
- **Out of Scope:** Publishing to external registries (manual run still acceptable), cross-platform builds beyond Linux.
- **Dependencies:** CART-M3-02 documentation, CART-M3-01 CLI readiness, docs/decision-log.md license decisions.
- **Acceptance Hints:** Release script builds crates/binaries, runs parity suite, uploads artifacts with checksums; pipeline fails if parity or perf regressions exceed ±5%; release checklist stored in `docs/release-checklist.md`; artifacts documented as reproducible with pinned dependencies and commit hash.
- **Performance Budget:** Release automation must reuse existing benchmarks and not duplicate logic; ensures release builds executed with `--profile release` and log throughput metrics; packaging avoids embedding unnecessary assets keeping binary size <5 MiB.
- **Simplicity Budget:** Scripts use bash + cargo; avoid bespoke YAML duplication; reuse `ci/lint.sh` where possible; configuration values centralized.
- **Testing Expectations:** Dry-run release executed locally with `DRY_RUN=1` check; add CI job to validate release script syntax; include regression gating via matrix.
- **Review Focus:** Idempotency of release process, reuse of local-first tooling, compliance with licensing.
- **Handoff Notes:** Update README with release steps, notify maintainers of ownership, and log decision in docs/decision-log.md.
- **Traceability:** docs/roadmap.md (M3 exit criteria), docs/charter.md (§Constraints & Boundaries), docs/architecture.md (§Observability & Operations).
- **Effort / Risk:** Effort M (~1 week); Risk Medium (CI environment parity, supply-chain considerations).

## Open Follow-Ups
- F1: Create or locate an authoritative risk register artifact; current repository lacks a dedicated register required by prompt guardrails (reference docs/context-report.md Risks & Gaps).
- F2: Confirm Minimum Supported Rust Version (MSRV) with stakeholders and record decision (ties to docs/open-questions.md).
- F3: Agree on exact memory ceiling (MiB) for large workloads to validate budgets defined in docs/performance-plan.md.
- F4: Decide CLI UX defaults for overwrite behavior, stdout piping, and RC4 key override naming (docs/open-questions.md).
- F5: Assign owners for profiling tool access (`perf`, `heaptrack`) and ensure developer workstations have required permissions before M2.
