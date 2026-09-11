#!/usr/bin/env python3
"""Count games thrown away after a persistent material advantage.

PLAN A.5. Rarog wins a piece and then fails to win the game: over 2,400 games
against the six HCE-era engines it drew 57 and lost 12 after holding at least a
minor piece for 12 plies. Basilisk 1.9.3, a WEAKER engine, threw away 40. That
comparison is what makes this a defect rather than a fact of life, and it is
what the C.5 conversion cluster exists to fix.

This is a DIAGNOSTIC LAYER, never an acceptance layer. It says whether a
conversion change did what it claimed. Only a registered SPRT accepts one.

INPUT IS PGN, DELIBERATELY. The seed for this tool read Colosseum's SQLite
directly, which tied the measurement to one program's schema on one machine.
PGN is portable and harness-neutral; `export_tournament_pgn.py` produces it and
is the only tool that knows about Colosseum. Keep PGN archives out of Git and
cite their sha256; the small JSON summary this writes is what gets tracked.

FIDELITY TO THE SEED. The 57/12 baseline was produced WITHOUT the lone-minor
exclusion, because the seed did not implement one. `--no-exclude-lone-minor`
reproduces that arithmetic exactly, which is how a port is proved faithful
before it is improved. The default excludes lone minors, so the corrected
figures are expected to be LOWER than 57/12 and are not comparable to it.

Usage:
  python tools/diag/conversion_audit.py --pgn games.pgn \\
      --engine "Rarog 2.4.0-dev" \\
      --opponents "Houdini 1.5a" "Critter 1.6a" "Rybka 4" "Fritz 16" \\
                  "Shredder 12" "HIARCS 14" \\
      --json analysis/conversion_baseline_v1.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

try:
    import chess
    import chess.pgn
except ImportError:  # pragma: no cover - environment problem, not logic
    sys.exit("python-chess is required: pip install chess")

PIECE_VALUE = {
    chess.PAWN: 1,
    chess.KNIGHT: 3,
    chess.BISHOP: 3,
    chess.ROOK: 5,
    chess.QUEEN: 9,
}
#: Plies the advantage must persist before a game counts as convertible.
PERSIST_PLIES = 12
#: Advantage in pawns that counts as "a piece up".
MIN_ADVANTAGE = 3
#: Non-pawn material at or below which the position counts as an endgame.
ENDGAME_NPM = 26


def material(board: chess.Board, color: chess.Color) -> int:
    return sum(PIECE_VALUE[p] * len(board.pieces(p, color)) for p in PIECE_VALUE)


def non_pawn_material(board: chess.Board) -> int:
    pieces = (chess.KNIGHT, chess.BISHOP, chess.ROOK, chess.QUEEN)
    return sum(
        PIECE_VALUE[p] * len(board.pieces(p, c))
        for p in pieces
        for c in (chess.WHITE, chess.BLACK)
    )


def is_lone_minor(board: chess.Board, color: chess.Color) -> bool:
    """King and one minor against a bare king - a dead draw, not a failure.

    Without this, `KB vs K` and `KN vs K` are scored as "drew a piece up", which
    blames the engine for a position no engine can win. Checked by material
    signature rather than by piece count alone so KBB, KBN and KNN are NOT
    excluded: those are winnable and a draw there IS a conversion failure.
    """
    if board.pieces(chess.PAWN, color) or board.pieces(chess.PAWN, not color):
        return False
    if any(board.pieces(p, color) for p in (chess.ROOK, chess.QUEEN)):
        return False
    if any(board.pieces(p, not color) for p in PIECE_VALUE):
        return False  # the opponent must be bare
    minors = len(board.pieces(chess.KNIGHT, color)) + len(board.pieces(chess.BISHOP, color))
    return minors == 1


def outcome_for(result: str, played_white: bool) -> str:
    """`W`, `D` or `L` from the named side's point of view."""
    if result in ("1/2-1/2", "Draw"):
        return "D"
    white_won = result in ("1-0", "WhiteWin")
    if result not in ("1-0", "0-1", "WhiteWin", "BlackWin"):
        return "?"
    return "W" if white_won == played_white else "L"


