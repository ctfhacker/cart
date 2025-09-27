# cart-rs

Rust workspace scaffolding for the CaRT codec and CLI rewrite.

## Local Development
- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-targets`
- Build: `cargo build --workspace --all-targets`
- Test: `cargo test --workspace --all-targets`
- Bench: `cargo bench -p cart-benches` (always use `--release` or the default release profile for meaningful numbers)
- CI helper: `./ci/lint.sh`
- Parity smoke: `scripts/run_parity_smoke.sh` (requires Python CaRT CLI in `.venv` or `CART_PYTHON`)
- Seed fixtures: `scripts/seed_cart_artifacts.py` writes `benchmarks/data/artifact.{bin,cart}` with 100 KiB synthetic data
- Rust perf runner: `cargo run --release -p cart-benches --bin run -- --iters 5` (ASCII table summary; run in release mode)

`cart-core` implements decode/metadata helpers plus a minimal encode pipeline that writes default CaRT headers and encrypted payloads. `cart-benches` exposes helper functions for timing encode/decode calls so we can compare against Python baselines, while `cart-cli` still returns `CartError::Unimplemented` until the CLI wiring lands. Run `cargo run -p cart-cli -- --help` to exercise the CLI stub and confirm wiring.

To study the Python parity implementation, inspect `.venv/lib64/python3.11/site-packages/cart/` (or the matching path for your Python version) alongside the vendored `.venv/lib64/python3.11/site-packages/cart/` sources.

## Docs
- Performance measurement plan: `docs/performance-plan.md`
