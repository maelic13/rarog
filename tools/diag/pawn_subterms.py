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

C.6.7 adds Rarog's own definitions beside the donor's (`eval_pawns`:
isolated, doubled, doubled-isolated, backward, weak unopposed, lever,
candidate passers, islands, support, phalanx and the passed set by rank),
and the donor's passed() over Rarog's passed set. `verify-rarog` checks
those counts against the engine's own trace, field by field
(`rarog-texel --dump-feature ... --full`); `compare` reads, for each
definition the two engines draw differently, whether the donor's form adds
outcome information beyond Rarog's.

  python tools/diag/pawn_subterms.py verify-rarog --dump <dir>/pawn_subterms.csv --engine-dumps <dir>/engine
  python tools/diag/pawn_subterms.py compare --dump <dir>/pawn_subterms.csv --out <dir>/compare.json

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
                mine: ts.Side, theirs_side: ts.Side, filter_blocked: bool = True) -> dict:
    """`Evaluation::passed<Us>()`, component by component. Each pawn's bonus
    depends on the board, not on the rest of the set, so the family is
    additive over pawns; `filter_blocked=False` skips the donor's removal of
    blocked candidates without help, to value a chosen group of pawns."""
    them = not us
    comp = {name: (0, 0) for name in PASSED_COMPONENTS}
    our_pawns = board.pawns & board.occupied_co[us]
    their_pawns = board.pawns & board.occupied_co[them]
    our_king, their_king = board.king(us), board.king(them)

    b = passed_set
    blocked_passers = b & down(us, their_pawns)
    if blocked_passers and filter_blocked:
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


# Rarog's own pawn and passer definitions (`eval_pawns`, `src/eval/pawns.rs`),
# counted White minus Black, so each donor form can be read against Rarog's.
RAROG_SCALARS = ["r_isolated", "r_doubled", "r_doubled_isolated", "r_backward",
                 "r_unopposed", "r_lever", "r_candidate", "r_islands"]
RAROG_RANKED = ["r_support", "r_phalanx", "r_passed"]
RAROG_COLUMNS = RAROG_SCALARS + [f"{n}_{r}" for n in RAROG_RANKED for r in range(8)]
# The donor's passed() over Rarog's passed set, beside its own set's family.
PASSED_ON_RAROG_SET = ["pr_total"]

# Each engine trace field, the offline column(s) it must equal and the sign it
# is traced with (penalties are traced negative).
ENGINE_FIELDS = {
    "pawn_isolated_eg": (["r_isolated"], -1),
    "pawn_doubled_eg": (["r_doubled"], -1),
    "pawn_doubled_isolated_eg": (["r_doubled_isolated"], -1),
    "pawn_backward_eg": (["r_backward"], -1),
    "pawn_weak_unopposed_eg": (["r_unopposed"], -1),
    "pawn_lever_eg": (["r_lever"], 1),
    "passed_candidate_eg": (["r_candidate"], 1),
    "pawn_islands_eg": (["r_islands"], -1),
    "pawn_connected_eg": ([f"r_support_{r}" for r in range(8)], 1),
    "pawn_phalanx_eg": ([f"r_phalanx_{r}" for r in range(8)], 1),
    "passed_eg": ([f"r_passed_{r}" for r in range(8)], 1),
}


def rarog_side(board: chess.Board, us: chess.Color) -> tuple[dict, int]:
    """`Evaluator::eval_pawns` for `us`: occurrence counts and the passed set."""
    them = not us
    ours = board.pawns & board.occupied_co[us]
    theirs = board.pawns & board.occupied_co[them]
    c = {name: 0 for name in RAROG_SCALARS}
    for name in RAROG_RANKED:
        c[name] = [0] * 8
    passed_set = 0
    for s in chess.scan_forward(ours):
        f = chess.square_file(s)
        r = rel_rank(us, s)
        adjacent = adjacent_files(s)
        supported = bool(chess.BB_PAWN_ATTACKS[them][s] & ours)
        if not (passed_span(us, s) & theirs):
            passed_set |= chess.BB_SQUARES[s]
            c["r_passed"][r] += 1
        elif r >= 3 and supported and not (theirs & adjacent & forward_ranks(us, s)):
            c["r_candidate"] += 1
        doubled = ts.popcount(ours & chess.BB_FILES[f]) > 1
        isolated = not (ours & adjacent)
        c["r_doubled"] += doubled
        c["r_isolated"] += isolated
        c["r_doubled_isolated"] += doubled and isolated
        if supported:
            c["r_support"][r] += 1
        phalanx = ((f > 0 and ours & chess.BB_SQUARES[s - 1])
                   or (f < 7 and ours & chess.BB_SQUARES[s + 1]))
        if phalanx:
            c["r_phalanx"][r] += 1
        c["r_lever"] += bool(chess.BB_PAWN_ATTACKS[us][s] & theirs)
        stop = step(us, s)
        backward = (not (ours & forward_ranks(them, s) & adjacent)
                    and 0 <= stop < 64
                    and bool(chess.BB_PAWN_ATTACKS[us][stop] & theirs))
        c["r_backward"] += backward
        unopposed = not (theirs & forward_file(us, s))
        c["r_unopposed"] += (isolated or backward) and unopposed
    files = 0
    for f in range(8):
        if ours & chess.BB_FILES[f]:
            files |= 1 << f
    c["r_islands"] = bin(files & ~(files << 1)).count("1")
    return c, passed_set


