"""Tests for tools/spsa_config_to_colosseum.py: rule 7c's graded perturbation,
its tune and run files, and every committed group's files still current."""

from __future__ import annotations

import importlib.util
import json
import math
import pathlib
import re
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
CONVERTER = ROOT / "tools" / "spsa_config_to_colosseum.py"


def load(name: str, path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader
    spec.loader.exec_module(module)
    return module


conv = load("spsa_config_to_colosseum", CONVERTER)


def c_ends(tune: str) -> dict[str, float]:
    values = {}
    for block in tune.split("[[parameters]]")[1:]:
        fields = dict(line.split(" = ", 1) for line in block.strip().splitlines()
                      if " = " in line and not line.startswith("#"))
        values[fields["name"].strip('"')] = float(fields["c_end"])
    return values


class Rule7cTests(unittest.TestCase):
    def test_the_perturbation_is_graded_by_room(self):
        # Two steps where the seed has the room, else the largest from one to
        # two that stays inside the range, never under one step.
        self.assertEqual(conv.graded_perturbation(10, 0, 100, 4), (2.0, 2.5))
        self.assertEqual(conv.graded_perturbation(8, 0, 100, 4), (2.0, 2.0))
        self.assertEqual(conv.graded_perturbation(6, 0, 100, 4), (1.5, 1.5))
        self.assertEqual(conv.graded_perturbation(94, 0, 100, 4), (1.5, 1.5))
        self.assertEqual(conv.graded_perturbation(2, 0, 100, 4), (1.0, 0.5))

    def test_c_end_scales_by_the_graded_steps_and_rounds_down(self):
        surface = {
            "Wide": {"value": 50, "min_value": 0, "max_value": 100, "step": 12.5},
            "Edge": {"value": 18, "min_value": 0, "max_value": 100, "step": 12.5},
            "Tight": {"value": 5, "min_value": 0, "max_value": 100, "step": 12.5},
        }
        tune, report = conv.rule7c_tune_file("test", surface, 6000)
        decay = 6000 ** -conv.GAMMA
        down = lambda value: math.floor(value * 1e4) / 1e4  # noqa: E731
        values = c_ends(tune)
        self.assertAlmostEqual(values["Wide"], down(2 * 12.5 * decay))
        self.assertAlmostEqual(values["Edge"], down(18 / 12.5 * 12.5 * decay))
        self.assertAlmostEqual(values["Tight"], down(12.5 * decay))
        # The runner's c0 never exceeds the room, so no first arm is clamped.
        self.assertLessEqual(values["Edge"] * 6000 ** conv.GAMMA, 18.0)
        self.assertIn("rule 7c ceiling N = 6000;", tune)
        self.assertEqual(len(report), 2)
        self.assertIn("1.44 steps", report[0])
        self.assertIn("under one step of room", report[1])

    def test_the_run_file_carries_rule_7cs_gain_ratio(self):
        text = conv.run_file("test", {"Hash": 64, "Threads": 1}, "the rule 7c ceiling N = 6000", conv.RULE7C_R_END)
        self.assertIn("r-end = 0.00155\n", text)
        self.assertIn('option = ["Hash=64", "Threads=1"]', text)

    def test_the_block_form_and_rule_7c_are_exclusive(self):
        for argv in (["b43", "--rule7c"], ["b43", "--iterations", "2000", "--ceiling", "6000"],
                     ["b43", "--rule7c", "--ceiling", "6000", "--iterations", "2000"],
                     ["c10long", "--rule7c", "--ceiling", "6000", "--check"]):
            with self.subTest(argv=argv):
                process = subprocess.run([sys.executable, str(CONVERTER), *argv], capture_output=True, text=True)
                self.assertNotEqual(process.returncode, 0)


class CommittedGroupTests(unittest.TestCase):
    def test_every_committed_group_is_current(self):
        # Each group's files reproduce from its JSON at the horizon its own
        # header records, so the generator cannot drift from what was run.
        tunes = sorted((ROOT / "tools" / "spsa_configs" / "colosseum").glob("*.tune.toml"))
        self.assertTrue(tunes)
        for tune in tunes:
            group = tune.name.removesuffix(".tune.toml")
            header = tune.read_text(encoding="utf-8")
            argv = [sys.executable, str(CONVERTER), group, "--check"]
            if match := re.search(r"rule 7c ceiling N = (\d+);", header):
                argv += ["--rule7c", "--ceiling", match.group(1)]
            else:
                argv += ["--iterations", re.search(r"at N = (\d+) iterations", header).group(1)]
                schedule = ROOT / "tools" / "spsa_configs" / f"schedule_{group}.json"
                if schedule.is_file():
                    argv += ["--r-end", str(json.loads(schedule.read_text(encoding="utf-8"))["r_end"])]
            with self.subTest(group=group):
                process = subprocess.run(argv, capture_output=True, text=True)
                self.assertEqual(process.returncode, 0, process.stderr)


if __name__ == "__main__":
    unittest.main()
