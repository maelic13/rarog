"""Tests for tools/spsa_read.py: the runner's signs, the statistics, synthetic
runs of every verdict, and Rarog's finished tunes where they are on disk."""

from __future__ import annotations

import importlib.util
import json
import math
import pathlib
import random
import struct
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
RESULTS = ROOT / "tools" / "results"


def load(name: str, path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader
    sys.modules[name] = module  # dataclasses resolve their module by name
    spec.loader.exec_module(module)
    return module


sr = load("spsa_read", ROOT / "tools" / "spsa_read.py")


class RunnerSignTests(unittest.TestCase):
    """Colosseum's own golden vectors (colosseum-core rng.rs and spsa.rs)."""

    MASTER = 0x0123_4567_89AB_CDEF

    def test_stream_seed_derivation(self):
        self.assertEqual(sr.stream_seed(self.MASTER, "spsa-perturbations").hex(),
                         "e58ef38a32c2012aa43f0ff3de0a4e1acabf563789afb301c0feff708776dc6f")

    def test_chacha12_block(self):
        key = struct.unpack("<8I", sr.stream_seed(self.MASTER, "opening-order"))
        self.assertEqual(sr._chacha12_block(key, 0).hex(),
                         "c2198a09daf1a2cd09160d69492bff7c848323cd90f7fa7a916110036907f455"
                         "3565d341820734b4348b985eb06a2ec5e975f2d492da09119596fc7ef6da02ff")

    def test_rademacher_draws_and_iteration_major_order(self):
        self.assertEqual(sr.perturbation_signs(self.MASTER, 1, 16)[0],
                         [-1, 1, 1, -1, 1, -1, 1, -1, -1, 1, -1, -1, 1, 1, 1, 1])
        self.assertEqual(sr.perturbation_signs(self.MASTER, 2, 3)[1], [-1, 1, -1])


class StatisticTests(unittest.TestCase):
    def test_familywise_lines(self):
        for n, line in ((8, 2.73), (15, 2.93), (29, 3.13), (49, 3.28)):
            self.assertAlmostEqual(sr.familywise_line(n), line, places=2)

    def test_forecast_floor_matches_the_recorded_shape(self):
        # B.5.6 forecast.txt: the floor of a quiet read at 2,000 for the next
        # 2,000 iterations at a mean D^2 of 11.6.
        elo = sr.elo_per_d(30)
        for r_end, floors in ((0.0031, (3.6, 5.0, 6.9, 9.0)), (0.00155, (1.8, 2.5, 3.5, 4.5))):
            schedule = sr.Schedule(6000, 0.601, 0.102, 600.0, r_end)
            factor = sr.forecast_factor(schedule, range(1000, 2000), range(2000, 4000), 11.6, elo)
            for n, floor in zip((8, 15, 29, 49), floors):
                self.assertAlmostEqual(2 * factor * math.sqrt(2 * n), floor, places=1)

    def test_the_gradient_counts_every_iteration(self):
        # Constant perturbation 8; D = delta on 50 iterations and 0 on 50. The
        # least-squares slope of D on the signed perturbation is
        # elo * (50 * 8) / (2 * 100 * 64); leaving the zeros out doubles it.
        delta = [[1 if k % 3 else -1] for k in range(100)]
        d = [delta[k][0] if k % 2 == 0 else 0 for k in range(100)]
        run = sr.Run(pathlib.Path("crafted"), ["K0"], [8.0], [0.0], [100.0],
                     sr.Schedule(100, 0.0, 0.0, 10.0, 0.00155), 6, [[50.0]] * 101, delta, d, "stopped", {})
        self.assertAlmostEqual(sr.gradient(run, delta, list(range(100)), 0), sr.elo_per_d(6) / 32.0, places=12)

    def test_a_forecast_is_scored_against_the_combined_sd(self):
        # A gain read 3 Elo from its forecast: outside twice the forecast's own
        # SD of 1, inside twice the combined SD once the gain read's noise counts.
        delta = sr.perturbation_signs(77, 400, 2)
        rng = random.Random(5)
        d = [rng.choice((-3, -1, 0, 1, 3)) for _ in range(400)]
        schedule = sr.Schedule(400, 0.601, 0.102, 40.0, 0.00155)
        centres = [[50.0, 50.0]]
        for k in range(400):
            centres.append([centres[-1][i] + 8.0 * schedule.perturbation_factor(k) * schedule.ratio(k) * d[k] * delta[k][i]
                            for i in range(2)])
        run = sr.Run(pathlib.Path("crafted"), ["K0", "K1"], [8.0] * 2, [0.0] * 2, [100.0] * 2, schedule, 6,
                     centres, delta, d, "completed", {})
        actual = sr.crossfit_gain(run, delta, sr.actual_moves(run), range(200, 400))
        noise = sr.null_gain_sd(run, range(200, 400), reps=20)
        self.assertGreater(noise, 0.0)
        near = {"at": 200, "forecast": {"segment": [200, 400], "elo": actual + 1.5 * noise, "sd": 0.1 * noise}}
        far = {"at": 200, "forecast": {"segment": [200, 400], "elo": actual + 2.5 * noise, "sd": 0.1 * noise}}
        scored = sr.score_forecasts(run, [near, far], 400, null_reps=20)
        self.assertAlmostEqual(scored[0]["gain_read_sd"], noise, places=2)
        self.assertGreater(scored[0]["combined_sd"], scored[0]["sd"])
        self.assertTrue(scored[0]["within_two_sd"])       # outside twice its own SD, inside the combined
        self.assertFalse(scored[1]["within_two_sd"])

    def test_rounding_is_the_runners(self):
        for value, expected in ((2.5, 3), (-2.5, -3), (2.4999, 2), (-0.5, -1), (7.945, 8)):
            self.assertEqual(sr.round_half_away(value), expected)


# --------------------------------------------------------------------------
# synthetic runs, played through the runner's own update rule

def simulate(directory: pathlib.Path, *, gradients, starts, bounds, c0, ceiling=3000, completed=None,
             games=6, seed=57001, master_seed=9001, faults=None, corrupt_at=None):
    """Write a run directory whose journal a real tune would leave behind.

    Each game is won by the plus side with probability 0.3 + b and lost with
    0.3 - b, b = 0.5 * sum(g_i * delta_i): g_i is coordinate i's gradient.
    """
    rng = random.Random(seed)
    completed = ceiling if completed is None else completed
    n = len(gradients)
    schedule = sr.Schedule(ceiling, 0.601, 0.102, ceiling / 10.0, 0.00155)
    delta = sr.perturbation_signs(master_seed, completed, n)
    centres = [list(map(float, starts))]
    games_out = []
    for k in range(completed):
        bias = 0.5 * sum(g * d for g, d in zip(gradients, delta[k]))
        win, loss = 0.3 + bias, 0.3 - bias
        score = 0
        for game in range(games):
            u = rng.random()
            result = "plus" if u < win else "minus" if u < win + loss else "draw"
            score += (result == "plus") - (result == "minus")
            plus_white = game % 2 == 0
            white_result = {"plus": "WhiteWin" if plus_white else "BlackWin",
                            "minus": "BlackWin" if plus_white else "WhiteWin", "draw": "Draw"}[result]
            games_out.append({"game": {"iteration": k, "white": "a" if plus_white else "b",
                                       "result": white_result, "sample": "official", "scorable": True}})
        r = schedule.ratio(k)
        nxt = []
        for i in range(n):
            c = c0[i] * schedule.perturbation_factor(k)
            nxt.append(min(max(centres[-1][i] + c * r * score * delta[k][i], bounds[i][0]), bounds[i][1]))
        centres.append(nxt)
    if corrupt_at is not None:
        centres[corrupt_at][0] += 0.25 * c0[0]
    names = [f"K{i}" for i in range(n)]
    plan = {
        "schedule": {"iterations": ceiling, "alpha": 0.601, "gamma": 0.102, "stability_constant": ceiling / 10.0},
        "r_end": 0.00155,
        "perturbations": {**sr.RNG, "master_seed": master_seed,
                          "stream_seed_sha256": sr.stream_seed(master_seed, sr.RNG["stream_name"]).hex()},
        "knobs": [{"name": names[i], "min": bounds[i][0], "max": bounds[i][1], "c0": c0[i],
                   "c_end": c0[i] * ceiling ** -0.102} for i in range(n)],
    }
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "spsa-schedule.json").write_text(json.dumps(plan), encoding="utf-8")
    (directory / "resolved-config.json").write_text(json.dumps({"settings": {"games_per_iteration": games}}),
                                                    encoding="utf-8")
    with (directory / "games.jsonl").open("w", encoding="utf-8") as handle:
        for record in games_out:
            handle.write(json.dumps(record) + "\n")
    history = {"report": {"run_status": "completed" if completed == ceiling else "stopped",
                          "completed_iterations": completed, "knobs": names,
                          "rows": [{"iteration": k, "centers": c, "clamped_at_min": [0] * n,
                                    "clamped_at_max": [0] * n} for k, c in enumerate(centres)]}}
    (directory / "history.json").write_text(json.dumps(history), encoding="utf-8")
    (directory / "checkpoint.json").write_text(json.dumps({"payload": {"faults": faults or {
        "engine_a": 0, "engine_b": 0, "time_losses_a": 0, "time_losses_b": 0, "infrastructure": 0}}}),
        encoding="utf-8")
    return directory


