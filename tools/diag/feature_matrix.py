#!/usr/bin/env python3
"""Check every shipped feature combination compiles (PLAN 4.10.12).

Which headers, items and modules arrive under a given `cfg` differs per feature
combination, so a module can compile in the default build and in `--features
diag` and still fail under both together. CI built four single-feature
configurations and no combination at all, which is the shape that lets a
configuration rot until the day it is needed -- and `tune` is what SPSA runs on,
so its rotting would be found at the worst possible moment.

**`--all-features` is not one of the shipped configurations.** It enables
`texel`, which bypasses the eval and pawn caches and must never be measured;
AGENTS.md records a depth sweep whose conclusion was reversed by exactly that
binary being left in `target/release/`. This tool therefore checks combinations
one at a time rather than reaching for `--all-features`, and says so when it
finds the combination that includes `texel`.

`cargo check --all-targets` rather than `build`: it catches the same
compilation errors -- which is what a feature matrix is for -- at a fraction of
the cost, so the whole matrix is runnable on demand rather than only in CI.

Example:

  python tools/diag/feature_matrix.py
  python tools/diag/feature_matrix.py --features tune,diag,ablate --release
"""

from __future__ import annotations

import argparse
import itertools
import subprocess
import sys
import time

# Every feature the crate declares. Kept in sync with Cargo.toml by
# `test_feature_matrix.py::test_the_matrix_covers_every_declared_feature`, so
# adding a feature and forgetting to check it fails the suite.
SHIPPED_FEATURES = ["tune", "diag", "ablate", "texel"]

# Features that change what is MEASURED rather than only what is exposed. A
# binary built with one of these must never be used for a strength number, and
# the matrix says so out loud when it checks such a combination.
NEVER_MEASURE = {"texel", "ablate", "tune"}


def combinations(features: list[str]) -> list[tuple[str, ...]]:
    """Every subset, smallest first, so a failure names the simplest case."""
    out: list[tuple[str, ...]] = []
    for size in range(len(features) + 1):
        out.extend(itertools.combinations(features, size))
    return out


def describe(combo: tuple[str, ...]) -> str:
    return "default" if not combo else ",".join(combo)


def check(combo: tuple[str, ...], release: bool, verbose: bool) -> tuple[bool, float]:
    cmd = ["cargo", "check", "--all-targets"]
    if release:
        cmd.append("--release")
    if combo:
        cmd += ["--features", ",".join(combo)]
    started = time.perf_counter()
    proc = subprocess.run(cmd, capture_output=not verbose, text=True)
    elapsed = time.perf_counter() - started
    if proc.returncode != 0 and not verbose:
        sys.stdout.write(proc.stdout or "")
        sys.stderr.write(proc.stderr or "")
    return proc.returncode == 0, elapsed


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--features", default=",".join(SHIPPED_FEATURES),
                    help="comma-separated features to enumerate over")
    ap.add_argument("--release", action="store_true")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    features = [f for f in args.features.split(",") if f]
    unknown = sorted(set(features) - set(SHIPPED_FEATURES))
    if unknown:
        ap.error(f"unknown feature(s): {', '.join(unknown)}")

    combos = combinations(features)
    print(f"{len(combos)} configurations over {len(features)} features "
          f"({'release' if args.release else 'debug'})\n")

    failures = []
    for combo in combos:
        label = describe(combo)
        ok, elapsed = check(combo, args.release, args.verbose)
        flag = ""
        if set(combo) & NEVER_MEASURE:
            flag = "  [never measure this binary]"
        print(f"  {'ok  ' if ok else 'FAIL'} {label:<28} {elapsed:6.1f}s{flag}",
              flush=True)
        if not ok:
            failures.append(label)

    print()
    if failures:
        print(f"FAIL: {len(failures)} configuration(s) do not compile: "
              f"{', '.join(failures)}")
        return 1
    print(f"all {len(combos)} configurations compile.")
    print("Reminder: --all-features is NOT a shipped configuration. It enables "
          "`texel`,\nwhich bypasses the eval and pawn caches; a binary left in "
          "target/release by\n`cargo test --all-features` must never be "
          "measured (AGENTS.md).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