# The pawns on which the two passed sets disagree, by why they differ: Rarog's
# rear pawn of a doubled passer (Rarog's set only); the donor's extra passers
# (only levers in the way; lever pushes outnumbered by the phalanx; a single
# blocker on the fifth rank or higher with a supported push), the last split
# by the donor's removal of blocked candidates without help.
GROUPS = ["g_rear", "g_lever", "g_leverpush", "g_blocked_kept", "g_blocked_dropped"]


def passed_groups(board: chess.Board, us: chess.Color, mine: ts.Side,
                  theirs_side: ts.Side) -> dict[str, int]:
    them = not us
    ours = board.pawns & board.occupied_co[us]
    theirs = board.pawns & board.occupied_co[them]
    double_attack_them = ts.pawn_double_attacks_bb(them, theirs)
    g = {name: 0 for name in GROUPS}
    blocked = 0
    for s in chess.scan_forward(ours):
        bit = chess.BB_SQUARES[s]
        r = rel_rank(us, s)
        s_up, s_down = step(us, s), step(them, s)
        stoppers = theirs & passed_span(us, s)
        own_ahead = forward_file(us, s) & ours
        if not stoppers:
            if own_ahead:
                g["g_rear"] |= bit
            continue
        if own_ahead:
            continue
        lever = theirs & chess.BB_PAWN_ATTACKS[us][s]
        lever_push = theirs & chess.BB_PAWN_ATTACKS[us][s_up]
        blocker = theirs & chess.BB_SQUARES[s_up]
        neighbours = ours & adjacent_files(s)
        phalanx = neighbours & chess.BB_RANKS[chess.square_rank(s)]
        support = neighbours & chess.BB_RANKS[chess.square_rank(s_down)]
        if not (stoppers ^ lever):
            g["g_lever"] |= bit
        elif not (stoppers ^ lever_push) and ts.popcount(phalanx) >= ts.popcount(lever_push):
            g["g_leverpush"] |= bit
        elif (stoppers == blocker and r >= 4
              and up(us, support) & ~(theirs | double_attack_them) & BB_ALL):
            blocked |= bit
    # Only the blocked group has an enemy pawn directly in front, so only it
    # meets the donor's removal of blocked candidates without help.
    helpers = (up(us, ours) & ~board.occupied_co[them]
               & ((~theirs_side.attacked2 & BB_ALL) | mine.attacked))
    kept = blocked & (shift_west(helpers) | shift_east(helpers))
    g["g_blocked_kept"] = kept
    g["g_blocked_dropped"] = blocked & ~kept & BB_ALL
    return g


def group_values_for(board: chess.Board) -> dict[str, int]:
    """Each group's passed() value White minus Black (mg, eg), and its pawn
    count over both sides."""
    white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
    gw = passed_groups(board, chess.WHITE, white, black)
    gb = passed_groups(board, chess.BLACK, black, white)
    out = {}
    for name in GROUPS:
        vw = passed_side(board, chess.WHITE, gw[name], white, black, filter_blocked=False)
        vb = passed_side(board, chess.BLACK, gb[name], black, white, filter_blocked=False)
        out[f"{name}_mg"] = sum(vw[n][0] for n in PASSED_COMPONENTS) - sum(vb[n][0] for n in PASSED_COMPONENTS)
        out[f"{name}_eg"] = sum(vw[n][1] for n in PASSED_COMPONENTS) - sum(vb[n][1] for n in PASSED_COMPONENTS)
        out[f"n_{name}"] = ts.popcount(gw[name]) + ts.popcount(gb[name])
    return out


