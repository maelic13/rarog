"""Tests for spsa_theta_audit.py: the schedule wire and the three lists."""

import sys
import unittest
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import spsa_theta_audit  # noqa: E402

ALPHA, GAMMA, STABILITY = 0.601, 0.102, 10.0


def synthetic(signs: dict, seeds: dict, steps: dict, ranges: dict, a0: float = 0.15):
    """A finished run whose every step follows the schedule: coordinate n moves
    by a_k / c_k * |difference_k| in direction signs[n][k]. Returns the result
    and its config."""
    names = list(signs)
    horizon = len(next(iter(signs.values())))
    diffs = [3.0] * horizon
    centre = np.array([float(seeds[n]) for n in names])
    c0 = np.array([float(steps[n]) for n in names])
    journal = []
    for k in range(horizon):
        kk = k + 1
        size = (a0 / (kk + STABILITY) ** ALPHA) / (c0 / kk ** GAMMA) * abs(diffs[k])
        centre = centre + size * np.array([signs[n][k] for n in names])
        journal.append({"iteration": k, "centers_after": list(centre), "score": {"difference": diffs[k]}})
    tuned = np.sign(centre) * np.floor(np.abs(centre) + 0.5)
    result = {
        "driver": {"status": "completed", "completed_iterations": journal},
        "schedule": {"schedule": {"alpha": ALPHA, "gamma": GAMMA, "stability_constant": STABILITY},
                     "knobs": [{"name": n, "c0": steps[n], "a0": a0} for n in names]},
        "tuned_result": {"settings": {"iterations": horizon}, "completed_iterations": horizon,
                         "parameters": [{"name": n, "original": seeds[n], "tuned": float(t),
                                         "min": ranges[n][0], "max": ranges[n][1]}
                                        for n, t in zip(names, tuned)]},
    }
    config = {n: {"value": seeds[n], "step": steps[n], "min_value": ranges[n][0], "max_value": ranges[n][1]}
              for n in names}
    return result, config