def crafted_run(push: int, centre: float, n: int = 4, at: int = 2000, ceiling: int = 3000):
    """A run whose window sets coordinate 0's z exactly: in the last 1,000
    iterations D = delta_0 on `push` of them (against it when negative), and
    on the rest D cancels in pairs, so z_0 = push / sqrt(1000); the others'
    signs cancel against D, so their z is 0. Coordinate 0
    sits at `centre` in [0, 100], within one perturbation of a bound when near
    0 or 100; the others sit mid-range. Only the reads are exercised."""
    delta = sr.perturbation_signs(4242, at, n)
    d = [0] * at
    sign = 1 if push >= 0 else -1
    lo = at - 1000
    for j, k in enumerate(range(lo, at)):
        if j < abs(push):
            d[k] = sign * delta[k][0]
        else:
            d[k] = delta[k][0] if (j - abs(push)) % 2 == 0 else -delta[k][0]
        for i in range(1, n):   # the other coordinates' z is exactly 0
            delta[k][i] = d[k] if j % 2 == 0 else -d[k]
    centres = [[centre] + [50.0] * (n - 1) for _ in range(at + 1)]
    return sr.Run(pathlib.Path("crafted"), [f"K{i}" for i in range(n)], [8.0] * n, [0.0] * n, [100.0] * n,
                  sr.Schedule(ceiling, 0.601, 0.102, ceiling / 10.0, 0.00155), 6, centres, delta, d, "stopped",
                  {})


