"""Fit the win-draw-loss model that turns Rarog's internal score into the score
it prints.

    python tools/wdl_fit.py <games.pgn> --engine <name> [--out fit.json]
                            [--check-rust src/uci/info.rs]

The model is the one Stockfish documents (github.com/official-stockfish/
WDL_model): with x the engine's own score of the position it searched and m
its material (pawn 1, knight and bishop 3, rook 5, queen 9, both sides,
clamped to [17, 78]),

    win  = 1 / (1 + exp((a(m) - x) / b(m)))
    loss = 1 / (1 + exp((a(m) + x) / b(m)))
    draw = 1 - win - loss

with a and b cubic polynomials in m / 58. The printed score is 100 x / a(m),
so +100 is the score at which the side wins half its games. The constants are
fitted here on the engine's own games, never taken from another engine.

The data are one engine's moves in a Colosseum PGN: the `s=` annotation is the
score the engine reported for the position before its move, from its own side;
the game's result is read from that side. Mate scores (`#n`), scores at or
beyond `--max-abs` (the clamped band a downgraded mate prints in, and
tablebase displays) and positions outside the material range are left out.

The fit has two stages: a and b by maximum likelihood for each material count
with at least `--min-count` positions, then each fitted by a cubic in m / 58,
weighted by the count. The report gives the fitted constants, every material
count's own fit, and the calibration: the observed and predicted outcome rates
in bands of the printed score.

`--check-rust` reads the constants in the engine's source (`WDL_A` and
`WDL_B`) and fails unless they equal the fit's, rounded as the fit prints
them.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import sys
from pathlib import Path

import numpy as np

MATERIAL_MIN = 17
MATERIAL_MAX = 78
ANCHOR = 58
DECIMALS = 8
SCORE = re.compile(r"(?:^|\s)s=(#?-?\d+)(?:\s|$)")


def material(board) -> int:
    import chess

    counts = {chess.PAWN: 1, chess.KNIGHT: 3, chess.BISHOP: 3, chess.ROOK: 5, chess.QUEEN: 9}
    return sum(len(board.pieces(piece, color)) * value
               for piece, value in counts.items() for color in (chess.WHITE, chess.BLACK))


def read_positions(pgn_path: Path, engine: str, max_abs: int) -> tuple[list[tuple[int, int, int]], dict]:
    """(score, material, outcome) for each of `engine`'s moves; outcome +1 win, 0 draw, -1 loss."""
    import chess.pgn

    rows: list[tuple[int, int, int]] = []
    counts = {"games": 0, "moves": 0, "mate": 0, "beyond_max_abs": 0, "no_score": 0, "unfinished": 0}
    with open(pgn_path, encoding="utf-8") as handle:
        while True:
            game = chess.pgn.read_game(handle)
            if game is None:
                break
            white, black = game.headers.get("White"), game.headers.get("Black")
            if engine not in (white, black):
                continue
            result = game.headers.get("Result")
            if result not in ("1-0", "0-1", "1/2-1/2"):
                counts["unfinished"] += 1
                continue
            counts["games"] += 1
            white_points = {"1-0": 1, "0-1": -1, "1/2-1/2": 0}[result]
            node = game
            while node.variations:
                child = node.variations[0]
                board = node.board()
                mover_is_engine = (board.turn == chess.WHITE and white == engine) or (
                    board.turn == chess.BLACK and black == engine)
                if mover_is_engine:
                    counts["moves"] += 1
                    found = SCORE.search(child.comment or "")
                    if found is None:
                        counts["no_score"] += 1
                    elif found.group(1).startswith("#"):
                        counts["mate"] += 1
                    else:
                        score = int(found.group(1))
                        if abs(score) >= max_abs:
                            counts["beyond_max_abs"] += 1
                        else:
                            outcome = white_points if board.turn == chess.WHITE else -white_points
                            rows.append((score, material(board), outcome))
                node = child
    return rows, counts


