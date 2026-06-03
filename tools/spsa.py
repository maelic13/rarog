"""
SPSA tuning driver for Rarog search/eval parameters.

Requires:
  - Python 3.8+
  - cutechess-cli at D:\\chess\\cutechess-cli\\cutechess-cli.exe
  - The engine must expose tunable parameters as UCI 'spin' options.
    This is Phase 1 work (src/search_options.rs).  Before Phase 1 is done,
    you can run this script with --dry-run to verify the config is valid.

Usage:
  python tools/spsa.py <config.json>
  python tools/spsa.py <config.json> --dry-run
  python tools/spsa.py <config.json> --resume results/spsa_<timestamp>/state.json

Config format: see tools/spsa_configs/phase1_lmr.json for a working example.

Algorithm:
  Standard SPSA with Bernoulli ±1 perturbations, following the Stockfish
  convention for chess engine tuning:
    c_t  = c_i / t^0.101          (perturbation size decreases slowly)
    a_t  = a_i / (A + t)^0.602    (learning rate decreases moderately)
  where A = 0.05 * max_iterations (stability constant).

  Each iteration runs a mini-match of `games_per_iter` games:
    engine(theta + c_t*delta)  vs  engine(theta - c_t*delta)
  The score of the '+' side is used to estimate the gradient.

Output:
  Progress is saved after every `save_interval` iterations to
  tools/results/spsa_<timestamp>/state.json.  Use --resume to continue.
"""

import json
import math
import os
import random
import re
import subprocess
import sys
import time
from copy import deepcopy
from datetime import datetime
from pathlib import Path


CUTECHESS = r"D:\chess\cutechess-cli\cutechess-cli.exe"
DEFAULT_BOOK = r"D:\chess\books\SuperGM_4mvs.pgn"
RESULTS_DIR = Path(__file__).parent / "results"

ALPHA = 0.602   # Learning rate decay exponent (Spall 1998 recommended)
GAMMA = 0.101   # Perturbation size decay exponent


def load_config(path: str) -> dict:
    with open(path) as f:
        cfg = json.load(f)

    # Fill defaults
    cfg.setdefault("cutechess", CUTECHESS)
    cfg.setdefault("book", DEFAULT_BOOK)
    cfg.setdefault("hash", 64)
    cfg.setdefault("concurrency", max(1, os.cpu_count() - 1))
    cfg.setdefault("games_per_iter", 100)
    cfg.setdefault("max_iterations", 3000)
    cfg.setdefault("save_interval", 50)

    for p in cfg["params"]:
        if "value" not in p:
            p["value"] = float(p["default"])
        else:
            p["value"] = float(p["value"])
        p["min"] = float(p["min"])
        p["max"] = float(p["max"])
        # c: perturbation size.  Default: ~5% of range, min 1.
        if "c" not in p:
            p["c"] = max(1.0, (p["max"] - p["min"]) * 0.05)
        p["c"] = float(p["c"])

    return cfg


def run_match(cfg: dict, theta_plus: dict, theta_minus: dict) -> float:
    """
    Run a mini-match of cfg['games_per_iter'] games:
      engine with theta_plus options  vs  engine with theta_minus options.

    Returns the score (0.0–1.0) for the theta_plus side.
    Raises RuntimeError if cutechess-cli fails.
    """
    engine_cmd = cfg["engine"]

    def option_args(params: dict) -> list[str]:
        args = []
        for name, val in params.items():
            args.append(f"option.{name}={int(round(val))}")
        return args

    games = cfg["games_per_iter"]
    rounds = games // 2  # each opening played once per colour
    concurrency = cfg["concurrency"]
    hash_mb = cfg["hash"]
    book = cfg["book"]

    plus_opts  = option_args(theta_plus)
    minus_opts = option_args(theta_minus)

    cmd = [
        cfg["cutechess"],
        "-engine", f"cmd={engine_cmd}", "name=Plus",  "proto=uci",
            f"option.Hash={hash_mb}", "option.Threads=1", *plus_opts,
        "-engine", f"cmd={engine_cmd}", "name=Minus", "proto=uci",
            f"option.Hash={hash_mb}", "option.Threads=1", *minus_opts,
        "-each", "st=0.1",
        "-openings", f"file={book}", "format=pgn", "order=random",
        "-rounds", str(rounds), "-games", "2", "-repeat",
        "-concurrency", str(concurrency),
        "-draw",   "movenumber=40", "movecount=10", "score=5",
        "-resign", "movecount=3", "score=600",
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"cutechess-cli failed:\n{result.stderr[-2000:]}")

    # Parse: "Score of Plus vs Minus: W - L - D [score]"
    m = re.search(
        r"Score of Plus vs Minus:\s+(\d+)\s+-\s+(\d+)\s+-\s+(\d+)",
        result.stdout,
    )
    if not m:
        raise RuntimeError(
            f"Could not parse cutechess-cli output:\n{result.stdout[-2000:]}"
        )
    wins, losses, draws = int(m.group(1)), int(m.group(2)), int(m.group(3))
    total = wins + losses + draws
    if total == 0:
        return 0.5
    return (wins + 0.5 * draws) / total


