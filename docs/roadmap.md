# CART-RS Roadmap

**Prioritization Lens:** Impact × Confidence ÷ Effort — favors milestones that unlock cross-language value quickly while keeping risk low.

## Milestones

### M0 – Workspace Bootstrapped & Measurement Guardrails
- **Outcome:** Rust workspace scaffolding, CI alignment, and a shared performance measurement doctrine so teams can iterate on parity with evidence.
- **Entrance Criteria:** Charter, context snapshot, decision log, and performance mantra ratified; access to Python reference implementation and benchmark datasets validated.
- **Exit Criteria:** Library and CLI crates compiling via `cargo check`; CI scripts updated to reference the workspace; cross-language smoke harness invoking `.venv/lib64/python3.11/site-packages/cart` fixtures lands in `tests/`; documented performance budgets and measurement plan (latency, throughput, memory) published in `docs/` (see `docs/performance-plan.md`) with linkage to `benchmarks/python_baseline.py`.
- **Dependencies:** Stakeholder agreement on provisional Minimum Supported Rust Version (MSRV) and development toolchain; confirmation that Python fixtures remain the authoritative source of truth.
- **Risks:** MSRV indecision or fixture ownership gaps could block workspace initialization; benchmarking environments may drift without pinned tool versions.
- **Checkpoints:** (1) Design review of crate layout/API ownership; (2) Measurement plan walkthrough confirming budgets, tooling, and reporting cadence.
- **Acceptance Signals:** CI run shows green builds and parity smoke test; measurement plan approved by maintainers and recorded in decision log.

### M1 – Minimal Encode/Decode Parity (Milestone Zero)
- **Outcome:** Safe Rust encode/decode paths delivering byte-for-byte parity with Python for representative files while logging baseline performance.
- **Entrance Criteria:** M0 complete; Rust workspace ready for feature work; fixture corpus and smoke harness available.
- **Exit Criteria:** Encode/decode APIs implement CaRT v1 semantics with automated cross-language tests covering small/large files; metadata read path mirrors Python behavior; baseline Rust vs. Python throughput, latency, and peak memory captured using shared measurement plan and stored alongside benchmark artifacts.
- **Dependencies:** Python parity harness, clarified handling of deterministic fields (timestamps, ordering), and available benchmarking datasets.
- **Risks:** Hidden CaRT edge cases or RC4 handling discrepancies could erode parity; measurement harness may show Rust lagging, threatening stakeholder confidence.
- **Checkpoints:** (1) Parity test coverage review focusing on critical fixtures; (2) Performance readout comparing Rust baseline to Python reference with mitigation backlog.
- **Acceptance Signals:** Automated parity suite passes; documented baseline metrics signed off in decision log with actionable gap analysis.

### M2 – Throughput & Memory Gains Delivered
- **Outcome:** Rust implementation meets or exceeds the ≥10× throughput target and holds memory usage within agreed bounds while staying correct and maintainable.
- **Entrance Criteria:** M1 exit criteria satisfied; baseline metrics highlight gaps; profiling instrumentation wired in.
- **Exit Criteria:** Profiling reports (CPU, allocations) inform targeted optimizations; throughput reaches ≥10× Python across benchmark datasets; latency variance and peak memory stay within documented budgets; performance ledger updated with before/after data and analyses following the measurement plan.
- **Dependencies:** Tooling for profiling (e.g., `perf`, `cargo-criterion`), prioritized optimization backlog, and agreement on memory thresholds.
- **Risks:** Optimization work could introduce correctness regressions; external tool access (perf permissions) may lag; insufficient profiling resolution could mask bottlenecks.
- **Checkpoints:** (1) Optimization strategy review grounded in profiling evidence; (2) Performance verification day with rerun benchmarks and guardrail comparison.
- **Acceptance Signals:** Benchmarks and regression tests run clean in CI; performance ledger reviewed by stakeholders with confidence score ≥0.7 on the prioritization lens.

### M3 – CLI Productization & Release Readiness
- **Outcome:** CLI delivers required workflows (encode, decode, metadata-only, round-trip verification) with documentation, packaging, and observability that support release.
- **Entrance Criteria:** M2 complete; API surface stable; stakeholders aligned on CLI UX expectations.
- **Exit Criteria:** CLI implements agreed flag set (overwrite/force, stdout piping, RC4 key handling) and parity tests with Python CLI; user documentation and onboarding guides added to `docs/` and README; distribution strategy (crates.io, binary releases) documented with automation plan; release checklist covers deterministic outputs and security posture messaging.
- **Dependencies:** Decisions on CLI UX defaults, distribution channels, and security communication; packaging infrastructure (GitHub Actions/Nix).
- **Risks:** Late-breaking UX requirements may expand scope; release automation could drift from Linux-only constraint; documentation might lag feature updates.
- **Checkpoints:** (1) CLI UX demo with stakeholder sign-off; (2) Dry-run release executing packaging scripts and validating artifacts against parity suite.
- **Acceptance Signals:** CLI e2e tests green, docs merged, and release plan approved with owners assigned for follow-up tasks.

## Risk & Assumption Ledger
- **A1 (All):** Python reference remains authoritative — *Validation:* reconfirm at each checkpoint; escalate if upstream changes land without notice.

## Roadmap Narrative
The roadmap rapidly establishes a Rust workspace and shared measurement guardrails before pursuing encode/decode parity with the Python reference. Once minimal parity lands, we focus on profiling-driven optimizations to achieve the ≥10× throughput and tight memory targets mandated by the charter. With performance secured, attention shifts to CLI productization, documentation, and release automation that respect Linux-only constraints. Risks and open questions emphasize toolchain choices, fixture completeness, and release channels so we can sequence decisions ahead of dependent milestones.