QUIET = dict(gradients=[0.0, 0.0, 0.0, 0.0], starts=[50, 50, 50, 50], bounds=[(0, 100)] * 4, c0=[8.0] * 4)
LIVE = dict(gradients=[0.12, 0.0, 0.0, 0.0], starts=[30, 50, 50, 50], bounds=[(0, 100)] * 4, c0=[8.0] * 4)
RAIL = dict(gradients=[0.12, 0.0, 0.0, 0.0], starts=[96, 50, 50, 50], bounds=[(0, 100)] * 4, c0=[8.0] * 4)


class SyntheticRunTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def run_of(self, name, spec, **extra):
        directory = simulate(self.root / name, **spec, **extra)
        return sr.load_run(directory, cli=None, history_json=directory / "history.json")

    def test_a_run_with_no_gradient_reads_quiet_and_stops(self):
        run = self.run_of("quiet", QUIET)
        result = sr.read(run, 2000, null_reps=5)
        self.assertFalse(result["live"], result["max_abs_z"])
        self.assertEqual(result["verdict"], "STOP_QUIET")
        self.assertLess(abs(result["gain"]["elo"]), 4 * max(result["gain"]["null_sd"], 0.5))

    def test_a_run_with_a_gradient_reads_live_and_continues_to_its_ceiling(self):
        run = self.run_of("live", LIVE)
        at_first = sr.read(run, 2000, null_reps=5)
        self.assertTrue(at_first["live"])
        self.assertEqual(at_first["verdict"], "CONTINUE")
        self.assertEqual(at_first["over_line"], ["K0"])
        self.assertGreater(at_first["coordinates"][0]["z"], sr.familywise_line(4))
        self.assertGreater(at_first["gain"]["elo"], 2 * at_first["gain"]["null_sd"])
        self.assertEqual(at_first["forecast"]["segment"], [2000, 3000])
        self.assertGreater(at_first["forecast"]["elo"], 0)
        at_ceiling = sr.read(run, 3000, null_reps=5)
        self.assertEqual(at_ceiling["verdict"], "STOP_CEILING")
        # No segment follows the ceiling, so no forecast floor is claimed there.
        self.assertIsNone(at_ceiling["floor"]["forecast_two_sd_elo"])
        self.assertIn("the ceiling is reached", sr.render(at_ceiling))
        self.assertIsNotNone(at_first["floor"]["forecast_two_sd_elo"])
        self.assertEqual(sr.read(run, 2500, null_reps=5)["verdict"], "INTERIM")

    def test_a_centre_at_its_bound_pushed_outward_is_a_rail(self):
        run = self.run_of("rail", RAIL)
        result = sr.read(run, 2000, null_reps=5)
        self.assertEqual(result["rails"], ["K0"])
        self.assertEqual(result["verdict"], "STOP_RAIL")
        self.assertGreater(result["coordinates"][0]["arm_clamped_share"], 0.5)
        # The true signs keep the clamped iterations; a sign read from the
        # moves loses them and points the gradient inward.
        self.assertGreater(result["coordinates"][0]["z"], sr.familywise_line(4))

    def test_a_weak_outward_push_is_a_candidate_and_stops_nothing(self):
        result = sr.read(crafted_run(push=70, centre=98.0), 2000, null_reps=3)
        self.assertGreaterEqual(result["coordinates"][0]["z"], sr.RAIL_CANDIDATE_Z)
        self.assertLess(result["coordinates"][0]["z"], result["familywise_line"])
        self.assertFalse(result["live"])
        self.assertEqual(result["rails"], [])
        self.assertEqual(result["rail_candidates"], ["K0"])
        self.assertEqual(result["verdict"], "STOP_QUIET")

    def test_an_outward_push_past_the_line_is_a_rail(self):
        result = sr.read(crafted_run(push=120, centre=98.0), 2000, null_reps=3)
        self.assertGreater(result["coordinates"][0]["z"], result["familywise_line"])
        self.assertEqual(result["rails"], ["K0"])
        self.assertEqual(result["verdict"], "STOP_RAIL")
        at_min = sr.read(crafted_run(push=-120, centre=2.0), 2000, null_reps=3)
        self.assertEqual(at_min["rails"], ["K0"])

    def test_an_inward_push_near_a_bound_is_no_rail(self):
        result = sr.read(crafted_run(push=-120, centre=98.0), 2000, null_reps=3)
        self.assertLess(result["coordinates"][0]["z"], -result["familywise_line"])
        self.assertEqual(result["rails"], [])
        self.assertEqual(result["rail_candidates"], [])
        self.assertTrue(result["live"])
        self.assertEqual(result["verdict"], "CONTINUE")

    def test_faults_stop_the_read(self):
        run = self.run_of("fault", QUIET, faults={"engine_a": 0, "time_losses_a": 1})
        self.assertEqual(sr.read(run, 2000, null_reps=5)["verdict"], "FAULT")

    def test_a_misread_journal_fails_loudly(self):
        directory = simulate(self.root / "corrupt", **QUIET, corrupt_at=1500)
        with self.assertRaisesRegex(sr.ReadError, "disagree with the runner's update"):
            sr.load_run(directory, cli=None, history_json=directory / "history.json")
        directory = simulate(self.root / "seed", **QUIET)
        plan = json.loads((directory / "spsa-schedule.json").read_text(encoding="utf-8"))
        plan["perturbations"]["master_seed"] += 1
        (directory / "spsa-schedule.json").write_text(json.dumps(plan), encoding="utf-8")
        with self.assertRaisesRegex(sr.ReadError, "stream seed"):
            sr.load_run(directory, cli=None, history_json=directory / "history.json")

    def test_a_partial_iteration_is_refused(self):
        directory = simulate(self.root / "short", **QUIET, completed=2000)
        lines = (directory / "games.jsonl").read_text(encoding="utf-8").splitlines()
        (directory / "games.jsonl").write_text("\n".join(lines[:-1]) + "\n", encoding="utf-8")
        with self.assertRaisesRegex(sr.ReadError, "scored games"):
            sr.load_run(directory, cli=None, history_json=directory / "history.json")

    def test_a_live_read_mid_run_uses_only_completed_iterations(self):
        run = self.run_of("midrun", LIVE, completed=2200)
        self.assertEqual(run.completed, 2200)
        self.assertEqual(sr.read(run, 2000, null_reps=5)["verdict"], "CONTINUE")
        self.assertEqual(sr.read(run, 2200, null_reps=5)["verdict"], "INTERIM")
        with self.assertRaises(sr.ReadError):
            sr.read(run, 2300, null_reps=5)

    def test_the_record_is_kept_once_and_scores_its_forecast(self):
        run = self.run_of("record", LIVE)
        first = sr.read(run, 2000, null_reps=5)
        self.assertEqual(sr.record(run.directory, first), "recorded")
        self.assertEqual(sr.record(run.directory, first), "already recorded")
        altered = dict(first, verdict="STOP_QUIET")
        with self.assertRaisesRegex(sr.ReadError, "different read"):
            sr.record(run.directory, altered)
        scored = sr.score_forecasts(run, sr.load_records(run.directory), 3000)
        self.assertEqual(len(scored), 1)
        self.assertEqual(scored[0]["segment"], [2000, 3000])
        self.assertIn("1 of 3", sr.forecast_status(scored))
        written = sr.write_theta(run.directory, sr.read(run, 3000, null_reps=5))
        text = written[0].read_text(encoding="utf-8")
        self.assertIn("setoption name K0 value", text)
        self.assertEqual(len(text.splitlines()), 4)


