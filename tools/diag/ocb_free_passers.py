#!/usr/bin/env python3
"""Free passed pawns in opposite-coloured-bishop endings: do they separate won
from drawn, and does one scale rule built on them work at every men count?

A passed pawn is *free* when the defending bishop neither attacks nor stands on
any square of its path to promotion and the defending king neither stands on
nor is next to one; *bishop-free* ignores the king. The engine's rule counts
every passer, and a count-only factor cannot tell won from drawn six-man
endings (RAR-E26, RAR-E27); the free count is the position knowledge a single
rule across the six-to-seven-men boundary would need.

  separation  win rate by passer and free-passer counts on Syzygy-labelled
              generated positions (`ocb_scale_screen.py drawn --emit` files
              dumped with `--scale`), White the side with more pawns
  rule        fit `s = clamp(a + b*pawns + c*q, 0, 48)` on a training dump's
              pure endings at every men count, q the passers, the free or the
              bishop-free passers of both sides, and read it on a held-out dump
              against the engine's own rule, by men region
  drawn       a fitted rule's static overclaim on draws and underclaim on wins
              on the Syzygy-labelled positions, against the engine's rule

Static loss ranks candidates and accepts nothing.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))

import ocb_scale_screen as oc  # noqa: E402

FORMS = ("count", "free", "bishop_free")
DIAGONALS = ((1, 1), (1, -1), (-1, 1), (-1, -1))


def bishop_reach(squares, at):
    """Squares a bishop on `at` attacks with the board's occupancy, plus its own."""
    reach = {at}
    for df, dr in DIAGONALS:
        f, r = at
        while True:
            f, r = f + df, r + dr
            if not (0 <= f < 8 and 0 <= r < 8):
                break
            reach.add((f, r))
            if (f, r) in squares:
                break
    return reach


def passer_kinds(squares):
    """For White and Black: (passers, free passers, bishop-free passers)."""
    out = {}
    for side, pawn, enemy_pawn, step, bishop, king in (
        ("w", "P", "p", 1, "b", "k"),
        ("b", "p", "P", -1, "B", "K"),
    ):
        enemies = [sq for sq, ch in squares.items() if ch == enemy_pawn]
        bsq = next((sq for sq, ch in squares.items() if ch == bishop), None)
        ksq = next(sq for sq, ch in squares.items() if ch == king)
        guard = bishop_reach(squares, bsq) if bsq is not None else set()
        counts = [0, 0, 0]
        for (f, r), ch in squares.items():
            if ch != pawn:
                continue
            ahead = (lambda er: er > r) if step == 1 else (lambda er: er < r)
            if any(abs(ef - f) <= 1 and ahead(er) for ef, er in enemies):
                continue
            path = [(f, rr) for rr in (range(r + 1, 8) if step == 1 else range(r - 1, -1, -1))]
            by_bishop = any(sq in guard for sq in path)
            by_king = any(max(abs(ksq[0] - pf), abs(ksq[1] - pr)) <= 1 for pf, pr in path)
            counts[0] += 1
            counts[1] += not (by_bishop or by_king)
            counts[2] += not by_bishop
        out[side] = tuple(counts)
    return out["w"], out["b"]


def q_of(form, w, b):
    index = {"count": 0, "free": 1, "bishop_free": 2}[form]
    return w[index] + b[index]


def load_pure(path: Path):
    """Pure opposite-bishop rows with pawns: arrays of pawns, the three passer
    counts (both sides), the engine's /48 factor, the score, the unscaled
    score and the label; plus the file's global squared-error inputs."""
    cols = {k: [] for k in ("pawns", "count", "free", "bishop_free", "s0", "score", "label")}
    all_scores, all_labels = [], []
    for fen, label, score, _engine in oc.read_dump(path):
        all_scores.append(score)
        all_labels.append(label)
        cohort, pawns, _pw, _pb, s0 = oc.classify(fen)
        if cohort != oc.PURE:
            continue
        w, b = passer_kinds(oc.parse_board(fen))
        cols["pawns"].append(pawns)
        for form in FORMS:
            cols[form].append(q_of(form, w, b))
        cols["s0"].append(s0)
        cols["score"].append(score)
        cols["label"].append(label)
    a = {k: np.array(v, dtype=float) for k, v in cols.items()}
    s0, score = a["s0"], a["score"]
    a["raw"] = np.where(s0 == oc.OCB_NORMAL, score,
                        np.round((score + 0.5 * np.sign(score)) * oc.OCB_NORMAL / s0))
    return a, np.array(all_scores, float), np.array(all_labels, float)


def rule_scale(params, pawns, q):
    a, b, c = params
    return np.clip(a + b * pawns + c * q, 0, oc.OCB_NORMAL)


