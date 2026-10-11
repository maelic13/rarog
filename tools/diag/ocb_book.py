#!/usr/bin/env python3
"""Build an opposite-coloured-bishop endgame-start book from game positions.

Endgame-start cohorts for a rule that only fires in one material class must be
sampled from where games actually go: generated placements are mostly not
game-like, and above six men no tablebase can label them anyway. This takes a
`rarog-texel --dump-scores ... --scale` file of self-play positions (from a
corpus the rule was not fitted on), keeps the opposite-bishop positions of
seven men or more in two strata, and writes an EPD book for `colosseum.ps1
-Book`:

  pure    one bishop each on opposite colours, pawns, nothing else; at least
          three pawns (seven men or more)
  near    the same plus exactly one further piece each, of the same type
          (both a knight, both a rook or both a queen); at least three pawns.
          These carry the decision whether to trade into a pure ending.

Filters, in order: legal and not in check, at least one legal move; the
dumped static score within `--max-abs-cp` (undecided starts); unique by the
first four FEN fields; not present in any `--exclude` file (the corpus the
rule was fitted on). Then `--per-stratum` positions are drawn from each
stratum with `--seed`. The labels in the dump are never read.

The manifest records every input's SHA-256, the counts at each filter and the
book's own SHA-256.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import chess  # noqa: E402

from ocb_scale_screen import PIECES, PURE, classify, read_dump  # noqa: E402


def epd_key(fen: str) -> str:
    return " ".join(fen.split()[:4])


def stratum(fen: str) -> str | None:
    """'pure', 'near' or None, by the definitions in the module docstring."""
    cohort, pawns, _w, _b, _s = classify(fen)
    if pawns < 3:
        return None
    if cohort == PURE:
        return "pure"
    if cohort != PIECES:
        return None
    placement = fen.split(" ", 1)[0]
    white = [ch for ch in placement if ch in "NRQ"]
    black = [ch.upper() for ch in placement if ch in "nrq"]
    if len(white) == 1 and white == black:
        return "near"
    return None


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest().upper()


def build(dump: Path, excludes: list[Path], max_abs_cp: int, per_stratum: int,
          seed: int) -> tuple[list[tuple[str, str, int]], dict]:
    counts = {"rows": 0, "stratum": {"pure": 0, "near": 0}, "legal": 0,
              "within_cp": 0, "unique": 0, "excluded": 0}
    candidates: dict[str, dict[str, tuple[str, int]]] = {"pure": {}, "near": {}}
    for fen, _label, score, _scale in read_dump(dump):
        counts["rows"] += 1
        name = stratum(fen)
        if name is None:
            continue
        counts["stratum"][name] += 1
        board = chess.Board(fen)
        if not board.is_valid() or board.is_check() or not any(board.legal_moves):
            continue
        counts["legal"] += 1
        if abs(score) > max_abs_cp:
            continue
        counts["within_cp"] += 1
        key = epd_key(fen)
        if key not in candidates[name]:
            candidates[name][key] = (fen, score)
    counts["unique"] = {k: len(v) for k, v in candidates.items()}
    excluded = set()
    for path in excludes:
        with path.open() as handle:
            for line in handle:
                sep = line.rfind(";")
                if sep > 0:
                    excluded.add(epd_key(line[:sep]))
    rng = random.Random(seed)
    book = []
    for name in ("pure", "near"):
        keys = sorted(k for k in candidates[name] if k not in excluded)
        counts["excluded"] += len(candidates[name]) - len(keys)
        if len(keys) < per_stratum:
            raise SystemExit(f"stratum {name}: only {len(keys)} positions, need {per_stratum}")
        for key in rng.sample(keys, per_stratum):
            fen, score = candidates[name][key]
            book.append((key, name, score))
    rng.shuffle(book)
    return book, counts


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--dump", required=True, type=Path)
    ap.add_argument("--exclude", type=Path, action="append", default=[])
    ap.add_argument("--max-abs-cp", type=int, default=300)
    ap.add_argument("--per-stratum", type=int, required=True)
    ap.add_argument("--seed", type=int, required=True)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()
    book, counts = build(args.dump, args.exclude, args.max_abs_cp, args.per_stratum, args.seed)
    lines = [f'{key} ; c0 "stratum={name} head_cp={score}"' for key, name, score in book]
    args.out.write_text("\n".join(lines) + "\n")
    manifest = {
        "schema": "rarog-ocb-book-v1",
        "book": str(args.out), "book_sha256": sha256(args.out),
        "positions": len(lines),
        "dump": str(args.dump), "dump_sha256": sha256(args.dump),
        "exclude": [{"path": str(p), "sha256": sha256(p)} for p in args.exclude],
        "max_abs_cp": args.max_abs_cp, "per_stratum": args.per_stratum, "seed": args.seed,
        "counts": counts,
    }
    args.out.with_suffix(".manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest["counts"], indent=2))
    print(f"{len(lines)} positions written to {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
