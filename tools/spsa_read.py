#!/usr/bin/env python3
"""Read a Colosseum SPSA run by its gradient test, at a checkpoint or live.

    python tools/spsa_read.py <run-dir> [--at K] [--record] [--json]

A tune is one run read at checkpoints (PLAN rule 7c). At a checkpoint K the
last --window iterations are read:

  z       for each coordinate, z = sum(delta * D) / sqrt(sum D^2) over the
          window, where delta is the sign the tuner drew for it and D the plus
          side's wins less its losses. With no gradient z has mean 0 and
          variance 1 exactly, because the tuner drew the signs.
  live    some |z| passes the familywise 5% line for n coordinates, or
          sum(z^2) - n is at least 2 * sqrt(2n). Otherwise quiet.
  floor   what quiet rules out: no coordinate is worth more than the line times
          a coordinate's standard error across one perturbation, and the
          forecast's upper reading for the next segment is under 2 SD.
  rails   a centre within one current perturbation of a bound, its z
          pointing outward past the familywise line: the window would stay
          live for a coordinate that cannot move. An outward z of
          RAIL_CANDIDATE_Z or more under the line is a candidate, reported
          and deciding nothing.
  gain    the tune's own read of what it has gained since iteration 0: the
          sum of gradient times move, cross-fitted (even iterations' gradient
          against odd iterations' move and the reverse) within segments of
          500 iterations, with a sign-randomised null SD.
  forecast  the next segment's gain at most, from the window's sum(z^2) - n
          held constant, with SD from the null. Frozen in the record and
          scored against the gain read once the segment has been played.

The verdict at a registered checkpoint (iteration --first, then every
--every, and the ceiling) is CONTINUE, STOP_QUIET, STOP_CEILING or STOP_RAIL.
Anywhere else the read is INTERIM and decides nothing. A run with any engine,
time or infrastructure fault reads FAULT at its checkpoint and stops there for
the maintainer; see --accept-faults. A stop writes theta, the rounded
centres at K, as theta-<K>.txt (setoption lines) and theta-<K>.toml; so does
--write-theta at a live checkpoint, when the maintainer stops the tune there.
A rail stop writes none: the range is registered again.

Inputs, all read without changing the run:
  - the centres after every completed iteration, from the pinned runner's
    `spsa history --json` (safe while the tune runs);
  - every iteration's score, rebuilt from games.jsonl (engine a is the plus
    side), only for iterations the centres cover;
  - spsa-schedule.json: each coordinate's c0 and bounds, the gain schedule,
    and the perturbation stream's master seed;
  - checkpoint.json: the run's fault counts.
The signs are the runner's own, regenerated from the master seed (ChaCha12
keyed by SHA-256 of "colosseum-rng-v1\\0", the seed and the stream name; one
little-endian u64 a draw, -1 when even; iteration-major, knob order), so a
centre held at a bound keeps its true sign. Before reading, every move is
checked against them and the schedule in closed form, move = r_k * c_k * D *
delta, a clamped one within it, so a misread journal or seed fails loudly.

--record appends the read to <run>/spsa-read.jsonl (once per checkpoint) and
rewrites <run>/spsa-read.txt; a read between checkpoints is printed and not
recorded. A forecast is scored against the gain read over its segment, within
twice the two reads' combined SD.

--accept-faults "<reason>" is the maintainer's decision after a FAULT read has
been analysed: it writes <run>/faults-accepted.json with the fault counts that
read recorded and the reason. The next read at that checkpoint then gives its
real verdict, recorded after the FAULT one, and the tune resumes or stops by
it. A fault beyond the accepted counts reads FAULT again.

The exit status is 0 for any read, 2 for an error.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import math
import random
import statistics
import struct
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from statistics import NormalDist

ROOT = Path(__file__).resolve().parents[1]
# The pinned runner, staged by tools/setup_tools.ps1; its name carries .exe on Windows only.
DEFAULT_CLI = ROOT / "tools" / "bin" / ("colosseum-cli.exe" if sys.platform == "win32" else "colosseum-cli")
GAIN_SEGMENT = 500
NULL_REPS = 40
NULL_SEED = 1
FORECAST_SPAN = 2000
RAIL_CANDIDATE_Z = 2.0
MOVE_TOLERANCE = 1e-6
SCHEMA = "rarog-spsa-read-v1"
ACCEPT_FILE = "faults-accepted.json"
ACCEPT_SCHEMA = "rarog-spsa-faults-accepted-v1"


class ReadError(Exception):
    pass


# --------------------------------------------------------------------------
# the runner's perturbation signs

RNG = {
    "algorithm": "chacha12-64-bit-counter-zero-stream-v1",
    "derivation": "sha256-colosseum-rng-v1",
    "u64_sampling": "little-endian-u64-rejection-v1",
    "sampler": "rademacher-minus-one-plus-one-v1",
    "stream_name": "spsa-perturbations",
    "draw_order": "iteration-major-knob-order",
}
_MASK = 0xFFFFFFFF


def _rotl(value: int, shift: int) -> int:
    return ((value << shift) | (value >> (32 - shift))) & _MASK


def _chacha12_block(key: tuple[int, ...], counter: int) -> bytes:
    state = [0x61707865, 0x3320646E, 0x79622D32, 0x6B206574, *key,
             counter & _MASK, (counter >> 32) & _MASK, 0, 0]
    x = list(state)

    def quarter(a: int, b: int, c: int, d: int) -> None:
        x[a] = (x[a] + x[b]) & _MASK
        x[d] = _rotl(x[d] ^ x[a], 16)
        x[c] = (x[c] + x[d]) & _MASK
        x[b] = _rotl(x[b] ^ x[c], 12)
        x[a] = (x[a] + x[b]) & _MASK
        x[d] = _rotl(x[d] ^ x[a], 8)
        x[c] = (x[c] + x[d]) & _MASK
        x[b] = _rotl(x[b] ^ x[c], 7)

    for _ in range(6):
        quarter(0, 4, 8, 12), quarter(1, 5, 9, 13), quarter(2, 6, 10, 14), quarter(3, 7, 11, 15)
        quarter(0, 5, 10, 15), quarter(1, 6, 11, 12), quarter(2, 7, 8, 13), quarter(3, 4, 9, 14)
    return struct.pack("<16I", *[(a + b) & _MASK for a, b in zip(x, state)])


def stream_seed(master_seed: int, stream_name: str) -> bytes:
    return hashlib.sha256(b"colosseum-rng-v1\x00" + struct.pack("<Q", master_seed)
                          + stream_name.encode("ascii")).digest()


def perturbation_signs(master_seed: int, iterations: int, knobs: int) -> list[list[int]]:
    """delta[k][i], the sign the runner drew for knob i at iteration k."""
    key = struct.unpack("<8I", stream_seed(master_seed, RNG["stream_name"]))
    need = iterations * knobs
    draws: list[int] = []
    counter = 0
    while len(draws) < need:
        draws.extend(struct.unpack("<8Q", _chacha12_block(key, counter)))
        counter += 1
    return [[1 if draws[k * knobs + i] & 1 else -1 for i in range(knobs)] for k in range(iterations)]


# --------------------------------------------------------------------------
# statistics

def familywise_line(n: int, family: float = 0.05) -> float:
    """|z| that n independent coordinates pass together with probability family."""
    per = 1.0 - (1.0 - family) ** (1.0 / n)
    return NormalDist().inv_cdf(1.0 - per / 2.0)


def elo_per_d(games_per_iteration: int) -> float:
    """Elo between the plus and minus sides per unit of mean D."""
    return 695.0 / (2.0 * games_per_iteration)


@dataclass
class Schedule:
    iterations: int
    alpha: float
    gamma: float
    stability: float
    r_end: float

    def perturbation_factor(self, k: int) -> float:
        """c_k / c0 at iteration k (0-based)."""
        return (k + 1) ** -self.gamma

    def ratio(self, k: int) -> float:
        n, a = self.iterations, self.stability
        return self.r_end * ((k + 1) / n) ** (2 * self.gamma) * ((a + n) / (a + k + 1)) ** self.alpha


def forecast_factor(schedule: Schedule, window: range, segment: range, mean_d2: float, elo: float) -> float:
    """Elo of the next segment's gain per unit of the window's sum(z^2) - n.

    A coordinate's gradient g (Elo across one perturbation) gives E[z^2] - 1 =
    g^2 * W / (elo^2 * mean D^2 * ...); held constant, the segment gains
    sum over its iterations of g * move. The coordinate's c0 cancels, so the
    factor is the same for every coordinate.
    """
    if not segment:
        return 0.0
    move = sum(schedule.ratio(k) * schedule.perturbation_factor(k) ** 2 for k in segment)
    held = statistics.mean(schedule.perturbation_factor(k) ** 2 for k in window)
    return move * mean_d2 * elo / (2.0 * held * len(window))


# --------------------------------------------------------------------------
# the run

@dataclass
class Run:
    directory: Path
    names: list[str]
    c0: list[float]
    minimum: list[float]
    maximum: list[float]
    schedule: Schedule
    games_per_iteration: int
    centres: list[list[float]]            # centres[k]: after k completed iterations
    delta: list[list[int]]                # delta[k][i]: the runner's sign at iteration k
    d: list[int]                          # D of iteration k
    status: str
    faults: dict = field(default_factory=dict)
    sources: dict = field(default_factory=dict)
    acceptance: dict = field(default_factory=dict)   # the maintainer's accepted faults, if any

    @property
    def completed(self) -> int:
        return len(self.d)

    @property
    def n(self) -> int:
        return len(self.names)

    def perturbation(self, k: int, i: int) -> float:
        return self.c0[i] * self.schedule.perturbation_factor(k)

    def arm_clamped(self, k: int, i: int) -> bool:
        """An arm of iteration k, centre +- c, left the range and was played at its end."""
        c = self.perturbation(k, i)
        return self.centres[k][i] - c < self.minimum[i] or self.centres[k][i] + c > self.maximum[i]


def load_history(run_dir: Path, cli: Path | None, history_json: Path | None) -> dict:
    if history_json is not None:
        return json.loads(history_json.read_text(encoding="utf-8"))
    if cli is None or not cli.is_file():
        raise ReadError(f"no runner at {cli}; pass --cli or --history-json")
    process = subprocess.run([str(cli), "spsa", "history", str(run_dir), "--json"],
                             capture_output=True, text=True)
    if process.returncode != 0:
        raise ReadError(f"spsa history failed ({process.returncode}): {process.stderr.strip()[-400:]}")
    return json.loads(process.stdout)


def load_scores(run_dir: Path, completed: int, games_per_iteration: int) -> list[int]:
    by_iteration: dict[int, list[dict]] = {}
    with (run_dir / "games.jsonl").open(encoding="utf-8") as handle:
        for line in handle:
            if not line.strip():
                continue
            game = json.loads(line)["game"]
            iteration = int(game["iteration"])
            if iteration < completed and game.get("sample") == "official" and game.get("scorable", True):
                by_iteration.setdefault(iteration, []).append(game)
    scores = []
    for k in range(completed):
        games = by_iteration.get(k, [])
        if len(games) != games_per_iteration:
            raise ReadError(f"iteration {k} has {len(games)} scored games, expected {games_per_iteration}")
        wins = losses = 0
        for game in games:
            plus_white = game["white"] == "a"
            if game["result"] == "WhiteWin":
                wins += plus_white
                losses += not plus_white
            elif game["result"] == "BlackWin":
                wins += not plus_white
                losses += plus_white
            elif game["result"] != "Draw":
                raise ReadError(f"iteration {k}: result {game['result']!r}")
        scores.append(wins - losses)
    return scores


def load_run(run_dir: Path, cli: Path | None = DEFAULT_CLI, history_json: Path | None = None) -> Run:
    plan = json.loads((run_dir / "spsa-schedule.json").read_text(encoding="utf-8"))
    knobs = plan["knobs"]
    sched = plan["schedule"]
    schedule = Schedule(int(sched["iterations"]), float(sched["alpha"]), float(sched["gamma"]),
                        float(sched["stability_constant"]), float(plan["r_end"]))
    resolved = json.loads((run_dir / "resolved-config.json").read_text(encoding="utf-8"))
    games_per_iteration = int(resolved["settings"]["games_per_iteration"])
    history = load_history(run_dir, cli, history_json)
    report = history["report"]
    names = [k["name"] for k in knobs]
    if report["knobs"] != names:
        raise ReadError("history and schedule name different coordinates")
    rows = report["rows"]
    if [int(r["iteration"]) for r in rows] != list(range(len(rows))):
        raise ReadError("history rows are not every iteration from 0; do not read a thinned history")
    completed = int(report["completed_iterations"])
    if len(rows) != completed + 1:
        raise ReadError(f"history has {len(rows)} rows for {completed} completed iterations")
    centres = [[float(x) for x in r["centers"]] for r in rows]
    stream = plan["perturbations"]
    for key, wanted in RNG.items():
        if stream.get(key) != wanted:
            raise ReadError(f"perturbation stream {key} is {stream.get(key)!r}; this reader knows {wanted!r}")
    master_seed = int(stream["master_seed"])
    if stream_seed(master_seed, RNG["stream_name"]).hex() != stream["stream_seed_sha256"]:
        raise ReadError("the regenerated stream seed disagrees with spsa-schedule.json")
    faults = {}
    checkpoint = run_dir / "checkpoint.json"
    if checkpoint.is_file():
        faults = json.loads(checkpoint.read_text(encoding="utf-8"))["payload"].get("faults", {})
    run = Run(run_dir, names, [float(k["c0"]) for k in knobs], [float(k["min"]) for k in knobs],
              [float(k["max"]) for k in knobs], schedule, games_per_iteration, centres,
              perturbation_signs(master_seed, completed, len(names)),
              load_scores(run_dir, completed, games_per_iteration), str(report["run_status"]), faults,
              {"history": "file" if history_json else str(cli)}, load_acceptance(run_dir))
    check_moves(run)
    return run


def load_acceptance(run_dir: Path) -> dict:
    path = run_dir / ACCEPT_FILE
    if not path.is_file():
        return {}
    acceptance = json.loads(path.read_text(encoding="utf-8"))
    if acceptance.get("schema") != ACCEPT_SCHEMA or not isinstance(acceptance.get("faults"), dict):
        raise ReadError(f"{path} is not a fault acceptance this reader knows")
    return acceptance


def accept_faults(run_dir: Path, reason: str) -> dict:
    """The maintainer accepts the faults of the run's last read, a FAULT at a checkpoint."""
    reason = " ".join(reason.split())
    if len(reason) < 10:
        raise ReadError("--accept-faults takes the reason in words: what the faults were and why they are accepted")
    records = load_records(run_dir)
    if not records or records[-1]["verdict"] != "FAULT":
        raise ReadError("the run's last recorded read is not a FAULT; faults are accepted only after the "
                        "checkpoint that read them")
    last = records[-1]
    acceptance = {
        "schema": ACCEPT_SCHEMA, "at": last["at"], "faults": {key: int(value) for key, value in last["faults"].items()},
        "reason": reason,
        "accepted_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M:%SZ"),
    }
    (run_dir / ACCEPT_FILE).write_text(json.dumps(acceptance, indent=2) + "\n", encoding="utf-8")
    return acceptance


