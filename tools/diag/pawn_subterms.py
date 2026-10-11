#!/usr/bin/env python3
"""C.6 (C.6.3): which parts of Stockfish's passed-pawn and pawn-structure
families carry their donor-direction residual.

Stockfish `9587eeeb`'s `Evaluation::passed()` (`evaluate.cpp`) and
`Pawns::evaluate()` (`pawns.cpp`) are rebuilt here from the FEN with its own
definitions, component by component: for passed pawns the rank bonus, the
enemy king's and our king's proximity to the block square (and to the next
square), the free-path bonus, the defended-block bonus and the file
adjustment, over the donor's passed set (its three passed conditions and its
removal of blocked candidates without help); for pawns the connected bonus,
isolated, weak unopposed, the doubled-isolated case, backward, doubled, weak
lever and blocked pawns. The attack maps come from `threat_subterms.Side`,
which reproduced the donor's threats and mobility exactly (RAR-E31).

`collect` writes every component per row beside the printed `Passed` and
`Pawns` rows of a `donor_residual.py` scores file; `verify` demands that the
components sum to the printed rows on every row where the donor computed
them (a specialised endgame prints every row zero, and the first lazy exit
prints `Passed` zero; those rows are zeroed the same way). `analyse` reads
the components as directions beside Rarog's score with `donor_residual.py`'s
two-fold held-out logistic models: each family, the family without each
component, and each component alone, by cohort.

  python tools/diag/pawn_subterms.py collect --scores <scores.csv> --out <dir>
  python tools/diag/pawn_subterms.py verify --dump <dir>/pawn_subterms.csv
  python tools/diag/pawn_subterms.py analyse --dump <dir>/pawn_subterms.csv --out <dir>/report.json

Static outcome loss ranks questions and accepts nothing; it is not Elo.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from concurrent.futures import ProcessPoolExecutor

import chess

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import donor_residual as dr  # noqa: E402
import threat_subterms as ts  # noqa: E402

PAWN_EG = ts.PAWN_EG

# Constants of 9587eeeb's pawns.cpp and evaluate.cpp, (mg, eg).
BACKWARD = (9, 24)
DOUBLED = (11, 56)
ISOLATED = (5, 15)
WEAK_LEVER = (0, 56)
WEAK_UNOPPOSED = (13, 27)
BLOCKED_PAWN = [(-11, -4), (-3, 4)]
CONNECTED = [0, 7, 8, 12, 29, 48, 86]
PASSED_RANK = [(0, 0), (10, 28), (17, 33), (15, 41), (62, 72), (168, 177), (276, 260)]
PASSED_FILE = (11, 8)

PASSED_COMPONENTS = ["p_rank", "p_king_them", "p_king_us", "p_path", "p_defended", "p_file"]
PAWN_COMPONENTS = ["w_connected", "w_isolated", "w_unopposed", "w_doubled_isolated",
                   "w_backward", "w_doubled", "w_lever", "w_blocked"]
COLUMNS = PASSED_COMPONENTS + PAWN_COMPONENTS
FAMILIES = {"passed": PASSED_COMPONENTS, "pawns": PAWN_COMPONENTS}

BB_ALL = chess.BB_ALL


def tdiv(a: int, b: int) -> int:
    """C++ integer division, truncating toward zero."""
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b > 0) else -q


def rel_rank(color: chess.Color, sq: int) -> int:
    r = chess.square_rank(sq)
    return r if color == chess.WHITE else 7 - r


def up(color: chess.Color, bb: int) -> int:
    return ts.north(bb) if color == chess.WHITE else ts.south(bb)


def down(color: chess.Color, bb: int) -> int:
    return ts.south(bb) if color == chess.WHITE else ts.north(bb)


def shift_west(bb: int) -> int:
    return (bb >> 1) & ~chess.BB_FILE_H & BB_ALL


def shift_east(bb: int) -> int:
    return (bb << 1) & ~chess.BB_FILE_A & BB_ALL


def adjacent_files(sq: int) -> int:
    f = chess.square_file(sq)
    bb = 0
    if f > 0:
        bb |= chess.BB_FILES[f - 1]
    if f < 7:
        bb |= chess.BB_FILES[f + 1]
    return bb


def forward_ranks(color: chess.Color, sq: int) -> int:
    """Every square on a rank strictly ahead of `sq` for `color`."""
    r = chess.square_rank(sq)
    ranks = range(r + 1, 8) if color == chess.WHITE else range(0, r)
    bb = 0
    for rr in ranks:
        bb |= chess.BB_RANKS[rr]
    return bb


def forward_file(color: chess.Color, sq: int) -> int:
    return forward_ranks(color, sq) & chess.BB_FILES[chess.square_file(sq)]


def passed_span(color: chess.Color, sq: int) -> int:
    return forward_ranks(color, sq) & (adjacent_files(sq) | chess.BB_FILES[chess.square_file(sq)])


def step(color: chess.Color, sq: int) -> int:
    return sq + 8 if color == chess.WHITE else sq - 8


def add(acc: dict, name: str, mg: int, eg: int) -> None:
    acc[name] = (acc[name][0] + mg, acc[name][1] + eg)


def pawns_side(board: chess.Board, us: chess.Color) -> tuple[dict, int]:
    """`Pawns::evaluate<Us>()`: the components and the donor's passed set."""
    them = not us
    ours = board.pawns & board.occupied_co[us]
    theirs = board.pawns & board.occupied_co[them]
    double_attack_them = ts.pawn_double_attacks_bb(them, theirs)
    comp = {name: (0, 0) for name in PAWN_COMPONENTS}
    passed_set = 0
    for s in chess.scan_forward(ours):
        r = rel_rank(us, s)
        s_up = step(us, s)
        s_down = step(them, s)
        opposed = theirs & forward_file(us, s)
        blocked = theirs & chess.BB_SQUARES[s_up]
        stoppers = theirs & passed_span(us, s)
        lever = theirs & chess.BB_PAWN_ATTACKS[us][s]
        lever_push = theirs & chess.BB_PAWN_ATTACKS[us][s_up]
        doubled = ours & chess.BB_SQUARES[s_down]
        neighbours = ours & adjacent_files(s)
        phalanx = neighbours & chess.BB_RANKS[chess.square_rank(s)]
        support = neighbours & chess.BB_RANKS[chess.square_rank(s_down)]
        backward = (not (neighbours & forward_ranks(them, s_up))) and bool(lever_push | blocked)

        passed = (not (stoppers ^ lever)
                  or (not (stoppers ^ lever_push)
                      and ts.popcount(phalanx) >= ts.popcount(lever_push))
                  or (stoppers == blocked and r >= 4
                      and bool(up(us, support) & ~(theirs | double_attack_them) & BB_ALL)))
        passed = passed and not (forward_file(us, s) & ours)
        if passed:
            passed_set |= chess.BB_SQUARES[s]

        if support | phalanx:
            v = (CONNECTED[r] * (2 + bool(phalanx) - bool(opposed))
                 + 21 * ts.popcount(support))
            add(comp, "w_connected", v, tdiv(v * (r - 2), 4))
        elif not neighbours:
            if opposed and (ours & forward_file(them, s)) and not (theirs & adjacent_files(s)):
                add(comp, "w_doubled_isolated", -DOUBLED[0], -DOUBLED[1])
            else:
                add(comp, "w_isolated", -ISOLATED[0], -ISOLATED[1])
                if not opposed:
                    add(comp, "w_unopposed", -WEAK_UNOPPOSED[0], -WEAK_UNOPPOSED[1])
        elif backward:
            add(comp, "w_backward", -BACKWARD[0], -BACKWARD[1])
            if not opposed:
                add(comp, "w_unopposed", -WEAK_UNOPPOSED[0], -WEAK_UNOPPOSED[1])

        if not support:
            if doubled:
                add(comp, "w_doubled", -DOUBLED[0], -DOUBLED[1])
            if ts.popcount(lever) > 1:
                add(comp, "w_lever", -WEAK_LEVER[0], -WEAK_LEVER[1])

        if blocked and r > 3:
            b = BLOCKED_PAWN[r - 4]
            add(comp, "w_blocked", b[0], b[1])
    return comp, passed_set


