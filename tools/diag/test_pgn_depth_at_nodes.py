"""Tests for the per-engine PGN depth reader.

The two mistakes these catch were both made: crediting a Black-to-move
opening's moves as if White moved first (RAR-O03's recorded depth gap), and
averaging a mate score's pseudo-depth into the depth an engine reached.
"""

import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import pgn_depth_at_nodes

PGN = """[White "A"]
[Black "B"]
[FEN "4k3/8/8/8/8/8/8/4K2R w K - 0 1"]

1. Kd2 {+1.00/10 0.010s} Kd7 {-1.00/20 0.020s} 2. Ke2 {+1.00/11 0.010s} 1/2-1/2

[White "B"]
[Black "A"]
[FEN "4k3/8/8/8/8/8/8/4K2R b K - 0 1"]

1... Kd7 {+1.00/12 0.010s} 2. Kd2 {-1.00/22 0.020s} Ke7 {+M3/245 0.010s, Draw by 3-fold repetition} 1/2-1/2
"""


class DepthTests(unittest.TestCase):
    def collect(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "g.pgn"
            path.write_text(PGN, encoding="utf-8")
            return pgn_depth_at_nodes.collect(path)

    def test_a_black_to_move_opening_credits_black_first(self):
        depths, _, _ = self.collect()
        # Game 2 starts Black to move and A has Black, so A moves first.
        # Attributing as if White moved first gives A [10, 11, 22].
        self.assertEqual(depths["A"], [10, 11, 12, 245])
        self.assertEqual(depths["B"], [20, 22])

    def test_the_last_move_with_the_termination_text_is_a_move(self):
        # fastchess appends the termination to the final comment after a
        # comma; dropping it would lose one move per game, the terminal one.
        depths, times, _ = self.collect()
        self.assertEqual(depths["A"][-1], 245)
        self.assertEqual(times["A"][-1], 0.010)

    def test_mate_scores_are_flagged(self):
        _, _, mates = self.collect()
        self.assertEqual(mates["A"], [False, False, False, True])
        self.assertEqual(mates["B"], [False, False])


if __name__ == "__main__":
    unittest.main()
