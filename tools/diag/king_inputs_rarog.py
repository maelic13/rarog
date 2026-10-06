"""Rarog's king-danger inputs recomputed from the FEN (RAR-E21, part b).

`eval_king_safety` adds its weighted inputs into one integer danger index and
reads a 40-entry table with it. Three inputs (`ks_weak_ring`, `ks_flank_attack`,
`ks_shelter_storm`) have weight 0, so the engine's trace never shows how often
they fire or how far one unit of weight would move the index. This script
rebuilds the zone, the attack maps and every input with python-chess, exactly as
`src/eval.rs` defines them, and reports their activation, their covariance with
the attacker-unit sum, and the index movement at weight 1.

It is validated before use against the tuner's own trace on the same rows:
the share of rows whose two kings read different table buckets, and the row
counts of the sparse buckets, must equal `rarog-texel --feature-support`'s.

  python tools/diag/king_inputs_rarog.py --scores tools/results/donor-residual-20261005/scores.csv \
      --out tools/results/king-subterms-20261006/rarog_inputs.csv \
      --summary tools/results/king-subterms-20261006/rarog_inputs.json
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from concurrent.futures import ProcessPoolExecutor

import chess

# `king_safety_table` and the active danger weights at the frozen head
# (src/eval.rs; RAR-E12's fit).
TABLE = [84, 84, 98, 98, 112, 112, 153, 153, 153, 218, 248, 253, 253, 253, 253, 254, 254, 256,
         256, 256, 256, 256, 256, 315, 315, 374, 374, 374, 374, 374, 374, 374, 374, 374, 374, 374,
         374, 374, 440, 515]
UNIT = {chess.KNIGHT: 2, chess.BISHOP: 2, chess.ROOK: 2, chess.QUEEN: 5}
SAFE_CHECK = {chess.KNIGHT: 8, chess.BISHOP: 8, chess.ROOK: 8, chess.QUEEN: 16}
PAWNLESS_FLANK = 17
QUEEN_RELIEF = 2
PHASE_W = {chess.KNIGHT: 1, chess.BISHOP: 1, chess.ROOK: 2, chess.QUEEN: 4}
TOTAL_PHASE = 24

FIELDS = ["units", "checks", "pawnless", "relief", "danger", "bucket", "weak", "flank", "deficit",
          "table_cp"]


def pawn_attacks(bb: int, white: bool) -> tuple[int, int]:
    """(attack set, squares attacked by two pawns) for one side's pawns."""
    if white:
        a = (bb << 9) & ~chess.BB_FILE_A & chess.BB_ALL
        b = (bb << 7) & ~chess.BB_FILE_H & chess.BB_ALL
    else:
        a = (bb >> 7) & ~chess.BB_FILE_A
        b = (bb >> 9) & ~chess.BB_FILE_H
    return a | b, a & b


def forward_ranks(white: bool, rank: int) -> int:
    """Ranks strictly ahead of `rank` for the side (eval.rs FORWARD_RANKS)."""
    if white:
        return sum(chess.BB_RANKS[r] for r in range(rank + 1, 8)) if rank < 7 else 0
    return sum(chess.BB_RANKS[r] for r in range(0, rank)) if rank > 0 else 0


def slider_from(sq: int, occ: int) -> tuple[int, int]:
    bishop = chess.BB_DIAG_ATTACKS[sq][chess.BB_DIAG_MASKS[sq] & occ]
    rook = (chess.BB_RANK_ATTACKS[sq][chess.BB_RANK_MASKS[sq] & occ]
            | chess.BB_FILE_ATTACKS[sq][chess.BB_FILE_MASKS[sq] & occ])
    return bishop, rook


