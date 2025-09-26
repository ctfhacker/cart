# CART-RS Decision Log

- Decision: Deliver both a Rust library and a CLI.
  - Rationale: Matches current and future usage patterns; parity with Python’s dual usage.
  - Stakeholders: Library consumers, pipeline/CLI users.
  - Evidence: Python reference exposes library API and CLI entrypoint (`.venv/lib64/python3.11/site-packages/cart/cart/cart.py`).

- Decision: Target 10x speedup over Python reference on representative workloads.
  - Rationale: Primary success criterion stated by stakeholders.
  - Stakeholders: All consumers demanding performance; maintainers.
  - Evidence: Stated requirement; to be validated with benchmarks.

- Decision: Strict compatibility with CaRT v1 artifacts produced/consumed by Python.
  - Rationale: Interop is non‑negotiable; avoids ecosystem fragmentation.
  - Stakeholders: Current CaRT users and maintainers.
  - Evidence: Reference behaviors in `.venv/lib64/python3.11/site-packages/cart/unittests/test_cart.py`.

- Decision: License under MIT, aligning with Python repo’s license.
  - Rationale: Consistency and permissive reuse.
  - Stakeholders: Legal/compliance, maintainers, adopters.
  - Evidence: `.venv/lib64/python3.11/site-packages/cart/LICENSE.md`.

- Decision: Target platform is Linux only.
  - Rationale: Stated constraint; simplifies support and CI.
  - Stakeholders: All users; CI/release maintainers.
  - Evidence: Stakeholder confirmation in discussion.

- Decision: Adopt the performance measurement plan and budgets in `docs/performance-plan.md`.
  - Rationale: Establishes shared methodology, tooling, and budgets required for roadmap milestone M0 exit criteria.
  - Stakeholders: Core maintainers, performance lead, benchmarking contributors.
  - Evidence: Approved plan recorded in `docs/performance-plan.md`; baseline data in `benchmarks/results/python-baseline.json`.
