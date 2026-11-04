#!/usr/bin/env python3
"""Check the universal x86-64 binary's tier selection under Intel SDE.

Each emulated CPU must start the tier the dispatcher's rule gives it, named
in the start-up line, and reproduce the declared `bench 13` there. A forced
tier the CPU cannot run must be refused before any engine code runs. The CPUs
cover every tier an Intel CPU selects (PLAN E.3.3):

- Pentium 4 Prescott (Linux only; Windows 11 refuses it under SDE): base. It
  has SSE3 but not SSSE3, SSE4 or POPCNT.
- Nehalem (Linux only, likewise), Sandy Bridge and Ivy Bridge: sse42. They
  have SSE4.2 and POPCNT but not AVX2.
- Haswell, Skylake, Ice Lake and Alder Lake: pext.

SDE cannot emulate AMD, so the slow-PEXT rule (family 15h and 17h) is the
unit truth table's (`src/cpu_tier.rs`).

    python tools/diag/check_universal.py --binary <universal> --sde <dir> --bench <nodes>
"""

from __future__ import annotations

import argparse
import os
import platform
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

SELECTION = [
    ("snb", "sse42"),
    ("ivb", "sse42"),
    ("hsw", "pext"),
    ("skl", "pext"),
    ("icl", "pext"),
    ("adl", "pext"),
]
LINUX_ONLY = [("p4p", "base"), ("nhm", "sse42")]
BANNER = re.compile(r"^Rarog .* \((base|sse42|avx2|pext)\)\s*$", re.MULTILINE)
NODES = re.compile(r"Nodes searched\s*:\s*(\d+)")
FORCE = "RAROG_ISA"


def run(sde: Path, cpu: str, binary: Path, args: list[str], forced: str | None,
        timeout: int) -> tuple[int, str]:
    env = dict(os.environ)
    env.pop(FORCE, None)
    if forced:
        env[FORCE] = forced
    try:
        result = subprocess.run([str(sde), f"-{cpu}", "--", str(binary), *args],
                                capture_output=True, text=True, env=env, timeout=timeout)
    except subprocess.TimeoutExpired:
        return -1, f"timed out after {timeout} s"
    return result.returncode, result.stdout + result.stderr


def check_selection(sde: Path, binary: Path, cpu: str, expected: str, nodes: str,
                    timeout: int) -> str | None:
    code, output = run(sde, cpu, binary, ["bench", "13"], None, timeout)
    banner = BANNER.search(output)
    counted = NODES.findall(output)
    tier = banner.group(1) if banner else "-"
    got = counted[-1] if counted else "-"
    if code != 0 or tier != expected or got != nodes:
        return (f"CPU {cpu}: expected {expected}/{nodes}, got {tier}/{got} (exit {code})\n"
                + "\n".join(output.splitlines()[-15:]))
    print(f"CPU {cpu}: {tier}, bench {got}")
    return None


def check_refusal(sde: Path, binary: Path, timeout: int) -> str | None:
    # Sandy Bridge has no AVX2: forcing pext must be refused, with the reason
    # on stdout, before any engine code runs.
    code, output = run(sde, "snb", binary, ["bench", "1"], "pext", timeout)
    if code != 1 or "cannot run" not in output or BANNER.search(output):
        return f"CPU snb with {FORCE}=pext: not refused (exit {code})\n{output[-2000:]}"
    print(f"CPU snb with {FORCE}=pext: refused")
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--sde", required=True, type=Path,
                        help="SDE's launcher (sde64, sde.exe) or the directory holding it")
    parser.add_argument("--bench", required=True, help="the declared bench 13 node count")
    parser.add_argument("--timeout", type=int, default=1800, help="seconds per emulated run")
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 2)
    args = parser.parse_args()
    if args.sde.is_dir():
        launchers = [args.sde / name for name in ("sde64", "sde64.exe", "sde.exe", "sde")]
        found = [path for path in launchers if path.is_file()]
        if not found:
            parser.error(f"no SDE launcher in {args.sde}")
        args.sde = found[0]
    if not args.binary.is_file():
        parser.error(f"binary not found: {args.binary}")

    pairs = SELECTION + (LINUX_ONLY if platform.system() == "Linux" else [])
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        jobs = [pool.submit(check_selection, args.sde, args.binary, cpu, tier, args.bench,
                            args.timeout) for cpu, tier in pairs]
        jobs.append(pool.submit(check_refusal, args.sde, args.binary, args.timeout))
        failures = [failure for job in jobs if (failure := job.result())]
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    if failures:
        return 1
    print(f"universal selection: PASS on {len(pairs)} CPUs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
