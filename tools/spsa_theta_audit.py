"""List the fitted constants a tune's analysis must read before its bake.

    python tools/spsa_theta_audit.py <run dir> <config.json> --at <K> [--movement-config <config.json>]
                                     [--out audit.json] [--draws 2000]

A rule 7c tune (PLAN rule 7c) is read with `--at K`, the checkpoint where it
stopped: the centres the runner recorded up to K, every move checked against
its regenerated signs by tools/spsa_read.py, and theta the rounded centres at
K, as the reader writes them to theta-<K>.txt. Without `--at` it reads a
finished block run's `result.json`, as the tunes before rule 7c left them.
Either way it reads the registered config the run used (seeds and steps). Every coordinate in one of three lists gets a line in
the tune's written analysis before theta is baked, saying what it shows about
the mechanism: a term tuned toward off, two terms moving to cancel, a margin
compensating for a mis-scaled input, a rail hiding a wanted value outside the
range. The lists never change theta (PROCESS, *SPSA go/no-go procedure*, step
9); a finding becomes a registration of its own.

- **Full-step movers:** net travel of at least one full step. The unit is the
  config's step, or `--movement-config`'s when the run's steps were scaled
  (the b43 steps stay the unit, PLAN rule 7c).
- **Rails:** the tuned value at a range end, or within one unit of it.
- **Reversals:** the centre first went at least one unit against the side
  it finished on, or overshot its final value by at least one unit and came
  back. A coordinate that drifts back part of a step is not one.

Also printed, as diagnostics: each coordinate's z (net travel over the root
sum of its squared steps; mean 0, variance 1 with no gradient) and the run's
mean z^2 against the null of every step's sign re-drawn (seed 20261021).

Wires (asserted): the knobs are the config's, in its order, with its seeds;
for a finished block run, the run completed every iteration, c0 is the
registered step, every step equals the schedule's a_k / c_k *
|difference_k| on at least 99.8% of entries (the rest are clamps) and the
last centres round to the tuned values; at a checkpoint, K is completed, c0
is one to two registered steps (rule 7c's graded perturbation) and the moves
pass the reader's check. A full step is the registered (b43) step.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import pathlib
import sys

import numpy as np

TOOLS = pathlib.Path(__file__).resolve().parent
DEFAULT_CLI = TOOLS / "bin" / ("colosseum-cli.exe" if sys.platform == "win32" else "colosseum-cli")


def spsa_read():
    """tools/spsa_read.py, loaded by path: the reader owns the run's loader."""
    spec = importlib.util.spec_from_file_location("spsa_read", TOOLS / "spsa_read.py")
    module = importlib.util.module_from_spec(spec)
    assert spec.loader
    sys.modules["spsa_read"] = module  # dataclasses resolve their module by name
    spec.loader.exec_module(module)
    return module


def load_checkpoint(run_dir: pathlib.Path, config: dict, at: int, cli: pathlib.Path | None = DEFAULT_CLI,
                    history_json: pathlib.Path | None = None) -> dict:
    """A rule 7c tune at checkpoint `at`: the path to it and theta, its rounded centres."""
    sr = spsa_read()
    run = sr.load_run(run_dir, cli=cli, history_json=history_json)
    assert 0 < at <= run.completed, f"--at {at}: the run has {run.completed} completed iterations"
    names = run.names
    assert names == list(config), "the run's knobs are not the config's, in order"
    start = np.array(run.centres[0], dtype=float)
    assert list(start) == [float(config[n]["value"]) for n in names], "seeds differ from the config"
    c0 = np.array(run.c0, dtype=float)
    step = np.array([float(config[n]["step"]) for n in names])
    # c0 is rebuilt from a c_end the tune file prints to four decimals, rounded down.
    assert np.all(c0 >= step * (1 - 1e-3)) and np.all(c0 <= 2 * step * (1 + 1e-6)), \
        "c0 is not one to two registered steps"
    path = np.array(run.centres[: at + 1], dtype=float)
    clamped = sum(run.arm_clamped(k, i) for k in range(at) for i in range(run.n))
    return {
        "names": names, "path": path, "steps": np.diff(path, axis=0), "c0": c0, "horizon": at,
        "tuned": np.array([float(sr.round_half_away(x)) for x in path[-1]]),
        "low": np.array(run.minimum, dtype=float), "high": np.array(run.maximum, dtype=float),
        "clamped_arms": int(clamped), "step": step,
    }