def passed_side(board: chess.Board, us: chess.Color, passed_set: int,
                mine: ts.Side, theirs_side: ts.Side) -> dict:
    """`Evaluation::passed<Us>()`, component by component."""
    them = not us
    comp = {name: (0, 0) for name in PASSED_COMPONENTS}
    our_pawns = board.pawns & board.occupied_co[us]
    their_pawns = board.pawns & board.occupied_co[them]
    our_king, their_king = board.king(us), board.king(them)

    b = passed_set
    blocked_passers = b & down(us, their_pawns)
    if blocked_passers:
        helpers = (up(us, our_pawns) & ~board.occupied_co[them]
                   & ((~theirs_side.attacked2 & BB_ALL) | mine.attacked))
        b &= (~blocked_passers & BB_ALL) | shift_west(helpers) | shift_east(helpers)

    def proximity(king: int, sq: int) -> int:
        return min(chess.square_distance(king, sq), 5)

    rooks_queens = board.rooks | board.queens
    for s in chess.scan_forward(b):
        r = rel_rank(us, s)
        add(comp, "p_rank", *PASSED_RANK[r])
        if r > 2:
            w = 5 * r - 13
            block = step(us, s)
            add(comp, "p_king_them", 0, (proximity(their_king, block) * 19) // 4 * w)
            us_part = -proximity(our_king, block) * 2 * w
            if r != 6:
                us_part -= proximity(our_king, step(us, block)) * w
            add(comp, "p_king_us", 0, us_part)
            if not board.piece_at(block):
                to_queen = forward_file(us, s)
                unsafe = passed_span(us, s)
                behind = forward_file(them, s) & rooks_queens
                if not (board.occupied_co[them] & behind):
                    unsafe &= theirs_side.attacked
                k = (35 if not unsafe else
                     20 if not (unsafe & to_queen) else
                     9 if not (unsafe & chess.BB_SQUARES[block]) else 0)
                add(comp, "p_path", k * w, k * w)
                if (board.occupied_co[us] & behind) or (mine.attacked & chess.BB_SQUARES[block]):
                    add(comp, "p_defended", 5 * w, 5 * w)
        f = chess.square_file(s)
        edge = min(f, 7 - f)
        add(comp, "p_file", -PASSED_FILE[0] * edge, -PASSED_FILE[1] * edge)
    return comp


def components_for(fen: str) -> dict[str, tuple[int, int]]:
    """White minus Black, in the donor's internal units, for every component."""
    board = chess.Board(fen)
    white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
    pw, passed_w = pawns_side(board, chess.WHITE)
    pb, passed_b = pawns_side(board, chess.BLACK)
    qw = passed_side(board, chess.WHITE, passed_w, white, black)
    qb = passed_side(board, chess.BLACK, passed_b, black, white)
    out = {}
    for name in PAWN_COMPONENTS:
        out[name] = (pw[name][0] - pb[name][0], pw[name][1] - pb[name][1])
    for name in PASSED_COMPONENTS:
        out[name] = (qw[name][0] - qb[name][0], qw[name][1] - qb[name][1])
    return out


def header() -> str:
    cells = ["idx", "label", "rarog", "passed_mg", "passed_eg", "pawns_mg", "pawns_eg"]
    for name in COLUMNS:
        cells += [f"{name}_mg", f"{name}_eg"]
    cells.append("fen")
    return ";".join(cells)


def score_chunk(rows) -> list[str]:
    out = []
    for idx, label, rarog, pmg, peg, wmg, weg, fen in rows:
        comps = components_for(fen)
        cells = [idx, label, rarog, pmg, peg, wmg, weg]
        for name in COLUMNS:
            cells += [str(comps[name][0]), str(comps[name][1])]
        cells.append(fen)
        out.append(";".join(cells))
    return out


def read_scores(path: str):
    rows = []
    with open(path, encoding="utf-8") as handle:
        head = handle.readline().rstrip("\n").split(";")
        col = {name: i for i, name in enumerate(head)}
        for line in handle:
            c = line.rstrip("\n").split(";")
            if len(c) != len(head) or c[col["rarog"]] == "invalid":
                continue
            rows.append((c[col["idx"]], c[col["label"]], c[col["rarog"]],
                         c[col["passed_mg"]], c[col["passed_eg"]],
                         c[col["pawns_mg"]], c[col["pawns_eg"]], c[col["fen"]]))
    return rows


def collect(args: argparse.Namespace) -> int:
    rows = read_scores(args.scores)
    if args.limit:
        rows = rows[:args.limit]
    os.makedirs(args.out, exist_ok=True)
    out_path = os.path.join(args.out, "pawn_subterms.csv")
    chunks = [rows[i:i + 2000] for i in range(0, len(rows), 2000)]
    with ProcessPoolExecutor(max_workers=args.workers) as pool, \
            open(out_path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(header() + "\n")
        for lines in pool.map(score_chunk, chunks):
            handle.write("\n".join(lines) + "\n")
    manifest = {
        "scores": os.path.abspath(args.scores),
        "scores_sha256": dr.sha256_file(args.scores),
        "rows": len(rows),
        "dump": os.path.abspath(out_path),
        "dump_sha256": dr.sha256_file(out_path),
    }
    with open(os.path.join(args.out, "collect-manifest.json"), "w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2)
    print(json.dumps(manifest, indent=2))
    return 0


def family_sums(col, data):
    return {fam: (sum(data[:, col[f"{n}_mg"]] for n in names),
                  sum(data[:, col[f"{n}_eg"]] for n in names))
            for fam, names in FAMILIES.items()}


def verify(args: argparse.Namespace) -> int:
    """Every row: each family's components sum to its printed row, except
    where the donor printed zero for a family it did not compute."""
    import numpy as np

    col, data, _ = ts.load_dump(args.dump)
    report = {"rows": int(data.shape[0])}
    for family, (mg, eg) in family_sums(col, data).items():
        pmg, peg = data[:, col[f"{family}_mg"]], data[:, col[f"{family}_eg"]]
        rmg, reg = np.round(mg / PAWN_EG, 2), np.round(eg / PAWN_EG, 2)
        match = (np.abs(rmg - pmg) < 0.006) & (np.abs(reg - peg) < 0.006)
        skipped = (pmg == 0) & (peg == 0) & ~match
        mismatch = ~match & ~skipped
        report[family] = {"match": int(match.sum()), "printed_zero_not_computed": int(skipped.sum()),
                          "mismatch": int(mismatch.sum()),
                          "first_mismatch_rows": [int(i) for i in np.flatnonzero(mismatch)[:10]]}
    report["pass"] = all(report[f]["mismatch"] == 0 for f in FAMILIES)
    print(json.dumps(report, indent=2))
    return 0 if report["pass"] else 1


def analyse(args: argparse.Namespace) -> int:
    import numpy as np

    col, data, fens = ts.load_dump(args.dump)
    n = data.shape[0]
    y = data[:, col["label"]]
    rarog = np.clip(data[:, col["rarog"]] * 100.0, -dr.CLIP_CP, dr.CLIP_CP)
    phase = np.array([dr.sf11_phase(f) for f in fens], dtype=float)
    queens = np.array([dr.has_queen(f) for f in fens])
    men = np.array([dr.men_count(f) for f in fens])

    def taper(mg, eg):
        return (mg * phase + eg * (128.0 - phase)) / 128.0 / PAWN_EG * 100.0

    comp = {name: taper(data[:, col[f"{name}_mg"]], data[:, col[f"{name}_eg"]]) for name in COLUMNS}
    zeroed = {}
    for family, names in FAMILIES.items():
        total = sum(comp[name] for name in names)
        printed = (data[:, col[f"{family}_mg"]] != 0) | (data[:, col[f"{family}_eg"]] != 0)
        active = printed | (total == 0)
        zeroed[family] = int((~active).sum())
        for name in names:
            comp[name] = comp[name] * active
    fam = {family: sum(comp[name] for name in names) for family, names in FAMILIES.items()}

    rng = np.random.default_rng(20261011)
    models: dict[str, np.ndarray] = {"rarog": rarog[:, None]}
    pairs = []
    for family, names in FAMILIES.items():
        models[f"rarog+{family}"] = np.column_stack([rarog, fam[family]])
        models[f"rarog+shuffled_{family}"] = np.column_stack([rarog, rng.permutation(fam[family])])
        for name in names:
            models[f"rarog+{family}-{name}"] = np.column_stack([rarog, fam[family] - comp[name]])
            models[f"rarog+{name}"] = np.column_stack([rarog, comp[name]])
            pairs.append((f"rarog+{family}", f"rarog+{family}-{name}"))
        models[f"rarog+{family}_components"] = np.column_stack([rarog] + [comp[x] for x in names])
        pairs.append((f"rarog+{family}", f"rarog+{family}_components"))
    models["rarog+passed+pawns"] = np.column_stack([rarog, fam["passed"], fam["pawns"]])

    cohorts = {
        "all": np.ones(n, dtype=bool),
        "phase>=96": phase >= 96,
        "phase 32-95": (phase >= 32) & (phase < 96),
        "phase<32": phase < 32,
        "men>=7": men >= 7,
        "no queens": ~queens,
        "queens": queens,
    }
    report = {
        "dump": os.path.abspath(args.dump),
        "dump_sha256": dr.sha256_file(args.dump),
        "rows": int(n),
        "rows_zeroed": zeroed,
        "cohorts": {},
    }
    for cname, mask in cohorts.items():
        count = int(mask.sum())
        errors = {name: (y[mask] - dr.held_out_predictions(x[mask], y[mask])) ** 2
                  for name, x in models.items()}
        base = errors["rarog"]
        rows = {}
        for mname, err in errors.items():
            diff = base - err
            rows[mname] = {
                "gain_vs_rarog_pct": float(100.0 * diff.mean() / base.mean()),
                "gain_se_pct": float(100.0 * diff.std(ddof=1) / np.sqrt(count) / base.mean()),
            }
        paired = {}
        for a, b in pairs:
            diff = errors[a] - errors[b]
            paired[f"{b} vs {a}"] = {
                "gain_pct": float(100.0 * diff.mean() / base.mean()),
                "se_pct": float(100.0 * diff.std(ddof=1) / np.sqrt(count) / base.mean()),
            }
        report["cohorts"][cname] = {"rows": count, "models": rows, "paired": paired}

    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2)
    print(f"rows {n}; zeroed {zeroed}")
    for cname, entry in report["cohorts"].items():
        print(f"\ncohort {cname} (n={entry['rows']})")
        print(f"  {'model':<40} {'gain vs rarog %':>16} {'se %':>8}")
        for mname, row in entry["models"].items():
            print(f"  {mname:<40} {row['gain_vs_rarog_pct']:>+16.3f} {row['gain_se_pct']:>8.3f}")
        print(f"  {'paired':<40} {'gain %':>16} {'se %':>8}")
        for pname, row in entry["paired"].items():
            print(f"  {pname:<40} {row['gain_pct']:>+16.3f} {row['se_pct']:>8.3f}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    c = sub.add_parser("collect")
    c.add_argument("--scores", required=True)
    c.add_argument("--out", required=True)
    c.add_argument("--workers", type=int, default=8)
    c.add_argument("--limit", type=int, default=0)
    v = sub.add_parser("verify")
    v.add_argument("--dump", required=True)
    a = sub.add_parser("analyse")
    a.add_argument("--dump", required=True)
    a.add_argument("--out", required=True)
    args = parser.parse_args()
    return {"collect": collect, "verify": verify, "analyse": analyse}[args.command](args)


if __name__ == "__main__":
    sys.exit(main())