GROUP_COLUMNS = [f"{name}_{part}" for name in GROUPS for part in ("mg", "eg")] + [f"n_{name}" for name in GROUPS]


def rarog_terms_for(fen: str) -> dict[str, int]:
    """Rarog's counts White minus Black, and the donor's passed() family
    (summed, in its internal units) over Rarog's passed set."""
    board = chess.Board(fen)
    rw, set_w = rarog_side(board, chess.WHITE)
    rb, set_b = rarog_side(board, chess.BLACK)
    out = {}
    for name in RAROG_SCALARS:
        out[name] = int(rw[name]) - int(rb[name])
    for name in RAROG_RANKED:
        for r in range(8):
            out[f"{name}_{r}"] = rw[name][r] - rb[name][r]
    white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
    qw = passed_side(board, chess.WHITE, set_w, white, black)
    qb = passed_side(board, chess.BLACK, set_b, black, white)
    out["pr_total_mg"] = sum(qw[n][0] for n in PASSED_COMPONENTS) - sum(qb[n][0] for n in PASSED_COMPONENTS)
    out["pr_total_eg"] = sum(qw[n][1] for n in PASSED_COMPONENTS) - sum(qb[n][1] for n in PASSED_COMPONENTS)
    return out


def header() -> str:
    cells = ["idx", "label", "rarog", "passed_mg", "passed_eg", "pawns_mg", "pawns_eg"]
    for name in COLUMNS:
        cells += [f"{name}_mg", f"{name}_eg"]
    cells += RAROG_COLUMNS + ["pr_total_mg", "pr_total_eg"] + GROUP_COLUMNS
    cells.append("fen")
    return ";".join(cells)


def score_chunk(rows) -> list[str]:
    out = []
    for idx, label, rarog, pmg, peg, wmg, weg, fen in rows:
        comps = components_for(fen)
        own = rarog_terms_for(fen)
        cells = [idx, label, rarog, pmg, peg, wmg, weg]
        for name in COLUMNS:
            cells += [str(comps[name][0]), str(comps[name][1])]
        cells += [str(own[name]) for name in RAROG_COLUMNS]
        cells += [str(own["pr_total_mg"]), str(own["pr_total_eg"])]
        groups = group_values_for(chess.Board(fen))
        cells += [str(groups[name]) for name in GROUP_COLUMNS]
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


def verify_rarog(args: argparse.Namespace) -> int:
    """Every row: the offline Rarog counts equal the engine's own trace, read
    from `rarog-texel --dump-feature <csv> <dir>/<field>.csv <field> --full`
    for each field in ENGINE_FIELDS. The dump's coefficient of an eg-only
    field is `count · (24 − phase) / 24`; at full phase it cannot be read and
    the row is counted as unreadable."""
    col, data, fens = ts.load_dump(args.dump)
    report = {"rows": len(fens), "fields": {}}
    ok = True
    for field, (names, sign) in ENGINE_FIELDS.items():
        path = os.path.join(args.engine_dumps, f"{field}.csv")
        engine_rows = {}
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                line = line.strip()
                if line:
                    fen, values, phase = line.rsplit(";", 2)
                    engine_rows[fen] = (values, phase)
        checked = unreadable = mismatched = missing = 0
        examples = []
        for i, fen in enumerate(fens):
            if fen not in engine_rows:
                missing += 1
                continue
            values, phase = engine_rows[fen]
            weight = (24 - int(phase)) / 24
            if weight <= 0:
                unreadable += 1
                continue
            checked += 1
            for name, value in zip(names, values.split(","), strict=True):
                engine = float(value) / weight
                offline = sign * data[i, col[name]]
                if abs(engine - offline) > 1e-6:
                    mismatched += 1
                    if len(examples) < 5:
                        examples.append((fen, name, engine, float(offline)))
                    break
        fine = mismatched == 0 and missing == 0 and checked > 0
        ok &= fine
        report["fields"][field] = {"checked": checked, "unreadable_full_phase": unreadable,
                                   "missing": missing, "mismatched_rows": mismatched,
                                   "examples": examples, "pass": fine}
    report["pass"] = ok
    print(json.dumps(report, indent=2))
    return 0 if ok else 1


