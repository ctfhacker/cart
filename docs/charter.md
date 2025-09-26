# CART-RS Charter

## Purpose & Problem Statement
- Deliver a Rust library and command-line interface (CLI) that encode and decode CaRT (Compressed and RC4-transported) files with byte-for-byte parity to the Python reference implementation.
- Provide faster, predictable tooling for pipelines that need deterministic packing, metadata inspection, and rapid round-tripping of binaries.

## Context & Evidence
- Python reference (`cart-py/cart/cart.py`) defines current CaRT v1 semantics; charter aligns with existing doc `docs/charter.md`.
- Cross-language parity is required so Rust artifacts interoperate with Python-generated CaRT files and vice versa.

## Scope
- Rust crate exposes in-memory and file-based encode/decode, metadata read without full decode, and CaRT detection utilities.
- CLI supports encode, decode, metadata-only inspection, and optional round-trip verification for automation workflows.
- Maintain compatibility with existing headers, footers, key semantics, and metadata structure defined in CaRT v1.

## Stakeholders & Users
- Rust service and tooling engineers embedding CaRT functionality.
- Pipeline operators using the CLI for fast deterministic packaging/unpacking.
- CaRT format maintainers responsible for cross-language consistency and test coverage.

## Success Metrics
- Functional: Rust implementation passes a test suite derived from Python fixtures and cross-verifies shared artifacts.
- Performance: ≥10× Python throughput on representative encode/decode benchmarks; publish baseline and achieved numbers.
- Memory: Peak usage remains near-constant relative to block size with no unbounded growth on large inputs.
- Usability: Ergonomic API with documented examples, intuitive CLI flags, semantic versioning, and changelog.
- Compatibility: Byte-identical outputs where the Python reference is deterministic for matching inputs and options.

## Constraints & Boundaries
- Licensing remains MIT; deliverables include publishable crate, CLI binary, reproducible benchmarks, and cross-verification tests.
- Target platform is Linux (default x86_64); additional architectures optional.
- Maintain current CaRT v1 format and cryptography for compatibility; no format evolution in this phase.
- Engineering policy: avoid `unsafe` Rust; adhere to safe abstractions throughout the crate and CLI.

## Non-Goals
- Introducing new CaRT versions, altering cryptography, or building network services, GUIs, or non-Rust bindings.

## Milestone Zero (First Valuable Outcome)
- Implement minimal encode/decode in Rust, cross-verify against Python for small and large files, and record baseline performance and memory metrics using the established benchmarking datasets.

## Assumptions & Risks
- Continued access to Python reference code and fixtures for verification.
- Legacy cryptography remains acceptable for compatibility; documentation must clarify security posture.
- Platform differences (file systems, line endings) may affect deterministic outputs; automated tests mitigate the risk.
- Representative datasets used by the Python reference remain available for benchmarking comparisons.

## Performance Mantra Alignment
- Measure before optimizing; treat allocations and copies as conscious decisions.
- Favor simple, direct code paths with clear ownership; keep hot paths small and predictable.
- Document rationale and data for any optimization work to maintain team-wide clarity.

## Decision Log
- D1 (Today): Reaffirmed existing charter scope and performance targets from `docs/charter.md` as authoritative baseline.
- D2 (Today): Adopted safe Rust-only policy and confirmed reuse of existing benchmarking datasets.

## Open Questions
- Q1: Confirm Minimum Supported Rust Version (MSRV); coordinate with maintainers for tooling compatibility.
- Q2: Define acceptable peak memory threshold in MiB for large files; align with pipeline operators.
