#!/usr/bin/env bash
set -euo pipefail

python_bin="${CART_PYTHON:-}" 
if [[ -z "$python_bin" ]]; then
  if [[ -x ".venv/bin/python3" ]]; then
    python_bin=".venv/bin/python3"
  else
    python_bin="python3"
  fi
fi

export CART_PYTHON="$python_bin"

echo "Using CART_PYTHON=$CART_PYTHON"

cargo test --test parity_smoke -- --ignored
