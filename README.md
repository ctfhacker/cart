# cart-rs

Rust workspace scaffolding for the CaRT codec and CLI rewrite.

## Local Development
- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-targets`
- Build: `cargo build --workspace --all-targets`
- Test: `cargo test --workspace --all-targets`
- CI helper: `./ci/lint.sh`

`cart-core`, `cart-cli`, and `cart-benches` currently expose stubbed APIs returning `CartError::Unimplemented` until feature work lands. Run `cargo run -p cart-cli -- --help` to exercise the CLI stub and confirm wiring.

To study the Python parity implementation, inspect `.venv/lib64/python3.11/site-packages/cart/` (or the matching path for your Python version) alongside the vendored `.venv/lib64/python3.11/site-packages/cart/` sources.

## Docs
- Performance measurement plan: `docs/performance-plan.md`
