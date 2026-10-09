"""Score RAR-E17's rows with Stockfish `9587eeeb`'s own term table (RAR-E22).

RAR-E17 took its per-family terms from Stockfish 11's `eval` trace and its
total from `9587eeeb`'s. This writes a file in `scores.csv`'s format whose
family columns hold `9587eeeb`'s rows (its `Winnable` row, which folds the
scale factor into its eg, stands in the `initiative` columns) and whose
`sf11_total` holds its `Final evaluation`, so `donor_residual.py analyse`
compares families and total from one version. `rarog`, `sf_control` and the
labels are copied from RAR-E17's file.

  python tools/diag/donor_terms_9587.py --scores tools/results/donor-residual-20261005/scores.csv \
      --sf <stockfish.exe> --out tools/results/king-subterms-20261006/scores-9587.csv
"""
from __future__ import annotations

import argparse
import os
import sys
from concurrent.futures import ProcessPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import donor_residual as dr  # noqa: E402

# `9587eeeb`'s row names in the order of donor_residual.SF11_TERMS; the last
# one differs by name only in what it prints.
ROW_FOR_TERM = {term: term for term in dr.SF11_TERMS}
ROW_FOR_TERM["Initiative"] = "Winnable"


def convert_block(block: str) -> dict[str, tuple[float, float]] | None:
    """The family terms keyed by Stockfish 11's names, plus the final total
    under `_total`, or None when the table or the total is missing."""
    terms, _ = dr.parse_sf11_trace(block)
    total = dr.parse_oracle_total(block)
    if total is None:
        return None
    out: dict[str, tuple[float, float]] = {}
    for term, row in ROW_FOR_TERM.items():
        if row not in terms:
            return None
        out[term] = terms[row]
    out["_total"] = (total, total)
    return out


def score_chunk(args: tuple[str, list[list[str]], dict[str, int]]) -> list[str]:
    sf, rows, col = args
    blocks = dr.run_eval(sf, [], [r[-1] for r in rows])
    out: list[str] = []
    for row, block in zip(rows, blocks):
        terms = convert_block(block)
        if terms is None:
            out.append(f"{row[col['idx']]};{row[col['label']]};invalid;{row[-1]}")
            continue
        cells = [row[col["idx"]], row[col["label"]], row[col["rarog"]], row[col["sf_control"]],
                 f"{terms['_total'][0]:.2f}"]
        for term in dr.SF11_TERMS:
            cells.append(f"{terms[term][0]:.2f}")
            cells.append(f"{terms[term][1]:.2f}")
        cells.append(row[-1])
        out.append(";".join(cells))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--scores", required=True)
    ap.add_argument("--sf", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--chunk", type=int, default=2000)
    ap.add_argument("--workers", type=int, default=16)
    args = ap.parse_args()

    with open(args.scores, encoding="utf-8") as handle:
        names = handle.readline().rstrip("\n").split(";")
        col = {n: i for i, n in enumerate(names)}
        rows = [line.rstrip("\n").split(";") for line in handle]
    if args.limit:
        rows = rows[: args.limit]
    assert dr.header() == ";".join(names), "scores.csv header differs from donor_residual's"
    chunks = [rows[i : i + args.chunk] for i in range(0, len(rows), args.chunk)]
    lines = [dr.header()]
    invalid = 0
    with ProcessPoolExecutor(max_workers=args.workers) as pool:
        for out in pool.map(score_chunk, [(args.sf, c, col) for c in chunks]):
            invalid += sum(1 for o in out if ";invalid;" in o)
            lines.extend(out)
    with open(args.out, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(lines) + "\n")
    print(f"wrote {args.out}: {len(rows)} rows, {invalid} invalid; sha256 {dr.sha256_file(args.out)}")
    return 1 if invalid else 0


if __name__ == "__main__":
    sys.exit(main())