def load(result: dict, config: dict) -> dict:
    tuned = result["tuned_result"]
    iterations = result["driver"]["completed_iterations"]
    horizon = int(tuned["settings"]["iterations"])
    assert result["driver"]["status"] == "completed", "the run did not complete"
    assert len(iterations) == horizon == int(tuned["completed_iterations"]), "the journal is incomplete"
    assert [it["iteration"] for it in iterations] == list(range(horizon)), "the journal is out of order"
    params = tuned["parameters"]
    names = [p["name"] for p in params]
    knobs = result["schedule"]["knobs"]
    assert [k["name"] for k in knobs] == names
    assert names == list(config), "the run's knobs are not the config's, in order"
    start = np.array([float(p["original"]) for p in params])
    assert list(start) == [float(config[n]["value"]) for n in names], "seeds differ from the config"
    c0 = np.array([float(k["c0"]) for k in knobs])
    # c0 is rebuilt from a c_end the tune file prints to four decimals.
    assert np.allclose(c0, [config[n]["step"] for n in names], rtol=1e-3), "c0 differs from the registered steps"
    centres = np.array([it["centers_after"] for it in iterations], dtype=float)
    diffs = np.array([float(it["score"]["difference"]) for it in iterations])
    path = np.vstack([start, centres])
    steps = np.diff(path, axis=0)
    sched = result["schedule"]["schedule"]
    alpha, gamma, stability = sched["alpha"], sched["gamma"], sched["stability_constant"]
    k = np.arange(1, horizon + 1, dtype=float)
    a0 = np.array([float(kn["a0"]) for kn in knobs])
    expected = (a0[None, :] / (k[:, None] + stability) ** alpha) / (c0[None, :] / k[:, None] ** gamma)
    expected = expected * np.abs(diffs)[:, None]
    off = np.abs(np.abs(steps) - expected) > 1e-6 * np.maximum(1.0, expected)
    assert off.mean() < 0.002, f"{off.sum()} steps differ from the schedule"
    tuned_values = np.array([float(p["tuned"]) for p in params])
    rounded = np.sign(path[-1]) * np.floor(np.abs(path[-1]) + 0.5)
    assert np.array_equal(rounded, tuned_values), "last centres do not round to the tuned values"
    return {
        "names": names, "path": path, "steps": steps, "c0": c0, "horizon": horizon,
        "tuned": tuned_values, "low": np.array([float(p["min"]) for p in params]),
        "high": np.array([float(p["max"]) for p in params]), "clamped_steps": int(off.sum()),
    }


def classify(run: dict, unit: np.ndarray) -> dict:
    """The three lists, per coordinate, in units of `unit`."""
    names, path, tuned = run["names"], run["path"], run["tuned"]
    travel = (path - path[0]) / unit
    net = travel[-1]
    movers = [n for n, m in zip(names, net) if abs(m) >= 1]
    rails = []
    for i, n in enumerate(names):
        if tuned[i] <= run["low"][i] + unit[i] or tuned[i] >= run["high"][i] - unit[i]:
            end = "min" if tuned[i] - run["low"][i] <= run["high"][i] - tuned[i] else "max"
            edge = run["low"][i] if end == "min" else run["high"][i]
            rails.append({"name": n, "end": end, "tuned": float(tuned[i]), "edge": float(edge),
                          "at_edge": bool(tuned[i] == edge)})
    reversals = []
    for i, n in enumerate(names):
        high, low = float(travel[:, i].max()), float(travel[:, i].min())
        if net[i] >= 0:
            turned = low <= -1 or high >= net[i] + 1
        else:
            turned = high >= 1 or low <= net[i] - 1
        if turned:
            reversals.append({"name": n, "net": float(net[i]), "peak": high, "trough": low})
    return {"steps_moved": dict(zip(names, map(float, net))), "full_step_movers": movers,
            "rails": rails, "reversals": reversals}