def king_inputs(board: chess.Board) -> tuple[int, dict[bool, dict[str, int]]]:
    occ = board.occupied
    attacked = {}
    attacked2 = {}
    attacked_by = {}
    pawns = {c: board.pawns & board.occupied_co[c] for c in (True, False)}
    phase = 0
    for color in (chess.WHITE, chess.BLACK):
        pa, double = pawn_attacks(pawns[color], color)
        att2 = double
        att = pa
        by = {chess.PAWN: pa}
        king_atk = chess.BB_KING_ATTACKS[board.king(color)]
        by[chess.KING] = king_atk
        att2 |= att & king_atk
        att |= king_atk
        for pt in (chess.KNIGHT, chess.BISHOP, chess.ROOK, chess.QUEEN):
            by[pt] = 0
            for sq in chess.scan_forward(board.pieces_mask(pt, color)):
                atks = int(board.attacks_mask(sq))
                by[pt] |= atks
                att2 |= att & atks
                att |= atks
                phase += PHASE_W[pt]
        attacked[color] = att
        attacked2[color] = att2
        attacked_by[color] = by
    phase = min(phase, TOTAL_PHASE)

    out = {}
    for color in (chess.WHITE, chess.BLACK):
        them = not color
        ksq = board.king(color)
        king_atk = chess.BB_KING_ATTACKS[ksq]
        zone = king_atk | chess.BB_SQUARES[ksq]
        zone |= ((king_atk << 8) & chess.BB_ALL) if color else (king_atk >> 8)

        units = 0
        for pt, unit in UNIT.items():
            for sq in chess.scan_forward(board.pieces_mask(pt, them)):
                if board.attacks_mask(sq) & zone:
                    units += unit

        weak = chess.popcount(zone & attacked[them] & (~attacked[color] | attacked2[them]))

        safe = ~attacked[color] & ~board.occupied_co[them] & chess.BB_ALL
        bishop_from, rook_from = slider_from(ksq, occ)
        knight_from = chess.BB_KNIGHT_ATTACKS[ksq]
        checks = 0
        checks += SAFE_CHECK[chess.KNIGHT] * chess.popcount(knight_from & attacked_by[them][chess.KNIGHT] & safe)
        checks += SAFE_CHECK[chess.BISHOP] * chess.popcount(bishop_from & attacked_by[them][chess.BISHOP] & safe)
        checks += SAFE_CHECK[chess.ROOK] * chess.popcount(rook_from & attacked_by[them][chess.ROOK] & safe)
        checks += SAFE_CHECK[chess.QUEEN] * chess.popcount((bishop_from | rook_from) & attacked_by[them][chess.QUEEN] & safe)

        kf = chess.square_file(ksq)
        krank = chess.square_rank(ksq)
        flank_bb = 0
        for f in range(kf - 1, kf + 2):
            if 0 <= f < 8:
                flank_bb |= chess.BB_FILES[f]
        flank = max(0, chess.popcount(attacked[them] & flank_bb) - chess.popcount(attacked[color] & flank_bb))

        pawnless = PAWNLESS_FLANK if not ((pawns[True] | pawns[False]) & flank_bb) else 0
        relief = QUEEN_RELIEF if not board.pieces_mask(chess.QUEEN, them) else 0

        deficit = 0
        if kf <= 2 or kf >= 5:
            ahead = forward_ranks(color, krank)
            for df in (-1, 0, 1):
                f = kf + df
                if not 0 <= f < 8:
                    continue
                if not (pawns[color] & chess.BB_FILES[f] & ahead):
                    deficit += 2 if df == 0 else 1
        for sq in chess.scan_forward(pawns[them] & flank_bb):
            rel = chess.square_rank(sq) if them else 7 - chess.square_rank(sq)
            if rel >= 3:
                deficit += rel - 2

        danger = units + checks + pawnless - relief
        bucket = min(max(danger, 0), len(TABLE) - 1)
        out[color] = {
            "units": units, "checks": checks, "pawnless": pawnless, "relief": relief,
            "danger": danger, "bucket": bucket, "weak": weak, "flank": flank, "deficit": deficit,
            "table_cp": TABLE[bucket] * phase // TOTAL_PHASE,
        }
    return phase, out


def score_chunk(chunk: list[tuple[str, str]]) -> list[str]:
    out = []
    for idx, fen in chunk:
        phase, sides = king_inputs(chess.Board(fen))
        cells = [idx, str(phase)]
        cells += [str(sides[chess.WHITE][f]) for f in FIELDS]
        cells += [str(sides[chess.BLACK][f]) for f in FIELDS]
        out.append(";".join(cells) + ";" + fen)
    return out