def check_moves(run: Run) -> None:
    """Every centre must be the runner's update of the one before it:
    clamp(centre + c_k * r_k * D * delta) to the range."""
    worst = 0.0
    for k, d in enumerate(run.d):
        r = run.schedule.ratio(k)
        for i in range(run.n):
            c = run.perturbation(k, i)
            expected = min(max(run.centres[k][i] + c * r * d * run.delta[k][i], run.minimum[i]), run.maximum[i])
            scale = max(c * r, 1e-12)
            worst = max(worst, abs(run.centres[k + 1][i] - expected) / scale)
    if worst > MOVE_TOLERANCE:
        raise ReadError(f"moves disagree with the runner's update (worst error {worst:.2e} of a unit move); "
                        "the journal, the schedule, the seed or the score is misread")
    run.sources["move_check_worst_error"] = worst


# --------------------------------------------------------------------------
# the reads

def signs(run: Run) -> list[list[int]]:
    return run.delta


def gradient_z(run: Run, delta: list[list[int]], window: range) -> list[float]:
    out = []
    for i in range(run.n):
        num = sum(delta[k][i] * run.d[k] for k in window)
        den = math.sqrt(sum(run.d[k] ** 2 for k in window))
        out.append(num / den if den else 0.0)
    return out


def gradient(run: Run, delta: list[list[int]], ks: list[int], i: int) -> float:
    """Coordinate i's gradient over iterations ks, in Elo per unit of the coordinate.

    E[D_k] = (2 c_k / elo) * delta_k * g for every iteration, so g is the
    least-squares slope of D on the signed perturbation. An iteration whose D
    came out 0 is an observation like any other and stays in the denominator:
    leaving it out selects on the outcome and reads g high by the share of
    such iterations, about a tenth at thirty games.
    """
    num = sum(delta[k][i] * run.d[k] * run.perturbation(k, i) for k in ks)
    den = sum(run.perturbation(k, i) ** 2 for k in ks)
    return elo_per_d(run.games_per_iteration) * num / (2.0 * den) if den else 0.0


