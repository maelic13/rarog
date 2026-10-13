#!/usr/bin/env python3
"""Write a neutral start vector for an initialization control (PLAN C.8).

Every coefficient the fitting manifest marks free and fits in the linear
stage is set to zero, except the piece values (`mg_val` and `eg_val`, pawn to
queen), which take a conventional 100 / 300 / 300 / 500 / 900 in both phases.
Fixed coefficients (structure, gauge anchors) and the coordinate stage's
king-danger coordinates keep the accepted vector's values, so the control
asks only whether the linear surface's start biases the fit.

  python tools/texel/neutral_vector.py --defaults <complete vector> \
      --manifest tools/texel/hce_fit_manifest_v5.tsv --out <neutral vector>
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

PIECE_VALUES = {0: 100, 1: 300, 2: 300, 3: 500, 4: 900}
VALUE_FIELDS = ("mg_val", "eg_val")


def read_vector(path: Path) -> list[tuple[str, int, int]]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        field, index, value = line.split()
        rows.append((field, int(index), int(value)))
    return rows


def read_manifest(path: Path) -> dict[tuple[str, int], tuple[str, str]]:
    out = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t")
        out[(parts[0], int(parts[1]))] = (parts[2], parts[3])
    return out


def neutral(vector, manifest) -> list[tuple[str, int, int]]:
    keys = {(f, i) for f, i, _ in vector}
    if keys != set(manifest):
        missing = sorted(set(manifest) - keys)[:5]
        extra = sorted(keys - set(manifest))[:5]
        raise ValueError(f"vector and manifest disagree: missing {missing}, extra {extra}")
    out = []
    for field, index, value in vector:
        status, instrument = manifest[(field, index)]
        if status == "free" and instrument == "linear":
            if field in VALUE_FIELDS and index in PIECE_VALUES:
                value = PIECE_VALUES[index]
            else:
                value = 0
        out.append((field, index, value))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--defaults", required=True, type=Path)
    ap.add_argument("--manifest", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()
    rows = neutral(read_vector(args.defaults), read_manifest(args.manifest))
    args.out.write_text("".join(f"{f} {i} {v}\n" for f, i, v in rows), encoding="utf-8")
    changed = sum(a != b for a, b in zip(read_vector(args.defaults), rows))
    print(f"{len(rows)} coefficients written to {args.out}; {changed} changed from the defaults")
    return 0


if __name__ == "__main__":
    sys.exit(main())
