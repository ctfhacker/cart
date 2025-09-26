# CART-RS Context Snapshot

## Project Purpose & Goals
- Charter targets a Rust library and CLI that are byte-compatible with the Python CaRT reference, aiming for ≥10× throughput, bounded memory, and safe Rust (no `unsafe`) while serving pipeline operators and embedding teams (source: charter.md; docs/charter.md; docs/decision-log.md).
- Success metrics focus on parity with Python fixtures, documented ergonomics, and performance baselines derived from existing datasets (source: charter.md; docs/charter.md; docs/decision-log.md).

## Current Implementation State
- Repository currently lacks any Rust crate artifacts (`Cargo.toml`, `.rs` sources) despite CI scripts expecting them (source: Command: find . -name 'Cargo.toml'; Command: rg --files -g '*.rs'; ci/lint.sh).
- Executable entry point is a placeholder Python script printing "Hello from cart-rs" and the root README is empty, indicating project scaffolding is incomplete (source: main.py; README.md).
- Working tree is detached (`HEAD (no branch)`) with numerous staged additions including the Python reference, promptfall assets, and `.direnv` outputs, signalling an in-progress restructure (source: Command: git status -sb).

## Repository Components
- `.venv/lib64/python3.11/site-packages/cart/` vendor directory houses the authoritative Python CaRT implementation, CLI behaviors, unit tests, and Azure DevOps pipelines for packaging, testing, and deployment, providing reference semantics and process expectations (source: .venv/lib64/python3.11/site-packages/cart/README.md; .venv/lib64/python3.11/site-packages/cart/cart/cart.py; .venv/lib64/python3.11/site-packages/cart/unittests/test_cart.py; .venv/lib64/python3.11/site-packages/cart/pipelines/azure-build.yaml).
- `benchmarks/` offers Python baseline tooling, synthetic datasets, and a captured JSON run to quantify current performance for pack/unpack/metadata workflows (source: benchmarks/README.md; benchmarks/python_baseline.py; Command: ls benchmarks/data; benchmarks/results/python-baseline.json).
- `promptfall/` replaces the earlier `prompts/` directory with role-specific prompt templates, documentation, and a runner script that orchestrates the Charter→Context→Roadmap flow; the runner expects prompt paths like `promptfall/prompts/context.md` (source: Command: ls promptfall/prompts; promptfall/prompts/context.md; promptfall/prompts/performance-mantra.md; promptfall/README.md; promptfall/scripts/runner.sh).
- `ci/lint.sh` codifies local-first CI expectations (format, clippy, build) but presumes a Cargo workspace that does not yet exist, highlighting divergence between scripts and code (source: ci/lint.sh).

## Tooling & Environment
- Python benchmarking package managed via `pyproject.toml` depends on `cart>=1.2.3` and exposes a `bench-py` entry; `scripts/uv_setup.sh` syncs dependencies via `uv` and suggests running benchmark commands (source: pyproject.toml; scripts/uv_setup.sh).
- Nix-based development shell pins nightly Rust with `rust-analyzer` and `uv`, while `.envrc` auto-enters the flake; Python runtime is pinned to 3.13 (source: flake.nix; .envrc; .python-version).

## Benchmarks & Performance Evidence
- Baseline JSON captures timings and throughput for pack/unpack/metadata/is_cart scenarios on representative datasets, establishing evidence for future Rust targets (source: benchmarks/results/python-baseline.json; benchmarks/README.md).
- Benchmark script reuses Python CaRT APIs (`cart.pack_stream`, `cart.unpack_stream`) and generates deterministic data, aligning with charter requirements for parity-focused perf measurement (source: benchmarks/python_baseline.py).

## Process & Governance Artifacts
- Prompt suite documents enforce evidence-backed workflows, context gathering, roadmap creation, testing discipline, and drift detection, with explicit references to the Performance Mantra (source: promptfall/prompts/context.md; promptfall/prompts/tests.md; promptfall/prompts/reality.md; promptfall/docs/prompt_agent_suite.md).
- Decision log reconfirms scope (Rust lib + CLI, MIT license, Linux focus) and performance targets; open-questions doc catalogues unresolved governance items (source: docs/decision-log.md; docs/open-questions.md).

## Risks & Gaps
- Absence of Rust code plus cargo-centric CI script creates guaranteed build failures until the crate is scaffolded (source: ci/lint.sh; Command: find . -name 'Cargo.toml').
- Benchmark evidence exists only for Python; no Rust metrics or integration tests enforce parity yet (source: benchmarks/results/python-baseline.json; benchmarks/python_baseline.py).
- Working tree includes generated `.direnv` artifacts slated for commit, raising risk of environment-specific noise if not cleaned (source: Command: git status -sb).
- Documentation such as the root README lacks onboarding or build instructions, impeding new contributors (source: README.md).

## Open Questions & Follow-Ups
- Confirm Minimum Supported Rust Version (MSRV), memory budgets, CLI UX expectations, async needs, determinism requirements, security posture messaging, distribution strategy, and fixture corpus ownership (source: docs/open-questions.md).
- Clarify plan to bootstrap the Rust crate, align `ci/lint.sh` with available code, and decide whether promptfall artifacts stay tracked within this repo or a separate docs location (source: ci/lint.sh; promptfall/scripts/runner.sh; Command: git status -sb).
- Determine how and where Rust performance baselines will be recorded to compare with `benchmarks/results/python-baseline.json` once implementations land (source: benchmarks/results/python-baseline.json).

## Sources
- charter.md
- docs/charter.md
- docs/decision-log.md
- docs/open-questions.md
- Command: find . -name 'Cargo.toml'
- Command: rg --files -g '*.rs'
- ci/lint.sh
- main.py
- README.md
- Command: git status -sb
- .venv/lib64/python3.11/site-packages/cart/README.md
- .venv/lib64/python3.11/site-packages/cart/cart/cart.py
- .venv/lib64/python3.11/site-packages/cart/unittests/test_cart.py
- .venv/lib64/python3.11/site-packages/cart/pipelines/azure-build.yaml
- benchmarks/README.md
- benchmarks/python_baseline.py
- Command: ls benchmarks/data
- benchmarks/results/python-baseline.json
- Command: ls promptfall/prompts
- promptfall/prompts/context.md
- promptfall/prompts/performance-mantra.md
- promptfall/prompts/tests.md
- promptfall/prompts/reality.md
- promptfall/README.md
- promptfall/scripts/runner.sh
- promptfall/docs/prompt_agent_suite.md
- pyproject.toml
- scripts/uv_setup.sh
- flake.nix
- .envrc
- .python-version
