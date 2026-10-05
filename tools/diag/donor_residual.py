#!/usr/bin/env python3
"""Donor-direction residual screen for the evaluation programme.

Question: on positions with known game outcomes, what does the classical
Stockfish evaluation predict about the outcome that Rarog's evaluation does
not, and in which of Stockfish's term families does that information sit?

A fitted evaluation's own features are uncorrelated with its outcome residual
(that is what the fit's first-order condition says), so a cohort residual on
Rarog's own trace cannot show a missing signal. A donor's term family is a
direction outside that span. If adding it to Rarog's score lowers held-out
outcome loss, the family carries conditioning Rarog's surface lacks; if it
does not, no coefficient or structure borrowed from that family is supported
at this layer. The layer is static outcome prediction: it ranks questions and
refutes; it never accepts a change and it is not Elo.

Two steps, both deterministic:

  collect   run three evaluators over a `fen;label` corpus and store one row
            per position: Rarog's HCE and the oracle's classical Stockfish
            total (both from the oracle package's `eval`, one binary), and the
            Stockfish 11 per-term trace.
  analyse   two-fold held-out logistic models on those rows, reported as mean
            squared error against the label with a per-row standard error.

Example:

  python tools/diag/donor_residual.py collect \
      --csv tools/texel/data/hce-v3-tb/validation.csv \
      --oracle tools/test_engines/oracle-hybrid-2.4.0/rarog-stockfish-hce-hybrid.exe \
      --sf11 D:/chess/engines/stockfish/stockfish-11-x64-bmi2.exe \
      --out tools/results/donor-residual-<date>
  python tools/diag/donor_residual.py analyse \
      --scores tools/results/donor-residual-<date>/scores.csv \
      --out tools/results/donor-residual-<date>/report.json
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import os
import re
import subprocess
import sys

# Stockfish 11's trace rows, in print order. "Total" is the table's own sum.
SF11_TERMS = [
    "Material",
    "Imbalance",
    "Pawns",
    "Knights",
    "Bishops",
    "Rooks",
    "Queens",
    "Mobility",
    "King safety",
    "Threats",
    "Passed",
    "Space",
    "Initiative",
]

# The families the screen reports. Material carries Stockfish's piece-square
# tables as well (its trace prints the incremental psq score there).
FAMILIES = {
    "material": ["Material", "Imbalance"],
    "pawns": ["Pawns"],
    "pieces": ["Knights", "Bishops", "Rooks", "Queens"],
    "mobility": ["Mobility"],
    "king": ["King safety"],
    "threats": ["Threats"],
    "passed": ["Passed"],
    "space": ["Space"],
    "initiative": ["Initiative"],
}

# Stockfish 11 phase constants (types.h): midgame piece values and the limits
# between which the game phase is interpolated.
SF11_NPM = {"n": 781, "b": 825, "r": 1276, "q": 2538}
SF11_MIDGAME_LIMIT = 15258
SF11_ENDGAME_LIMIT = 3915

# Totals are clipped before they enter a model. Stockfish's specialised
# endgame functions return known-win values near 50 pawns where Rarog scores
# the same position at 4 to 7; unclipped, a few hundred such rows set the
# slope for every other row.
CLIP_CP = 2000.0

_FINAL_RE = re.compile(r"[Ff]inal evaluation:\s*([+-]?\d+(?:\.\d+)?)\s*\(white side\)")
_TOTAL_RE = re.compile(r"Total evaluation:\s*([+-]?\d+(?:\.\d+)?)\s*\(white side\)")
_ROW_RE = re.compile(
    r"^\s*([A-Za-z][A-Za-z ]*?)\s*\|[^|]*\|[^|]*\|\s*([+-]?\d+\.\d+)\s+([+-]?\d+\.\d+)\s*$"
)


def parse_oracle_total(block: str) -> float | None:
    """White-side total in pawns from the oracle package's `eval`, or None."""
    m = _FINAL_RE.search(block)
    return float(m.group(1)) if m else None


def parse_sf11_trace(block: str) -> tuple[dict[str, tuple[float, float]], float | None]:
    """Per-term (mg, eg) totals and the white-side total, both in pawns.

    A position in check prints no table and `none (in check)`; the caller
    drops such a row rather than scoring it as zero.
    """
    terms: dict[str, tuple[float, float]] = {}
    for line in block.splitlines():
        m = _ROW_RE.match(line)
        if m:
            terms[m.group(1)] = (float(m.group(2)), float(m.group(3)))
    m = _TOTAL_RE.search(block)
    return terms, (float(m.group(1)) if m else None)


