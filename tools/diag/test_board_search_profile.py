import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

import chess


SCRIPT = Path(__file__).with_name("board_search_profile.py")
SPEC = importlib.util.spec_from_file_location("board_search_profile", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class BoardSearchProfileTests(unittest.TestCase):
    def test_frozen_suite_has_balanced_required_cohorts(self):
        cases = MODULE.load_suite(MODULE.DEFAULT_SUITE)
        counts = {}
        for case in cases:
            counts[case.cohort] = counts.get(case.cohort, 0) + 1
        self.assertEqual(set(counts), MODULE.REQUIRED_COHORTS)
        self.assertEqual(set(counts.values()), {4})
        self.assertEqual(len({case.name for case in cases}), 20)

    def test_frozen_cohort_labels_have_live_board_semantics(self):
        cases = MODULE.load_suite(MODULE.DEFAULT_SUITE)
        for case in cases:
            board = chess.Board(case.fen)
            self.assertTrue(board.is_valid(), case.name)
            self.assertFalse(board.is_game_over(), case.name)
            if case.cohort == "check-heavy":
                self.assertTrue(board.is_check(), case.name)
                self.assertTrue(any(board.legal_moves), case.name)
            elif case.cohort == "promotion":
                self.assertTrue(any(move.promotion for move in board.legal_moves), case.name)
            elif case.cohort == "sparse-endgame":
                self.assertLessEqual(len(board.piece_map()), 10, case.name)

    def test_loader_rejects_thin_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "thin.epd"
            rows = []
            for cohort in sorted(MODULE.REQUIRED_COHORTS):
                rows.append(
                    f"8/8/8/8/8/8/8/K6k w - - 0 1 ; cohort {cohort} ; "
                    f"name {cohort} ; src test\n"
                )
            path.write_text("".join(rows), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "at least four roots"):
                MODULE.load_suite(path)

    def test_parser_rejects_duplicate_counter_dump(self):
        case = MODULE.ProfileCase(
            "sample", "opening", "8/8/8/8/8/8/8/K6k w - - 0 1", "test"
        )
        lines = [
            "info depth 1 seldepth 1 score cp 0 nodes 1 nps 1 time 1 pv a1a2",
            "info string diag board_gen_vec_calls 1",
            "info string diag board_gen_vec_calls 1",
            "bestmove a1a2",
        ]
        with self.assertRaisesRegex(RuntimeError, "duplicate diagnostic counter"):
            MODULE.parse_search(lines, case, False)

    def test_identity_ignores_time_but_not_search_answer(self):
        common = {
            "name": "sample",
            "cohort": "opening",
            "fen": "fen",
            "source": "test",
            "repeat": 1,
            "depth": 4,
            "seldepth": 6,
            "reported_nodes": 100,
            "reported_nps": 1000,
            "reported_time_ms": 100,
            "score_type": "cp",
            "score": 12,
            "bestmove": "e2e4",
            "counters": {},
        }
        other = dict(common, reported_nps=500, reported_time_ms=200)
        MODULE.compare_identity({"searches": [common]}, {"searches": [other]})
        other["bestmove"] = "d2d4"
        with self.assertRaisesRegex(RuntimeError, "identity failed"):
            MODULE.compare_identity({"searches": [common]}, {"searches": [other]})

    def test_aggregate_records_the_root_generation_reset_boundary(self):
        rows = [
            {
                "cohort": "opening",
                "reported_nodes": 10,
                "reported_time_ms": 2,
                "counters": {"board_gen_full_calls": 3},
            },
            {
                "cohort": "opening",
                "reported_nodes": 12,
                "reported_time_ms": 3,
                "counters": {"board_gen_full_calls": 4},
            },
        ]
        aggregate = MODULE.aggregate(rows)["opening"]
        self.assertEqual(aggregate["root_legal_generation_calls_outside_diag"], 2)
        self.assertEqual(aggregate["counters"]["board_gen_full_calls"], 7)


if __name__ == "__main__":
    unittest.main()