def crossfit_gain(run: Run, delta: list[list[int]], moves: list[list[float]], span: range) -> float:
    """Sum over coordinates of gradient times move, cross-fitted within segments."""
    total = 0.0
    starts = list(range(span.start, span.stop, GAIN_SEGMENT))
    if len(starts) > 1 and span.stop - starts[-1] < GAIN_SEGMENT:
        starts.pop()            # a short tail joins the segment before it
    for index, lo in enumerate(starts):
        hi = starts[index + 1] if index + 1 < len(starts) else span.stop
        even = [k for k in range(lo, hi) if k % 2 == 0]
        odd = [k for k in range(lo, hi) if k % 2 == 1]
        for i in range(run.n):
            total += (gradient(run, delta, even, i) * sum(moves[k][i] for k in odd)
                      + gradient(run, delta, odd, i) * sum(moves[k][i] for k in even))
    return total


def actual_moves(run: Run) -> list[list[float]]:
    return [[run.centres[k + 1][i] - run.centres[k][i] for i in range(run.n)] for k in range(run.completed)]


def null_gain_sd(run: Run, span: range, reps: int = NULL_REPS, seed: int = NULL_SEED) -> float:
    """SD of the gain read over span with every coordinate's signs re-drawn at random."""
    rng = random.Random(seed)
    values = []
    for _ in range(reps):
        # Rows outside the span are never read; they stay empty.
        delta = [[rng.choice((-1, 1)) if run.d[k] != 0 else 0 for _ in range(run.n)] if k in span else []
                 for k in range(run.completed)]
        moves = [[delta[k][i] * run.schedule.ratio(k) * run.perturbation(k, i) * run.d[k]
                  for i in range(run.n)] if k in span else [] for k in range(run.completed)]
        values.append(crossfit_gain(run, delta, moves, span))
    return statistics.pstdev(values)


