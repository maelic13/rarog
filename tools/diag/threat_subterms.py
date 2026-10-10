#!/usr/bin/env python3
"""C.4 (RAR-E31): which part of Stockfish's threats family carries its
donor-direction residual, and whether its mobility residual is the area, the
x-rays or the pin restriction.

Stockfish `9587eeeb`'s threats() and its per-piece mobility are rebuilt here
from the FEN with its own definitions (the weak and strongly-protected sets,
safe pawns and safe pushes, the king threat, the queen threats on safe squares;
the mobility area without blocked or low pawns, the king, the queen and the
king's blockers, bishop and rook x-rays through queens and own rooks, pinned
pieces cut to the pin line). `collect` writes every component per row beside
the printed `Threats` and `Mobility` rows of a `donor_residual.py` scores
file, and `verify` demands that the components sum to the printed rows on
every row where the donor computed them (its lazy exits and specialised
endgames print zero; those rows are zeroed the same way, so the family
direction is the one the residual screen read). `analyse` then reads the
components as directions beside Rarog's score, two-fold held-out logistic
models exactly as `donor_residual.py` fits them: the family without each
component, each component alone, the mobility tables under four area
definitions, and the ungated counterparts of the gated terms, so that
conditioning is told from pricing.

  python tools/diag/threat_subterms.py collect --scores <scores-9587-head.csv> --out <dir>
  python tools/diag/threat_subterms.py verify --dump <dir>/subterms.csv
  python tools/diag/threat_subterms.py analyse --dump <dir>/subterms.csv --out <dir>/report.json

Static outcome loss ranks questions and accepts nothing; it is not Elo.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from concurrent.futures import ProcessPoolExecutor

import chess

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import donor_residual as dr  # noqa: E402

# Stockfish's internal unit: the trace prints value / PawnValueEg.
PAWN_EG = 206.0

# Constants of 9587eeeb's evaluate.cpp, (mg, eg).
MOBILITY_BONUS = {
    chess.KNIGHT: [(-62, -81), (-53, -56), (-12, -31), (-4, -16), (3, 5), (13, 11),
                   (22, 17), (28, 20), (33, 25)],
    chess.BISHOP: [(-48, -59), (-20, -23), (16, -3), (26, 13), (38, 24), (51, 42),
                   (55, 54), (63, 57), (63, 65), (68, 73), (81, 78), (81, 86),
                   (91, 88), (98, 97)],
    chess.ROOK: [(-60, -78), (-20, -17), (2, 23), (3, 39), (3, 70), (11, 99),
                 (22, 103), (31, 121), (40, 134), (40, 139), (41, 158), (48, 164),
                 (57, 168), (57, 169), (62, 172)],
    chess.QUEEN: [(-30, -48), (-12, -30), (-8, -7), (-9, 19), (20, 40), (23, 55),
                  (23, 59), (35, 75), (38, 78), (53, 96), (64, 96), (65, 100),
                  (65, 121), (66, 127), (67, 131), (67, 133), (72, 136), (72, 141),
                  (77, 147), (79, 150), (93, 151), (108, 168), (108, 168), (108, 171),
                  (110, 182), (114, 182), (114, 192), (116, 219)],
}
# Indexed by the victim's piece type (python-chess numbers them as Stockfish
# does: pawn 1 to king 6; the king entry is zero in the donor's array).
THREAT_BY_MINOR = [(0, 0), (5, 32), (57, 41), (77, 56), (88, 119), (79, 161), (0, 0)]
THREAT_BY_ROOK = [(0, 0), (3, 46), (37, 68), (42, 60), (0, 38), (58, 41), (0, 0)]
HANGING = (69, 36)
THREAT_BY_KING = (24, 89)
WEAK_QUEEN_PROTECTION = (14, 0)
RESTRICTED_PIECE = (7, 7)
THREAT_BY_SAFE_PAWN = (173, 94)
THREAT_BY_PAWN_PUSH = (48, 39)
KNIGHT_ON_QUEEN = (16, 11)
SLIDER_ON_QUEEN = (60, 18)

# The threats components, in the donor's order, and the Rarog-shaped
# counterparts of the gated ones (every attacked piece scored, pawn threats
# from any pawn, a push that needs only a pawn-free push square).
THREAT_COMPONENTS = [
    "by_minor", "by_rook", "by_king", "hanging", "weak_queen_protection",
    "restricted", "safe_pawn", "pawn_push", "knight_on_queen", "slider_on_queen",
]
UNGATED_COMPONENTS = ["by_minor_ungated", "by_rook_ungated", "pawn_ungated", "push_ungated"]
GATED_FOR = {"by_minor_ungated": "by_minor", "by_rook_ungated": "by_rook",
             "pawn_ungated": "safe_pawn", "push_ungated": "pawn_push"}
# Mobility under: the donor's definition (v0); true-occupancy attacks instead
# of x-rays (no_xray); pins ignored, in the area and in the attacks (no_pin);
# both (no_xray_no_pin); Rarog's area with Rarog's attacks (rarog).
MOBILITY_VARIANTS = ["v0", "no_xray", "no_pin", "no_xray_no_pin", "rarog"]

BB_ALL = chess.BB_ALL
NOT_FILE_A = BB_ALL & ~chess.BB_FILE_A
NOT_FILE_H = BB_ALL & ~chess.BB_FILE_H


def north(bb: int) -> int:
    return (bb << 8) & BB_ALL


def south(bb: int) -> int:
    return bb >> 8


def pawn_attacks_bb(color: chess.Color, pawns: int) -> int:
    if color == chess.WHITE:
        return ((pawns << 7) & NOT_FILE_H) | ((pawns << 9) & NOT_FILE_A)
    return ((pawns >> 9) & NOT_FILE_H) | ((pawns >> 7) & NOT_FILE_A)


def pawn_double_attacks_bb(color: chess.Color, pawns: int) -> int:
    if color == chess.WHITE:
        return ((pawns << 7) & NOT_FILE_H) & ((pawns << 9) & NOT_FILE_A)
    return ((pawns >> 9) & NOT_FILE_H) & ((pawns >> 7) & NOT_FILE_A)


def bishop_attacks(sq: int, occ: int) -> int:
    return chess.BB_DIAG_ATTACKS[sq][chess.BB_DIAG_MASKS[sq] & occ]


def rook_attacks(sq: int, occ: int) -> int:
    return (chess.BB_RANK_ATTACKS[sq][chess.BB_RANK_MASKS[sq] & occ]
            | chess.BB_FILE_ATTACKS[sq][chess.BB_FILE_MASKS[sq] & occ])


def popcount(bb: int) -> int:
    return bb.bit_count()


def add(a: tuple[int, int], b: tuple[int, int], n: int = 1) -> tuple[int, int]:
    return a[0] + n * b[0], a[1] + n * b[1]


def blockers_for_king(board: chess.Board, color: chess.Color) -> int:
    """Stockfish's `blockers_for_king(color)`: pieces of either colour that
    alone stand between an enemy slider and this king (snipers are not
    occupancy for each other)."""
    ksq = board.king(color)
    them = not color
    snipers = ((rook_attacks(ksq, 0) & (board.rooks | board.queens))
               | (bishop_attacks(ksq, 0) & (board.bishops | board.queens))) & board.occupied_co[them]
    occupancy = board.occupied ^ snipers
    blockers = 0
    for s in chess.scan_forward(snipers):
        b = chess.between(ksq, s) & occupancy
        if b and popcount(b) == 1:
            blockers |= b
    return blockers


class Side:
    """One colour's attack maps and mobility, by the donor's definitions and
    by the variants under test."""

    def __init__(self, board: chess.Board, color: chess.Color):
        self.color = color
        them = not color
        occ = board.occupied
        own = board.occupied_co[color]
        pawns = board.pawns & own
        their_pawns = board.pawns & board.occupied_co[them]
        ksq = board.king(color)
        low_ranks = (chess.BB_RANK_2 | chess.BB_RANK_3) if color == chess.WHITE else (chess.BB_RANK_7 | chess.BB_RANK_6)
        ahead_occupied = south(occ) if color == chess.WHITE else north(occ)
        self.pawn_attacks = pawn_attacks_bb(color, pawns)
        self.their_pawn_attacks = pawn_attacks_bb(them, their_pawns)
        self.blockers = blockers_for_king(board, color)
        stuck = pawns & (ahead_occupied | low_ranks)
        king_queen = (board.kings | board.queens) & own
        self.area = BB_ALL & ~(stuck | king_queen | self.blockers | self.their_pawn_attacks)
        area_no_pin = BB_ALL & ~(stuck | king_queen | self.their_pawn_attacks)
        area_rarog = BB_ALL & ~(self.their_pawn_attacks | own)

        king_atk = chess.BB_KING_ATTACKS[ksq]
        self.attacked_by = {chess.PAWN: self.pawn_attacks, chess.KING: king_atk,
                            chess.KNIGHT: 0, chess.BISHOP: 0, chess.ROOK: 0, chess.QUEEN: 0}
        self.attacked = king_atk | self.pawn_attacks
        self.attacked2 = pawn_double_attacks_bb(color, pawns) | (king_atk & self.pawn_attacks)
        self.mobility = {name: (0, 0) for name in MOBILITY_VARIANTS}
        # Each piece's donor attacks, for the threat terms that read them.
        self.attacks_of = {}
        xray_occ = {chess.BISHOP: occ ^ board.queens,
                    chess.ROOK: occ ^ board.queens ^ (board.rooks & own)}
        for pt in (chess.KNIGHT, chess.BISHOP, chess.ROOK, chess.QUEEN):
            table = MOBILITY_BONUS[pt]
            pieces = board.pieces_mask(pt, color)
            for s in chess.scan_forward(pieces):
                if pt == chess.KNIGHT:
                    plain = xray = chess.BB_KNIGHT_ATTACKS[s]
                elif pt == chess.BISHOP:
                    plain = bishop_attacks(s, occ)
                    xray = bishop_attacks(s, xray_occ[pt])
                elif pt == chess.ROOK:
                    plain = rook_attacks(s, occ)
                    xray = rook_attacks(s, xray_occ[pt])
                else:
                    plain = xray = bishop_attacks(s, occ) | rook_attacks(s, occ)
                pinned = bool(self.blockers & chess.BB_SQUARES[s])
                line = chess.ray(ksq, s) if pinned else BB_ALL
                b = xray & line
                self.attacks_of[s] = b
                self.attacked2 |= self.attacked & b
                self.attacked_by[pt] |= b
                self.attacked |= b
                counts = {
                    "v0": popcount(b & self.area),
                    "no_xray": popcount(plain & line & self.area),
                    "no_pin": popcount(xray & area_no_pin),
                    "no_xray_no_pin": popcount(plain & area_no_pin),
                    "rarog": popcount(plain & area_rarog),
                }
                for name, mob in counts.items():
                    self.mobility[name] = add(self.mobility[name], table[min(mob, len(table) - 1)])


def threats(board: chess.Board, us: Side, them: Side) -> dict[str, tuple[int, int]]:
    """9587eeeb's threats() for `us`, component by component, plus the
    ungated counterparts."""
    color = us.color
    occ = board.occupied
    own_pawns = board.pawns & board.occupied_co[color]
    enemies = board.occupied_co[not color]
    non_pawn_enemies = enemies & ~board.pawns
    strongly_protected = them.attacked_by[chess.PAWN] | (them.attacked2 & ~us.attacked2)
    defended = non_pawn_enemies & strongly_protected
    weak = enemies & ~strongly_protected & us.attacked
    out = {name: (0, 0) for name in THREAT_COMPONENTS + UNGATED_COMPONENTS}
    minor_att = us.attacked_by[chess.KNIGHT] | us.attacked_by[chess.BISHOP]
    rook_att = us.attacked_by[chess.ROOK]

    def victims(bb: int, table) -> tuple[int, int]:
        total = (0, 0)
        for s in chess.scan_forward(bb):
            total = add(total, table[board.piece_type_at(s)])
        return total

    if defended | weak:
        out["by_minor"] = victims((defended | weak) & minor_att, THREAT_BY_MINOR)
        out["by_rook"] = victims(weak & rook_att, THREAT_BY_ROOK)
        if weak & us.attacked_by[chess.KING]:
            out["by_king"] = THREAT_BY_KING
        b = ~them.attacked | (non_pawn_enemies & us.attacked2)
        out["hanging"] = add((0, 0), HANGING, popcount(weak & b))
        out["weak_queen_protection"] = add((0, 0), WEAK_QUEEN_PROTECTION,
                                           popcount(weak & them.attacked_by[chess.QUEEN]))
    out["by_minor_ungated"] = victims(enemies & minor_att, THREAT_BY_MINOR)
    out["by_rook_ungated"] = victims(enemies & rook_att, THREAT_BY_ROOK)

    restricted = them.attacked & ~strongly_protected & us.attacked
    out["restricted"] = add((0, 0), RESTRICTED_PIECE, popcount(restricted))

    safe = ~them.attacked | us.attacked
    b = pawn_attacks_bb(color, own_pawns & safe) & non_pawn_enemies
    out["safe_pawn"] = add((0, 0), THREAT_BY_SAFE_PAWN, popcount(b))
    out["pawn_ungated"] = add((0, 0), THREAT_BY_SAFE_PAWN,
                              popcount(pawn_attacks_bb(color, own_pawns) & non_pawn_enemies))

    empty = BB_ALL & ~occ
    if color == chess.WHITE:
        push = north(own_pawns) & empty
        push |= north(push & chess.BB_RANK_3) & empty
    else:
        push = south(own_pawns) & empty
        push |= south(push & chess.BB_RANK_6) & empty
    push_rarog = push & ~them.attacked_by[chess.PAWN]
    push &= ~them.attacked_by[chess.PAWN] & safe
    out["pawn_push"] = add((0, 0), THREAT_BY_PAWN_PUSH,
                           popcount(pawn_attacks_bb(color, push) & non_pawn_enemies))
    out["push_ungated"] = add((0, 0), THREAT_BY_PAWN_PUSH,
                              popcount(pawn_attacks_bb(color, push_rarog) & non_pawn_enemies))

    their_queens = board.queens & enemies
    if popcount(their_queens) == 1:
        imbalance = 1 if popcount(board.queens) == 1 else 0
        qsq = chess.msb(their_queens)
        safe_q = us.area & ~own_pawns & ~strongly_protected
        b = us.attacked_by[chess.KNIGHT] & chess.BB_KNIGHT_ATTACKS[qsq]
        out["knight_on_queen"] = add((0, 0), KNIGHT_ON_QUEEN, popcount(b & safe_q) * (1 + imbalance))
        b = ((us.attacked_by[chess.BISHOP] & bishop_attacks(qsq, occ))
             | (us.attacked_by[chess.ROOK] & rook_attacks(qsq, occ)))
        out["slider_on_queen"] = add((0, 0), SLIDER_ON_QUEEN,
                                     popcount(b & safe_q & us.attacked2) * (1 + imbalance))
    return out


def components_for(fen: str) -> dict[str, tuple[int, int]]:
    """White minus Black, in the donor's internal units, for every threats
    component, its ungated counterpart and every mobility variant."""
    board = chess.Board(fen)
    white, black = Side(board, chess.WHITE), Side(board, chess.BLACK)
    tw, tb = threats(board, white, black), threats(board, black, white)
    out = {}
    for name in THREAT_COMPONENTS + UNGATED_COMPONENTS:
        out[name] = (tw[name][0] - tb[name][0], tw[name][1] - tb[name][1])
    for name in MOBILITY_VARIANTS:
        out[f"mob_{name}"] = (white.mobility[name][0] - black.mobility[name][0],
                              white.mobility[name][1] - black.mobility[name][1])
    return out


COLUMNS = THREAT_COMPONENTS + UNGATED_COMPONENTS + [f"mob_{v}" for v in MOBILITY_VARIANTS]


def header() -> str:
    cells = ["idx", "label", "rarog", "threats_mg", "threats_eg", "mobility_mg", "mobility_eg"]
    for name in COLUMNS:
        cells += [f"{name}_mg", f"{name}_eg"]
    cells.append("fen")
    return ";".join(cells)


def score_chunk(rows: list[tuple[str, str, str, str, str, str, str, str]]) -> list[str]:
    out = []
    for idx, label, rarog, tmg, teg, mmg, meg, fen in rows:
        comps = components_for(fen)
        cells = [idx, label, rarog, tmg, teg, mmg, meg]
        for name in COLUMNS:
            cells += [str(comps[name][0]), str(comps[name][1])]
        cells.append(fen)
        out.append(";".join(cells))
    return out


def read_scores(path: str):
    """(idx, label, rarog, threats mg, eg, mobility mg, eg, fen) per row of a
    `donor_residual.py` scores file."""
    rows = []
    with open(path, encoding="utf-8") as handle:
        head = handle.readline().rstrip("\n").split(";")
        col = {name: i for i, name in enumerate(head)}
        for line in handle:
            c = line.rstrip("\n").split(";")
            if len(c) != len(head) or c[col["rarog"]] == "invalid":
                continue
            rows.append((c[col["idx"]], c[col["label"]], c[col["rarog"]],
                         c[col["threats_mg"]], c[col["threats_eg"]],
                         c[col["mobility_mg"]], c[col["mobility_eg"]], c[col["fen"]]))
    return rows


def collect(args: argparse.Namespace) -> int:
    rows = read_scores(args.scores)
    if args.limit:
        rows = rows[:args.limit]
    os.makedirs(args.out, exist_ok=True)
    out_path = os.path.join(args.out, "subterms.csv")
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


def load_dump(path: str):
    import numpy as np

    with open(path, encoding="utf-8") as handle:
        head = handle.readline().rstrip("\n").split(";")
        col = {name: i for i, name in enumerate(head)}
        fens, numeric = [], []
        for line in handle:
            c = line.rstrip("\n").split(";")
            fens.append(c[col["fen"]])
            numeric.append([float(v) for v in c[:-1]])
    return col, np.array(numeric), fens


def printed(value_mg: float, value_eg: float) -> tuple[float, float]:
    """The donor's trace prints value / PawnValueEg to two decimals."""
    return round(value_mg / PAWN_EG, 2), round(value_eg / PAWN_EG, 2)


