#!/usr/bin/env python3
"""Seed local CaRT artifacts with synthetic data when missing.

This script creates `artifact.bin` filled with 100 KiB of random bytes and
invokes the Python CaRT CLI to produce `artifact.cart` if either file is
absent.

It honours the `CART_PYTHON` environment variable. Otherwise it prefers the
workspace virtual environment (`.venv/bin/python3`) and falls back to the
system `python3`.
"""

from __future__ import annotations

import os
import pathlib
import secrets
import subprocess
from typing import List

DATA_BYTES = 100_000
BENCHMARK_DIR = pathlib.Path("benchmarks/data")
ARTIFACT_BIN = BENCHMARK_DIR / "artifact.bin"
ARTIFACT_CART = BENCHMARK_DIR / "artifact.cart"


def main() -> None:
    python_exe = resolve_python()
    ensure_cart_cli_available(python_exe)
    created = False

    BENCHMARK_DIR.mkdir(parents=True, exist_ok=True)

    if not ARTIFACT_BIN.exists():
        write_random_payload(ARTIFACT_BIN, DATA_BYTES)
        print(f"Wrote {DATA_BYTES} bytes to {ARTIFACT_BIN}")
        created = True
    else:
        print(f"{ARTIFACT_BIN} already exists; leaving as-is")

    if not ARTIFACT_CART.exists() or created:
        pack_cart(python_exe, ARTIFACT_BIN, ARTIFACT_CART)
        print(f"Generated {ARTIFACT_CART}")
    else:
        print(f"{ARTIFACT_CART} already exists; leaving as-is")


def resolve_python() -> pathlib.Path:
    env_bin = os.environ.get("CART_PYTHON")
    if env_bin:
        return pathlib.Path(env_bin)

    venv_candidate = pathlib.Path(".venv") / "bin" / "python3"
    if venv_candidate.exists():
        return venv_candidate

    return pathlib.Path("python3")


def write_random_payload(path: pathlib.Path, length: int) -> None:
    payload = secrets.token_bytes(length)
    path.write_bytes(payload)


def pack_cart(python_exe: pathlib.Path, source: pathlib.Path, target: pathlib.Path) -> None:
    cmd: List[str] = [str(python_exe), "-m", "cart.cart", "-f", "-o", str(target), str(source)]
    try:
        subprocess.run(cmd, check=True, capture_output=True)
    except subprocess.CalledProcessError as exc:  # pragma: no cover - exec path
        stderr = exc.stderr.decode(errors="replace") if exc.stderr else ""
        raise SystemExit(
            f"Failed to produce {target} using cart (exit {exc.returncode}):\n{stderr}"
        ) from exc


def ensure_cart_cli_available(python_exe: pathlib.Path) -> None:
    try:
        subprocess.run(
            [str(python_exe), "-c", "import cart.cart"],
            check=True,
            capture_output=True,
        )
    except subprocess.CalledProcessError as exc:  # pragma: no cover - exec path
        hint = (
            "Python CaRT CLI is missing. Run `scripts/uv_setup.sh` or otherwise install"
            " the cart package into your environment."
        )
        stderr = exc.stderr.decode(errors="replace") if exc.stderr else ""
        raise SystemExit(f"Cannot import cart.cart using {python_exe}:\n{stderr}\n{hint}") from exc


if __name__ == "__main__":  # pragma: no cover - script entry point
    main()
