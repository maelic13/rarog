"""C.0.4 (RAR-E21): which part of Stockfish's king term carries the donor-direction
residual, and what its scale factor and complexity add as directions.

Stockfish `9587eeeb` built with `tools/diag/patches/sf9587_king_subterms.patch`
prints a `ksdump` line after its `eval` table: the king() components per colour
(shelter/storm, king-to-pawn distance, the danger index and every term that
sums into it, the pawnless-flank and flank-attack inputs) and the winnable()
internals (complexity, the capped bonuses, the scale factor and the branch that
set it).  `collect` scores RAR-E17's rows with it; `analyse` rebuilds the five
sub-terms and the danger index without each of its components, and compares
held-out outcome models exactly as `donor_residual.py` does (two folds,
logistic with an intercept, gain in percent of Rarog's held-out squared error).

  python tools/diag/king_subterms.py collect --scores tools/results/donor-residual-20261005/scores.csv \
      --sf <instrumented stockfish.exe> --out tools/results/king-subterms-20261006
  python tools/diag/king_subterms.py analyse --dump .../ksdump.csv --out .../report.json

Static outcome loss ranks questions and accepts nothing.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from concurrent.futures import ProcessPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import donor_residual as dr  # noqa: E402

# Stockfish's internal units: one pawn is PawnValueEg.
PAWN_EG = 206.0
# Constants of king() at 9587eeeb, needed to rebuild the danger index and the
# two flat penalties from the dumped inputs.
PAWNLESS_FLANK = (17, 95)
FLANK_ATTACKS = 8
KING_FIELDS = [
    "shelter_mg", "shelter_eg", "kpdist", "kd", "kd_mg", "kd_eg", "pawnless",
    "flank_att", "flank_def", "att_count", "att_weight", "weak", "unsafe",
    "blockers", "king_attacks", "checks", "mob_diff", "no_queen", "knight_def",
    "feedback", "rook_checks", "queen_checks", "bishop_checks", "knight_checks",
    "ring", "ran",
]
WIN_FIELDS = [
    "complexity", "u", "v", "mg", "eg", "strong", "sf_specific", "sf", "branch",
    "outflanking", "both_flanks", "almost_unwinnable", "infiltration",
    "passed_count", "no_pieces", "phase", "value", "specialized", "lazy1", "lazy2",
]
BRANCH_NAMES = {
    -1: "specific", 0: "none", 1: "ocb-pure", 2: "ocb-other", 3: "rook-one-flank",
    4: "single-queen", 5: "pawn-count",
}


def parse_ksdump(block: str) -> tuple[dict[str, int], dict[str, int], dict[str, int]] | None:
    """White and black king() inputs and the winnable() internals, or None."""
    for line in block.splitlines():
        if not line.startswith("ksdump "):
            continue
        tokens = line.split()
        if len(tokens) != 3 + 1 + 26 + 1 + 26 + 1 + 20:
            raise ValueError(f"ksdump line of {len(tokens)} tokens: {line[:80]}")
        assert tokens[1] == "v" and tokens[3] == "w" and tokens[30] == "b" and tokens[57] == "win"
        white = dict(zip(KING_FIELDS, map(int, tokens[4:30])))
        black = dict(zip(KING_FIELDS, map(int, tokens[31:57])))
        win = dict(zip(WIN_FIELDS, map(int, tokens[58:78])))
        win["final"] = int(tokens[2])
        return white, black, win
    return None


def danger_components(side: dict[str, int]) -> dict[str, int]:
    """The terms that sum into king()'s danger index, in its own units."""
    fa = side["flank_att"]
    return {
        "attackers": side["att_count"] * side["att_weight"],
        "weak_ring": 185 * side["weak"],
        "unsafe_checks": 148 * side["unsafe"],
        "blockers": 98 * side["blockers"],
        "king_attacks": 69 * side["king_attacks"],
        "flank_sq": int(3 * fa * fa / 8),
        "mobility_diff": side["mob_diff"],
        "no_queen": -873 * side["no_queen"],
        "knight_defender": -100 * side["knight_def"],
        "shelter_feedback": side["feedback"],
        "flank_defense": -4 * side["flank_def"],
        "constant": 37,
        "safe_checks": side["checks"],
    }