def verify(args: argparse.Namespace) -> int:
    """Every row: the components sum to the printed Threats row and `mob_v0`
    equals the printed Mobility row, except where the donor printed zero for
    a family it did not compute (a lazy exit or a specialised endgame)."""
    import numpy as np

    col, data, _ = load_dump(args.dump)
    tmg = sum(data[:, col[f"{n}_mg"]] for n in THREAT_COMPONENTS)
    teg = sum(data[:, col[f"{n}_eg"]] for n in THREAT_COMPONENTS)
    report = {"rows": int(data.shape[0])}
    for family, mg, eg in (("threats", tmg, teg),
                           ("mobility", data[:, col["mob_v0_mg"]], data[:, col["mob_v0_eg"]])):
        pmg, peg = data[:, col[f"{family}_mg"]], data[:, col[f"{family}_eg"]]
        rmg, reg = np.round(mg / PAWN_EG, 2), np.round(eg / PAWN_EG, 2)
        match = (np.abs(rmg - pmg) < 0.006) & (np.abs(reg - peg) < 0.006)
        skipped = (pmg == 0) & (peg == 0) & ~match
        mismatch = ~match & ~skipped
        worst = [int(i) for i in np.flatnonzero(mismatch)[:10]]
        report[family] = {"match": int(match.sum()), "printed_zero_not_computed": int(skipped.sum()),
                          "mismatch": int(mismatch.sum()), "first_mismatch_rows": worst}
    report["pass"] = report["threats"]["mismatch"] == 0 and report["mobility"]["mismatch"] == 0
    print(json.dumps(report, indent=2))
    return 0 if report["pass"] else 1