def sf11_phase(fen: str) -> int:
    """Stockfish 11's game phase for a FEN, 0 (endgame) to 128 (midgame)."""
    npm = sum(SF11_NPM.get(c.lower(), 0) for c in fen.split()[0])
    npm = max(SF11_ENDGAME_LIMIT, min(SF11_MIDGAME_LIMIT, npm))
    return ((npm - SF11_ENDGAME_LIMIT) * 128) // (SF11_MIDGAME_LIMIT - SF11_ENDGAME_LIMIT)


def men_count(fen: str) -> int:
    """Pieces and pawns on the board, both kings included."""
    return sum(1 for c in fen.split()[0] if c.isalpha())


def has_queen(fen: str) -> bool:
    board = fen.split()[0]
    return "q" in board or "Q" in board


def run_eval(exe: str, setup: list[str], fens: list[str]) -> list[str]:
    """One engine process over `fens`; returns one output block per position.

    `isready` after every `eval` delimits the blocks, so a position that
    prints nothing still yields an (empty) block and the count is checked.
    """
    lines = list(setup)
    lines.append("isready")
    for fen in fens:
        lines.append(f"position fen {fen}")
        lines.append("eval")
        lines.append("isready")
    lines.append("quit")
    # The oracle loads its evaluation library from its own directory, so the
    # process starts there; the path must therefore be absolute.
    exe = os.path.abspath(exe)
    proc = subprocess.run(
        [exe],
        input="\n".join(lines) + "\n",
        capture_output=True,
        text=True,
        cwd=os.path.dirname(exe),
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"{exe} exited {proc.returncode}: {proc.stderr[-400:]}")
    blocks: list[str] = []
    current: list[str] = []
    for line in proc.stdout.splitlines():
        if line.strip() == "readyok":
            blocks.append("\n".join(current))
            current = []
        else:
            current.append(line)
    blocks = blocks[1:]  # the block before the first position is the banner
    if len(blocks) != len(fens):
        raise RuntimeError(f"{exe}: {len(blocks)} blocks for {len(fens)} positions")
    return blocks


def read_corpus(path: str, limit: int) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            fen, sep, label = line.rpartition(";")
            if not sep or label not in ("0", "0.5", "1"):
                raise ValueError(f"not a `fen;label` WDL row: {line[:80]}")
            rows.append((fen, label))
            if limit and len(rows) >= limit:
                break
    if not rows:
        raise ValueError(f"no rows in {path}")
    return rows


def sha256_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def score_chunk(args: tuple[str, str, list[tuple[int, str, str]]]) -> list[str]:
    oracle, sf11, chunk = args
    fens = [fen for _, fen, _ in chunk]
    rarog = run_eval(oracle, ["setoption name Use Rarog HCE value true"], fens)
    control = run_eval(oracle, ["setoption name Use Rarog HCE value false"], fens)
    donor = run_eval(sf11, [], fens)
    out: list[str] = []
    for (idx, fen, label), b_r, b_s, b_d in zip(chunk, rarog, control, donor):
        r = parse_oracle_total(b_r)
        s = parse_oracle_total(b_s)
        terms, total = parse_sf11_trace(b_d)
        if r is None or s is None or total is None or any(t not in terms for t in SF11_TERMS):
            out.append(f"{idx};{label};invalid;{fen}")
            continue
        cells = [str(idx), label, f"{r:.2f}", f"{s:.2f}", f"{total:.2f}"]
        for term in SF11_TERMS:
            cells.append(f"{terms[term][0]:.2f}")
            cells.append(f"{terms[term][1]:.2f}")
        cells.append(fen)
        out.append(";".join(cells))
    return out


def header() -> str:
    cells = ["idx", "label", "rarog", "sf_control", "sf11_total"]
    for term in SF11_TERMS:
        key = term.lower().replace(" ", "_")
        cells += [f"{key}_mg", f"{key}_eg"]
    cells.append("fen")
    return ";".join(cells)


