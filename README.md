# CaRT

## Rust bench

```
cargo run -r -p cart-benches --bin run -- --iters 3
```

## Python bench

```
uv run -p python3.11 python3 benchmarks/python_baseline.py --iters 3 --profile unittests --out benchmarks/results/python-
```
