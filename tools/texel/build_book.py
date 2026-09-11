#!/usr/bin/env python3
"""Build a phase-WEIGHTED datagen start book from the read-only position store.

WHY THIS EXISTS. `beast_seed.epd` holds exactly 150,000 positions in each of
the five extractor phase buckets. That looks balanced and is the wrong shape,
because "phase" is MATERIAL, not ply: a game started at phase 12 can never
produce an `opening` row. Measured on the 20,000-game RAR-E08 pilot, yield per
game conditioned on the START bucket is

    start bucket    opening rows/game   all buckets
    opening               3.4392          13.30
    early_mid             0.0008          12.39
    middlegame            0.0000          11.65
    endgame               0.0000           9.85
    deep_endgame          0.0000           9.10

Only an opening start feeds the opening bucket, and an opening start is also
the most productive overall because one game traverses every phase on the way
down. So four fifths of a balanced book is structurally incapable of filling
the bucket that binds, and the extractor preflight -- which sizes GAMES -- has
no way to say so. It asked for 1,113,504 games when the book was the problem.

Maximising the worst-off bucket over the matrix above puts the optimum at
68/10/0/0/22. The default here is the hedged 50/10/10/10/20 instead: the corner
buys its extra rows by making every middlegame and endgame row a REACHED
position, correlated with the opening play that led there, and 10% direct
starts in each keeps those buckets independently sampled. That costs 26% of the
theoretical yield and is worth it. Against the balanced book the hedge still
takes 3.0M rows from 1,090,116 games to 436,127.

Supply is not a constraint: the store is ~125M positions, 36.8% of them
opening-bucket, with a 0.02% exact-duplicate rate. 150,000 was a quota.

THE OUTPUT IS SHUFFLED, AND THAT IS LOAD-BEARING. fastchess consumes openings
in order from `-Start`, and `datagen.ps1` hands out contiguous segments. A book
grouped by bucket would give each segment a single phase. Shuffling makes every
contiguous segment carry the target composition.

Positions are filtered to match the `beast_seed.epd` contract, which was
measured rather than assumed: 0 of 30,000 sampled entries were in check or
terminal, and side-to-move was 49.7% white. This keeps all three.

The store is READ-ONLY and is only ever opened for reading.

Usage:

  python tools/texel/build_book.py --count 1000000 --out tools/texel/data/phase_book_v2.epd
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import random
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

try:
    import chess
except ImportError:
    print("ERROR: python-chess not installed. Run: pip install chess", file=sys.stderr)
    sys.exit(1)

# Single source of truth: drift between the book's phase definition and the
# extractor's would make the composition silently wrong.
from extract import PHASE_BUCKETS, PHASE_W  # noqa: E402

BUCKET_NAMES = tuple(name for name, _, _ in PHASE_BUCKETS)
DEFAULT_STORE = "A:/Chess/Beast/data/txt/positions.txt"

# byte -> phase weight, derived from the extractor's piece-type table.
_W: dict[bytes, int] = {}
for _pt, _w in PHASE_W.items():
    _sym = chess.piece_symbol(_pt)
    _W[_sym.encode()] = _w
    _W[_sym.upper().encode()] = _w
W_ITEMS = tuple(_W.items())

_BUCKET_OF = [""] * 25
for _name, _lo, _hi in PHASE_BUCKETS:
    for _p in range(_lo, _hi + 1):
        _BUCKET_OF[_p] = _name


def phase_of(board: bytes) -> int:
    total = 0
    for ch, w in W_ITEMS:
        total += board.count(ch) * w
    return total if total < 24 else 24


def parse_weights(text: str) -> list[float]:
    parts = [float(x) for x in text.split(",")]
    if len(parts) != 5:
        raise argparse.ArgumentTypeError("--weights needs 5 comma-separated numbers")
    if any(p < 0 for p in parts) or sum(parts) <= 0:
        raise argparse.ArgumentTypeError("--weights must be non-negative and not all zero")
    total = sum(parts)
    return [p / total for p in parts]


def sample_store(store: Path, quotas: dict[str, int], seed: int) -> dict[str, list[str]]:
    """One sequential pass, reservoir sampling per bucket (algorithm R)."""
    rng = random.Random(seed)
    reservoir: dict[str, list[str]] = {n: [] for n in BUCKET_NAMES}
    seen: dict[str, int] = {n: 0 for n in BUCKET_NAMES}
    size = store.stat().st_size
    read = 0
    t0 = time.time()
    with store.open("rb") as fh:
        for raw in fh:
            read += len(raw)
            parts = raw.split()
            if len(parts) < 4:
                continue
            name = _BUCKET_OF[phase_of(parts[0])]
            cap = quotas[name]
            if cap <= 0:
                continue
            n = seen[name]
            seen[name] = n + 1
            res = reservoir[name]
            if n < cap:
                res.append(b" ".join(parts[:4]).decode("ascii", "replace"))
            else:
                j = rng.randrange(n + 1)
                if j < cap:
                    res[j] = b" ".join(parts[:4]).decode("ascii", "replace")
            if read % (512 * 1024 * 1024) < len(raw):
                print(f"  ... {read / size:5.1%} of the store, {time.time() - t0:5.0f}s",
                      flush=True)
    print(f"  store pass complete in {time.time() - t0:.0f}s")
    for name in BUCKET_NAMES:
        print(f"    {name:<14} seen {seen[name]:>12,}  held {len(reservoir[name]):>9,}")
    return reservoir


def keep(fen4: str) -> bool | None:
    """True if the position matches the beast_seed contract; None if unparsable."""
    try:
        board = chess.Board(fen4 + " 0 1")
    except ValueError:
        return None
    if not board.is_valid():
        return False
    return not board.is_check() and not board.is_game_over()


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--store", type=Path, default=Path(DEFAULT_STORE))
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--count", type=int, required=True, help="total positions to write")
    ap.add_argument("--weights", type=parse_weights, default=parse_weights("50,10,10,10,20"),
                    metavar="O,EM,MG,EG,DE",
                    help="per-bucket share, any scale (default 50,10,10,10,20)")
    ap.add_argument("--oversample", type=float, default=1.30, metavar="X",
                    help="reservoir headroom to survive validation and dedup")
    ap.add_argument("--seed", type=int, default=0x5EED2)
    args = ap.parse_args()

    if args.count <= 0:
        raise SystemExit("--count must be positive")
    if not args.store.is_file():
        raise SystemExit(f"store not found: {args.store}")
    if args.out.exists():
        raise SystemExit(f"refusing to overwrite {args.out}; books are never rewritten")

    target = {n: int(round(args.count * w)) for n, w in zip(BUCKET_NAMES, args.weights)}
    quotas = {n: int(round(v * args.oversample)) for n, v in target.items()}
    print(f"store   : {args.store} ({args.store.stat().st_size:,} bytes, read-only)")
    print("target  : " + "  ".join(f"{n}={v:,}" for n, v in target.items()))
    print(f"sampling with {args.oversample:g}x headroom ...", flush=True)

    reservoir = sample_store(args.store, quotas, args.seed)

    rng = random.Random(args.seed ^ 0xA5A5)
    out: list[str] = []
    print("validating and balancing side to move ...", flush=True)
    shortfall = {}
    for name in BUCKET_NAMES:
        want = target[name]
        if want <= 0:
            continue
        pool = reservoir[name]
        rng.shuffle(pool)
        by_stm: dict[str, list[str]] = {"w": [], "b": []}
        dedup = set()
        bad = 0
        for fen in pool:
            if fen in dedup:
                continue
            verdict = keep(fen)
            if verdict is None or verdict is False:
                bad += 1
                continue
            dedup.add(fen)
            by_stm[fen.split(" ")[1]].append(fen)
        half = want // 2
        take_w = min(half, len(by_stm["w"]))
        take_b = min(want - take_w, len(by_stm["b"]))
        take_w = min(want - take_b, len(by_stm["w"]))
        chosen = by_stm["w"][:take_w] + by_stm["b"][:take_b]
        if len(chosen) < want:
            shortfall[name] = (len(chosen), want)
        out.extend(chosen)
        print(f"    {name:<14} kept {len(chosen):>9,} of {want:>9,} "
              f"(rejected {bad:,}, w/b {take_w:,}/{take_b:,})", flush=True)

    if shortfall:
        print("\nWARNING: buckets short of target (raise --oversample):")
        for n, (got, want) in shortfall.items():
            print(f"  {n}: {got:,} < {want:,}")

    # Load-bearing: datagen hands out contiguous segments.
    rng.shuffle(out)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    tmp = args.out.with_suffix(args.out.suffix + ".tmp")
    tmp.write_text("\n".join(out) + "\n", encoding="ascii", newline="\n")
    os.replace(tmp, args.out)

    digest = hashlib.sha256(args.out.read_bytes()).hexdigest().upper()
    counts = {n: 0 for n in BUCKET_NAMES}
    whites = 0
    for fen in out:
        counts[_BUCKET_OF[phase_of(fen.split(" ", 1)[0].encode())]] += 1
        whites += fen.split(" ")[1] == "w"
    manifest = {
        "schema": "rarog-texel-book-v1",
        "source": str(args.store),
        "positions": len(out),
        "seed": args.seed,
        "weights": dict(zip(BUCKET_NAMES, args.weights)),
        "composition": counts,
        "white_to_move_pct": round(100.0 * whites / len(out), 2),
        "sha256": digest,
        "shuffled": True,
    }
    man = args.out.with_suffix(".manifest.json")
    man.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n",
                   encoding="utf-8", newline="\n")

    print(f"\nwrote {len(out):,} positions to {args.out}")
    print(f"sha256 {digest}")
    for n in BUCKET_NAMES:
        print(f"  {n:<14} {counts[n]:>9,}  {100.0 * counts[n] / len(out):5.1f}%")
    print(f"  white to move {manifest['white_to_move_pct']}%")
    print(f"manifest {man}")
    return 1 if shortfall else 0


if __name__ == "__main__":
    sys.exit(main())