def collect(args: argparse.Namespace) -> int:
    rows = read_corpus(args.csv, args.limit)
    indexed = [(i, fen, label) for i, (fen, label) in enumerate(rows)]
    size = max(1, -(-len(indexed) // (args.workers * 4)))
    chunks = [indexed[i : i + size] for i in range(0, len(indexed), size)]
    os.makedirs(args.out, exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
        parts = list(pool.map(score_chunk, [(args.oracle, args.sf11, c) for c in chunks]))
    lines = [line for part in parts for line in part]
    invalid = sum(1 for line in lines if line.split(";", 3)[2] == "invalid")
    scores = os.path.join(args.out, "scores.csv")
    with open(scores, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(header() + "\n")
        for line in lines:
            if line.split(";", 3)[2] != "invalid":
                handle.write(line + "\n")
    oracle_dir = os.path.dirname(os.path.abspath(args.oracle))
    manifest = {
        "corpus": os.path.abspath(args.csv),
        "corpus_sha256": sha256_file(args.csv),
        "limit": args.limit,
        "rows_read": len(rows),
        "rows_scored": len(lines) - invalid,
        "rows_invalid": invalid,
        "oracle": os.path.abspath(args.oracle),
        "oracle_sha256": sha256_file(args.oracle),
        "oracle_dll_sha256": sha256_file(os.path.join(oracle_dir, "rarog_hce.dll")),
        "sf11": os.path.abspath(args.sf11),
        "sf11_sha256": sha256_file(args.sf11),
        "scores_sha256": sha256_file(scores),
    }
    with open(os.path.join(args.out, "collect-manifest.json"), "w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2)
    print(json.dumps(manifest, indent=2))
    if len(lines) != len(rows):
        print("ERROR: scored rows do not match corpus rows", file=sys.stderr)
        return 1
    return 0


# ---------------------------------------------------------------------------
# Analysis
# ---------------------------------------------------------------------------


def fit_logistic(x, y, iterations: int = 40):
    """Logistic regression with an intercept by Newton steps (fractional y)."""
    import numpy as np

    n, k = x.shape
    design = np.hstack([np.ones((n, 1)), x])
    w = np.zeros(k + 1)
    for _ in range(iterations):
        p = 1.0 / (1.0 + np.exp(-(design @ w)))
        grad = design.T @ (y - p)
        hess = (design * (p * (1.0 - p))[:, None]).T @ design + 1e-9 * np.eye(k + 1)
        step = np.linalg.solve(hess, grad)
        w += step
        if float(np.max(np.abs(step))) < 1e-10:
            break
    return w


def predict(w, x):
    import numpy as np

    return 1.0 / (1.0 + np.exp(-(w[0] + x @ w[1:])))


def held_out_predictions(x, y):
    """Two-fold: each row is predicted by the model fitted on the other fold.

    Columns are standardised on the fitting fold only.
    """
    import numpy as np

    n = x.shape[0]
    fold = np.arange(n) % 2
    out = np.empty(n)
    for held in (0, 1):
        fit = fold != held
        mean = x[fit].mean(axis=0)
        std = x[fit].std(axis=0)
        std[std == 0] = 1.0
        w = fit_logistic((x[fit] - mean) / std, y[fit])
        out[~fit] = predict(w, (x[~fit] - mean) / std)
    return out


def load_scores(path: str):
    import numpy as np

    with open(path, encoding="utf-8") as handle:
        names = handle.readline().rstrip("\n").split(";")
        raw = [line.rstrip("\n").split(";") for line in handle]
    col = {name: i for i, name in enumerate(names)}
    numeric = np.array([[float(v) for v in row[: len(names) - 1]] for row in raw])
    fens = [row[-1] for row in raw]
    return col, numeric, fens


def analyse(args: argparse.Namespace) -> int:
    import numpy as np

    col, data, fens = load_scores(args.scores)
    n = data.shape[0]
    y = data[:, col["label"]]
    rarog = np.clip(data[:, col["rarog"]] * 100.0, -CLIP_CP, CLIP_CP)
    control = np.clip(data[:, col["sf_control"]] * 100.0, -CLIP_CP, CLIP_CP)
    phase = np.array([sf11_phase(f) for f in fens], dtype=float)
    queens = np.array([has_queen(f) for f in fens])

    def tapered(term: str):
        key = term.lower().replace(" ", "_")
        mg = data[:, col[f"{key}_mg"]]
        eg = data[:, col[f"{key}_eg"]]
        return (mg * phase + eg * (128.0 - phase)) / 128.0 * 100.0

    family = {name: sum(tapered(t) for t in terms) for name, terms in FAMILIES.items()}
    family_sum = sum(family.values())
    sf11_total = np.clip(data[:, col["sf11_total"]] * 100.0, -CLIP_CP, CLIP_CP)

    def rank_corr(a, b) -> float:
        ra = np.argsort(np.argsort(a)).astype(float)
        rb = np.argsort(np.argsort(b)).astype(float)
        return float(np.corrcoef(ra, rb)[0, 1])

    rng = np.random.default_rng(20261005)
    models: dict[str, np.ndarray] = {
        "rarog": rarog[:, None],
        "stockfish": control[:, None],
        "rarog+stockfish": np.column_stack([rarog, control]),
        "rarog+phase_slope": np.column_stack([rarog, rarog * (phase / 128.0 - 0.5)]),
        "rarog+all_families": np.column_stack([rarog] + [family[f] for f in FAMILIES]),
        "rarog+shuffled_king": np.column_stack([rarog, rng.permutation(family["king"])]),
    }
    for name in FAMILIES:
        models[f"rarog+{name}"] = np.column_stack([rarog, family[name]])

    cohorts = {
        "all": np.ones(n, dtype=bool),
        "phase>=96": phase >= 96,
        "phase 32-95": (phase >= 32) & (phase < 96),
        "phase<32": phase < 32,
        "queens": queens,
        "no queens": ~queens,
    }
    if args.within:
        # A monotone recalibration of Rarog's own score: what it gains is
        # magnitude calibration, not information from the donor.
        models["rarog+magnitude"] = np.column_stack([rarog, rarog * np.abs(rarog) / 1000.0])
        men = np.array([men_count(f) for f in fens])
        cohorts["men<=6"] = men <= 6
        cohorts["men>=7"] = men >= 7
        cohorts["|rarog|<=500"] = np.abs(rarog) <= 500.0
        cohorts["|rarog|>500"] = np.abs(rarog) > 500.0

    def squared_errors(mask):
        return {
            name: (y[mask] - held_out_predictions(x[mask], y[mask])) ** 2
            for name, x in models.items()
        }

    # Default: one fit over every row, cohorts read from it. `--within` refits
    # inside each cohort, so a cohort's weights are not set by the others.
    global_errors = None if args.within else squared_errors(np.ones(n, dtype=bool))

    report: dict[str, object] = {
        "scores": os.path.abspath(args.scores),
        "scores_sha256": sha256_file(args.scores),
        "rows": int(n),
        "fit": "within each cohort" if args.within else "all rows",
        "wire": {
            "rank_corr_control_vs_sf11_total": rank_corr(control, sf11_total),
            "rank_corr_family_sum_vs_sf11_total": rank_corr(family_sum, sf11_total),
            "rank_corr_rarog_vs_control": rank_corr(rarog, control),
            "rows_clipped_control": int(np.sum(np.abs(data[:, col["sf_control"]]) * 100.0 > CLIP_CP)),
            "rows_clipped_rarog": int(np.sum(np.abs(data[:, col["rarog"]]) * 100.0 > CLIP_CP)),
            "mean_abs_rarog_cp": float(np.mean(np.abs(rarog))),
            "mean_abs_control_cp": float(np.mean(np.abs(control))),
        },
        "cohorts": {},
    }
    for cname, mask in cohorts.items():
        count = int(mask.sum())
        if global_errors is None:
            errors = squared_errors(mask)
        else:
            errors = {name: err[mask] for name, err in global_errors.items()}
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

    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2)

    print(f"rows {n}")
    for key, value in report["wire"].items():
        print(f"  wire {key}: {value:.4f}" if isinstance(value, float) else f"  wire {key}: {value}")
    for cname, entry in report["cohorts"].items():
        print(f"\ncohort {cname} (n={entry['rows']})")
        print(f"  {'model':<24} {'mse':>10} {'gain vs rarog %':>16} {'se %':>8}")
        for mname, row in entry["models"].items():
            print(
                f"  {mname:<24} {row['mse']:>10.6f} "
                f"{row['gain_vs_rarog_pct']:>+16.3f} {row['gain_se_pct']:>8.3f}"
            )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    c = sub.add_parser("collect")
    c.add_argument("--csv", required=True)
    c.add_argument("--oracle", required=True)
    c.add_argument("--sf11", required=True)
    c.add_argument("--out", required=True)
    c.add_argument("--workers", type=int, default=8)
    c.add_argument("--limit", type=int, default=0)
    c.set_defaults(func=collect)
    a = sub.add_parser("analyse")
    a.add_argument("--scores", required=True)
    a.add_argument("--out", required=True)
    a.add_argument(
        "--within",
        action="store_true",
        help="refit inside each cohort and add the men-count and magnitude cohorts",
    )
    a.set_defaults(func=analyse)
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
