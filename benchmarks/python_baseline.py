#!/usr/bin/env python3
"""
Benchmark the Python CaRT reference implementation to establish a baseline
that we will replicate in Rust.

Outputs JSON with per-iteration timings and computed throughput (MiB/s).
"""
import argparse
import json
import os
import platform
import struct
import sys
import time
from io import BytesIO
from statistics import mean
from typing import Dict, List, Tuple

try:
    import cart  # noqa: E402
except Exception:
    sys.stderr.write("ERROR: Failed to import 'cart'. Install the 'cart' package.\n")
    raise


MiB = 1024 * 1024
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, os.pardir))
DATA_DIR = os.path.join(HERE, "data")


def rdtsc() -> float:
    return time.perf_counter()


def _summarize(values: List[float]) -> Dict[str, float]:
    return {
        "min": min(values) if values else 0.0,
        "max": max(values) if values else 0.0,
        "mean": mean(values) if values else 0.0,
        "median": sorted(values)[len(values) // 2] if values else 0.0,
    }


def _gen_bytes(kind: str, size: int) -> bytes:
    if size <= 0:
        return b""
    if kind == "zeros":
        return b"0" * size
    if kind == "pattern":
        return (b"0123456789" * ((size // 10) + 1))[:size]
    if kind == "random":
        # Deterministic but pseudo-random bytes
        # Avoid importing random for speed; use a simple LCG
        buf = bytearray(size)
        a, c, m = 1664525, 1013904223, 2**32
        x = 42
        for i in range(size):
            x = (a * x + c) % m
            buf[i] = (x >> 16) & 0xFF
        return bytes(buf)
    raise ValueError(f"unknown kind: {kind}")


def _ensure_data_file(
    label: str,
    *,
    kind: str | None = None,
    size: int | None = None,
    data: bytes | None = None,
) -> str:
    os.makedirs(DATA_DIR, exist_ok=True)
    path = os.path.join(DATA_DIR, f"{label}.bin")
    expected_size = len(data) if data is not None else size
    need_write = not os.path.exists(path)
    if not need_write and expected_size is not None:
        actual_size = os.path.getsize(path)
        if actual_size != expected_size:
            need_write = True
    if need_write:
        if data is None:
            if kind is None or size is None:
                raise ValueError("either data or (kind and size) must be provided")
            data = _gen_bytes(kind, size)
        with open(path, "wb") as f:
            f.write(data)
    return path


def _load_dataset(
    label: str,
    *,
    kind: str | None = None,
    size: int | None = None,
    data: bytes | None = None,
) -> Tuple[bytes, str]:
    path = _ensure_data_file(label, kind=kind, size=size, data=data)
    with open(path, "rb") as f:
        content = f.read()
    rel_path = os.path.relpath(path, ROOT)
    return content, rel_path


def _bench_pack(data: bytes, header: dict, footer: dict, iters: int) -> Tuple[List[float], List[float]]:
    times: List[float] = []
    thrus: List[float] = []
    for _ in range(iters):
        istream = BytesIO(data)
        ostream = BytesIO()
        t0 = rdtsc()
        cart.pack_stream(istream, ostream, header, footer)
        dt = rdtsc() - t0
        times.append(dt)
        thrus.append(len(data) / dt / MiB if dt > 0 else 0.0)
    return times, thrus


def _bench_unpack(cart_bytes: bytes, iters: int) -> Tuple[List[float], List[float]]:
    times: List[float] = []
    thrus: List[float] = []
    # Determine original size by actually unpacking once (not counted) to get length
    tmp_in = BytesIO(cart_bytes)
    tmp_out = BytesIO()
    cart.unpack_stream(tmp_in, tmp_out)
    plain = tmp_out.getvalue()
    src_len = len(plain)

    for _ in range(iters):
        istream = BytesIO(cart_bytes)
        ostream = BytesIO()
        t0 = rdtsc()
        cart.unpack_stream(istream, ostream)
        dt = rdtsc() - t0
        times.append(dt)
        thrus.append(src_len / dt / MiB if dt > 0 else 0.0)
    return times, thrus


def _bench_metadata(cart_bytes: bytes, iters: int) -> List[float]:
    import tempfile

    times: List[float] = []
    # Write cart to a temp file once, then measure metadata extraction time
    with tempfile.NamedTemporaryFile(delete=True) as tf:
        tf.write(cart_bytes)
        tf.flush()
        for _ in range(iters):
            t0 = rdtsc()
            cart.get_metadata_only(tf.name)
            dt = rdtsc() - t0
            times.append(dt)
    return times


def _bench_is_cart(cart_bytes: bytes, iters: int) -> List[float]:
    times: List[float] = []
    hdr_len = struct.calcsize(cart.MANDATORY_HEADER_FMT)
    header_bytes = cart_bytes[:hdr_len]
    for _ in range(iters):
        t0 = rdtsc()
        cart.is_cart(header_bytes)
        dt = rdtsc() - t0
        times.append(dt)
    return times


def run_benchmarks(profile: str, iters: int) -> Dict:
    results: Dict = {
        "env": {
            "python": sys.version.split(" ")[0],
            "platform": platform.platform(),
            "machine": platform.machine(),
            "implementation": f"{cart.version.major}.{cart.version.minor}.{cart.version.micro}",
        },
        "config": {"iters": iters, "profile": profile},
        "scenarios": {},
    }

    def record(
        name: str,
        *,
        times: List[float],
        thrus: List[float] | None = None,
        bytes_in: int | None = None,
        data_file: str | None = None,
    ):
        results["scenarios"][name] = {
            "iters": iters,
            "bytes_in": bytes_in,
            "time_s": times,
            "time_summary": _summarize(times),
        }
        if thrus is not None:
            results["scenarios"][name]["throughput_mib_s"] = thrus
            results["scenarios"][name]["throughput_summary"] = _summarize(thrus)
        if data_file is not None:
            results["scenarios"][name]["data_file"] = data_file

    # Map to unit tests
    # 1) test_empty
    data = b""
    t, th = _bench_pack(data, {}, {}, iters)
    record("pack_empty", times=t, thrus=th, bytes_in=len(data))

    cart_bytes = BytesIO()
    cart.pack_stream(BytesIO(data), cart_bytes, {}, {})
    t, th = _bench_unpack(cart_bytes.getvalue(), iters)
    record("unpack_empty", times=t, thrus=th, bytes_in=len(data))

    # 2) test_small (1 byte)
    data, data_path = _load_dataset("small_1B", data=b"a")
    header = {"testkey": "testvalue"}
    footer = {"complete": "yes"}
    t, th = _bench_pack(data, header, footer, iters)
    record("pack_small_1B", times=t, thrus=th, bytes_in=len(data), data_file=data_path)

    cart_bytes = BytesIO()
    cart.pack_stream(BytesIO(data), cart_bytes, header, footer)
    t, th = _bench_unpack(cart_bytes.getvalue(), iters)
    record("unpack_small_1B", times=t, thrus=th, bytes_in=len(data), data_file=data_path)

    # 3) test_large (128 MiB zeros)
    data, data_path = _load_dataset("zeros_128MiB", kind="zeros", size=128 * MiB)
    header = {}
    footer = {}
    t, th = _bench_pack(data, header, footer, iters)
    record(
        "pack_large_128MiB_zeros",
        times=t,
        thrus=th,
        bytes_in=len(data),
        data_file=data_path,
    )

    cart_bytes = BytesIO()
    cart.pack_stream(BytesIO(data), cart_bytes, header, footer)
    t, th = _bench_unpack(cart_bytes.getvalue(), iters)
    record(
        "unpack_large_128MiB_zeros",
        times=t,
        thrus=th,
        bytes_in=len(data),
        data_file=data_path,
    )

    # 4) test_simple (metadata + is_cart)
    data, data_path = _load_dataset("pattern_100KB", kind="pattern", size=10 * 10_000)
    header = {"name": "hello.txt"}
    footer = {"digest": "done"}
    # pack
    cart_bytes = BytesIO()
    cart.pack_stream(BytesIO(data), cart_bytes, header, footer)
    cb = cart_bytes.getvalue()
    # unpack perf
    t, th = _bench_unpack(cb, iters)
    record(
        "unpack_simple_pattern_100KB",
        times=t,
        thrus=th,
        bytes_in=len(data),
        data_file=data_path,
    )
    # metadata perf
    t_md = _bench_metadata(cb, iters)
    record("metadata_only", times=t_md)
    # is_cart perf
    t_ic = _bench_is_cart(cb, iters)
    record("is_cart", times=t_ic)

    # Optional (not in unit tests): random 128 MiB for a tougher case
    if profile != "unittests":
        data, data_path = _load_dataset("random_128MiB", kind="random", size=128 * MiB)
        t, th = _bench_pack(data, {}, {}, iters)
        record(
            "pack_large_128MiB_random",
            times=t,
            thrus=th,
            bytes_in=len(data),
            data_file=data_path,
        )
        cart_bytes = BytesIO()
        cart.pack_stream(BytesIO(data), cart_bytes, {}, {})
        t, th = _bench_unpack(cart_bytes.getvalue(), iters)
        record(
            "unpack_large_128MiB_random",
            times=t,
            thrus=th,
            bytes_in=len(data),
            data_file=data_path,
        )

    # Additional random payload sizes
    random_sizes = [
        (10_000, "random_10KB"),
        (100_000, "random_100KB"),
        (1_000_000, "random_1MB"),
        (10_000_000, "random_10MB"),
    ]

    for size, label in random_sizes:
        data, data_path = _load_dataset(label, kind="random", size=size)
        cart_bytes = BytesIO()
        cart.pack_stream(BytesIO(data), cart_bytes, {}, {})
        cart_buf = cart_bytes.getvalue()

        t, th = _bench_pack(data, {}, {}, iters)
        record(
            f"pack_{label}",
            times=t,
            thrus=th,
            bytes_in=len(data),
            data_file=data_path,
        )

        t, th = _bench_unpack(cart_buf, iters)
        record(
            f"unpack_{label}",
            times=t,
            thrus=th,
            bytes_in=len(data),
            data_file=data_path,
        )

    return results


def _format_bytes(num: int | None) -> str:
    if num is None:
        return "-"
    if num == 0:
        return "0 B"
    if num >= MiB:
        return f"{num / MiB:.2f} MiB"
    if num >= 1024:
        return f"{num / 1024:.1f} KiB"
    return f"{num} B"


def _print_results_table(results: Dict) -> None:
    impl = f"{cart.version.major}.{cart.version.minor}.{cart.version.micro}"
    print(f"cart version: {impl}")

    headers = [
        "Scenario",
        "Bytes In",
        "Mean Time (ms)",
        "Min Time (ms)",
        "Max Time (ms)",
        "Mean Throughput (MiB/s)",
        "Data File",
    ]
    rows: List[List[str]] = []
    for name, data in results["scenarios"].items():
        summary = data["time_summary"]
        mean_ms = f"{summary['mean'] * 1_000:.3f}"
        min_ms = f"{summary['min'] * 1_000:.3f}"
        max_ms = f"{summary['max'] * 1_000:.3f}"
        bytes_str = _format_bytes(data.get("bytes_in"))
        thru = data.get("throughput_summary")
        if thru:
            thru_str = f"{thru['mean']:.3f}"
        else:
            thru_str = "-"
        data_file = data.get("data_file", "-")
        rows.append([name, bytes_str, mean_ms, min_ms, max_ms, thru_str, data_file])

    widths = [len(h) for h in headers]
    for row in rows:
        for idx, cell in enumerate(row):
            widths[idx] = max(widths[idx], len(cell))

    def _format_line(cells: List[str]) -> str:
        return " | ".join(cell.ljust(widths[idx]) for idx, cell in enumerate(cells))

    print(_format_line(headers))
    print("-+-".join("-" * w for w in widths))
    for row in rows:
        print(_format_line(row))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--iters", type=int, default=3, help="iterations per scenario")
    ap.add_argument(
        "--profile",
        choices=["unittests", "full"],
        default="unittests",
        help="scenario set to run",
    )
    ap.add_argument("--out", type=str, default="", help="write JSON results to file")
    args = ap.parse_args()

    results = run_benchmarks(args.profile, args.iters)
    out_json = json.dumps(results, indent=2)

    _print_results_table(results)

    if args.out:
        out_path = args.out
        os.makedirs(os.path.dirname(out_path), exist_ok=True)
        with open(out_path, "w", encoding="utf-8") as f:
            f.write(out_json)


if __name__ == "__main__":
    main()