def read(run: Run, at: int, window_size: int = 1000, first: int = 2000, every: int = 1000,
         null_reps: int = NULL_REPS) -> dict:
    if not 0 < at <= run.completed:
        raise ReadError(f"--at {at}: the run has {run.completed} completed iterations")
    if at < window_size:
        raise ReadError(f"--at {at} is shorter than the window of {window_size}")
    ceiling = run.schedule.iterations
    window = range(at - window_size, at)
    delta = signs(run)
    z = gradient_z(run, delta, window)
    n = run.n
    line = familywise_line(n)
    excess = sum(x * x for x in z) - n
    excess_line = 2.0 * math.sqrt(2.0 * n)
    over = [run.names[i] for i in range(n) if abs(z[i]) >= line]
    live = bool(over) or excess >= excess_line
    elo = elo_per_d(run.games_per_iteration)
    d2 = [run.d[k] ** 2 for k in window]
    se_elo = elo * math.sqrt(sum(d2)) / window_size
    coordinates = []
    rails = []
    candidates = []
    for i, name in enumerate(run.names):
        centre = run.centres[at][i]
        perturbation = run.perturbation(at - 1, i)
        num = sum(delta[k][i] * run.d[k] for k in window)
        near_min = centre - run.minimum[i] <= perturbation
        near_max = run.maximum[i] - centre <= perturbation
        # Near both bounds of a narrow range, either push is outward.
        outward = (abs(z[i]) if near_max and near_min else z[i] if near_max
                   else -z[i] if near_min else 0.0)
        rail = outward >= line
        if rail:
            rails.append(name)
        elif outward >= RAIL_CANDIDATE_Z:
            candidates.append(name)
        coordinates.append({
            "name": name, "z": round(z[i], 4), "elo_across_perturbation": round(elo * num / window_size, 3),
            "centre": centre, "start": run.centres[0][i], "theta": round_half_away(centre),
            "arm_clamped_share": round(sum(run.arm_clamped(k, i) for k in range(at)) / at, 5),
            "perturbation": perturbation, "rail": rail, "rail_candidate": name in candidates,
        })
    segment = range(at, min(at + FORECAST_SPAN, ceiling))
    factor = forecast_factor(run.schedule, window, segment, statistics.mean(d2), elo)
    span = range(0, at)
    gain = crossfit_gain(run, delta, actual_moves(run), span)
    gain_sd = null_gain_sd(run, span, reps=null_reps)
    faults = {key: int(value) for key, value in run.faults.items()}
    accepted = run.acceptance.get("faults", {})
    unaccepted = {key: value for key, value in faults.items() if value > int(accepted.get(key, 0))}
    checkpoint = at >= first and ((at - first) % every == 0 or at == ceiling) or at == ceiling
    if unaccepted:
        verdict = "FAULT"
    elif not checkpoint:
        verdict = "INTERIM"
    elif rails:
        verdict = "STOP_RAIL"
    elif not live:
        verdict = "STOP_QUIET"
    elif at >= ceiling:
        verdict = "STOP_CEILING"
    else:
        verdict = "CONTINUE"
    return {
        "schema": SCHEMA,
        "run": str(run.directory),
        "at": at, "completed": run.completed, "ceiling": ceiling, "run_status": run.status,
        "window": [window.start, window.stop], "first": first, "every": every,
        "coordinates_n": n, "familywise_line": round(line, 4),
        "max_abs_z": round(max(abs(x) for x in z), 4), "over_line": over,
        "excess": round(excess, 3), "excess_sd": round(excess / math.sqrt(2.0 * n), 3),
        "excess_line": round(excess_line, 3), "live": live,
        "floor": {
            "coordinate_elo": round(line * se_elo, 2),
            "coordinate_se_elo": round(se_elo, 3),
            # At the ceiling no segment follows, so the forecast rules nothing out.
            "forecast_two_sd_elo": round(2.0 * factor * math.sqrt(2.0 * n), 2) if segment else None,
        },
        "gain": {"elo": round(gain, 2), "null_sd": round(gain_sd, 2), "span": [0, at], "null_reps": null_reps},
        "forecast": {"segment": [segment.start, segment.stop], "elo": round(factor * excess, 2),
                     "sd": round(factor * math.sqrt(2.0 * n), 2)},
        "rails": rails, "rail_candidates": candidates, "faults": faults, "faults_unaccepted": unaccepted,
        "faults_accepted": dict(run.acceptance) if any(faults.values()) and not unaccepted else None,
        "verdict": verdict,
        "coordinates": coordinates,
        "move_check_worst_error": run.sources.get("move_check_worst_error"),
        "clamped_over_one_percent": [c["name"] for c in coordinates if c["arm_clamped_share"] > 0.01],
    }


