"""Tests for the Colosseum/fastchess configuration parity check.

Two things are checked, and the second is the one that matters. First, the
recorded pair agrees: `sprt.ps1`'s manifest for RAR-S77's gate and the Colosseum
dry run of the same two binaries at the same cap and seed resolve to the same
conditions in every comparable field. Second, each comparison is LIVE: mutating
one field of the dry run must make that field, and no other, report a
difference. A parity check that cannot fail proves nothing.

Fixtures are in `colosseum_parity_v1/`. The two rows that hash local binaries
are checked only when those binaries are present, so the suite still runs on a
host that does not carry them.
"""

import copy
import json
import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import colosseum_parity

FIXTURES = pathlib.Path(__file__).resolve().parent / "colosseum_parity_v1"
MANIFEST = FIXTURES / "sprt_theta5000_vs_theta3900_20260920_082658.manifest.txt"
DRY_RUN = FIXTURES / "colosseum_sprt_theta5000_vs_theta3900.dry-run.json"
NEEDS_LOCAL_FILE = {"engineA.sha256", "engineB.sha256", "book_sha256"}


def rows_for(dry):
    manifest = colosseum_parity.read_manifest(MANIFEST)
    return {row["field"]: row for row in colosseum_parity.compare(manifest, dry)}


def load_dry():
    return json.loads(DRY_RUN.read_text(encoding="utf-8"))


class ParityTests(unittest.TestCase):
    def test_the_recorded_pair_agrees_in_every_comparable_field(self):
        rows = rows_for(load_dry())
        self.assertGreaterEqual(len(rows), 25, "the comparison lost fields")
        for field, row in rows.items():
            if field in NEEDS_LOCAL_FILE and str(row["colosseum"]).startswith("("):
                continue  # the binary or book is not on this host
            self.assertTrue(row["equal"], f"{field}: {row['fastchess']!r} vs {row['colosseum']!r}")

    def test_every_field_the_check_claims_is_a_field_it_would_notice(self):
        # One mutation per comparison class. Each must flip its own row and
        # leave the rest alone; a check that cannot fail is not a check.
        mutations = {
            "sprt.elo1": lambda d: d["resolved_configuration"]["design"]["parameters"].update({"elo1": 10.0}),
            "sprt.model": lambda d: d["resolved_configuration"]["design"]["parameters"].update({"model": "logistic"}),
            "game_budget": lambda d: d["resolved_configuration"]["design"].update({"max_pairs": 19999}),
            "engine_a_time_control.base_ms": lambda d: d["resolved_configuration"]["engine_a_time_control"]["control"]["Increment"].update({"base_ms": 10000}),
            "engine_b_time_control.margin_ms": lambda d: d["resolved_configuration"]["engine_b_time_control"].update({"margin_ms": 2000}),
            "adjudication": lambda d: d["resolved_configuration"]["adjudication"].update({"resign": {"moves": 3, "score_cp": 600}}),
            # Naming a different executable moves its hash row too, which is the
            # point of that row; it is declared rather than suppressed.
            "engineA.executable": lambda d: d["resolved_configuration"]["engine_a"].update({"executable": r"D:\code\rarog\tools\test_engines\other.exe"}),
            "engineA.Hash": lambda d: d["resolved_configuration"]["engine_a"]["options"]["Hash"].update({"value": 256}),
            "engineB.Threads": lambda d: d["resolved_configuration"]["engine_b"]["options"]["Threads"].update({"value": 4}),
            "engineA.extra_options": lambda d: d["resolved_configuration"]["engine_a"]["options"].update({"CoreRazorGuards": {"kind": "string", "value": 1}}),
            "opening_order": lambda d: d["resolved_configuration"]["openings"].update({"order": "Sequential"}),
            "opening_seed": lambda d: d["resolved_configuration"].update({"master_seed": 1}),
            "concurrency": lambda d: d["resolved_configuration"]["execution"].update({"concurrency": 16}),
            "affinity_cpus": lambda d: d["resolved_configuration"]["execution"]["slots"][0]["engine_a"]["allocation"]["cpus"].append({"group": 0, "number": 0}),
        }
        expected_collateral = {"engineA.executable": ["engineA.sha256"]}
        baseline = rows_for(load_dry())
        for field, mutate in mutations.items():
            with self.subTest(field=field):
                dry = load_dry()
                mutate(dry)
                rows = rows_for(dry)
                self.assertFalse(rows[field]["equal"], f"mutating {field} did not fail its own check")
                collateral = [
                    other
                    for other, row in rows.items()
                    if other != field and row["equal"] != baseline[other]["equal"]
                ]
                self.assertEqual(
                    collateral,
                    expected_collateral.get(field, []),
                    f"mutating {field} also moved {collateral}",
                )

    def test_a_book_change_is_noticed(self):
        dry = load_dry()
        dry["resolved_configuration"]["openings"]["path"] = r"D:\code\rarog\tools\books\IM_4mvs.pgn"
        rows = rows_for(dry)
        self.assertFalse(rows["book"]["equal"])

    def test_an_adjudicated_run_never_reports_parity_with_an_unadjudicated_one(self):
        dry = load_dry()
        dry["resolved_configuration"]["adjudication"]["draw"] = {"move": 40, "moves": 8, "score_cp": 10}
        rows = rows_for(dry)
        self.assertFalse(rows["adjudication"]["equal"])

    def test_the_comparison_refuses_a_file_that_is_not_a_manifest(self):
        with self.assertRaises(SystemExit):
            colosseum_parity.read_manifest(DRY_RUN)


if __name__ == "__main__":
    unittest.main()