class CommandLineTests(unittest.TestCase):
    """The reader as the wrapper runs it: --record, theta at a stop only."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def read(self, directory, *extra):
        import subprocess
        process = subprocess.run([sys.executable, str(ROOT / "tools" / "spsa_read.py"), str(directory),
                                  "--at", "2000", "--history-json", str(directory / "history.json"),
                                  "--null-reps", "3", "--record", "--json", *extra],
                                 capture_output=True, text=True)
        self.assertEqual(process.returncode, 0, process.stderr)
        return json.loads(process.stdout)

    def test_theta_is_written_at_a_stop_and_by_the_maintainer_only(self):
        quiet = simulate(self.root / "quiet", **QUIET)
        self.assertEqual(self.read(quiet)["verdict"], "STOP_QUIET")
        self.assertTrue((quiet / "theta-2000.txt").is_file())
        rail = simulate(self.root / "rail", **RAIL)
        self.assertEqual(self.read(rail)["verdict"], "STOP_RAIL")
        self.assertFalse((rail / "theta-2000.txt").exists())
        live = simulate(self.root / "live", **LIVE)
        self.assertEqual(self.read(live)["verdict"], "CONTINUE")
        self.assertFalse((live / "theta-2000.txt").exists())
        self.assertEqual(self.read(live, "--write-theta")["record"], "already recorded")
        self.assertTrue((live / "theta-2000.txt").is_file())

    def test_the_maintainer_accepts_a_fault_after_its_checkpoint(self):
        import subprocess
        reader = [sys.executable, str(ROOT / "tools" / "spsa_read.py")]
        one_forfeit = {"engine_a": 1, "engine_b": 0, "time_losses_a": 1, "time_losses_b": 0, "infrastructure": 0}
        run = simulate(self.root / "forfeit", **QUIET, faults=one_forfeit)

        def accept(directory, reason):
            return subprocess.run([*reader, str(directory), "--accept-faults", reason], capture_output=True, text=True)

        # Nothing to accept before the checkpoint has read the fault.
        self.assertEqual(accept(run, "one time forfeit during a host update").returncode, 2)
        self.assertEqual(self.read(run)["verdict"], "FAULT")
        self.assertFalse((run / "theta-2000.txt").exists())
        self.assertEqual(self.read(run)["record"], "already recorded")
        # The reason is required, in words.
        self.assertEqual(accept(run, "ok").returncode, 2)
        self.assertFalse((run / sr.ACCEPT_FILE).exists())
        accepted = accept(run, "one time forfeit at iteration 19 during a host update; it moves no centre by 0.01 step")
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        acceptance = json.loads((run / sr.ACCEPT_FILE).read_text(encoding="utf-8"))
        self.assertEqual(acceptance["faults"], one_forfeit)
        self.assertEqual(acceptance["at"], 2000)
        # The same checkpoint now reads its real verdict, recorded after the FAULT.
        after = self.read(run)
        self.assertEqual(after["verdict"], "STOP_QUIET")
        self.assertIn("fault acceptance", after["record"])
        self.assertIn("host update", after["faults_accepted"]["reason"])
        self.assertTrue((run / "theta-2000.txt").is_file())
        self.assertEqual([r["verdict"] for r in sr.load_records(run)], ["FAULT", "STOP_QUIET"])
        self.assertIn("accepted by the maintainer", (run / "spsa-read.txt").read_text(encoding="utf-8"))
        self.assertEqual(self.read(run)["record"], "already recorded")
        # A read that is not a FAULT leaves nothing to accept.
        self.assertEqual(accept(run, "a second acceptance with nothing new to accept").returncode, 2)
        # A fault beyond the accepted counts is a FAULT again.
        checkpoint = json.loads((run / "checkpoint.json").read_text(encoding="utf-8"))
        checkpoint["payload"]["faults"]["time_losses_a"] = 2
        checkpoint["payload"]["faults"]["engine_a"] = 2
        (run / "checkpoint.json").write_text(json.dumps(checkpoint), encoding="utf-8")
        again = sr.read(sr.load_run(run, cli=None, history_json=run / "history.json"), 2000, null_reps=3)
        self.assertEqual(again["verdict"], "FAULT")
        self.assertEqual(again["faults_unaccepted"], {"engine_a": 2, "time_losses_a": 2})
        self.assertIsNone(again["faults_accepted"])

    def test_a_read_between_checkpoints_is_not_recorded(self):
        # The wrapper takes the record's last line as the tune's state, and
        # the first three recorded forecasts decide whether it may be used.
        import subprocess
        live = simulate(self.root / "peek", **LIVE, completed=2500)
        process = subprocess.run([sys.executable, str(ROOT / "tools" / "spsa_read.py"), str(live),
                                  "--history-json", str(live / "history.json"), "--null-reps", "3",
                                  "--record", "--json"], capture_output=True, text=True)
        self.assertEqual(process.returncode, 0, process.stderr)
        result = json.loads(process.stdout)
        self.assertEqual(result["at"], 2500)
        self.assertEqual(result["verdict"], "INTERIM")
        self.assertIn("not recorded", result["record"])
        self.assertFalse((live / "spsa-read.jsonl").exists())
        self.assertFalse((live / "theta-2500.txt").exists())

    def test_an_error_exits_2(self):
        import subprocess
        directory = simulate(self.root / "bad", **QUIET, corrupt_at=10)
        process = subprocess.run([sys.executable, str(ROOT / "tools" / "spsa_read.py"), str(directory),
                                  "--history-json", str(directory / "history.json")],
                                 capture_output=True, text=True)
        self.assertEqual(process.returncode, 2)
        self.assertIn("disagree", process.stderr)


# --------------------------------------------------------------------------
# Rarog's finished Colosseum tunes, where they are on disk (tools/results is
# not in Git, so CI skips these)

RECORDED = {
    # At iteration 2,000 with the production checkpoints (window 1,000, first
    # 2,000): max |z|, the familywise line, excess SD, the coordinates over
    # the line and the verdict. Read 2026-11-13 by this reader
    # (analysis/spsa_method_adoption_2026-11-13.md); they pin it.
    "b27all-spsa": (2.57, 3.42, 1.4, [], "STOP_QUIET"),
    "b33m-spsa-1": (2.22, 3.19, -1.2, [], "STOP_QUIET"),
    "b33m-spsa-2": (2.02, 3.19, 0.3, [], "STOP_QUIET"),
    "b33r-spsa-1": (3.70, 3.19, 0.8, ["CoreSingularMargin"], "FAULT"),
    "b43-spsa-1": (2.79, 2.91, 1.4, [], "STOP_QUIET"),
    "c10-long": (3.50, 3.42, 1.9, ["QsDeltaMargin"], "CONTINUE"),
    "d14-spsa-block1": (2.28, 2.88, 0.4, [], "STOP_QUIET"),
    "rar-e24-block": (2.69, 3.12, 0.6, [], "STOP_QUIET"),
    "rar-e25-block": (2.90, 3.12, 2.2, [], "STOP_CEILING"),
    "rar-r21-fit": (2.49, 2.93, 0.8, [], "STOP_QUIET"),
}


def history_from_result(directory: pathlib.Path, out: pathlib.Path) -> pathlib.Path:
    """The history the runner would print, from a finished tune's result.json."""
    result = json.loads((directory / "result.json").read_text(encoding="utf-8"))
    names = [k["name"] for k in result["schedule"]["knobs"]]
    rows = [{"iteration": 0, "centers": [float(p["original"]) for p in result["tuned_result"]["parameters"]]}]
    for k, iteration in enumerate(result["driver"]["completed_iterations"]):
        rows.append({"iteration": k + 1, "centers": iteration["centers_after"]})
    for row in rows:
        row["clamped_at_min"] = row["clamped_at_max"] = [0] * len(names)
    out.write_text(json.dumps({"report": {"run_status": result["driver"]["status"],
                                          "completed_iterations": len(rows) - 1,
                                          "knobs": names, "rows": rows}}), encoding="utf-8")
    return out


