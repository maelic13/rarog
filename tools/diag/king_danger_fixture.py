"""C.3.1's king-danger fixture: the donor's king-danger components on 500
positions, as Rarog's producer and king term must reproduce them.

`sample` draws 700 candidate rows (seed 20261008) from RAR-E21's scores file, skipping
positions in check (the donor's `eval` prints no trace there), and writes them
in the scores format. `king_subterms.py collect` then scores the sample with
Stockfish `9587eeeb` built with `patches/sf9587_king_subterms.patch` followed
by `patches/sf9587_plain_maps.patch` (the donor's piece attacks made plain, as
Rarog's are: no x-ray through queens or own rooks, no pin-line restriction).
`write` keeps the first 500 rows whose king term ran for both kings (the
donor's lazy exit skips it, leaving zeros it never computed) and turns them
into the versioned fixture, one row per position: the
FEN, then for White's king and Black's king the ring attackers' count and
weight, the weak ring squares, the unsafe checks, the blockers, the
king-adjacent attacks, the safe checks by rook, queen, bishop and knight, and
the ring size.

  python tools/diag/king_danger_fixture.py sample --scores tools/results/donor-residual-20261005/scores.csv --out <dir>/sample.csv
  python tools/diag/king_subterms.py collect --scores <dir>/sample.csv --sf <plain stockfish.exe> --out <dir>
  python tools/diag/king_danger_fixture.py write --dump <dir>/ksdump.csv --out tests/data/king-danger-9587eeeb-plain.tsv
"""
from __future__ import annotations

import argparse
import random
import sys

import chess

ROWS = 500
CANDIDATES = 700
SEED = 20261008
FIELDS = ["att_count", "att_weight", "weak", "unsafe", "blockers", "king_attacks",
          "rook_checks", "queen_checks", "bishop_checks", "knight_checks", "ring"]


def sample(args: argparse.Namespace) -> int:
    with open(args.scores, encoding="utf-8") as handle:
        header = handle.readline()
        rows = [line for line in handle if line.strip()]
    order = list(range(len(rows)))
    random.Random(SEED).shuffle(order)
    chosen = []
    for i in order:
        fen = rows[i].rstrip("\n").split(";")[-1]
        if not chess.Board(fen).is_check():
            chosen.append(i)
        if len(chosen) == CANDIDATES:
            break
    chosen.sort()
    with open(args.out, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(header)
        handle.writelines(rows[i] for i in chosen)
    print(f"sampled {len(chosen)} of {len(rows)} rows (seed {SEED})")
    return 0


def write(args: argparse.Namespace) -> int:
    with open(args.dump, encoding="utf-8") as handle:
        names = handle.readline().rstrip("\n").split(";")
        lines = [line.rstrip("\n").split(";") for line in handle if line.strip()]
    if any("invalid" in cells for cells in lines):
        sys.exit("the dump has invalid rows; the fixture needs every position scored")
    col = {name: i for i, name in enumerate(names)}
    lines = [c for c in lines if c[col["w_ran"]] == "1" and c[col["b_ran"]] == "1"][:ROWS]
    if len(lines) < ROWS:
        sys.exit(f"only {len(lines)} rows ran the king term; sample more candidates")
    out = ["fen\t" + "\t".join([f"w_{f}" for f in FIELDS] + [f"b_{f}" for f in FIELDS])]
    for cells in lines:
        values = [cells[col[f"w_{f}"]] for f in FIELDS] + [cells[col[f"b_{f}"]] for f in FIELDS]
        out.append(cells[-1] + "\t" + "\t".join(values))
    with open(args.out, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(out) + "\n")
    print(f"wrote {args.out}: {len(lines)} positions")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("sample")
    s.add_argument("--scores", required=True)
    s.add_argument("--out", required=True)
    w = sub.add_parser("write")
    w.add_argument("--dump", required=True)
    w.add_argument("--out", required=True)
    args = ap.parse_args()
    return sample(args) if args.cmd == "sample" else write(args)


if __name__ == "__main__":
    sys.exit(main())
