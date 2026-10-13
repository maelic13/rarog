"""Tests for ending_games.py: family matching in both colours, and the
summary's affected-game rule on a hand-built record."""

import unittest

import chess

import ending_games as eg


class Matching(unittest.TestCase):
    def setUp(self):
        self.families = {n: eg.signature(n) for n in ("KRP-KR", "KP-K", "KP-KP")}

    def test_either_colour_matches_and_names_the_strong_side(self):
        white_strong = chess.Board("4k3/8/8/8/4P3/8/r7/R3K3 w - - 0 1")
        self.assertEqual(eg.match(white_strong, self.families), ("KRP-KR", chess.WHITE))
        black_strong = chess.Board("4k3/8/8/4p3/8/8/r7/R3K3 w - - 0 1")
        self.assertEqual(eg.match(black_strong, self.families), ("KRP-KR", chess.BLACK))

    def test_exact_material_only(self):
        extra = chess.Board("4k3/8/8/8/4P3/8/r6P/R3K3 w - - 0 1")
        self.assertIsNone(eg.match(extra, self.families))
        self.assertEqual(eg.match(chess.Board("4k3/8/8/4p3/4P3/8/8/4K3 w - - 0 1"),
                                  self.families)[0], "KP-KP")


class Summary(unittest.TestCase):
    def test_affected_games_and_the_bound(self):
        base = {"official": True, "family": "KP-K", "ply": 10, "fen_before": "", "move": "",
                "capture": True, "mover": "strong"}
        entries = [
            # 1: a win traded into a draw by its mover.
            {**base, "game": 1, "mover_before": "win", "mover_after": "draw",
             "white_after": "draw", "white_result": "draw"},
            # 2: a drawn entry the game lost.
            {**base, "game": 2, "mover_before": "draw", "mover_after": "draw",
             "white_after": "draw", "white_result": "loss"},
            # 3: as the tablebase said.
            {**base, "game": 3, "mover_before": "win", "mover_after": "win",
             "white_after": "win", "white_result": "win"},
        ]
        report = eg.summarise({"games": 100, "official": 100, "entries": entries}, ["KP-K"])
        self.assertEqual(report["affected_games"], [1, 2])
        self.assertAlmostEqual(report["expected_value_upper_bound_elo"], 2 / 100 * 0.5 * 100 * 6.95, 3)
        self.assertEqual(report["by_family"]["KP-K"]["worsening_entries"], 1)


if __name__ == "__main__":
    unittest.main()