# Each definition the two engines draw differently: the donor's form (its
# columns in the dump) against Rarog's (counts, split into mg and eg parts by
# the donor's phase so the model may price them freely).
DEFINITIONS = {
    "backward": (["w_backward"], ["r_backward"]),
    "weak_unopposed": (["w_unopposed"], ["r_unopposed"]),
    "isolated": (["w_isolated"], ["r_isolated"]),
    "doubled": (["w_doubled"], ["r_doubled"]),
    "doubled_isolated": (["w_doubled_isolated"], ["r_doubled_isolated"]),
    "connected": (["w_connected"], [f"r_support_{r}" for r in range(1, 7)]
                  + [f"r_phalanx_{r}" for r in range(1, 7)]),
    "lever": (["w_lever"], ["r_lever"]),
    "blocked": (["w_blocked"], []),
    "passed_set": (["p_total"], ["pr_total"]),
}


def compare(args: argparse.Namespace) -> int:
    """For each definition: Rarog's form, the donor's, and both, beside
    Rarog's score; the donor's form over Rarog's (paired) is what Rarog's
    definition misses."""
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

    def donor(name):
        if name == "p_total":
            return sum(taper(data[:, col[f"{c}_mg"]], data[:, col[f"{c}_eg"]])
                       for c in PASSED_COMPONENTS)
        return taper(data[:, col[f"{name}_mg"]], data[:, col[f"{name}_eg"]])

    def rarog_cols(name):
        if name == "pr_total":
            return [taper(data[:, col["pr_total_mg"]], data[:, col["pr_total_eg"]])]
        count = data[:, col[name]]
        return [count * phase / 128.0, count * (128.0 - phase) / 128.0]

    rng = np.random.default_rng(20261012)
    models: dict[str, np.ndarray] = {"rarog": rarog[:, None]}
    pairs = []
    for name, (dcols, rcols) in DEFINITIONS.items():
        d = [donor(c) for c in dcols]
        r = [x for c in rcols for x in rarog_cols(c)]
        models[f"{name}:rarog_form"] = np.column_stack([rarog] + r)
        models[f"{name}:donor_form"] = np.column_stack([rarog] + d)
        models[f"{name}:both"] = np.column_stack([rarog] + r + d)
        pairs.append((f"{name}:rarog_form", f"{name}:both", f"{name}: donor's beyond Rarog's"))
        pairs.append((f"{name}:rarog_form", f"{name}:donor_form", f"{name}: donor's instead of Rarog's"))
    # The null: the backward pair with the donor's column shuffled.
    rb = [x for x in rarog_cols("r_backward")]
    models["null:both"] = np.column_stack([rarog] + rb + [rng.permutation(donor("w_backward"))])
    pairs.append(("backward:rarog_form", "null:both", "null: shuffled donor backward beyond Rarog's"))

    cohorts = {
        "all": np.ones(n, dtype=bool),
        "phase<32": phase < 32,
        "no queens": ~queens,
        "men>=7": men >= 7,
        "phase 32-95": (phase >= 32) & (phase < 96),
    }
    report = {"dump": os.path.abspath(args.dump), "dump_sha256": dr.sha256_file(args.dump),
              "rows": int(n), "cohorts": {}}
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
        for a, b, label in pairs:
            diff = errors[a] - errors[b]
            paired[label] = {
                "gain_pct": float(100.0 * diff.mean() / base.mean()),
                "se_pct": float(100.0 * diff.std(ddof=1) / np.sqrt(count) / base.mean()),
            }
        report["cohorts"][cname] = {"rows": count, "models": rows, "paired": paired}
    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2)
    for cname, entry in report["cohorts"].items():
        print(f"\ncohort {cname} (n={entry['rows']})")
        for label, row in entry["paired"].items():
            print(f"  {label:<52} {row['gain_pct']:>+8.3f} {row['se_pct']:>7.3f}")
    return 0


def verify_split(args: argparse.Namespace) -> int:
    """Every row: the donor's passed() over its own set equals the family over
    Rarog's set, less Rarog's rear doubled passers, plus the donor's lever,
    lever-push and kept blocked passers, exactly, in mg and in eg."""
    col, data, _ = ts.load_dump(args.dump)
    report = {"rows": int(data.shape[0])}
    bad = 0
    for part in ("mg", "eg"):
        donor = sum(data[:, col[f"{c}_{part}"]] for c in PASSED_COMPONENTS)
        rebuilt = (data[:, col[f"pr_total_{part}"]] - data[:, col[f"g_rear_{part}"]]
                   + data[:, col[f"g_lever_{part}"]] + data[:, col[f"g_leverpush_{part}"]]
                   + data[:, col[f"g_blocked_kept_{part}"]])
        mismatch = int((abs(donor - rebuilt) > 1e-9).sum())
        report[f"mismatch_{part}"] = mismatch
        bad += mismatch
    report["rows_with_pawns_in_group"] = {name: int((data[:, col[f"n_{name}"]] > 0).sum())
                                          for name in GROUPS}
    report["pass"] = bad == 0
    print(json.dumps(report, indent=2))
    return 0 if report["pass"] else 1