@unittest.skipUnless(all((RESULTS / run / "result.json").is_file() for run in RECORDED),
                     "the recorded tunes are not on disk")
class RecordedTuneTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.reads = {}
        for name in RECORDED:
            history = history_from_result(RESULTS / name, pathlib.Path(cls.tmp.name) / f"{name}.json")
            run = sr.load_run(RESULTS / name, cli=None, history_json=history)
            cls.reads[name] = (run, sr.read(run, 2000, null_reps=3))

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_every_recorded_move_is_the_runners_update(self):
        # The signs are regenerated from each run's master seed; every centre
        # the runner recorded must be its update of the one before.
        for name, (run, _) in self.reads.items():
            with self.subTest(run=name):
                self.assertLess(run.sources["move_check_worst_error"], 1e-9)

    def test_the_recorded_reads_at_2000(self):
        for name, (max_z, line, excess, over, verdict) in RECORDED.items():
            _, result = self.reads[name]
            with self.subTest(run=name):
                self.assertAlmostEqual(result["max_abs_z"], max_z, places=2)
                self.assertAlmostEqual(result["familywise_line"], line, places=2)
                self.assertAlmostEqual(result["excess_sd"], excess, places=1)
                self.assertEqual(result["over_line"], over)
                self.assertEqual(result["verdict"], verdict)


if __name__ == "__main__":
    unittest.main()
