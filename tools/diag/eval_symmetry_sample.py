"""The versioned sample behind `tests/eval_invariants.rs`'s colour-symmetry sweep.

The evaluation must score a position and its colour-flipped vertical mirror
the same. Ten seed games never reached three rooks of one colour, where
connected rooks were once paired by square order and so scored differently in
the mirror; a sweep over corpus rows did. This writes the rows the standing
test sweeps, one FEN per line under a source tag:

- `sample`: SAMPLE_ROWS distinct rows drawn with SEED from the held-out file
  (`hce-v6-tb/validation.csv`, `FEN;label` rows; the label is dropped);
- `audit`: the four held-out rows that exposed the three-rook asymmetry;
- `promoted`: constructed legal positions with promoted material (three
  rooks, three queens, three knights) for White, for Black and for both.

The source file's SHA-256 is written into the header, and the run refuses a
source whose row count differs from the one the sample was drawn from, so the
same command reproduces every row (the file is written with LF endings; a
checkout may convert them).

  python tools/diag/eval_symmetry_sample.py --source tools/texel/data/hce-v6-tb/validation.csv --out tests/data/eval-symmetry-v1.tsv
"""
from __future__ import annotations

import argparse
import hashlib
import random
import sys

import chess

SAMPLE_ROWS = 2000
SEED = 20261015
SOURCE_ROWS = 194444

AUDIT = [
    "8/8/2P4p/r6k/8/6Q1/r1r5/1K6 w - - 0 17",
    "3Q4/5k2/2P4p/r6P/8/8/r1r5/1K6 w - - 0 14",
    "8/8/2PQ3p/r5kP/8/8/r1r5/1K6 w - - 4 16",
    "8/8/2P4p/r2k4/6Q1/8/r1r5/1K6 w - - 8 21",
]

PROMOTED = [
    # Three rooks. a1, a2, h2: the square-order pair (a1, h2) is not aligned
    # but its mirror's (a7, a8) is.
    "4k3/8/8/8/8/8/R6R/R3K3 w - - 0 1",
    "r3k3/r6r/8/8/8/8/8/4K3 b - - 0 1",
    "r3k2r/r7/8/8/8/8/R7/R3K2R w - - 0 1",
    # Three rooks on one rank: the outer pair is blocked by the middle rook.
    "4k3/8/8/8/8/8/8/R2R1KR1 w - - 0 1",
    "r2r1kr1/pp4pp/8/8/8/8/PP4PP/R2R1KR1 b - - 0 1",
    "2r1k3/2r5/8/1R6/1R6/8/1R6/6K1 w - - 0 1",
    # Three queens.
    "4k3/8/8/8/8/8/QQ6/Q5K1 w - - 0 1",
    "q5k1/qq6/8/8/8/8/8/4K3 b - - 0 1",
    "q3k2q/q7/8/8/8/8/Q7/Q3K2Q w - - 0 1",
    # Three knights.
    "4k3/8/8/8/8/2N5/1N6/1N2K3 w - - 0 1",
    "1n2k3/1n6/2n5/8/8/8/8/4K3 b - - 0 1",
    "1n2k1n1/4n3/8/3p4/3P4/8/4N3/1N2K1N1 w - - 0 1",
]


def checked(fen: str) -> str:
    board = chess.Board(fen)
    if not board.is_valid():
        raise SystemExit(f"constructed position is not legal: {fen} ({board.status()!r})")
    return fen


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--source", required=True, help="hce-v6-tb validation.csv")
    parser.add_argument("--out", required=True, help="tests/data/eval-symmetry-v1.tsv")
    args = parser.parse_args()

    with open(args.source, "rb") as handle:
        raw = handle.read()
    digest = hashlib.sha256(raw).hexdigest()
    lines = [line for line in raw.decode("utf-8").splitlines() if line.strip()]
    if len(lines) != SOURCE_ROWS:
        print(f"source has {len(lines)} rows, the sample was drawn from {SOURCE_ROWS}", file=sys.stderr)
        return 1

    picks = sorted(random.Random(SEED).sample(range(len(lines)), SAMPLE_ROWS))
    rows = [("sample", lines[i].split(";")[0].strip()) for i in picks]
    rows += [("audit", checked(fen)) for fen in AUDIT]
    rows += [("promoted", checked(fen)) for fen in PROMOTED]

    with open(args.out, "w", encoding="utf-8", newline="\n") as out:
        out.write("# eval-symmetry-v1: source<TAB>FEN; each row must evaluate equal to its colour-flipped mirror\n")
        out.write(f"# sample: {SAMPLE_ROWS} rows, seed {SEED}, from hce-v6-tb/validation.csv sha256 {digest}\n")
        out.write("# written by tools/diag/eval_symmetry_sample.py\n")
        for source, fen in rows:
            out.write(f"{source}\t{fen}\n")
    print(f"wrote {len(rows)} rows to {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
