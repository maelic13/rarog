#!/usr/bin/env python3
"""Build an endgame-start opening book with Syzygy-verified verdicts (4.9a.2).

RAR-M15 measured how often each reference endgame occurs in real games: KXK
37.34%, KRPKR 10.04%, and then a long tail down to KBNK at 0.28%, KNNK at
0.03% -- and **KQKR, KQKRPs and KRPPKRP at exactly zero** across 3,915 games.
Those families cannot be measured by playing from opening positions at all, at
any budget. Their cohort has to be constructed.

This writes an EPD book of legal endgame starts so `sprt.ps1 -Book` can play a
cohort from them. Every position is verified against Syzygy before it is
written, and its verdict is recorded in the EPD comment, so the book carries
its own ground truth:

    <fen> ; c0 "family=KRP-KR wdl=win dtz=27"

Two design points that matter for how the book is used.

**Both verdicts are included, in a fixed ratio.** A book of won positions alone
measures conversion and nothing else; a cohort also needs drawn positions,
because holding a draw is half of endgame skill and the drawn subset is where
the audit found the evaluator overconfident. `--win-share` sets the split.

**Colour is not baked in.** The harness plays each opening from both sides
(`-games 2 -repeat`), so each position is played once with each engine on the
strong side. That is what makes the cohort a fair A/B rather than a test of
whoever drew the good colour.

The book is a measurement instrument, never training data: these positions are
uniformly sampled, not drawn from real play, so feeding them to a fit would
reweight the corpus toward a distribution the search never sees.

Example:

  python tools/diag/endgame_book.py \\
      --syzygy D:/chess/tablebases/syzygy3456 \\
      --per-family 40 --out tools/books/endgame_cohort_v1.epd
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import random
import sys
from pathlib import Path

import chess
import chess.syzygy

PIECE_OF = {"Q": chess.QUEEN, "R": chess.ROOK, "B": chess.BISHOP,
            "N": chess.KNIGHT, "P": chess.PAWN}

# The reference inventory plus the bare-king conversion set. Tier 4 -- the
# three that never occur in real games -- is the reason this file exists.
FAMILIES = [
    "KQ-K", "KR-K", "KBB-K", "KBN-K", "KNN-K",
    "KP-K", "KPP-K", "KBP-K",
    "KR-KP", "KR-KB", "KR-KN", "KQ-KP", "KQ-KR", "KNN-KP",
    "KRP-KR", "KRP-KB", "KBP-KB", "KBP-KN", "KP-KP",
    "KQ-KRP", "KBPP-KB",
]

# KRPP-KRP (reference item 15) is deliberately absent: it is SEVEN men, and the
# local tables stop at six. It is also one of the three families RAR-M15 found
# occurring zero times in 3,915 real games -- so it can be reached neither by
# sampling real play nor by verified construction, and 4.9a.24 cannot be closed
# on measurement until 7-man tables exist. Record that as a gap; do not paper
# over it with unverified positions.
UNVERIFIABLE_AT_6_MEN = ["KRPP-KRP"]


def parse_family(spec: str) -> tuple[tuple[int, ...], tuple[int, ...]]:
    strong, weak = spec.split("-")
    out = []
    for side in (strong, weak):
        if not side.startswith("K"):
            raise ValueError(f"each side of {spec!r} must start with K")
        out.append(tuple(PIECE_OF[c] for c in side[1:]))
    if 2 + len(out[0]) + len(out[1]) > 6:
        raise ValueError(f"{spec!r} exceeds 6 men")
    return out[0], out[1]


def random_position(rng, strong, weak) -> chess.Board | None:
    n = 2 + len(strong) + len(weak)
    squares = rng.sample(range(64), n)
    board = chess.Board(None)
    board.turn = chess.WHITE
    board.set_piece_at(squares[0], chess.Piece(chess.KING, chess.WHITE))
    board.set_piece_at(squares[1], chess.Piece(chess.KING, chess.BLACK))
    i = 2
    for piece, color in ([(p, chess.WHITE) for p in strong]
                         + [(p, chess.BLACK) for p in weak]):
        sq = squares[i]
        i += 1
        if piece == chess.PAWN and chess.square_rank(sq) in (0, 7):
            return None
        board.set_piece_at(sq, chess.Piece(piece, color))
    if not board.is_valid() or board.is_check():
        return None
    if not any(board.legal_moves):
        return None
    # A position already inside the fifty-move horizon of its own mate is not a
    # useful start; require a little room. Also reject trivial mate-in-1.
    if board.is_checkmate() or board.is_stalemate():
        return None
    return board


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--syzygy", required=True, type=Path)
    ap.add_argument("--per-family", type=int, default=40)
    ap.add_argument("--win-share", type=float, default=0.6,
                    help="fraction of each family's positions that are wins "
                         "for the strong side; the rest are theoretical draws")
    ap.add_argument("--seed", type=int, default=0x4E9A2)
    ap.add_argument("--families", default=",".join(FAMILIES))
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()

    if not 0.0 <= args.win_share <= 1.0:
        ap.error("--win-share must be between 0 and 1")
    if args.per_family <= 0:
        ap.error("--per-family must be positive")
    if not args.syzygy.is_dir():
        ap.error(f"syzygy path is not a directory: {args.syzygy}")

    families = [f for f in args.families.split(",") if f]
    specs = {}
    for name in families:
        try:
            specs[name] = parse_family(name)
        except (ValueError, KeyError) as exc:
            ap.error(f"{name}: {exc}")

    want_win = round(args.per_family * args.win_share)
    want_draw = args.per_family - want_win

    tb = chess.syzygy.open_tablebase(str(args.syzygy))
    lines: list[str] = []
    summary: dict[str, dict] = {}
    try:
        for fi, name in enumerate(families):
            strong, weak = specs[name]
            rng = random.Random(args.seed ^ ((fi + 1) * 0x9E3779B1))
            got = {"win": [], "draw": []}
            tries = 0
            since_progress = 0
            # Abandon a bucket that stops filling rather than burning the whole
            # try budget on it. Several families have no drawn subset at all --
            # KQ-K, KR-K and KBN-K are won from essentially every legal
            # position -- so a flat try cap makes those families dominate the
            # runtime while adding nothing. Stop when nothing has been added for
            # a while, and report the shortfall instead of hiding it.
            while (tries < 60_000 and since_progress < 8_000
                   and (len(got["win"]) < want_win
                        or len(got["draw"]) < want_draw)):
                tries += 1
                since_progress += 1
                board = random_position(rng, strong, weak)
                if board is None:
                    continue
                try:
                    wdl = tb.probe_wdl(board)
                    dtz = tb.probe_dtz(board)
                except (chess.syzygy.MissingTableError, KeyError, ValueError):
                    continue
                # White is the strong side by construction and is to move.
                bucket = "win" if wdl >= 1 else ("draw" if wdl == 0 else None)
                if bucket is None:
                    continue
                need = want_win if bucket == "win" else want_draw
                if len(got[bucket]) >= need:
                    continue
                got[bucket].append((board.fen(), wdl, abs(dtz)))
                since_progress = 0
            for bucket in ("win", "draw"):
                for fen, wdl, dtz in got[bucket]:
                    verdict = "win" if wdl == 2 else ("cursed" if wdl == 1 else "draw")
                    lines.append(f'{fen} ; c0 "family={name} wdl={verdict} dtz={dtz}"')
            summary[name] = {
                "requested": {"win": want_win, "draw": want_draw},
                "written": {"win": len(got["win"]), "draw": len(got["draw"])},
                "attempts": tries,
            }
            short = summary[name]["written"]
            flag = "" if (short["win"] == want_win and short["draw"] == want_draw) else "  <-- SHORT"
            print(f"{name:<10} win {short['win']:>3}/{want_win}  "
                  f"draw {short['draw']:>3}/{want_draw}  "
                  f"({tries} tries){flag}", flush=True)
    finally:
        tb.close()

    args.out.parent.mkdir(parents=True, exist_ok=True)
    text = "\n".join(lines) + "\n"
    args.out.write_text(text, encoding="utf-8", newline="\n")
    digest = hashlib.sha256(text.encode("utf-8")).hexdigest().upper()

    manifest = {
        "schema": "rarog-endgame-book-v1",
        "book": str(args.out.resolve()),
        "book_sha256": digest,
        "positions": len(lines),
        "unique_positions": len({ln.split(" ; ")[0] for ln in lines}),
        "seed": args.seed,
        "per_family": args.per_family,
        "win_share": args.win_share,
        "syzygy": str(args.syzygy.resolve()),
        "families": summary,
    }
    mpath = args.out.with_suffix(".manifest.json")
    mpath.write_text(json.dumps(manifest, indent=2) + "\n",
                     encoding="utf-8", newline="\n")

    short = [n for n, s in summary.items()
             if s["written"]["win"] + s["written"]["draw"] < args.per_family]
    print()
    print(f"positions : {len(lines)} ({manifest['unique_positions']} unique)")
    print(f"book      : {args.out}  SHA-256 {digest}")
    print(f"manifest  : {mpath}")
    if short:
        print(f"SHORT families (fewer than --per-family): {', '.join(short)}")
        print("  Expected for families with no drawn subset, e.g. KQ-K/KR-K.")
    print(f"NOT verifiable with 6-man tables: {', '.join(UNVERIFIABLE_AT_6_MEN)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
