#!/usr/bin/env python3
"""Write the HCE fitting manifest: every evaluation coefficient with its status.

  free      receives gradient in the linear stage (`--tune complete`)
  excluded  outside the linear model; fitted by the coordinate stage
            (`--tune-kingsafety`) and carried as a fixed residual per row in
            the linear stage
  fixed     never fitted: an algebraic gauge, an invariant, or a slot the
            evaluation cannot activate

The coefficient list comes from a complete vector (`rarog-texel
--write-defaults`), so it cannot drift from `EVAL_PARAM_NAMES`. Every slot
classified as never activating is checked against a `--feature-support`
report: it must be listed there with zero activations, or the script fails.
`rarog-texel` reads the manifest and checks that it covers every coefficient
exactly once.

  python tools/texel/fit_manifest.py --defaults <vector.txt> \\
      --feature-support <feature-support.txt> --out tools/texel/hce_fit_manifest_v1.tsv
"""
from __future__ import annotations

import argparse
import re
import sys

SCHEMA = "rarog-hce-fit-manifest-v1"

# The danger-index selectors (the tuner's KS_DANGER_INPUTS) and the table they
# index: a perturbation moves the bucket, not a coefficient.
DANGER_SELECTORS = [
    "king_safety_unit_minor",
    "king_safety_unit_rook",
    "king_safety_unit_queen",
    "ks_weak_ring",
    "ks_safe_check_knight",
    "ks_safe_check_bishop",
    "ks_safe_check_rook",
    "ks_safe_check_queen",
    "ks_flank_attack",
    "ks_pawnless_flank",
    "ks_shelter_storm",
    "ks_queen_relief",
]


def load_vector(path: str) -> list[tuple[str, int]]:
    """(field, length) in file order, from `name index value` lines."""
    order: list[str] = []
    lengths: dict[str, int] = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            parts = line.split()
            if len(parts) != 3 or line.startswith("#"):
                continue
            name, idx = parts[0], int(parts[1])
            if name not in lengths:
                order.append(name)
                lengths[name] = 0
            if idx != lengths[name]:
                sys.exit(f"{path}: {name} index {idx} out of order")
            lengths[name] += 1
    return [(name, lengths[name]) for name in order]


def load_zero_slots(path: str) -> set[tuple[str, int]]:
    """Slots a feature-support report lists with zero activations."""
    zero = set()
    pattern = re.compile(r"^\s+(\w+)\[(\d+)\]:\s+(\d+)\s*$")
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            m = pattern.match(line)
            if m and int(m.group(3)) == 0:
                zero.add((m.group(1), int(m.group(2))))
    return zero


def classify(field: str, idx: int) -> tuple[str, str, str, bool]:
    """(status, instrument, reason, must_be_inactive) for one coefficient."""
    if field in DANGER_SELECTORS:
        return ("excluded", "coordinate",
                "king-danger selector: moves the table bucket, not a coefficient", False)
    if field == "king_safety_table":
        return ("excluded", "coordinate",
                "king-danger table: shaped against the selectors' index distribution", False)
    if field in ("pst_mg", "pst_eg") and idx % 64 == 0 and idx // 64 < 5:
        return ("fixed", "none",
                "material/PST gauge anchor: a piece value plus C with its 64 squares "
                "minus C is the same evaluator", False)
    if field in ("mg_val", "eg_val") and idx == 5:
        return ("fixed", "none", "king value: both kings are always present, net count zero", False)
    if field in ("imbalance_ours", "imbalance_theirs"):
        pt1, pt2 = divmod(idx, 6)
        if pt2 > pt1:
            return ("fixed", "none",
                    "imbalance upper triangle: eval_imbalance reads only pt2 <= pt1", True)
        if field == "imbalance_theirs" and pt1 == pt2:
            return ("fixed", "none",
                    "imbalance_theirs diagonal: the two sides' products cancel", True)
    if field in ("pawn_connected_mg", "pawn_connected_eg") and idx in (0, 1, 7):
        return ("fixed", "none",
                "pawn support on relative rank 1, 2 or 8: no pawn there, or none behind it", True)
    if field in ("passed_mg", "passed_eg", "pawn_phalanx_mg", "pawn_phalanx_eg") and idx in (0, 7):
        return ("fixed", "none", "no pawn on relative rank 1 or 8", True)
    if field in ("pst_mg", "pst_eg") and idx < 64 and (idx < 8 or idx >= 56):
        return ("fixed", "none", "pawn square on rank 1 or 8", True)
    return ("free", "linear", "", False)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--defaults", required=True)
    ap.add_argument("--feature-support", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    fields = load_vector(args.defaults)
    zero = load_zero_slots(args.feature_support)
    if not zero:
        sys.exit(f"{args.feature_support}: no zero-activation slots parsed")
    names = {name for name, _ in fields}
    for name in DANGER_SELECTORS + ["king_safety_table"]:
        if name not in names:
            sys.exit(f"{name} is not in {args.defaults}")

    rows = []
    counts = {"free": 0, "fixed": 0, "excluded": 0}
    active_fixed = []
    for name, length in fields:
        for idx in range(length):
            status, instrument, reason, inactive = classify(name, idx)
            if inactive and (name, idx) not in zero:
                active_fixed.append(f"{name}[{idx}]")
            counts[status] += 1
            rows.append(f"{name}\t{idx}\t{status}\t{instrument}\t{reason}")
    if active_fixed:
        sys.exit("classified as never active but not zero in the report: " + ", ".join(active_fixed))

    total = sum(counts.values())
    with open(args.out, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(f"# {SCHEMA}\n")
        handle.write("# Generated by tools/texel/fit_manifest.py; read by rarog-texel --manifest.\n")
        handle.write(f"# coefficients {total}: free {counts['free']}, fixed {counts['fixed']}, "
                     f"excluded {counts['excluded']}\n")
        handle.write("# field\tindex\tstatus\tinstrument\treason\n")
        for row in rows:
            handle.write(row + "\n")
    print(f"{args.out}: {total} coefficients, free {counts['free']}, fixed {counts['fixed']}, "
          f"excluded {counts['excluded']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