def spsa_step(cfg: dict, params: list[dict], iteration: int) -> list[dict]:
    """
    Run one SPSA iteration.  Returns updated params list.
    """
    n_params = len(params)
    A = 0.05 * cfg["max_iterations"]

    # Bernoulli ±1 perturbation vector
    delta = [random.choice([-1, 1]) for _ in range(n_params)]

    # Per-parameter c_t and a_t
    c_t = [p["c"] / (iteration ** GAMMA) for p in params]
    a_t = [p["c"] / (A + iteration) ** ALPHA for p in params]

    # Build theta_plus and theta_minus UCI option dicts
    theta_plus  = {}
    theta_minus = {}
    for i, p in enumerate(params):
        v_plus  = max(p["min"], min(p["max"], p["value"] + c_t[i] * delta[i]))
        v_minus = max(p["min"], min(p["max"], p["value"] - c_t[i] * delta[i]))
        theta_plus[p["name"]]  = v_plus
        theta_minus[p["name"]] = v_minus

    # Play the mini-match
    score = run_match(cfg, theta_plus, theta_minus)
    gradient = 2.0 * score - 1.0  # mapped to [-1, +1]

    # SPSA update: theta += a_t * gradient / (2*c_t) * delta
    updated = deepcopy(params)
    for i, p in enumerate(updated):
        update = a_t[i] * gradient / (2.0 * c_t[i]) * delta[i]
        new_val = p["value"] + update
        p["value"] = max(p["min"], min(p["max"], new_val))

    return updated, score, gradient


def save_state(out_dir: Path, cfg: dict, params: list[dict], iteration: int):
    out_dir.mkdir(parents=True, exist_ok=True)
    state = {
        "iteration": iteration,
        "params": [{k: v for k, v in p.items()} for p in params],
    }
    with open(out_dir / "state.json", "w") as f:
        json.dump(state, f, indent=2)

    # Also write a human-readable summary (can be fed directly to UCI engine)
    with open(out_dir / "params_current.txt", "w") as f:
        f.write(f"# SPSA results after iteration {iteration}\n")
        for p in params:
            f.write(f"# {p['name']}: {p['value']:.2f}  (default: {p.get('default', '?')})\n")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="SPSA tuning driver for Rarog")
    parser.add_argument("config", help="Path to JSON config file")
    parser.add_argument("--dry-run", action="store_true",
                        help="Validate config and print the first match command without running it")
    parser.add_argument("--resume", metavar="STATE_JSON",
                        help="Resume from a saved state file")
    args = parser.parse_args()

    cfg = load_config(args.config)
    params = cfg["params"]
    start_iter = 1

    if args.resume:
        with open(args.resume) as f:
            state = json.load(f)
        start_iter = state["iteration"] + 1
        saved = {p["name"]: p["value"] for p in state["params"]}
        for p in params:
            if p["name"] in saved:
                p["value"] = saved[p["name"]]
        print(f"Resuming from iteration {start_iter}")

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    out_dir = RESULTS_DIR / f"spsa_{timestamp}"

    print("\nSPSA Tuning Configuration")
    print(f"  Engine:        {cfg['engine']}")
    print(f"  Book:          {Path(cfg['book']).name}")
    print(f"  Games/iter:    {cfg['games_per_iter']}")
    print(f"  Max iters:     {cfg['max_iterations']}")
    print(f"  Concurrency:   {cfg['concurrency']}")
    print(f"  Parameters:    {len(params)}")
    for p in params:
        print(f"    {p['name']:30s} start={p['value']:8.2f}  range=[{p['min']:.0f}, {p['max']:.0f}]  c={p['c']:.1f}")
    print()

    if args.dry_run:
        print("DRY RUN — printing first match command and exiting.")
        A = 0.05 * cfg["max_iterations"]
        delta = [1] * len(params)
        c_t = [p["c"] / (1.0 ** GAMMA) for p in params]
        theta_plus  = {p["name"]: p["value"] + c_t[i] for i, p in enumerate(params)}
        theta_minus = {p["name"]: p["value"] - c_t[i] for i, p in enumerate(params)}
        print("theta_plus options:")
        for name, val in theta_plus.items():
            print(f"  option.{name}={int(round(val))}")
        print("theta_minus options:")
        for name, val in theta_minus.items():
            print(f"  option.{name}={int(round(val))}")
        return

    print(f"Results will be saved to: {out_dir}")
    print()

    for iteration in range(start_iter, cfg["max_iterations"] + 1):
        t0 = time.monotonic()
        try:
            params, score, gradient = spsa_step(cfg, params, iteration)
        except RuntimeError as e:
            print(f"[iter {iteration}] ERROR: {e}")
            continue

        elapsed = time.monotonic() - t0
        param_summary = "  ".join(f"{p['name']}={p['value']:.1f}" for p in params)
        print(f"[{iteration:5d}/{cfg['max_iterations']}]  score={score:.3f}  grad={gradient:+.3f}  "
              f"({elapsed:.1f}s)  {param_summary}")

        if iteration % cfg["save_interval"] == 0:
            save_state(out_dir, cfg, params, iteration)
            print(f"  -> saved to {out_dir / 'state.json'}")

    save_state(out_dir, cfg, params, cfg["max_iterations"])
    print(f"\nTuning complete. Final state: {out_dir / 'state.json'}")


if __name__ == "__main__":
    main()
