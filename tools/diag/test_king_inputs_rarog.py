"""Tests for king_inputs_rarog.py: the weights come from src/eval/params.rs, the
start position is quiet, a direct queen attack on the zone counts one queen
unit, a safe rook check counts the rook's check weight, every input enters the
index at its engine weight, the shelter/storm deficit follows src/eval/king.rs's
definition, and the index clamps to the table."""

import unittest

import chess

import king_inputs_rarog as ki


class WeightTests(unittest.TestCase):
    def test_loader_reads_multiline_fields_and_skips_comments(self):
        text = "    a: 2 = [1, -2];\n    b: 3 = [4,\n        5, 6];\n    // c: 1 = [9];\n"
        self.assertEqual(ki.load_weights(text), {"a": [1, -2], "b": [4, 5, 6]})

    def test_loader_refuses_a_length_mismatch(self):
        with self.assertRaises(ValueError):
            ki.load_weights("    a: 3 = [1, 2];\n")

    def test_the_engine_weights_are_read(self):
        weights = ki.load_weights(ki.PARAMS_RS.read_text(encoding="utf-8"))
        self.assertEqual(ki.TABLE, weights["king_safety_table"])
        self.assertEqual(len(ki.TABLE), 40)
        self.assertEqual(ki.TABLE, sorted(ki.TABLE), "the tuner keeps the table non-decreasing")
        self.assertEqual(ki.UNIT[chess.QUEEN], weights["king_safety_unit_queen"][0])
        self.assertEqual(ki.WEAK_RING, weights["ks_weak_ring"][0])


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

    def test_queen_on_the_zone_is_one_queen_unit_and_relief_is_off(self):
        # Black queen on h4 attacks f2 and g3 next to the castled white king
        # (zone: f1 g1 h1 f2 g2 h2 f3 g3 h3 after the forward shift).
        board = chess.Board("6k1/8/8/8/7q/8/5PPP/6K1 w - - 0 1")
        _, sides = ki.king_inputs(board)
        w = sides[chess.WHITE]
        self.assertEqual(w["units"], ki.UNIT[chess.QUEEN])
        self.assertEqual(w["relief"], 0)
        b = sides[chess.BLACK]
        self.assertEqual(b["units"], 0)
        self.assertEqual(b["relief"], ki.QUEEN_RELIEF)

    def test_safe_rook_check_counts_the_rook_check_weight(self):
        # White king g1 with no defenders of the first rank's a1; black rook on a8
        # can give a safe check from a1.
        board = chess.Board("r5k1/8/8/8/8/8/5PPP/6K1 w - - 0 1")
        _, sides = ki.king_inputs(board)
        self.assertEqual(sides[chess.WHITE]["checks"], ki.SAFE_CHECK[chess.ROOK])
        self.assertEqual(sides[chess.WHITE]["units"], 0)

    def test_every_input_enters_the_index_at_its_engine_weight(self):
        # Knight f3 and queen g3 against the castled king: two weak ring squares.
        board = chess.Board("6k1/5ppp/8/8/8/5nq1/5PPP/6K1 w - - 0 1")
        _, sides = ki.king_inputs(board)
        w = sides[chess.WHITE]
        self.assertGreater(w["weak"], 0, "the fixture must exercise the weak-ring input")
        self.assertEqual(
            w["danger"],
            w["units"] + ki.WEAK_RING * w["weak"] + w["checks"] + ki.FLANK_ATTACK * w["flank"]
            + w["pawnless"] - w["relief"] + ki.SHELTER_STORM * w["deficit"],
        )

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
        self.assertLessEqual(w["bucket"], ki.TOP)
        self.assertEqual(w["bucket"], min(w["danger"], ki.TOP))


if __name__ == "__main__":
    unittest.main()