def danger_identity_holds(side: dict[str, int]) -> bool:
    return not side["ran"] or sum(danger_components(side).values()) == side["kd"]


def danger_score(kd):
    """mg and eg of the danger penalty for an index array (positive = penalty)."""
    import numpy as np

    kd = np.asarray(kd, dtype=np.int64)
    active = kd > 100
    mg = np.where(active, kd * kd // 4096, 0)
    eg = np.where(active, kd // 16, 0)
    return mg.astype(float), eg.astype(float)


def king_parts(side: dict[str, int]) -> dict[str, tuple[int, int]]:
    """The five sub-terms of one king, (mg, eg) in Stockfish units, signed as
    they enter that side's score (penalties negative)."""
    kd = side["kd"]
    kd_mg = kd * kd // 4096 if kd > 100 else 0
    kd_eg = kd // 16 if kd > 100 else 0
    return {
        "shelter": (side["shelter_mg"], side["shelter_eg"]),
        "kpdist": (0, -16 * side["kpdist"]),
        "danger": (-kd_mg, -kd_eg),
        "pawnless": (-PAWNLESS_FLANK[0] * side["pawnless"], -PAWNLESS_FLANK[1] * side["pawnless"]),
        "flankatt": (-FLANK_ATTACKS * side["flank_att"], 0),
    }


def king_row_matches(white: dict[str, int], black: dict[str, int], row_mg: float, row_eg: float) -> bool:
    """The printed `King safety` total (pawns) equals the sum of the parts."""
    if not white["ran"]:
        return row_mg == 0.0 and row_eg == 0.0
    pw, pb = king_parts(white), king_parts(black)
    mg = sum(v[0] for v in pw.values()) - sum(v[0] for v in pb.values())
    eg = sum(v[1] for v in pw.values()) - sum(v[1] for v in pb.values())
    return abs(mg / PAWN_EG - row_mg) <= 0.011 and abs(eg / PAWN_EG - row_eg) <= 0.011


def header() -> str:
    cells = ["idx", "label", "rarog"]
    cells += [f"w_{f}" for f in KING_FIELDS] + [f"b_{f}" for f in KING_FIELDS]
    cells += [f"win_{f}" for f in WIN_FIELDS] + ["win_final", "king_mg", "king_eg", "fen"]
    return ";".join(cells)


def score_chunk(args: tuple[str, list[tuple[str, str, str, str]]]) -> list[str]:
    sf, chunk = args
    blocks = dr.run_eval(sf, [], [fen for _, _, _, fen in chunk])
    out: list[str] = []
    for (idx, label, rarog, fen), block in zip(chunk, blocks):
        parsed = parse_ksdump(block)
        terms, _ = dr.parse_sf11_trace(block)
        if parsed is None or "King safety" not in terms:
            out.append(f"{idx};{label};{rarog};invalid;{fen}")
            continue
        white, black, win = parsed
        if not (danger_identity_holds(white) and danger_identity_holds(black)):
            raise RuntimeError(f"danger identity failed: {fen}")
        if not king_row_matches(white, black, *terms["King safety"]):
            raise RuntimeError(f"king row does not match its parts: {fen}")
        cells = [idx, label, rarog]
        cells += [str(white[f]) for f in KING_FIELDS] + [str(black[f]) for f in KING_FIELDS]
        cells += [str(win[f]) for f in WIN_FIELDS] + [str(win["final"])]
        cells += [f"{terms['King safety'][0]:.2f}", f"{terms['King safety'][1]:.2f}", fen]
        out.append(";".join(cells))
    return out


def collect(args: argparse.Namespace) -> int:
    rows: list[tuple[str, str, str, str]] = []
    with open(args.scores, encoding="utf-8") as handle:
        names = handle.readline().rstrip("\n").split(";")
        col = {n: i for i, n in enumerate(names)}
        for line in handle:
            cells = line.rstrip("\n").split(";")
            rows.append((cells[col["idx"]], cells[col["label"]], cells[col["rarog"]], cells[-1]))
            if args.limit and len(rows) >= args.limit:
                break
    os.makedirs(args.out, exist_ok=True)
    chunks = [rows[i : i + args.chunk] for i in range(0, len(rows), args.chunk)]
    print(f"{len(rows)} rows in {len(chunks)} chunks, {args.workers} workers", flush=True)
    lines = [header()]
    invalid = 0
    with ProcessPoolExecutor(max_workers=args.workers) as pool:
        for done, out in enumerate(pool.map(score_chunk, [(args.sf, c) for c in chunks]), 1):
            invalid += sum(1 for o in out if ";invalid;" in o)
            lines.extend(out)
            if done % 10 == 0 or done == len(chunks):
                print(f"  {done}/{len(chunks)} chunks", flush=True)
    dump = os.path.join(args.out, "ksdump.csv")
    with open(dump, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(lines) + "\n")
    manifest = {
        "scores": os.path.abspath(args.scores),
        "scores_sha256": dr.sha256_file(args.scores),
        "stockfish": os.path.abspath(args.sf),
        "stockfish_sha256": dr.sha256_file(args.sf),
        "rows": len(rows),
        "invalid": invalid,
        "dump": os.path.abspath(dump),
        "dump_sha256": dr.sha256_file(dump),
    }
    with open(os.path.join(args.out, "collect-manifest.json"), "w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2)
    print(f"wrote {dump}: {len(rows)} rows, {invalid} invalid")
    return 1 if invalid else 0


def load_dump(path: str):
    import numpy as np

    with open(path, encoding="utf-8") as handle:
        names = handle.readline().rstrip("\n").split(";")
        raw = [line.rstrip("\n").split(";") for line in handle]
    col = {name: i for i, name in enumerate(names)}
    bad = [r for r in raw if len(r) != len(names)]
    if bad:
        raise ValueError(f"{len(bad)} invalid rows in {path}")
    data = np.array([[float(v) for v in row[:-1]] for row in raw])
    fens = [row[-1] for row in raw]
    return col, data, fens


def analyse(args: argparse.Namespace) -> int:
    import numpy as np

    col, data, fens = load_dump(args.dump)
    n = data.shape[0]
    y = data[:, col["label"]]
    rarog = np.clip(data[:, col["rarog"]] * 100.0, -dr.CLIP_CP, dr.CLIP_CP)
    clock = np.array([float(f.split()[4]) for f in fens])
    phase = data[:, col["win_phase"]]  # the trace's own game phase, 0..128
    ran = data[:, col["w_ran"]] > 0

    def w(f):
        return data[:, col[f"w_{f}"]]

    def b(f):
        return data[:, col[f"b_{f}"]]

    def taper(mg, eg):
        return (mg * phase + eg * (128.0 - phase)) / 128.0 * 100.0 / PAWN_EG

    # The five sub-terms, white minus black, in cp.
    kd_w_mg, kd_w_eg = danger_score(w("kd"))
    kd_b_mg, kd_b_eg = danger_score(b("kd"))
    parts = {
        "shelter": taper(w("shelter_mg") - b("shelter_mg"), w("shelter_eg") - b("shelter_eg")),
        "kpdist": taper(0.0 * phase, -16.0 * (w("kpdist") - b("kpdist"))),
        "danger": taper(-(kd_w_mg - kd_b_mg), -(kd_w_eg - kd_b_eg)),
        "pawnless": taper(
            -PAWNLESS_FLANK[0] * (w("pawnless") - b("pawnless")),
            -PAWNLESS_FLANK[1] * (w("pawnless") - b("pawnless")),
        ),
        "flankatt": taper(-FLANK_ATTACKS * (w("flank_att") - b("flank_att")), 0.0 * phase),
    }
    king = sum(parts.values())
    king_row = taper(data[:, col["king_mg"]] * PAWN_EG, data[:, col["king_eg"]] * PAWN_EG)
    wire_king_max_err = float(np.max(np.abs(king - king_row)))

    # The danger index without each component, mapped through the same curve.
    comps_w = {k: np.zeros(n) for k in danger_components({f: 0 for f in KING_FIELDS})}
    comps_b = {k: np.zeros(n) for k in comps_w}
    for i in range(n):
        side_w = {f: int(data[i, col[f"w_{f}"]]) for f in KING_FIELDS}
        side_b = {f: int(data[i, col[f"b_{f}"]]) for f in KING_FIELDS}
        for k, v in danger_components(side_w).items():
            comps_w[k][i] = v
        for k, v in danger_components(side_b).items():
            comps_b[k][i] = v
    kd_sum_w = sum(comps_w.values())
    kd_sum_b = sum(comps_b.values())
    identity_rows = int(np.sum(ran & ((kd_sum_w != w("kd")) | (kd_sum_b != b("kd")))))
    danger_without = {}
    for k in comps_w:
        mg_w, eg_w = danger_score(np.where(ran, w("kd") - comps_w[k], 0))
        mg_b, eg_b = danger_score(np.where(ran, b("kd") - comps_b[k], 0))
        danger_without[k] = taper(-(mg_w - mg_b), -(eg_w - eg_b))
    kd_linear = taper(-(w("kd") - b("kd")) * np.where(ran, 1.0, 0.0), -(w("kd") - b("kd")) * np.where(ran, 1.0, 0.0) / 16.0)

    # Scale and complexity from winnable().
    sf = data[:, col["win_sf"]]
    eg_before = data[:, col["win_eg"]]
    v_bonus = data[:, col["win_v"]]
    u_bonus = data[:, col["win_u"]]
    sf_term = (eg_before + v_bonus) * (sf - 64.0) / 64.0 * (128.0 - phase) / 128.0 * 100.0 / PAWN_EG
    complexity = taper(u_bonus, v_bonus)
    rarog_scaled = rarog * (sf - 64.0) / 64.0 * (128.0 - phase) / 128.0
    branch = data[:, col["win_branch"]]
    specialized = data[:, col["win_specialized"]] > 0
    lazy1 = data[:, col["win_lazy1"]] > 0

    rng = np.random.default_rng(20261006)
    models: dict[str, np.ndarray] = {
        "rarog": rarog[:, None],
        "rarog+king9587": np.column_stack([rarog, king]),
        "rarog+king_parts": np.column_stack([rarog] + [parts[p] for p in parts]),
        "rarog+shuffled_danger": np.column_stack([rarog, rng.permutation(parts["danger"])]),
        "rarog+kd_linear": np.column_stack([rarog, kd_linear]),
        "rarog+sf_term": np.column_stack([rarog, sf_term]),
        "rarog+complexity": np.column_stack([rarog, complexity]),
        "rarog+sf_term+complexity": np.column_stack([rarog, sf_term, complexity]),
        "rarog+rarog_scaled": np.column_stack([rarog, rarog_scaled]),
        "rarog+rarog_clock": np.column_stack([rarog, rarog * clock / 100.0]),
        "rarog+king9587+sf_term": np.column_stack([rarog, king, sf_term]),
    }
    for p in parts:
        models[f"rarog+ks_{p}"] = np.column_stack([rarog, parts[p]])
        models[f"rarog+king_minus_{p}"] = np.column_stack([rarog, king - parts[p]])
    for k in danger_without:
        models[f"rarog+king_danger_without_{k}"] = np.column_stack(
            [rarog, king - parts["danger"] + danger_without[k]]
        )

    sf11_phase = np.array([dr.sf11_phase(f) for f in fens], dtype=float)
    queens = np.array([dr.has_queen(f) for f in fens])
    men = np.array([dr.men_count(f) for f in fens])
    cohorts = {
        "all": np.ones(n, dtype=bool),
        "phase>=96": sf11_phase >= 96,
        "phase 32-95": (sf11_phase >= 32) & (sf11_phase < 96),
        "phase<32": sf11_phase < 32,
        "queens": queens,
        "no queens": ~queens,
        "men<=6": men <= 6,
        "men>=7": men >= 7,
        "|rarog|<=500": np.abs(rarog) <= 500.0,
        "|rarog|>500": np.abs(rarog) > 500.0,
    }
    for code, name in BRANCH_NAMES.items():
        mask = (branch == code) & (men >= 7) & ~specialized
        if mask.sum() >= 2000:
            cohorts[f"men>=7 sf-branch {name}"] = mask

    def squared_errors(mask):
        return {
            name: (y[mask] - dr.held_out_predictions(x[mask], y[mask])) ** 2
            for name, x in models.items()
        }

    report: dict[str, object] = {
        "dump": os.path.abspath(args.dump),
        "dump_sha256": dr.sha256_file(args.dump),
        "rows": int(n),
        "fit": "within each cohort",
        "wire": {
            "rows_king_ran": int(ran.sum()),
            "rows_specialized": int(specialized.sum()),
            "rows_lazy_skip_1": int(lazy1.sum()),
            "danger_identity_failures": identity_rows,
            "king_bundle_vs_row_max_cp": wire_king_max_err,
            "mean_abs_king_cp": float(np.mean(np.abs(king))),
            "mean_abs_sf_term_cp": float(np.mean(np.abs(sf_term))),
            "rows_sf_below_64": int(np.sum(sf < 64)),
        },
        "cohorts": {},
        "magnitude": {},
    }
    for cname, mask in cohorts.items():
        count = int(mask.sum())
        errors = squared_errors(mask)
        base = errors["rarog"]
        rows = {}
        for mname, err in errors.items():
            diff = base - err
            rows[mname] = {
                "mse": float(err.mean()),
                "gain_vs_rarog_pct": float(100.0 * diff.mean() / base.mean()),
                "gain_se_pct": float(100.0 * diff.std(ddof=1) / np.sqrt(count) / base.mean()),
            }
        report["cohorts"][cname] = {"rows": count, "models": rows}

    # Magnitude by |rarog| band: what the donor's king term and scale add where
    # the frozen search expects the current distribution.
    bands = [(0, 100), (101, 300), (301, 600), (601, 1200), (1201, 100000)]
    sign = np.sign(rarog)
    for lo, hi in bands:
        mask = (np.abs(rarog) >= lo) & (np.abs(rarog) <= hi)
        if mask.sum() == 0:
            continue
        report["magnitude"][f"|rarog| {lo}-{hi}"] = {
            "rows": int(mask.sum()),
            "mean_abs_king_cp": float(np.mean(np.abs(king[mask]))),
            "mean_king_toward_sign_cp": float(np.mean((king * sign)[mask])),
            "p90_abs_king_cp": float(np.percentile(np.abs(king[mask]), 90)),
            "mean_abs_danger_cp": float(np.mean(np.abs(parts["danger"][mask]))),
            "mean_abs_shelter_cp": float(np.mean(np.abs(parts["shelter"][mask]))),
            "mean_sf_term_toward_sign_cp": float(np.mean((sf_term * sign)[mask])),
            "share_sf_below_64": float(np.mean(sf[mask] < 64)),
        }

    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2)

    print(f"rows {n}")
    for key, value in report["wire"].items():
        print(f"  wire {key}: {value:.4f}" if isinstance(value, float) else f"  wire {key}: {value}")
    for cname, entry in report["cohorts"].items():
        print(f"\ncohort {cname} (n={entry['rows']})")
        print(f"  {'model':<42} {'mse':>10} {'gain vs rarog %':>16} {'se %':>8}")
        for mname, row in entry["models"].items():
            print(
                f"  {mname:<42} {row['mse']:>10.6f} "
                f"{row['gain_vs_rarog_pct']:>+16.3f} {row['gain_se_pct']:>8.3f}"
            )
    print("\nmagnitude by |rarog| band")
    for bname, entry in report["magnitude"].items():
        print(f"  {bname:<18} " + " ".join(f"{k}={v:.3f}" if isinstance(v, float) else f"{k}={v}" for k, v in entry.items()))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    c = sub.add_parser("collect")
    c.add_argument("--scores", required=True)
    c.add_argument("--sf", required=True)
    c.add_argument("--out", required=True)
    c.add_argument("--limit", type=int, default=0)
    c.add_argument("--chunk", type=int, default=2000)
    c.add_argument("--workers", type=int, default=16)
    c.set_defaults(func=collect)
    a = sub.add_parser("analyse")
    a.add_argument("--dump", required=True)
    a.add_argument("--out", required=True)
    a.set_defaults(func=analyse)
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
