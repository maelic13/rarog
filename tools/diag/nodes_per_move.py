#!/usr/bin/env python3
"""Measure actual nodes per move at a real time control (PLAN 4.10.6).

Every fixed-node screen in this project implies a claim: that its budget stands
in for what the engine really gets at the time control it is tested and shipped
at. That claim was never measured here. It is measured now, because a screen
budget far below the deployment budget produces PROVISIONAL verdicts -- a
losing move a 200,000-node search sees can be invisible at 60,000, which is how
Basilisk rejected its own leading KBNK candidate on a two-ply tactic (BAS-E45).

Method: play self-play games under a real clock, exactly as a match would, and
record `nodes` from each search. The clock is managed here rather than by the
harness so the engine receives genuine `wtime/btime/winc/binc` and exercises
its own time management -- a fixed `movetime` would measure a different code
path (RAR-M01).

The phase split is by MOVE NUMBER and is descriptive only. Nodes per move rises
through a game as the position simplifies and the tree narrows, so a single
mean hides the thing a screen budget needs to match.

Example:

  python tools/diag/nodes_per_move.py \\
      --engine target/release/rarog.exe --games 4 --tc 3+0.03
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
import time
from pathlib import Path

import chess
import chess.engine


def parse_tc(spec: str) -> tuple[float, float]:
    """`3+0.03` -> (3.0, 0.03) seconds."""
    if "+" not in spec:
        raise ValueError(f"time control must look like 3+0.03, got {spec!r}")
    base, inc = spec.split("+", 1)
    return float(base), float(inc)


def play_game(
    engine: chess.engine.SimpleEngine,
    base: float,
    inc: float,
    max_plies: int,
    game_token: object,
) -> list[tuple[int, int]]:
    """One self-play game. Returns [(ply, nodes), ...].

    The clock is decremented by MEASURED wall time and incremented after each
    move, so an engine that overspends is not silently given the time back.
    """
    board = chess.Board()
    clock = {chess.WHITE: base, chess.BLACK: base}
    samples: list[tuple[int, int]] = []

    for ply in range(max_plies):
        if board.is_game_over(claim_draw=True):
            break
        side = board.turn
        limit = chess.engine.Limit(
            white_clock=clock[chess.WHITE], black_clock=clock[chess.BLACK],
            white_inc=inc, black_inc=inc,
        )
        started = time.perf_counter()
        result = engine.play(
            board, limit, info=chess.engine.INFO_ALL, game=game_token
        )
        elapsed = time.perf_counter() - started
        if result.move is None:
            break
        nodes = result.info.get("nodes")
        if nodes is not None:
            samples.append((ply, int(nodes)))
        board.push(result.move)

        clock[side] = clock[side] - elapsed + inc
        if clock[side] <= 0:
            # A forfeit ends the sample honestly rather than being papered over.
            print(f"  clock exhausted at ply {ply} for "
                  f"{'white' if side == chess.WHITE else 'black'}", flush=True)
            break
    return samples


def quantiles(values: list[int]) -> dict:
    ordered = sorted(values)
    n = len(ordered)
    def at(q: float) -> int:
        return ordered[min(n - 1, max(0, int(round(q * (n - 1)))))]
    return {
        "n": n,
        "min": ordered[0],
        "p25": at(0.25),
        "median": int(statistics.median(ordered)),
        "p75": at(0.75),
        "p90": at(0.90),
        "max": ordered[-1],
        "mean": int(statistics.fmean(ordered)),
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--engine", required=True, type=Path)
    ap.add_argument("--tc", default="3+0.03", help="base+increment in seconds")
    ap.add_argument("--games", type=int, default=4)
    ap.add_argument("--max-plies", type=int, default=200)
    ap.add_argument("--hash", type=int, default=64)
    ap.add_argument("--output", type=Path)
    args = ap.parse_args()

    engine_path = args.engine.resolve()
    if not engine_path.is_file():
        ap.error(f"engine not found: {engine_path}")
    try:
        base, inc = parse_tc(args.tc)
    except ValueError as exc:
        ap.error(str(exc))
    if args.games < 1:
        ap.error("games must be at least 1")

    engine = chess.engine.SimpleEngine.popen_uci(str(engine_path))
    all_samples: list[tuple[int, int]] = []
    try:
        options = {}
        if "Hash" in engine.options:
            options["Hash"] = args.hash
        if "Threads" in engine.options:
            options["Threads"] = 1
        if options:
            engine.configure(options)
        for game in range(args.games):
            samples = play_game(engine, base, inc, args.max_plies, object())
            all_samples.extend(samples)
            print(f"game {game + 1}/{args.games}: {len(samples)} moves", flush=True)
    finally:
        engine.quit()

    if not all_samples:
        print("no samples: the engine reported no node counts", file=sys.stderr)
        return 1

    nodes = [n for _, n in all_samples]
    report = {
        "schema": "rarog-nodes-per-move-v1",
        "layer": "run_condition",
        "layer_note": (
            "not a measurement of play. This sizes the node budget that a "
            "fixed-node screen must justify itself against "
            "(analysis/endgame_measurement_layers.md)."
        ),
        "engine": str(engine_path),
        "time_control": args.tc,
        "games": args.games,
        "threads": 1,
        "hash_mb": args.hash,
        "overall": quantiles(nodes),
        "by_phase": {},
    }
    # Descriptive split by move number: the tree narrows as the game simplifies,
    # so one mean hides what a screen budget has to match.
    bands = [("opening", 0, 30), ("middlegame", 30, 80), ("endgame", 80, 10**9)]
    for name, low, high in bands:
        band = [n for ply, n in all_samples if low <= ply < high]
        if band:
            report["by_phase"][name] = quantiles(band)

    rendered = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8", newline="\n")
        print(f"Report: {args.output.resolve()}")
    o = report["overall"]
    print(f"\nnodes/move at {args.tc}, {o['n']} moves over {args.games} games:")
    print(f"  median {o['median']:,}   mean {o['mean']:,}")
    print(f"  p25 {o['p25']:,}   p75 {o['p75']:,}   p90 {o['p90']:,}")
    print(f"  min {o['min']:,}   max {o['max']:,}")
    for name, q in report["by_phase"].items():
        print(f"  {name:<11} median {q['median']:,} over {q['n']} moves")
    return 0


if __name__ == "__main__":
    sys.exit(main())