def probabilities(x, a, b):
    # A rate this far out is 0 or 1; the overflow it takes to get there is not an error.
    with np.errstate(over="ignore"):
        win = 1.0 / (1.0 + np.exp((a - x) / b))
        loss = 1.0 / (1.0 + np.exp((a + x) / b))
    return win, 1.0 - win - loss, loss


def negative_log_likelihood(x, outcome, a, b) -> float:
    if a <= 0 or b <= 0:
        return math.inf
    win, draw, loss = probabilities(x, a, b)
    p = np.where(outcome > 0, win, np.where(outcome < 0, loss, draw))
    if np.any(p <= 0):
        return math.inf
    return float(-np.sum(np.log(p)))


def fit_ab(x, outcome, start=(250.0, 60.0)) -> tuple[float, float]:
    """Maximum likelihood (a, b) by Newton's method on numerical derivatives,
    with a step halved until the likelihood improves."""
    theta = np.array(start, dtype=float)
    f = lambda t: negative_log_likelihood(x, outcome, t[0], t[1])  # noqa: E731
    value = f(theta)
    for _ in range(200):
        h = np.maximum(np.abs(theta) * 1e-4, 1e-3)
        grad = np.zeros(2)
        hess = np.zeros((2, 2))
        for i in range(2):
            e = np.zeros(2)
            e[i] = h[i]
            grad[i] = (f(theta + e) - f(theta - e)) / (2 * h[i])
            for j in range(2):
                d = np.zeros(2)
                d[j] = h[j]
                hess[i, j] = (f(theta + e + d) - f(theta + e - d) - f(theta - e + d) + f(theta - e - d)) / (
                    4 * h[i] * h[j])
        try:
            step = -np.linalg.solve(hess, grad)
            if not np.all(np.isfinite(step)) or grad @ step >= 0:
                raise np.linalg.LinAlgError
        except np.linalg.LinAlgError:
            step = -grad / max(np.abs(grad).max(), 1e-12) * 0.1 * np.abs(theta)
        scale = 1.0
        while scale > 1e-8:
            candidate = theta + scale * step
            new = f(candidate)
            if new < value:
                break
            scale /= 2
        else:
            break
        converged = abs(value - new) < 1e-9 * max(1.0, abs(value))
        theta, value = candidate, new
        if converged:
            break
    return float(theta[0]), float(theta[1])


def poly(coefficients, m):
    """A cubic in m / 58, coefficients from the cube down, as Stockfish writes them."""
    v = np.clip(np.asarray(m, dtype=float), MATERIAL_MIN, MATERIAL_MAX) / ANCHOR
    c = coefficients
    return ((c[0] * v + c[1]) * v + c[2]) * v + c[3]


def fit(rows, min_count: int) -> dict:
    data = np.array(rows, dtype=float)
    x, m, outcome = data[:, 0], data[:, 1].astype(int), data[:, 2]
    keep = (m >= MATERIAL_MIN) & (m <= MATERIAL_MAX)
    x, m, outcome = x[keep], m[keep], outcome[keep]
    bins = []
    for count in range(MATERIAL_MIN, MATERIAL_MAX + 1):
        sel = m == count
        n = int(sel.sum())
        if n < min_count:
            continue
        a, b = fit_ab(x[sel], outcome[sel])
        bins.append({"material": count, "n": n, "a": a, "b": b})
    if len(bins) < 4:
        raise SystemExit(f"only {len(bins)} material counts have {min_count} positions; a cubic needs 4")
    v = np.array([row["material"] for row in bins]) / ANCHOR
    w = np.sqrt([row["n"] for row in bins])
    a_coef = [round(float(c), DECIMALS) for c in np.polyfit(v, [row["a"] for row in bins], 3, w=w)]
    b_coef = [round(float(c), DECIMALS) for c in np.polyfit(v, [row["b"] for row in bins], 3, w=w)]
    a_all, b_all = poly(a_coef, m), poly(b_coef, m)
    if np.any(a_all <= 0) or np.any(b_all <= 0):
        raise SystemExit("the fitted polynomials leave the range where the model is defined")
    win, draw, loss = probabilities(x, a_all, b_all)
    p = np.where(outcome > 0, win, np.where(outcome < 0, loss, draw))
    printed = 100.0 * x / a_all
    bands = []
    for low in range(-300, 300, 25):
        sel = (printed >= low) & (printed < low + 25)
        if sel.sum() == 0:
            continue
        bands.append({
            "printed": [low, low + 25], "n": int(sel.sum()),
            "observed": [float(np.mean(outcome[sel] > 0)), float(np.mean(outcome[sel] == 0)),
                         float(np.mean(outcome[sel] < 0))],
            "predicted": [float(np.mean(win[sel])), float(np.mean(draw[sel])), float(np.mean(loss[sel]))],
        })
    return {"a": a_coef, "b": b_coef, "positions": int(len(x)), "material_bins": bins,
            "mean_log_loss": float(-np.mean(np.log(p))), "calibration": bands}


