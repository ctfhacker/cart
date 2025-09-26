# CART-RS Open Questions

- Platforms: Resolved — Linux only (confirm exact CI images and arches).
- MSRV: What minimum Rust version should we guarantee? Any `no_std` or `unsafe` policy constraints?
- Performance budgets: What concrete throughput/latency/memory targets and representative file corpora define “10x faster”?
- CLI UX: Required flags and behaviors (e.g., overwrite/force, metadata‑only, stdout piping, RC4 key override naming)?
- Async support: Do we need asynchronous streaming APIs as in Python, and which runtime(s) to target?
- Determinism: Are deterministic outputs required across platforms (e.g., metadata ordering, timestamp handling)?
- Security posture: How should we document compatibility crypto vs. modern security expectations; any optional hardened mode planned?
- Distribution: How should we ship the CLI (crates.io install, prebuilt binaries, internal registries)?
- Testing: Do we have/need a shared fixture corpus and cross‑language conformance tests beyond unit tests?