def round_half_away(value: float) -> int:
    """The runner's rounding of a centre to the value it sends."""
    return int(math.ceil(value - 0.5)) if value < 0 else int(math.floor(value + 0.5))


def score_forecasts(run: Run, records: list[dict], at: int, null_reps: int = NULL_REPS) -> list[dict]:
    """Each earlier forecast whose segment has been played, against the gain read over it.

    The gain read it is scored against is itself noisy, usually more than the
    forecast, so the two SDs combine: a forecast is within 2 SD when it is
    within twice sqrt(forecast SD^2 + gain read's null SD^2) of the gain read.
    """
    delta = signs(run)
    moves = actual_moves(run)
    scored = []
    for record in records:
        lo, hi = record["forecast"]["segment"]
        if hi <= lo or hi > at:
            continue
        actual = crossfit_gain(run, delta, moves, range(lo, hi))
        actual_sd = null_gain_sd(run, range(lo, hi), reps=null_reps)
        forecast, sd = record["forecast"]["elo"], record["forecast"]["sd"]
        combined = math.sqrt(sd ** 2 + actual_sd ** 2)
        scored.append({"made_at": record["at"], "segment": [lo, hi], "forecast": forecast, "sd": sd,
                       "gain_read": round(actual, 2), "gain_read_sd": round(actual_sd, 2),
                       "combined_sd": round(combined, 2),
                       "within_two_sd": abs(actual - forecast) <= 2.0 * combined})
    return scored