def rust_constants(source: str) -> tuple[list[float], list[float]]:
    found = {}
    for name in ("WDL_A", "WDL_B"):
        match = re.search(rf"const {name}: \[f64; 4\] = \[([^\]]*)\];", source)
        if match is None:
            raise SystemExit(f"no `const {name}: [f64; 4]` in the source")
        found[name] = [float(v.strip().replace("_", "")) for v in match.group(1).split(",") if v.strip()]
    return found["WDL_A"], found["WDL_B"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("pgn", type=Path)
    parser.add_argument("--engine", required=True, help="the PGN player name whose moves are fitted")
    parser.add_argument("--max-abs", type=int, default=4000)
    parser.add_argument("--min-count", type=int, default=300)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--check-rust", type=Path)
    args = parser.parse_args()

    rows, counts = read_positions(args.pgn, args.engine, args.max_abs)
    if counts["games"] == 0:
        raise SystemExit(f"no finished game of {args.engine!r} in {args.pgn}")
    result = fit(rows, args.min_count)
    result["source"] = {"pgn": str(args.pgn), "sha256": hashlib.sha256(args.pgn.read_bytes()).hexdigest(),
                        "engine": args.engine, "max_abs": args.max_abs, "min_count": args.min_count,
                        "counts": counts}
    print(f"{counts['games']} games, {counts['moves']} moves of {args.engine}: {result['positions']} positions "
          f"fitted (mate {counts['mate']}, at or beyond {args.max_abs} {counts['beyond_max_abs']}, "
          f"no score {counts['no_score']})")
    print(f"material counts fitted: {len(result['material_bins'])} "
          f"({result['material_bins'][0]['material']} to {result['material_bins'][-1]['material']})")
    print(f"WDL_A = {result['a']}")
    print(f"WDL_B = {result['b']}")
    print(f"mean log loss {result['mean_log_loss']:.4f}")
    print("a(m) and b(m) at material 20, 40, 58, 78:",
          [round(float(poly(result["a"], m)), 1) for m in (20, 40, 58, 78)],
          [round(float(poly(result["b"], m)), 1) for m in (20, 40, 58, 78)])
    print("calibration by printed score (n; observed W/D/L; predicted W/D/L):")
    for band in result["calibration"]:
        o, p = band["observed"], band["predicted"]
        print(f"  [{band['printed'][0]:+4}, {band['printed'][1]:+4}) {band['n']:6}  "
              f"{o[0]:.3f} {o[1]:.3f} {o[2]:.3f}   {p[0]:.3f} {p[1]:.3f} {p[2]:.3f}")
    if args.out:
        args.out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    if args.check_rust:
        a, b = rust_constants(args.check_rust.read_text(encoding="utf-8"))
        if a != result["a"] or b != result["b"]:
            print(f"{args.check_rust}: WDL_A {a}, WDL_B {b} differ from the fit", file=sys.stderr)
            return 1
        print(f"{args.check_rust}: the constants equal the fit")
    return 0


if __name__ == "__main__":
    sys.exit(main())