def cmd_rule(args) -> int:
    k = args.k
    grid = {"a": range(0, oc.OCB_NORMAL + 1), "b": range(0, 13), "c": range(0, 25)}
    train, _, _ = load_pure(args.train)
    held, held_scores, held_labels = load_pure(args.held_out)
    global_err = (held_labels - oc.sigmoid(held_scores, k)) ** 2
    head_err = (held["label"] - oc.sigmoid(oc.scaled(held["raw"], held["s0"]), k)) ** 2
    regions = {"all": held["pawns"] >= 1, "men<=6": held["pawns"] <= 2, "men>=7": held["pawns"] >= 3}
    report = {"k": k, "train_rows": int(len(train["pawns"])), "held_out_rows": int(len(held["pawns"])),
              "held_out_global_mse": float(global_err.mean()), "forms": {}}
    for form in FORMS:
        table = oc.cell_losses(train["raw"], train["label"], train["pawns"], train[form], k)
        params = oc.fit_rule(table, grid)
        s = rule_scale(params, held["pawns"], held[form])
        err = (held["label"] - oc.sigmoid(oc.scaled(held["raw"], s), k)) ** 2
        entry = {"params_a_b_c": list(params), "regions": {}}
        for name, mask in regions.items():
            gain, se = oc.paired(head_err[mask], err[mask])
            entry["regions"][name] = {"rows": int(mask.sum()), "gain_vs_head_pct": round(gain, 3),
                                      "se_pct": round(se, 3)}
        entry["global_gain_pct"] = round(100.0 * (head_err - err).sum() / global_err.sum(), 4)
        entry["_err"] = err
        report["forms"][form] = entry
    # Paired comparisons between forms, by region.
    report["form_vs_form"] = {}
    for x, y in (("free", "count"), ("bishop_free", "count"), ("free", "bishop_free")):
        ex, ey = report["forms"][x]["_err"], report["forms"][y]["_err"]
        report["form_vs_form"][f"{x}_over_{y}"] = {
            name: dict(zip(("gain_pct", "se_pct"), [round(v, 3) for v in oc.paired(ey[m], ex[m])]))
            for name, m in regions.items()}
    for form in FORMS:
        report["forms"][form].pop("_err")
    text = json.dumps(report, indent=2)
    print(text)
    if args.output:
        args.output.write_text(text + "\n")
    return 0


def truth_rows(path: Path):
    """(family, fen, label for White, score, s0, pawns, (w kinds), (b kinds))."""
    for fen, label, score, _engine in oc.read_dump(path):
        cohort, pawns, _pw, _pb, s0 = oc.classify(fen)
        if cohort != oc.PURE:
            continue
        w, b = passer_kinds(oc.parse_board(fen))
        yield oc.material_key(fen), fen, label, score, s0, pawns, w, b


def cmd_separation(args) -> int:
    table = {}
    for family, fen, label, _score, _s0, _pawns, w, _b in truth_rows(args.dump):
        fam = table.setdefault(family, {})
        for form, idx in (("passers", 0), ("free", 1), ("bishop_free", 2)):
            cell = fam.setdefault(form, {}).setdefault(str(w[idx]), [0, 0])
            cell[0] += 1
            cell[1] += label == 1.0
        if family == "KBPP-KB":
            files = sorted(f for (f, _r), ch in oc.parse_board(fen).items() if ch == "P")
            dist = str(min(files[1] - files[0], 3))
            cell = fam.setdefault("file_distance_3plus", {}).setdefault(dist, [0, 0])
            cell[0] += 1
            cell[1] += label == 1.0
    out = {fam: {form: {k: {"n": n, "won": wn, "win_rate": round(wn / n, 3)}
                        for k, (n, wn) in sorted(cells.items())}
                 for form, cells in forms.items()}
           for fam, forms in sorted(table.items())}
    text = json.dumps(out, indent=2)
    print(text)
    if args.output:
        args.output.write_text(text + "\n")
    return 0


def cmd_drawn(args) -> int:
    params = tuple(int(x) for x in args.candidate.split(","))
    out = {}
    for family, _fen, label, score, s0, pawns, w, b in truth_rows(args.dump):
        raw = score if s0 == oc.OCB_NORMAL else (
            round((score + 0.5 * math.copysign(1, score)) * oc.OCB_NORMAL / s0) if score else 0)
        s1 = int(min(max(params[0] + params[1] * pawns + params[2] * q_of(args.form, w, b), 0),
                     oc.OCB_NORMAL))
        cand = int(math.trunc(raw * s1 / oc.OCB_NORMAL))
        fam = out.setdefault(family, {"draws": 0, "head_over": 0, "cand_over": 0,
                                      "wins": 0, "head_under": 0, "cand_under": 0})
        if label == 0.5:
            fam["draws"] += 1
            fam["head_over"] += score > args.threshold
            fam["cand_over"] += cand > args.threshold
        elif label == 1.0:
            fam["wins"] += 1
            fam["head_under"] += score < args.threshold
            fam["cand_under"] += cand < args.threshold
    text = json.dumps({"form": args.form, "candidate": list(params),
                       "threshold_cp": args.threshold, "families": out}, indent=2)
    print(text)
    if args.output:
        args.output.write_text(text + "\n")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("separation")
    p.add_argument("--dump", required=True, type=Path)
    p.add_argument("--output", type=Path)
    p = sub.add_parser("rule")
    p.add_argument("--train", required=True, type=Path)
    p.add_argument("--held-out", required=True, type=Path)
    p.add_argument("--k", required=True, type=float)
    p.add_argument("--output", type=Path)
    p = sub.add_parser("drawn")
    p.add_argument("--dump", required=True, type=Path)
    p.add_argument("--form", required=True, choices=FORMS)
    p.add_argument("--candidate", required=True)
    p.add_argument("--threshold", type=int, default=100)
    p.add_argument("--output", type=Path)
    args = ap.parse_args()
    return {"separation": cmd_separation, "rule": cmd_rule, "drawn": cmd_drawn}[args.cmd](args)


if __name__ == "__main__":
    sys.exit(main())