def forecast_status(scored: list[dict]) -> str:
    first_three = scored[:3]
    if len(first_three) < 3:
        return f"decides nothing: {len(scored)} of 3 forecasts scored"
    if all(s["within_two_sd"] for s in first_three):
        return "the first three forecasts each fell within 2 combined SD: the forecast may be used"
    return "decides nothing: a forecast among the first three fell outside 2 combined SD"


# --------------------------------------------------------------------------
# record and report

def render(result: dict) -> str:
    lines = [
        f"SPSA read of {result['run']} at iteration {result['at']} "
        f"(window {result['window'][0]}-{result['window'][1]}, ceiling {result['ceiling']}, "
        f"run {result['run_status']})",
        f"  verdict: {result['verdict']}",
        f"  {result['coordinates_n']} coordinates; familywise line {result['familywise_line']:.2f}; "
        f"max |z| {result['max_abs_z']:.2f}; over the line: {', '.join(result['over_line']) or 'none'}",
        f"  sum z^2 - n = {result['excess']:+.1f} ({result['excess_sd']:+.1f} SD; line "
        f"{result['excess_line']:.1f}); {'LIVE' if result['live'] else 'quiet'}",
        f"  floor: no coordinate above {result['floor']['coordinate_elo']:.1f} Elo across a perturbation "
        f"(SE {result['floor']['coordinate_se_elo']:.2f}); "
        + (f"the forecast's 2 SD is {result['floor']['forecast_two_sd_elo']:.1f} Elo"
           if result['floor']['forecast_two_sd_elo'] is not None else "no forecast floor at the ceiling"),
        f"  the tune's own read of its gain since iteration 0: {result['gain']['elo']:+.1f} Elo "
        f"(null SD {result['gain']['null_sd']:.1f})",
        (f"  forecast for iterations {result['forecast']['segment'][0]}-{result['forecast']['segment'][1]}: "
         f"at most {result['forecast']['elo']:+.1f} +/- {result['forecast']['sd']:.1f} Elo"
         if result['forecast']['segment'][0] < result['forecast']['segment'][1]
         else "  forecast: none, the ceiling is reached"),
        f"  arms clamped on over 1% of iterations: {', '.join(result['clamped_over_one_percent']) or 'none'}",
        f"  rails: {', '.join(result['rails']) or 'none'}; rail candidates (outward z of "
        f"{RAIL_CANDIDATE_Z:g} or more, under the line; they decide nothing): "
        f"{', '.join(result['rail_candidates']) or 'none'}",
        f"  faults: "
        f"{', '.join(f'{k} {v}' for k, v in result['faults'].items() if v) or 'none'}"
        + (f" (accepted by the maintainer at iteration {result['faults_accepted']['at']}: "
           f"{result['faults_accepted']['reason']})" if result.get("faults_accepted") else ""),
    ]
    for item in result.get("scored_forecasts", []):
        lines.append(f"  forecast made at {item['made_at']} for {item['segment'][0]}-{item['segment'][1]}: "
                     f"{item['forecast']:+.1f} +/- {item['sd']:.1f}; gain read {item['gain_read']:+.1f} "
                     f"+/- {item['gain_read_sd']:.1f}; "
                     f"{'within' if item['within_two_sd'] else 'outside'} 2 combined SD ({item['combined_sd']:.1f})")
    if "forecast_status" in result:
        lines.append(f"  forecast: {result['forecast_status']}")
    lines.append("  coordinates (z, Elo across one perturbation, centre, theta):")
    for c in sorted(result["coordinates"], key=lambda c: -abs(c["z"])):
        lines.append(f"    {c['name']:32} z {c['z']:+6.2f}  {c['elo_across_perturbation']:+6.2f} Elo  "
                     f"{c['centre']:12.4f}  {c['theta']}{'  RAIL' if c['rail'] else '  rail candidate' if c['rail_candidate'] else ''}")
    return "\n".join(lines) + "\n"