def header() -> str:
    return ";".join(["idx", "phase"] + [f"w_{f}" for f in FIELDS] + [f"b_{f}" for f in FIELDS] + ["fen"])


def summarise(rows: list[list[str]]) -> dict:
    import numpy as np

    col = {name: i for i, name in enumerate(header().split(";"))}
    data = np.array([[float(v) for v in r[:-1]] for r in rows])

    def w(f):
        return data[:, col[f"w_{f}"]]

    def b(f):
        return data[:, col[f"b_{f}"]]

    n = data.shape[0]
    wb, bb = w("bucket").astype(int), b("bucket").astype(int)
    differ = wb != bb
    slot_rows = {}
    for s in range(len(TABLE)):
        slot_rows[s] = int(np.sum(((wb == s) & (bb != s)) | ((bb == s) & (wb != s))))
    kings = {f: np.concatenate([w(f), b(f)]) for f in FIELDS}
    attacked = kings["units"] > 0
    rise = {}
    for f in ("weak", "flank", "deficit"):
        count = kings[f]
        clamped_now = np.minimum(np.maximum(kings["danger"], 0), 39)
        clamped_w1 = np.minimum(np.maximum(kings["danger"] + count, 0), 39)
        rise[f] = {
            "share_active_all_kings": float(np.mean(count > 0)),
            "share_active_attacked_kings": float(np.mean(count[attacked] > 0)),
            "mean_count_all_kings": float(np.mean(count)),
            "mean_count_attacked_kings": float(np.mean(count[attacked])),
            "p90_count_attacked_kings": float(np.percentile(count[attacked], 90)),
            "mean_bucket_rise_at_w1_attacked_kings": float(np.mean((clamped_w1 - clamped_now)[attacked])),
            "share_reaching_top_bucket_at_w1_attacked_kings": float(np.mean(clamped_w1[attacked] >= 39)),
            "corr_with_units_all_kings": float(np.corrcoef(count, kings["units"])[0, 1]),
            "corr_with_units_attacked_kings": float(np.corrcoef(count[attacked], kings["units"][attacked])[0, 1]),
            "corr_with_danger_attacked_kings": float(np.corrcoef(count[attacked], kings["danger"][attacked])[0, 1]),
        }
    hist = np.bincount(np.concatenate([wb, bb]), minlength=len(TABLE))
    return {
        "rows": int(n),
        "validation": {
            "share_rows_with_differing_buckets": float(np.mean(differ)),
            "rows_with_differing_buckets": int(differ.sum()),
            "sparse_slot_rows": {str(s): slot_rows[s] for s in (1, 3, 28, 30, 36)},
        },
        "kings_with_units": int(attacked.sum()),
        "share_kings_with_units": float(np.mean(attacked)),
        "mean_units_attacked_kings": float(np.mean(kings["units"][attacked])),
        "mean_checks_attacked_kings": float(np.mean(kings["checks"][attacked])),
        "share_kings_in_top_bucket": float(np.mean(np.concatenate([wb, bb]) >= 39)),
        "bucket_histogram_kings": [int(x) for x in hist],
        "inputs": rise,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--scores", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--summary", required=True)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--workers", type=int, default=12)
    args = ap.parse_args()

    rows: list[tuple[str, str]] = []
    with open(args.scores, encoding="utf-8") as handle:
        names = handle.readline().rstrip("\n").split(";")
        col = {n: i for i, n in enumerate(names)}
        for line in handle:
            cells = line.rstrip("\n").split(";")
            rows.append((cells[col["idx"]], cells[-1]))
            if args.limit and len(rows) >= args.limit:
                break
    chunks = [rows[i : i + 2000] for i in range(0, len(rows), 2000)]
    lines: list[str] = []
    with ProcessPoolExecutor(max_workers=args.workers) as pool:
        for out in pool.map(score_chunk, chunks):
            lines.extend(out)
    os.makedirs(os.path.dirname(os.path.abspath(args.out)), exist_ok=True)
    with open(args.out, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(header() + "\n" + "\n".join(lines) + "\n")
    summary = summarise([l.split(";") for l in lines])
    with open(args.summary, "w", encoding="utf-8") as handle:
        json.dump(summary, handle, indent=2)
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
