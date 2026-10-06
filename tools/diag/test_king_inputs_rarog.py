"""Tests for king_inputs_rarog.py: the start position is quiet, a direct queen
attack on the zone counts as five units, a safe rook check counts eight, the
shelter/storm deficit follows src/eval/king.rs's definition, and the index clamps to
the table."""

import unittest

import chess

import king_inputs_rarog as ki


class InputTests(unittest.TestCase):
    def test_start_position_is_quiet(self):
        phase, sides = ki.king_inputs(chess.Board())
        self.assertEqual(phase, 24)
        for color in (chess.WHITE, chess.BLACK):
            s = sides[color]
            self.assertEqual((s["units"], s["checks"], s["weak"], s["flank"], s["pawnless"], s["relief"]), (0, 0, 0, 0, 0, 0))
            # King on the e-file: the castled-flank shelter gate is closed;
            # no enemy pawn has reached its fourth rank.
            self.assertEqual(s["deficit"], 0)
            self.assertEqual(s["bucket"], 0)
            self.assertEqual(s["table_cp"], ki.TABLE[0])

    def test_queen_on_the_zone_is_five_units_and_relief_is_off(self):
        # Black queen on h4 attacks f2 and g3 next to the castled white king
        # (zone: f1 g1 h1 f2 g2 h2 f3 g3 h3 after the forward shift).
        board = chess.Board("6k1/8/8/8/7q/8/5PPP/6K1 w - - 0 1")
        _, sides = ki.king_inputs(board)
        w = sides[chess.WHITE]
        self.assertEqual(w["units"], 5)
        self.assertEqual(w["relief"], 0)
        b = sides[chess.BLACK]
        self.assertEqual(b["units"], 0)
        self.assertEqual(b["relief"], ki.QUEEN_RELIEF)

    def test_safe_rook_check_counts_eight(self):
        # White king g1 with no defenders of the first rank's a1; black rook on a8
        # can give a safe check from a1.
        board = chess.Board("r5k1/8/8/8/8/8/5PPP/6K1 w - - 0 1")
        _, sides = ki.king_inputs(board)
        self.assertEqual(sides[chess.WHITE]["checks"], 8)
        self.assertEqual(sides[chess.WHITE]["units"], 0)

    def test_shelter_deficit_counts_missing_files_and_storm_pawns(self):
        # White king g1, no f/g/h pawns (deficit 1+2+1 = 4); black pawn on h3
        # (rank 6 from Black's view, 0-based 5) adds 5-2 = 3.
        board = chess.Board("6k1/8/8/8/8/7p/8/6K1 w - - 0 1")
        _, sides = ki.king_inputs(board)
        self.assertEqual(sides[chess.WHITE]["deficit"], 7)

    def test_index_clamps_to_the_table(self):
        board = chess.Board("6k1/8/8/8/3q3q/5n2/8/6K1 w - - 0 1")
        _, sides = ki.king_inputs(board)
        w = sides[chess.WHITE]
        self.assertGreaterEqual(w["danger"], 0)
        self.assertLessEqual(w["bucket"], len(ki.TABLE) - 1)
        self.assertEqual(w["bucket"], min(w["danger"], len(ki.TABLE) - 1))


if __name__ == "__main__":
    unittest.main()
