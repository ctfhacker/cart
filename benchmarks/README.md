# Python Baseline Benchmarks

This benchmark script measures the Python reference implementation to establish a performance baseline we will replicate in Rust.

- Scenarios mirror unit tests in `.venv/lib64/python3.11/site-packages/cart/unittests/test_cart.py` (empty, small, large, metadata, is_cart).
- Outputs JSON with per-iteration timings and throughput (MiB/s) for pack/unpack scenarios.

## Run

Ensure the `cart` package is importable. The script auto-adds `.venv/lib64/python3.11/site-packages/cart` to `PYTHONPATH` if run from the repo root.

Examples:

- Default unit-test aligned scenarios (recommended):
  `python3 benchmarks/python_baseline.py --iters 3 --profile unittests --out benchmarks/results/python-baseline.json`

- Extended scenarios (adds 128 MiB random cases):
  `python3 benchmarks/python_baseline.py --iters 3 --profile full --out benchmarks/results/python-baseline-full.json`

Results are written as JSON to the given `--out` path or printed to stdout if omitted.

## Notes

- Large test uses a 128 MiB input, matching the unit tests (zeros). This compresses heavily; extended profile adds an incompressible case for contrast.
- The script records environment details (Python version, platform, machine) to help compare against Rust runs on the same host.
- Linux-only platform constraint is assumed for reproducibility.