def write_theta(run_dir: Path, result: dict) -> list[Path]:
    at = result["at"]
    txt = run_dir / f"theta-{at}.txt"
    toml = run_dir / f"theta-{at}.toml"
    txt.write_text("".join(f"setoption name {c['name']} value {c['theta']}\n" for c in result["coordinates"]),
                   encoding="utf-8")
    toml.write_text("[engine.options]\n" + "".join(f"{c['name']} = {c['theta']}\n"
                                                   for c in sorted(result["coordinates"], key=lambda c: c["name"])),
                    encoding="utf-8")
    return [txt, toml]


def load_records(run_dir: Path) -> list[dict]:
    path = run_dir / "spsa-read.jsonl"
    if not path.is_file():
        return []
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def record(run_dir: Path, result: dict) -> str:
    """Append once per checkpoint; an existing record for it must agree.

    The one read that may follow another at the same checkpoint is the real
    verdict after the maintainer accepted the faults of a FAULT read there.
    """
    records = load_records(run_dir)
    same = [r for r in records if r["at"] == result["at"]]
    after_acceptance = bool(same and same[-1]["verdict"] == "FAULT" and result["verdict"] != "FAULT"
                            and result.get("faults_accepted"))
    if same and not after_acceptance:
        keep = ("verdict", "excess", "max_abs_z", "forecast")
        if any(same[-1][key] != result[key] for key in keep):
            raise ReadError(f"spsa-read.jsonl already holds a different read at {result['at']}")
        return "already recorded"
    with (run_dir / "spsa-read.jsonl").open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(result) + "\n")
    (run_dir / "spsa-read.txt").write_text("".join(render(r) + "\n" for r in [*records, result]), encoding="utf-8")
    return "recorded after the maintainer's fault acceptance" if after_acceptance else "recorded"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("run_dir", type=Path)
    parser.add_argument("--at", type=int, default=0, help="checkpoint iteration (default: the last completed)")
    parser.add_argument("--window", type=int, default=1000)
    parser.add_argument("--first", type=int, default=2000)
    parser.add_argument("--every", type=int, default=1000)
    parser.add_argument("--record", action="store_true")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--cli", type=Path, default=DEFAULT_CLI)
    parser.add_argument("--history-json", type=Path, default=None, help="a saved `spsa history --json` (tests)")
    parser.add_argument("--null-reps", type=int, default=NULL_REPS)
    parser.add_argument("--write-theta", action="store_true",
                        help="with --record: the maintainer stops a live tune at this checkpoint; write its theta")
    parser.add_argument("--accept-faults", metavar="REASON", default=None,
                        help="the maintainer accepts the faults of the last read, a FAULT, for this reason; no read")
    args = parser.parse_args()
    if args.accept_faults is not None:
        try:
            acceptance = accept_faults(args.run_dir, args.accept_faults)
        except (ReadError, OSError, KeyError, ValueError, json.JSONDecodeError) as error:
            print(f"spsa_read: {error}", file=sys.stderr)
            return 2
        counts = ", ".join(f"{key} {value}" for key, value in acceptance["faults"].items() if value)
        print(f"accepted at iteration {acceptance['at']}: {counts}\n  reason: {acceptance['reason']}\n"
              f"  wrote {args.run_dir / ACCEPT_FILE}; run the tune's command again to read the checkpoint and go on")
        return 0
    try:
        run = load_run(args.run_dir, args.cli, args.history_json)
        at = args.at or run.completed
        result = read(run, at, args.window, args.first, args.every, args.null_reps)
        # One forecast a checkpoint: a read after a fault acceptance repeats the FAULT read's.
        earlier = list({r["at"]: r for r in load_records(args.run_dir) if r["at"] < at}.values())
        result["scored_forecasts"] = score_forecasts(run, earlier, at, args.null_reps)
        result["forecast_status"] = forecast_status(result["scored_forecasts"])
        if args.record and result["verdict"] == "INTERIM":
            # Only a registered checkpoint enters the record: a read between
            # checkpoints decides nothing, and its forecast is never scored.
            result["record"] = "not recorded: not a registered checkpoint"
        elif args.record:
            result["record"] = record(args.run_dir, result)
            stop = result["verdict"] in ("STOP_QUIET", "STOP_CEILING")
            chosen = args.write_theta and result["verdict"] == "CONTINUE"
            if stop or chosen:
                result["theta_files"] = [str(p) for p in write_theta(args.run_dir, result)]
    except (ReadError, OSError, KeyError, ValueError, json.JSONDecodeError) as error:
        print(f"spsa_read: {error}", file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2) if args.json else render(result), end="" if not args.json else "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
