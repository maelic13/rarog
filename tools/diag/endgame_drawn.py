#!/usr/bin/env python3
"""Drawn-subset cohort: does the engine claim an advantage in dead-drawn endings?

`endgame_truth.py` measures what happens in positions that are theoretically
WON -- conversion, win-preservation, DTZ progress. That is the right instrument
for a mate drive, and the wrong one for a SCALE function, whose entire job is to
stop the evaluator from scoring a theoretically drawn position as winning.

The distinction matters concretely. RAR-E11 measured Stockfish 18 converting
KRP-KR at only 47.9% against Rarog's 43.8% at the same node budget, so the
conversion gap in that family is about four points, not the fifty-five that
"52% conversion" suggests against an imagined 100%. The defect a scale function
addresses is elsewhere: an engine that scores a drawn rook ending at +200 will
STEER INTO IT from a position it could have won differently, and no conversion
measurement taken inside the ending can see that happen.

So this measures the complementary cohort. Sample positions of a family, keep
the ones the tablebase calls a clean draw, search each at a fixed node budget,
and report how often the engine claims a meaningful advantage anyway.

  overclaim rate   fraction of drawn positions scored beyond `--threshold` cp
                   for the strong side. This is the number a scale function
                   must move, and the one to put in a ledger row.
  mean/median      the score distribution, which says whether a change moved
                   the whole cohort or clipped a tail.

Scores are from the STRONG side's perspective and come from a real search at
`--nodes`, not from a `tune`-feature static eval, so the number describes the
production engine. A mate score is clamped to `--mate-cp` rather than dropped:
claiming mate in a drawn position is the worst case, not a missing sample.

Positions are seeded from the family NAME, matching `endgame_truth.py`, so a
single-family run samples exactly what the same family samples inside a longer
run. Index seeding silently broke that comparison once already.

Usage:

  python tools/diag/endgame_drawn.py --engine target/release/rarog.exe \\
      --syzygy D:/chess/tablebases/syzygy3456 --families KRP-KR
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import statistics
import sys
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

try:
    import chess
    import chess.engine
    import chess.syzygy
except ImportError:
    print("ERROR: python-chess not installed. Run: pip install chess", file=sys.stderr)
    sys.exit(1)

from endgame_truth import (  # noqa: E402
    cohort_digest, configure_engine, generate_family, parse_family,
    wdl_for_white,
)

# Below this many DRAWN positions a family's overclaim rate is reported as thin
# rather than as a number -- the same discipline as `endgame_floors.MIN_ELIGIBLE`
# and for the same reason. Some families have almost no drawn subset at all
# (KQ-K and KR-K have none by construction), and an overclaim rate over three
# positions would read as a measurement while being noise.
MIN_DRAWN = 25


def _worker(task):
    """Score one shard of drawn positions. Returns [(index, cp, is_mate), ...].

    Engine and tablebase lifetimes are the task's, explicitly. The pool
    initializer version of this deadlocked in 4.10.3 and there is no reason to
    rediscover that.
    """
    engine_path, syzygy, hash_mb, nodes, mate_cp, items = task
    tb = chess.syzygy.open_tablebase(syzygy)
    engine = chess.engine.SimpleEngine.popen_uci(engine_path)
    try:
        configure_engine(engine, hash_mb)
        out = []
        for index, fen in items:
            board = chess.Board(fen)
            try:
                if wdl_for_white(tb, board) != 0:
                    continue
            except (chess.syzygy.MissingTableError, KeyError, ValueError):
                continue
            # `game=object()` forces `ucinewgame` before every position, so
            # the transposition table starts empty each time.
            #
            # Without it the engine carries its table across positions and a
            # position's score depends on which positions preceded it. That is
            # not a theoretical worry: it was caught by the serial-vs-sharded
            # byte-identity check at 4.11.4, where KBP-KB's overclaim rate read
            # 0.702 serially and 0.750 over six workers on the SAME positions.
            # A census must be position-local to be shardable at all, and
            # order-independence is worth having on its own account.
            info = engine.analyse(
                board, chess.engine.Limit(nodes=nodes), game=object()
            )
            pov = info["score"].white()
            if pov.is_mate():
                mate = pov.mate()
                out.append((index, mate_cp if mate is not None and mate > 0
                            else -mate_cp, True))
            else:
                out.append((index, int(pov.score()), False))
        return out
    finally:
        engine.quit()
        tb.close()


def shard(items, workers: int):
    """Fixed-index round-robin, so parallel output equals serial output."""
    buckets = [[] for _ in range(workers)]
    for i, item in enumerate(items):
        buckets[i % workers].append(item)
    return [b for b in buckets if b]


def summarize(name, scores, sampled, mates, threshold, nodes, digest):
    """One family's overclaim statistics, or a thin-sample refusal."""
    entry = {
        "sampled": sampled,
        "drawn": len(scores),
        "threshold_cp": threshold,
        "nodes": nodes,
        "cohort_sha256": digest,
    }
    if len(scores) < MIN_DRAWN:
        entry["thin"] = (
            f"only {len(scores)} drawn positions (need {MIN_DRAWN}); reported "
            "as empty rather than as a rate"
        )
        entry["overclaim_rate"] = None
        return entry
    over = [s for s in scores if s > threshold]
    entry.update({
        "overclaimed": len(over),
        "overclaim_rate": round(len(over) / len(scores), 6),
        "mate_claims": mates,
        "mean_cp": round(statistics.mean(scores), 2),
        "median_cp": statistics.median(scores),
        "max_cp": max(scores),
    })
    return entry


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--engine", required=True, type=Path)
    ap.add_argument("--syzygy", required=True, type=Path)
    ap.add_argument("--families", required=True,
                    help="comma-separated, e.g. KRP-KR or KRP-KR,KRP-KB")
    ap.add_argument("--positions", type=int, default=400,
                    help="positions SAMPLED per family; the drawn subset is "
                         "whatever fraction of them the tablebase calls a draw")
    ap.add_argument("--nodes", type=int, default=60_000)
    ap.add_argument("--threshold", type=int, default=100,
                    help="cp beyond which a drawn position counts as overclaimed")
    ap.add_argument("--mate-cp", type=int, default=10_000,
                    help="score substituted for a mate claim")
    ap.add_argument("--seed", type=int, default=0x5E9D18,
                    help="matches endgame_truth.py, so cohorts line up")
    ap.add_argument("--hash", type=int, default=16)
    ap.add_argument(
        "--workers", type=int, default=1,
        help="independent one-thread engine processes. Changes wall time only; "
             "results are reassembled by fixed index (PLAN 4.10.3)",
    )
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()

    names = [n.strip() for n in args.families.split(",") if n.strip()]
    specs = {n: parse_family(n) for n in names}

    # Generate and fingerprint every family's positions BEFORE opening an
    # engine, so a sharded run and a serial run address identical positions by
    # index -- the same contract as `endgame_truth.py` (PLAN 4.10.2/4.10.3).
    fens = {}
    digests = {}
    for name in names:
        strong, weak = specs[name]
        boards = generate_family(args.seed, name, strong, weak, args.positions)
        fens[name] = [b.fen() for b in boards]
        digests[name] = cohort_digest(fens[name])

    # Shards mix families, so work is addressed in ONE flat index space and
    # re-split afterwards. That is what lets the round-robin balance across
    # families whose cost differs by an order of magnitude.
    offsets = {}
    running = 0
    for name in names:
        offsets[name] = running
        running += len(fens[name])
    by_index = {offsets[name] + i: name
                for name in names for i in range(len(fens[name]))}

    indexed = [(offsets[name] + i, fen)
               for name in names for i, fen in enumerate(fens[name])]
    print(f"{len(indexed)} positions over {len(names)} families, "
          f"{args.workers} worker(s)", flush=True)
    tasks = [(str(args.engine.resolve()), str(args.syzygy.resolve()), args.hash,
              args.nodes, args.mate_cp, items)
             for items in shard(indexed, args.workers)]
    flat = {name: {} for name in names}

    if args.workers == 1:
        produced = [_worker(tasks[0])] if tasks else []
    else:
        with ProcessPoolExecutor(max_workers=len(tasks)) as pool:
            produced = list(pool.map(_worker, tasks))
    for batch in produced:
        for gi, cp, is_mate in batch:
            name = by_index[gi]
            flat[name][gi - offsets[name]] = (cp, is_mate)

    report: dict[str, dict] = {}
    for name in names:
        got = flat[name]
        scores = [got[i][0] for i in sorted(got)]
        mates = sum(1 for i in got if got[i][1])
        entry = summarize(name, scores, len(fens[name]), mates,
                          args.threshold, args.nodes, digests[name])
        report[name] = entry
        if entry["overclaim_rate"] is None:
            print(f"{name:<9} drawn {entry['drawn']:>4}/{entry['sampled']}  "
                  f"THIN -- {entry['thin']}", flush=True)
        else:
            print(
                f"{name:<9} drawn {entry['drawn']:>4}/{entry['sampled']}  "
                f"overclaim {entry['overclaim_rate']:.4f} "
                f"({entry['overclaimed']}/{entry['drawn']} over "
                f"{args.threshold}cp)  mean {entry['mean_cp']:+.1f}  "
                f"median {entry['median_cp']:+}  max {entry['max_cp']:+}  "
                f"mate claims {mates}",
                flush=True,
            )

    if args.output:
        # mkdir first. endgame_truth.py does this; this tool did not, so a
        # completed census died on the write and lost 28,500 positions of work.
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(
            json.dumps(
                {
                    "schema": "rarog-endgame-drawn-v1",
                    # NOT one of the four layers: this measures the COMPLEMENT
                    # of conversion -- does the evaluator claim won what theory
                    # says is drawn. A SCALE function is validated here and is
                    # invisible in conversion; a VERDICT function is the other
                    # way round. Reading 4.9a.7 off conversion nearly called a
                    # working change a failure.
                    "layer": "drawn_share_bias",
                    "layer_note": (
                        "static evaluation of theoretically drawn positions; "
                        "plays no games, so it is unaffected by the RAR-E14 "
                        "playout defect. Never reports Elo."
                    ),
                    "engine": str(args.engine),
                    "seed": args.seed,
                    "families": report,
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
            newline="\n",
        )
        print(f"\nReport: {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
