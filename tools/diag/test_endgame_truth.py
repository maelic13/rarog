"""Unit tests for the endgame truth harness's termination rule (PLAN 4.10.1).

These are the tests RAR-E14 says were missing. The defect they cover is not a
crash: the old harness ended a game the moment the strong side's piece count
dropped, which reads as a plausible "material lost" outcome and silently scores
correct pawn technique as a failure. Nothing about the output looked wrong.

So each test below asserts a BEHAVIOUR, and the schema tests assert that the
guards FAIL on a known-bad input rather than merely pass on a good one -- which
is 4.10.4's rule applied to the change that motivated it.

Run:
  python -m unittest discover -s tools/diag -p "test_*.py"
"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

import chess

sys.path.insert(0, str(Path(__file__).resolve().parent))
import endgame_budget_bracket
import endgame_floors
import endgame_truth


class StubEngine:
    """Plays a scripted move list, then the first legal move forever.

    `play_and_grade` only needs `.play(board, limit, game=...)`. Scripting the
    moves is what makes the termination rule testable without an engine binary,
    a tablebase or a node budget.
    """

    def __init__(self, moves):
        self.moves = list(moves)
        self.calls = 0

    def play(self, board, limit, game=None):
        self.calls += 1
        if self.moves:
            move = chess.Move.from_uci(self.moves.pop(0))
        else:
            move = next(iter(board.legal_moves))

        class Result:
            pass

        result = Result()
        result.move = move
        return result


class StubTablebase:
    """WDL/DTZ from a caller-supplied table, defaulting to a clean White win."""

    def __init__(self, wdl=2, dtz=20):
        self.wdl = wdl
        self.dtz = dtz

    def probe_wdl(self, board):
        # play_and_grade converts to White's point of view by the side to move.
        return self.wdl if board.turn == chess.WHITE else -self.wdl

    def probe_dtz(self, board):
        return self.dtz


class TerminationRuleTests(unittest.TestCase):
    def test_pawn_sacrifice_does_not_end_the_game(self):
        """The defect itself: KPP-K, White gives one pawn to promote the other.

        Under the old rule this returned `material_lost` at ply 1 with the win
        still intact. It must now play on and record the shed ply instead.
        """
        # White Kd6, pawns c7 and d5; Black Kc8 to move and able to take on c7.
        # Losing the c-pawn leaves K+P vs K, which is NOT insufficient material
        # and is often still won -- exactly the case the old rule aborted.
        board = chess.Board("2k5/2P5/3K4/3P4/8/8/8/8 b - - 0 1")
        self.assertEqual(chess.popcount(board.occupied_co[chess.WHITE]), 3)

        played = endgame_truth.play_and_grade(
            StubEngine(["c8c7"]), StubTablebase(), board,
            nodes=1, max_plies=12, game_token=object(),
        )
        self.assertNotEqual(played["outcome"], "material_lost")
        self.assertIsNotNone(played["shed_material_ply"])
        self.assertGreater(played["plies"], played["shed_material_ply"])

    def test_material_lost_is_no_longer_a_possible_outcome(self):
        """The whole point of 4.10.1: nothing terminates on material any more."""
        source = Path(endgame_truth.__file__).read_text(encoding="utf-8")
        self.assertNotIn('outcome = "material_lost"', source)

    def test_shed_ply_records_the_first_drop_only(self):
        # The capture happens on ply 0, so the drop is first observed by the
        # ply-1 check. A later second capture must not overwrite it.
        board = chess.Board("2k5/2P5/3K4/3P4/8/8/8/8 b - - 0 1")
        played = endgame_truth.play_and_grade(
            StubEngine(["c8c7"]), StubTablebase(), board,
            nodes=1, max_plies=20, game_token=object(),
        )
        self.assertEqual(played["shed_material_ply"], 1)

    def test_bare_king_shed_is_caught_by_insufficient_material_first(self):
        """The isolation proof, asserted rather than assumed.

        In every bare-king family a strong-side loss reaches a position the
        insufficient-material test terminates on. If that ordering is ever
        changed, this fails.
        """
        for fen, name in [
            ("4k3/8/8/8/8/8/8/4K1N1 b - - 0 1", "KN-K after a KNN-K shed"),
            ("4k3/8/8/8/8/8/8/4K1B1 b - - 0 1", "KB-K after a KBN-K shed"),
        ]:
            with self.subTest(name):
                board = chess.Board(fen)
                self.assertTrue(board.is_insufficient_material())
                played = endgame_truth.play_and_grade(
                    StubEngine([]), StubTablebase(wdl=0), board,
                    nodes=1, max_plies=8, game_token=object(),
                )
                self.assertEqual(played["outcome"], "insufficient_material")

    def test_report_records_the_new_diagnostic(self):
        board = chess.Board("2k5/2P5/3K4/3P4/8/8/8/8 b - - 0 1")
        played = endgame_truth.play_and_grade(
            StubEngine(["c8c7"]), StubTablebase(), board,
            nodes=1, max_plies=12, game_token=object(),
        )
        self.assertIn("shed_material_ply", played)
        self.assertIn("first_discard_ply", played)


class CohortIdentityTests(unittest.TestCase):
    """4.10.2: a report must be able to say which positions it measured.

    The golden digests below were verified position-for-position, in order,
    against `tools/results/e08-accepted/endgame-truth.json` -- 1900/1900 -- so
    they pin the generator that produced every artifact currently on disk. If
    one of these changes, the position set changed, and every baseline measured
    on it is superseded whether or not anyone noticed.
    """

    SEED = 6200600
    KBP_KB = "b730954492fafc8a30a8a3a4ee6e6d83eb3fdf8031fa8a9e1a6584eb830d32cb"
    OVERALL = "fe4866045506636f884ee30526b4188c3def9ca9747f5960ea5c5e7cba5dbb5e"

    def _digest(self, name, positions=100, seed=None):
        strong, weak = endgame_truth.parse_family(name)
        boards = endgame_truth.generate_family(
            self.SEED if seed is None else seed, name, strong, weak, positions)
        return endgame_truth.cohort_digest(b.fen() for b in boards)

    def test_family_digest_matches_the_shipped_generator(self):
        self.assertEqual(self._digest("KBP-KB"), self.KBP_KB)

    def test_overall_digest_matches_the_shipped_generator(self):
        names = endgame_truth.DEFAULT_FAMILIES
        per_family = {n: self._digest(n) for n in names}
        self.assertEqual(
            endgame_truth.overall_cohort_digest(names, per_family), self.OVERALL)

    def test_digest_changes_with_the_seed(self):
        self.assertNotEqual(self._digest("KBP-KB", seed=self.SEED + 1),
                            self.KBP_KB)

    def test_digest_changes_with_the_position_count(self):
        self.assertNotEqual(self._digest("KBP-KB", positions=99), self.KBP_KB)

    def test_a_family_does_not_depend_on_what_else_was_generated(self):
        """Why a subset run is comparable with a full run.

        The family seed derives from the family NAME, so generating other
        families first cannot shift this one. Under the old index seeding it
        did, and `--families KBP-KB` silently measured 34 theoretical wins
        where the full run measured 47.
        """
        for other in ("KQ-K", "KRP-KR", "KP-KP"):
            self._digest(other, positions=7)
        self.assertEqual(self._digest("KBP-KB"), self.KBP_KB)

    def test_family_seeds_are_distinct(self):
        seen = {endgame_truth.family_seed(self.SEED, n): n
                for n in endgame_truth.DEFAULT_FAMILIES}
        self.assertEqual(len(seen), len(endgame_truth.DEFAULT_FAMILIES))


class CohortGuardTests(unittest.TestCase):
    """The guard must FAIL on two runs over different positions (4.10.4)."""

    def _report(self, digests):
        return {
            "schema": "rarog-endgame-truth-v2",
            "cohort": {"seed": 1, "positions_per_family": 2,
                       "sha256": "whatever",
                       "family_sha256": dict(digests)},
            "families": {name: {"cohort_sha256": d}
                         for name, d in digests.items()},
        }

    def _floors(self, digests):
        return {"schema": "rarog-endgame-floors-v2",
                "truth_schema": endgame_floors.TRUTH_SCHEMA,
                "cohort": {"family_sha256": dict(digests)},
                "families": {}}

    def test_matching_cohorts_are_accepted(self):
        d = {"KRP-KR": "aa" * 32, "KQ-K": "bb" * 32}
        endgame_floors.check_cohorts(self._floors(d), self._report(d))

    def test_a_differing_family_is_refused(self):
        floors = self._floors({"KRP-KR": "aa" * 32, "KQ-K": "bb" * 32})
        report = self._report({"KRP-KR": "cc" * 32, "KQ-K": "bb" * 32})
        with self.assertRaises(SystemExit) as caught:
            endgame_floors.check_cohorts(floors, report)
        message = str(caught.exception)
        self.assertIn("KRP-KR", message)
        self.assertNotIn("KQ-K", message)

    def test_a_subset_report_is_still_comparable(self):
        floors = self._floors({"KRP-KR": "aa" * 32, "KQ-K": "bb" * 32})
        endgame_floors.check_cohorts(floors, self._report({"KRP-KR": "aa" * 32}))

    def test_floors_without_cohort_digests_are_refused(self):
        floors = {"schema": "rarog-endgame-floors-v2",
                  "truth_schema": endgame_floors.TRUTH_SCHEMA, "families": {}}
        with self.assertRaises(SystemExit) as caught:
            endgame_floors.check_cohorts(floors, self._report({"KQ-K": "bb" * 32}))
        self.assertIn("4.11.2", str(caught.exception))

    def test_a_report_without_a_digest_is_refused(self):
        report = {"families": {"KQ-K": {}}}
        with self.assertRaises(SystemExit) as caught:
            endgame_floors.cohorts(report)
        self.assertIn("cohort_sha256", str(caught.exception))


class ShardTests(unittest.TestCase):
    """4.10.3: sharding may change wall time and nothing else."""

    def test_every_item_appears_exactly_once(self):
        items = [("f", i, "fen%d" % i) for i in range(23)]
        for workers in (1, 2, 5, 23):
            with self.subTest(workers=workers):
                buckets = endgame_truth.shard(items, workers)
                flat = [x for b in buckets for x in b]
                self.assertEqual(sorted(flat), sorted(items))
                self.assertEqual(len(flat), len(items))

    def test_empty_buckets_are_dropped(self):
        buckets = endgame_truth.shard([("f", 0, "a")], 8)
        self.assertEqual(len(buckets), 1)
        self.assertTrue(all(buckets))

    def test_one_worker_preserves_order(self):
        items = [("f", i, "fen%d" % i) for i in range(6)]
        self.assertEqual(endgame_truth.shard(items, 1), [items])

    def test_round_robin_spreads_a_family(self):
        """A slow family must not land wholly on one worker."""
        items = [("slow", i, "x") for i in range(10)]
        buckets = endgame_truth.shard(items, 5)
        self.assertEqual([len(b) for b in buckets], [2, 2, 2, 2, 2])

    def test_workers_hold_no_module_level_engine(self):
        """Regression guard for a deadlock this step actually hit.

        The first version kept the engine in a module global filled by a pool
        initializer and closed by `atexit`. Every shard finished, `24/24
        positions` printed, and the pool then hung forever with five live
        `rarog.exe` children -- closing a python-chess engine from an `atexit`
        handler races its asyncio loop thread. The engine's lifetime is now the
        task's, explicitly.
        """
        source = Path(endgame_truth.__file__).read_text(encoding="utf-8")
        # Match the CONSTRUCTS, not the words: the comment above `_worker_run`
        # names the bug on purpose, and a bare `assertNotIn("atexit", ...)`
        # failed on that comment rather than on any code.
        self.assertNotIn("import atexit", source)
        self.assertNotIn("atexit.register", source)
        self.assertNotIn("initializer=", source)
        self.assertNotIn("initargs=", source)


def _family(rate, n, cohort="aa" * 32):
    """A truth-report family entry with the three floor metrics at one rate."""
    return {
        "spec": "X", "theory": {}, "outcomes": {},
        "theoretically_won": n, "converted": int(round(rate * n)),
        "conversion_rate": rate,
        "graded_moves": n, "win_preserving_moves": int(round(rate * n)),
        "win_preserving_rate": rate,
        "dtz_checked_moves": n, "dtz_progress_moves": int(round(rate * n)),
        "dtz_progress_rate": rate,
        "cohort_sha256": cohort,
    }


def _report(families):
    return {
        "schema": "rarog-endgame-truth-v2",
        "cohort": {"seed": 1, "positions_per_family": 100, "sha256": "z",
                   "family_sha256": {k: v["cohort_sha256"]
                                     for k, v in families.items()}},
        "families": families,
    }


class ThinSampleTests(unittest.TestCase):
    """4.10.4: refuse to report a statistic over an eligible set that is tiny.

    The failure prevented is a CONFIDENT wrong reading, not a wrong verdict: a
    family with one eligible position that fails reads as 0.0%, which looks
    like catastrophe and is emptiness.
    """

    def test_a_thin_family_is_not_given_a_rate(self):
        report = _report({"THIN": _family(0.0, 1), "REAL": _family(0.5, 40)})
        self.assertNotIn("THIN", endgame_floors.rates(report))
        self.assertIn("REAL", endgame_floors.rates(report))

    def test_a_thin_family_is_reported_as_thin_rather_than_dropped(self):
        report = _report({"THIN": _family(0.0, 1)})
        self.assertEqual(
            [(f, m) for f, m, _ in endgame_floors.thin(report)],
            [("THIN", "conversion_rate"), ("THIN", "dtz_progress_rate"),
             ("THIN", "win_preserving_rate")],
        )

    def test_the_boundary_is_inclusive(self):
        at = _report({"F": _family(0.5, endgame_floors.MIN_ELIGIBLE)})
        under = _report({"F": _family(0.5, endgame_floors.MIN_ELIGIBLE - 1)})
        self.assertIn("F", endgame_floors.rates(at))
        self.assertNotIn("F", endgame_floors.rates(under))


class FloorGateTests(unittest.TestCase):
    """The floor gate must BLOCK on a regression, not merely pass on equality."""

    def _run(self, floors_families, report_families, sigma="3"):
        import subprocess
        with tempfile.TemporaryDirectory() as d:
            report = _report(report_families)
            rp = Path(d) / "truth.json"
            rp.write_text(json.dumps(report), encoding="utf-8")
            fl = Path(d) / "floors.json"
            fl.write_text(json.dumps({
                "schema": "rarog-endgame-floors-v2",
                "truth_schema": endgame_floors.TRUTH_SCHEMA,
                "cohort": {"family_sha256": {k: v["cohort_sha256"]
                                             for k, v in floors_families.items()}},
                "families": endgame_floors.rates(_report(floors_families)),
            }), encoding="utf-8")
            proc = subprocess.run(
                [sys.executable, str(Path(endgame_floors.__file__)),
                 "--report", str(rp), "--floors", str(fl), "--sigma", sigma],
                capture_output=True, text=True)
            return proc.returncode, proc.stdout + proc.stderr

    def test_an_equal_report_passes(self):
        fam = {"KQ-K": _family(0.95, 100)}
        code, out = self._run(fam, fam)
        self.assertEqual(code, 0, out)

    def test_a_large_family_regression_blocks(self):
        code, out = self._run({"KQ-K": _family(0.95, 100)},
                              {"KQ-K": _family(0.50, 100)})
        self.assertEqual(code, 1, out)
        self.assertIn("BELOW FLOOR", out)

    def test_a_small_family_dip_does_not_block(self):
        code, out = self._run({"KQ-K": _family(0.95, 100)},
                              {"KQ-K": _family(0.94, 100)})
        self.assertEqual(code, 0, out)

    def test_a_thin_family_cannot_manufacture_a_verdict(self):
        """n=1 at 0% must not read as a catastrophic regression."""
        code, out = self._run({"THIN": _family(1.0, 1), "KQ-K": _family(0.95, 100)},
                              {"THIN": _family(0.0, 1), "KQ-K": _family(0.95, 100)})
        self.assertEqual(code, 0, out)
        self.assertIn("thin samples", out)


class MeasurementLayerTests(unittest.TestCase):
    """4.10.5: an instrument must say which QUESTION it answers.

    Enforced rather than documented, because the contract's whole purpose is to
    stop a number being read as answering something it never asked -- which is
    what 4.9a.7 nearly did to a working scale function, and what RAR-E14 found
    a conversion baseline doing.
    """

    DIAG = Path(__file__).resolve().parent
    CONTRACT = DIAG.parents[1] / "analysis" / "endgame_measurement_layers.md"

    def test_the_contract_exists_and_names_all_four_layers(self):
        text = self.CONTRACT.read_text(encoding="utf-8")
        for layer in ("Theory truth", "Move quality", "Conversion",
                      "Game strength", "Occurrence"):
            self.assertIn(layer, text)

    def test_the_contract_states_the_precedence_rules(self):
        text = self.CONTRACT.read_text(encoding="utf-8")
        self.assertIn("Truth is an absolute veto", text)
        self.assertIn("Conversion NEVER establishes strength", text)
        self.assertIn("Layers are never aggregated", text)
        self.assertIn("Bench identity is provenance", text)

    def test_every_playing_instrument_stamps_its_layer(self):
        for tool in ("endgame_truth.py", "endgame_conversion.py",
                     "endgame_drawn.py"):
            with self.subTest(tool):
                source = (self.DIAG / tool).read_text(encoding="utf-8")
                self.assertRegex(source, r'"layers?":')

    def test_the_truth_report_declares_all_four_layers(self):
        source = (self.DIAG / "endgame_truth.py").read_text(encoding="utf-8")
        for key in ("1_theory_truth", "2_move_quality", "3_conversion",
                    "4_game_strength"):
            self.assertIn(key, source)

    def test_the_truth_report_disclaims_strength(self):
        source = (self.DIAG / "endgame_truth.py").read_text(encoding="utf-8")
        self.assertIn("NOT MEASURED HERE", source)


class BudgetBracketTests(unittest.TestCase):
    """4.10.6: one budget is a guess, and a bracket must not mix cohorts."""

    def test_budgets_parse_sorted_and_deduplicated(self):
        self.assertEqual(
            endgame_budget_bracket.parse_budgets("600000, 60000,200000"),
            [60000, 200000, 600000])

    def test_a_duplicate_budget_is_refused(self):
        with self.assertRaises(ValueError):
            endgame_budget_bracket.parse_budgets("60000,60000")

    def test_a_nonpositive_budget_is_refused(self):
        for spec in ("0", "-1", ""):
            with self.subTest(spec), self.assertRaises(ValueError):
                endgame_budget_bracket.parse_budgets(spec)

    def _arm(self, digest):
        return {"cohort": {"sha256": digest}, "families": {}}

    def test_matching_cohorts_tabulate(self):
        arms = {60000: self._arm("aa" * 32), 200000: self._arm("aa" * 32)}
        self.assertIsNone(endgame_budget_bracket.cohorts_agree(arms))

    def test_a_bracket_over_different_positions_is_refused(self):
        """Otherwise the bracket is RAR-E14 defect B with extra steps."""
        arms = {60000: self._arm("aa" * 32), 200000: self._arm("bb" * 32)}
        problem = endgame_budget_bracket.cohorts_agree(arms)
        self.assertIsNotNone(problem)
        self.assertIn("different position sets", problem)

    def test_a_missing_digest_is_refused(self):
        arms = {60000: self._arm("aa" * 32), 200000: {"families": {}}}
        self.assertIn("no cohort digest",
                      endgame_budget_bracket.cohorts_agree(arms))


class NodeBudgetEvidenceTests(unittest.TestCase):
    """The measured deployment budget must stay recorded, not remembered."""

    NOTE = (Path(__file__).resolve().parents[2] / "analysis"
            / "node_budget_2026-09-04.md")

    def test_the_measurement_is_recorded(self):
        text = self.NOTE.read_text(encoding="utf-8")
        self.assertIn("153,466", text)
        self.assertIn("3+0.03", text)

    def test_it_states_the_gap_to_the_screen_budget(self):
        text = self.NOTE.read_text(encoding="utf-8")
        self.assertIn("60,000", text)
        self.assertIn("PROVISIONAL", text)


class SchemaGuardTests(unittest.TestCase):
    """4.10.4: a guard is not verified until it FAILS on a known-bad input."""

    def _write(self, directory, name, doc):
        path = Path(directory) / name
        path.write_text(json.dumps(doc), encoding="utf-8")
        return path

    def test_floors_reject_a_v1_truth_report(self):
        with tempfile.TemporaryDirectory() as d:
            bad = self._write(d, "truth.json", {
                "schema": "rarog-endgame-truth-v1", "families": {}})
            with self.assertRaises(SystemExit) as caught:
                endgame_floors.load_report(bad)
            self.assertIn("v1", str(caught.exception))

    def test_floors_accept_a_v2_truth_report(self):
        with tempfile.TemporaryDirectory() as d:
            good = self._write(d, "truth.json", {
                "schema": "rarog-endgame-truth-v2", "families": {}})
            self.assertEqual(endgame_floors.load_report(good)["families"], {})

    def test_the_committed_floors_carry_their_provenance(self):
        """Inverted at 4.11.2, when the floors were re-derived.

        Between 4.10.1 and 4.11.2 this asserted the OPPOSITE -- that the
        committed floors were stale and must fail closed -- which was true and
        was the point. 4.11.2 rebuilt them from the corrected head arm, so the
        assertion flips to the state that must now hold and must not silently
        regress: stamped with the truth schema that produced them, and carrying
        the cohort they were measured on.

        Nothing is weakened by the flip. That a MISMATCHED floors file is
        refused is covered independently, on synthetic inputs, by
        `test_floors_reject_a_v1_truth_report` and by `CohortGuardTests` --
        those keep proving the guard fires regardless of what the committed
        file happens to contain.
        """
        doc = json.loads(
            endgame_floors.DEFAULT_FLOORS.read_text(encoding="utf-8"))
        self.assertEqual(doc.get("truth_schema"), endgame_floors.TRUTH_SCHEMA)
        self.assertIn("family_sha256", doc.get("cohort", {}))
        self.assertTrue(doc["cohort"]["family_sha256"])

    def test_the_superseded_floors_are_kept_and_still_refused(self):
        """v1 is preserved unmodified as the historical artifact."""
        v1 = endgame_floors.DEFAULT_FLOORS.with_name("endgame_floors_v1.json")
        self.assertTrue(v1.is_file(), "the superseded floors must not be deleted")
        doc = json.loads(v1.read_text(encoding="utf-8"))
        self.assertNotEqual(doc.get("truth_schema"), endgame_floors.TRUTH_SCHEMA)

    def test_truth_runner_emits_v2(self):
        source = Path(endgame_truth.__file__).read_text(encoding="utf-8")
        self.assertIn('"schema": "rarog-endgame-truth-v2"', source)


if __name__ == "__main__":
    unittest.main()