def classify_game(game, engine: str, exclude_lone_minor: bool = True) -> dict | None:
    """Replay one game and describe what the named engine did with it.

    Returns `None` when the engine did not play in this game.
    """
    white = game.headers.get("White", "")
    black = game.headers.get("Black", "")
    if engine == white:
        color = chess.WHITE
    elif engine == black:
        color = chess.BLACK
    else:
        return None

    board = game.board()
    run_up = run_down = 0
    had_up = had_up_endgame = had_down = False
    suppressed_lone_minor = False
    for move in game.mainline_moves():
        board.push(move)
        advantage = material(board, color) - material(board, not color)
        run_up = run_up + 1 if advantage >= MIN_ADVANTAGE else 0
        run_down = run_down + 1 if advantage <= -MIN_ADVANTAGE else 0
        if run_up >= PERSIST_PLIES:
            if exclude_lone_minor and is_lone_minor(board, color):
                # Report suppressions, so a guard that never fires cannot hide.
                # Note this does NOT retract an earlier qualifying window: an
                # engine that was a rook up with pawns and only later degenerated
                # to KB-v-K failed to convert something winnable, and the bare
                # ending is the consequence of that failure, not an excuse for it.
                suppressed_lone_minor = True
            else:
                had_up = True
                if non_pawn_material(board) <= ENDGAME_NPM:
                    had_up_endgame = True
        if run_down >= PERSIST_PLIES:
            had_down = True

    return {
        "opponent": black if color == chess.WHITE else white,
        "outcome": outcome_for(game.headers.get("Result", "*"), color == chess.WHITE),
        "termination": game.headers.get("Termination", "unknown"),
        "persistent_advantage": had_up,
        "persistent_advantage_endgame": had_up_endgame,
        "persistent_deficit": had_down,
        "lone_minor_window_suppressed": suppressed_lone_minor,
        "final_advantage": material(board, color) - material(board, not color),
    }


def audit(games, engine: str, opponents: set[str] | None, exclude_lone_minor: bool) -> dict:
    counts: Counter[str] = Counter()
    by_termination: Counter[str] = Counter()
    for game in games:
        verdict = classify_game(game, engine, exclude_lone_minor)
        if verdict is None:
            continue
        if opponents is not None and verdict["opponent"] not in opponents:
            continue

        counts["games"] += 1
        outcome = verdict["outcome"]
        counts[{"W": "wins", "D": "draws", "L": "losses", "?": "unfinished"}[outcome]] += 1

        if outcome == "D" and verdict["persistent_advantage"]:
            counts["draw_after_persistent_advantage"] += 1
            by_termination[verdict["termination"]] += 1
            if verdict["persistent_advantage_endgame"]:
                counts["draw_after_persistent_advantage_endgame"] += 1
        if outcome == "L" and verdict["persistent_advantage"]:
            counts["loss_after_persistent_advantage"] += 1
        if outcome == "D" and verdict["persistent_deficit"]:
            counts["draw_saved_from_persistent_deficit"] += 1
        if outcome == "W" and verdict["persistent_deficit"]:
            counts["win_from_persistent_deficit"] += 1
        if verdict["lone_minor_window_suppressed"]:
            counts["lone_minor_windows_suppressed"] += 1
            if not verdict["persistent_advantage"]:
                counts["excused_by_lone_minor_only"] += 1

    thrown = (
        counts["draw_after_persistent_advantage"] + counts["loss_after_persistent_advantage"]
    )
    return {
        "engine": engine,
        "opponents": sorted(opponents) if opponents else "all",
        "parameters": {
            "persist_plies": PERSIST_PLIES,
            "min_advantage": MIN_ADVANTAGE,
            "endgame_non_pawn_material": ENDGAME_NPM,
            "exclude_lone_minor": exclude_lone_minor,
        },
        "counts": dict(sorted(counts.items())),
        "draw_terminations": dict(sorted(by_termination.items())),
        "thrown_away": thrown,
    }


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_games(paths: list[Path]):
    for path in paths:
        with path.open(encoding="utf-8", errors="replace") as handle:
            while True:
                game = chess.pgn.read_game(handle)
                if game is None:
                    break
                yield game


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--pgn", required=True, nargs="+", type=Path)
    parser.add_argument("--engine", required=True, help='exactly as it appears in the PGN')
    parser.add_argument("--opponents", nargs="*", default=None)
    parser.add_argument(
        "--no-exclude-lone-minor",
        dest="exclude_lone_minor",
        action="store_false",
        help="score KB-v-K and KN-v-K draws as conversion failures; reproduces "
        "the 2026-09-09 seed arithmetic, and is wrong for any other purpose",
    )
    parser.add_argument("--json", type=Path, help="write the frozen summary here")
    args = parser.parse_args()

    for path in args.pgn:
        if not path.exists():
            raise SystemExit(f"PGN not found: {path}")

    summary = audit(
        read_games(args.pgn),
        args.engine,
        set(args.opponents) if args.opponents else None,
        args.exclude_lone_minor,
    )
    # Cite the evidence by CONTENT, not by path. A path is a promise that a
    # file still exists somewhere; a hash lets anyone confirm they are
    # holding the same archive this summary was computed from.
    summary["pgn"] = [
        {"path": str(path).replace("\\", "/"), "sha256": sha256_of(path)}
        for path in args.pgn
    ]
    summary["generated_utc"] = datetime.now(timezone.utc).isoformat(timespec="seconds")

    counts = summary["counts"]
    print(f"{summary['engine']}: {counts.get('games', 0)} games")
    for key in sorted(counts):
        if key != "games":
            print(f"  {key:<46} {counts[key]}")
    print(f"  {'THROWN AWAY (draws + losses a piece up)':<46} {summary['thrown_away']}")
    if summary["draw_terminations"]:
        print("  draw terminations:")
        for term, n in summary["draw_terminations"].items():
            print(f"    {term:<44} {n}")

    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        print(f"\nfrozen summary: {args.json}")


if __name__ == "__main__":
    main()