def analyse(args: argparse.Namespace) -> int:
    import numpy as np

    col, data, fens = load_dump(args.dump)
    n = data.shape[0]
    y = data[:, col["label"]]
    rarog = np.clip(data[:, col["rarog"]] * 100.0, -dr.CLIP_CP, dr.CLIP_CP)
    phase = np.array([dr.sf11_phase(f) for f in fens], dtype=float)
    queens = np.array([dr.has_queen(f) for f in fens])
    men = np.array([dr.men_count(f) for f in fens])

    def taper(mg, eg):
        return (mg * phase + eg * (128.0 - phase)) / 128.0 / PAWN_EG * 100.0

    # Rows the donor did not compute print zero; the components are zeroed
    # there too, so every direction below is the family the screen read.
    comp = {name: taper(data[:, col[f"{name}_mg"]], data[:, col[f"{name}_eg"]]) for name in COLUMNS}
    threats_sum = sum(comp[name] for name in THREAT_COMPONENTS)
    threats_printed = (data[:, col["threats_mg"]] != 0) | (data[:, col["threats_eg"]] != 0)
    active_t = threats_printed | (threats_sum == 0)
    mobility_printed = (data[:, col["mobility_mg"]] != 0) | (data[:, col["mobility_eg"]] != 0)
    active_m = mobility_printed | (comp["mob_v0"] == 0)
    for name in THREAT_COMPONENTS + UNGATED_COMPONENTS:
        comp[name] = comp[name] * active_t
    for name in MOBILITY_VARIANTS:
        comp[f"mob_{name}"] = comp[f"mob_{name}"] * active_m
    threats_sum = sum(comp[name] for name in THREAT_COMPONENTS)

    rng = np.random.default_rng(20261010)
    models: dict[str, np.ndarray] = {"rarog": rarog[:, None]}
    models["rarog+threats"] = np.column_stack([rarog, threats_sum])
    models["rarog+shuffled_threats"] = np.column_stack([rarog, rng.permutation(threats_sum)])
    for name in THREAT_COMPONENTS:
        models[f"rarog+threats-{name}"] = np.column_stack([rarog, threats_sum - comp[name]])
        models[f"rarog+{name}"] = np.column_stack([rarog, comp[name]])
    models["rarog+threats_components"] = np.column_stack([rarog] + [comp[n] for n in THREAT_COMPONENTS])
    ungated_sum = threats_sum
    for ungated, gated in GATED_FOR.items():
        ungated_sum = ungated_sum - comp[gated] + comp[ungated]
        models[f"rarog+threats_with_{ungated}"] = np.column_stack(
            [rarog, threats_sum - comp[gated] + comp[ungated]])
    models["rarog+threats_ungated"] = np.column_stack([rarog, ungated_sum])
    for name in MOBILITY_VARIANTS:
        models[f"rarog+mob_{name}"] = np.column_stack([rarog, comp[f"mob_{name}"]])
    models["rarog+mob_rarog+mob_v0"] = np.column_stack([rarog, comp["mob_rarog"], comp["mob_v0"]])

    cohorts = {
        "all": np.ones(n, dtype=bool),
        "phase>=96": phase >= 96,
        "phase 32-95": (phase >= 32) & (phase < 96),
        "phase<32": phase < 32,
        "men>=7": men >= 7,
        "no queens": ~queens,
        "queens": queens,
    }
    # Paired differences read beside each model's gain: every leave-one-out
    # against the whole family, every ungated form against the gated one,
    # every mobility variant against the donor's.
    pairs = [("rarog+threats", f"rarog+threats-{name}") for name in THREAT_COMPONENTS]
    pairs += [("rarog+threats", f"rarog+threats_with_{u}") for u in GATED_FOR]
    pairs += [("rarog+threats", "rarog+threats_ungated"), ("rarog+threats", "rarog+threats_components")]
    pairs += [("rarog+mob_v0", f"rarog+mob_{v}") for v in MOBILITY_VARIANTS if v != "v0"]
    pairs += [("rarog+mob_rarog", "rarog+mob_rarog+mob_v0")]

    report = {
        "dump": os.path.abspath(args.dump),
        "dump_sha256": dr.sha256_file(args.dump),
        "rows": int(n),
        "rows_threats_zeroed": int((~active_t).sum()),
        "rows_mobility_zeroed": int((~active_m).sum()),
        "cohorts": {},
    }
    for cname, mask in cohorts.items():
        count = int(mask.sum())
        if count == 0:
            continue
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
    print(f"rows {n}; threats zeroed {report['rows_threats_zeroed']}, "
          f"mobility zeroed {report['rows_mobility_zeroed']}")
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