class Lists(unittest.TestCase):
    def setUp(self):
        n = 40
        self.result, self.config = synthetic(
            signs={
                "Mover": [1] * n,                          # travels out and stays
                "Still": [1, -1] * (n // 2),               # goes nowhere
                "Railed": [-1] * n,                        # runs into its min
                "Overshot": [1] * 20 + [-1] * 20,          # out, then back past a step
                "WrongWayFirst": [-1] * 16 + [1] * 24,     # a step against, then back
                "Drifted": [1] * 22 + [-1] * 6 + [1, -1] * 6,  # out, back part of a step
            },
            seeds={"Mover": 50, "Still": 50, "Railed": 4, "Overshot": 50, "WrongWayFirst": 50, "Drifted": 50},
            steps={"Mover": 1, "Still": 1, "Railed": 1, "Overshot": 1, "WrongWayFirst": 1, "Drifted": 1},
            ranges={n: (0, 200) for n in
                    ("Mover", "Still", "Railed", "Overshot", "WrongWayFirst", "Drifted")},
        )
        self.run = spsa_theta_audit.load(self.result, self.config)
        self.lists = spsa_theta_audit.classify(self.run, self.run["c0"])

    def test_movers_are_named_by_net_travel(self):
        moved = self.lists["steps_moved"]
        self.assertIn("Mover", self.lists["full_step_movers"])
        self.assertNotIn("Still", self.lists["full_step_movers"])
        self.assertGreaterEqual(moved["Mover"], 1)
        self.assertLess(abs(moved["Still"]), 1)

    def test_a_value_at_or_near_a_range_end_is_a_rail(self):
        rails = {r["name"]: r for r in self.lists["rails"]}
        self.assertEqual(set(rails), {"Railed"})
        self.assertEqual(rails["Railed"]["end"], "min")
        self.assertEqual(rails["Railed"]["edge"], 0)

    def test_reversals_need_a_full_step_against_the_finish(self):
        names = {r["name"] for r in self.lists["reversals"]}
        self.assertIn("Overshot", names)
        self.assertIn("WrongWayFirst", names)
        self.assertNotIn("Mover", names)
        self.assertNotIn("Drifted", names)
        self.assertNotIn("Still", names)

    def test_the_unit_can_come_from_another_config(self):
        lists = spsa_theta_audit.classify(self.run, self.run["c0"] * 100)
        self.assertEqual(lists["full_step_movers"], [])
        self.assertEqual(lists["reversals"], [])


class Wires(unittest.TestCase):
    def make(self):
        return synthetic(signs={"A": [1, -1, 1, 1], "B": [1, 1, -1, 1]}, seeds={"A": 10, "B": 20},
                         steps={"A": 2, "B": 3}, ranges={"A": (0, 50), "B": (0, 50)})

    def test_a_clean_run_loads(self):
        result, config = self.make()
        self.assertEqual(spsa_theta_audit.load(result, config)["clamped_steps"], 0)

    def test_an_unfinished_run_is_refused(self):
        result, config = self.make()
        result["driver"]["status"] = "stopped"
        with self.assertRaises(AssertionError):
            spsa_theta_audit.load(result, config)

    def test_seeds_other_than_the_config_are_refused(self):
        result, config = self.make()
        config["A"]["value"] = 11
        with self.assertRaises(AssertionError):
            spsa_theta_audit.load(result, config)

    def test_steps_off_the_schedule_are_refused(self):
        result, config = self.make()
        for it in result["driver"]["completed_iterations"]:
            it["score"]["difference"] *= 2
        with self.assertRaises(AssertionError):
            spsa_theta_audit.load(result, config)

    def test_a_tuned_value_that_is_not_the_rounded_centre_is_refused(self):
        result, config = self.make()
        result["tuned_result"]["parameters"][0]["tuned"] += 1
        with self.assertRaises(AssertionError):
            spsa_theta_audit.load(result, config)


class CheckpointTests(unittest.TestCase):
    """A rule 7c tune read where it stopped, through the reader's loader."""

    def setUp(self):
        import importlib.util
        import tempfile
        spec = importlib.util.spec_from_file_location(
            "test_spsa_read_for_audit", Path(__file__).resolve().parent / "test_spsa_read.py")
        self.helpers = importlib.util.module_from_spec(spec)
        sys.modules["test_spsa_read_for_audit"] = self.helpers
        spec.loader.exec_module(self.helpers)
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = self.helpers.simulate(Path(self.tmp.name) / "run", gradients=[0.12, 0.0], starts=[30, 50],
                                         bounds=[(0, 100), (0, 100)], c0=[8.0, 6.0], ceiling=3000, completed=2000)
        # Registered steps of 4 and 6: coordinate 0 perturbs by two, coordinate 1 by one.
        self.config = {"K0": {"value": 30, "min_value": 0, "max_value": 100, "step": 4},
                       "K1": {"value": 50, "min_value": 0, "max_value": 100, "step": 6}}

    def tearDown(self):
        self.tmp.cleanup()

    def test_theta_is_the_checkpoints_rounded_centres(self):
        run = spsa_theta_audit.load_checkpoint(self.dir, self.config, 2000, cli=None,
                                               history_json=self.dir / "history.json")
        history = __import__("json").loads((self.dir / "history.json").read_text(encoding="utf-8"))
        centres = history["report"]["rows"][2000]["centers"]
        self.assertEqual(list(run["tuned"]), [float(self.helpers.sr.round_half_away(x)) for x in centres])
        self.assertEqual(run["horizon"], 2000)
        self.assertEqual(run["path"].shape, (2001, 2))
        # A full step is the registered step, not the graded perturbation.
        self.assertEqual(list(run["step"]), [4.0, 6.0])

    def test_an_earlier_checkpoint_reads_the_path_to_it(self):
        run = spsa_theta_audit.load_checkpoint(self.dir, self.config, 1000, cli=None,
                                               history_json=self.dir / "history.json")
        self.assertEqual(run["path"].shape, (1001, 2))

    def test_a_checkpoint_past_the_run_is_refused(self):
        with self.assertRaises(AssertionError):
            spsa_theta_audit.load_checkpoint(self.dir, self.config, 2500, cli=None,
                                             history_json=self.dir / "history.json")

    def test_a_perturbation_outside_one_to_two_steps_is_refused(self):
        config = dict(self.config, K0=dict(self.config["K0"], step=3))
        with self.assertRaises(AssertionError):
            spsa_theta_audit.load_checkpoint(self.dir, config, 2000, cli=None,
                                             history_json=self.dir / "history.json")


if __name__ == "__main__":
    unittest.main()