def z_scores(steps: np.ndarray, draws: int) -> tuple[np.ndarray, float, np.ndarray]:
    norm = np.sqrt((steps ** 2).sum(axis=0))
    safe = np.where(norm > 0, norm, 1.0)
    z = steps.sum(axis=0) / safe
    rng = np.random.default_rng(20261021)
    mag = np.abs(steps)
    null = np.empty(draws)
    for d in range(draws):
        signs = rng.integers(0, 2, size=steps.shape, dtype=np.int8) * 2 - 1
        null[d] = (((mag * signs).sum(axis=0) / safe) ** 2).mean()
    return z, float((z ** 2).mean()), null


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("run_dir", type=pathlib.Path)
    ap.add_argument("config", type=pathlib.Path)
    ap.add_argument("--movement-config", type=pathlib.Path, default=None,
                    help="a config whose steps are the unit of 'a full step' (the b43 steps) when the run's own steps were scaled")
    ap.add_argument("--out", type=pathlib.Path, default=None)
    ap.add_argument("--draws", type=int, default=2000)
    ap.add_argument("--at", type=int, default=None, help="a rule 7c tune's checkpoint, where it stopped")
    ap.add_argument("--cli", type=pathlib.Path, default=DEFAULT_CLI, help="the pinned runner, for --at")
    args = ap.parse_args()
    config = json.loads(args.config.read_text(encoding="utf-8"))
    if args.at is not None:
        run = load_checkpoint(args.run_dir, config, args.at, cli=args.cli)
        unit = run["step"]
    else:
        result = json.loads((args.run_dir / "result.json").read_text(encoding="utf-8"))
        run = load(result, config)
        unit = run["c0"]
    names = run["names"]
    if args.movement_config is not None:
        movement = json.loads(args.movement_config.read_text(encoding="utf-8"))
        missing = [n for n in names if n not in movement]
        assert not missing, f"movement config lacks {missing}"
        unit = np.array([float(movement[n]["step"]) for n in names])
    lists = classify(run, unit)
    z, stat, null = z_scores(run["steps"], args.draws)
    p95 = float(np.quantile(null, 0.95))
    report = {
        "run_dir": str(args.run_dir), "coordinates": len(names), "iterations": run["horizon"],
        "clamped": run.get("clamped_steps", run.get("clamped_arms")), **lists,
        "mean_z2": stat, "null_mean": float(null.mean()), "null_p95": p95,
        "p": float((np.sum(null >= stat) + 1) / (args.draws + 1)),
        "abs_z_ge_2": [n for n, v in zip(names, z) if abs(v) >= 2],
        "z": dict(zip(names, map(float, z))),
    }
    if args.out is not None:
        args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    clamps = (f"{run['clamped_steps']} clamped steps" if "clamped_steps" in run
              else f"{run['clamped_arms']} clamped arms; theta at iteration {run['horizon']}")
    print(f"{len(names)} coordinates x {run['horizon']} iterations ({clamps})")
    print(f"full-step movers ({len(lists['full_step_movers'])}): "
          + (", ".join(f"{n} {lists['steps_moved'][n]:+.2f}" for n in lists["full_step_movers"]) or "none"))
    print(f"rails ({len(lists['rails'])}): "
          + (", ".join(f"{r['name']} {r['tuned']:g} {'at' if r['at_edge'] else 'near'} {r['end']} {r['edge']:g}"
                       for r in lists["rails"]) or "none"))
    print(f"reversals ({len(lists['reversals'])}): "
          + (", ".join(f"{r['name']} net {r['net']:+.2f} (range {r['trough']:+.2f}..{r['peak']:+.2f})"
                       for r in lists["reversals"]) or "none"))
    print(f"mean z^2 {stat:.3f}  null mean {null.mean():.3f}  null 95% {p95:.3f}  p {report['p']:.4f}")
    print(f"|z| >= 2 ({len(report['abs_z_ge_2'])}): {', '.join(report['abs_z_ge_2']) or 'none'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