def split(args: argparse.Namespace) -> int:
    """Each disagreeing group's passed() value beside Rarog's score and the
    family over Rarog's set: what that group adds beyond Rarog's set."""
    import numpy as np

    col, data, fens = ts.load_dump(args.dump)
    n = data.shape[0]
    y = data[:, col["label"]]
    rarog = np.clip(data[:, col["rarog"]] * 100.0, -dr.CLIP_CP, dr.CLIP_CP)
    phase = np.array([dr.sf11_phase(f) for f in fens], dtype=float)
    queens = np.array([dr.has_queen(f) for f in fens])
    men = np.array([dr.men_count(f) for f in fens])

    def taper(name):
        return ((data[:, col[f"{name}_mg"]] * phase + data[:, col[f"{name}_eg"]] * (128.0 - phase))
                / 128.0 / PAWN_EG * 100.0)

    base = taper("pr_total")
    rng = np.random.default_rng(20261012)
    models: dict[str, np.ndarray] = {"rarog": rarog[:, None],
                                     "rarog+rarog_set": np.column_stack([rarog, base])}
    pairs = []
    for name in GROUPS:
        models[f"rarog+rarog_set+{name}"] = np.column_stack([rarog, base, taper(name)])
        pairs.append(("rarog+rarog_set", f"rarog+rarog_set+{name}", f"{name} beyond Rarog's set"))
    models["rarog+rarog_set+all_groups"] = np.column_stack([rarog, base] + [taper(g) for g in GROUPS])
    pairs.append(("rarog+rarog_set", "rarog+rarog_set+all_groups", "all groups beyond Rarog's set"))
    models["rarog+rarog_set+null"] = np.column_stack([rarog, base, rng.permutation(taper("g_lever"))])
    pairs.append(("rarog+rarog_set", "rarog+rarog_set+null", "null: shuffled lever group"))

    cohorts = {
        "all": np.ones(n, dtype=bool),
        "phase<32": phase < 32,
        "no queens": ~queens,
        "men>=7": men >= 7,
    }
    report = {"dump": os.path.abspath(args.dump), "dump_sha256": dr.sha256_file(args.dump),
              "rows": int(n), "cohorts": {}}
    for cname, mask in cohorts.items():
        count = int(mask.sum())
        errors = {name: (y[mask] - dr.held_out_predictions(x[mask], y[mask])) ** 2
                  for name, x in models.items()}
        ref = errors["rarog"]
        paired = {}
        for a, b, label in pairs:
            diff = errors[a] - errors[b]
            paired[label] = {"gain_pct": float(100.0 * diff.mean() / ref.mean()),
                             "se_pct": float(100.0 * diff.std(ddof=1) / np.sqrt(count) / ref.mean())}
        present = {name: int((data[mask, col[f"n_{name}"]] > 0).sum()) for name in GROUPS}
        report["cohorts"][cname] = {"rows": count, "rows_with_group": present, "paired": paired}
    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2)
    for cname, entry in report["cohorts"].items():
        print(f"\ncohort {cname} (n={entry['rows']}); rows with each group: {entry['rows_with_group']}")
        for label, row in entry["paired"].items():
            print(f"  {label:<40} {row['gain_pct']:>+8.3f} {row['se_pct']:>7.3f}")
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
    vr = sub.add_parser("verify-rarog")
    vr.add_argument("--dump", required=True)
    vr.add_argument("--engine-dumps", required=True,
                    help="directory of <field>.csv files from rarog-texel --dump-feature --full")
    cp = sub.add_parser("compare")
    cp.add_argument("--dump", required=True)
    cp.add_argument("--out", required=True)
    vs = sub.add_parser("verify-split")
    vs.add_argument("--dump", required=True)
    sp = sub.add_parser("split")
    sp.add_argument("--dump", required=True)
    sp.add_argument("--out", required=True)
    args = parser.parse_args()
    return {"collect": collect, "verify": verify, "analyse": analyse,
            "verify-rarog": verify_rarog, "compare": compare,
            "verify-split": verify_split, "split": split}[args.command](args)


if __name__ == "__main__":
    sys.exit(main())
