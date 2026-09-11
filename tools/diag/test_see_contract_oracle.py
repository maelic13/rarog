"""Structural checks for independent exchange truth, including its limits."""
import unittest
from unittest.mock import patch

import chess
import see_contract_oracle as oracle


class SeeOracleTests(unittest.TestCase):
    def test_committed_fixture_and_wrong_arithmetic_negative_control(self):
        self.assertEqual(oracle.OUTPUT.read_text(encoding="utf-8"), oracle.render())
        self.assertEqual(oracle.REPAIR_OUTPUT.read_text(encoding="utf-8"),
                         oracle.render(oracle.REPAIR_CASES, "see-repair-v1"))
        wrong = list(oracle.CASES[0])
        wrong[-1] += 1
        with patch.object(oracle, "CASES", [tuple(wrong)]):
            with self.assertRaises(AssertionError):
                oracle.render()

    def test_color_mirrors_and_restoration(self):
        for name, _, fen, uci, expected in oracle.CASES:
            with self.subTest(name=name):
                board = chess.Board(fen).mirror()
                mv = chess.Move.from_uci(uci)
                mirrored = chess.Move(chess.square_mirror(mv.from_square),
                                      chess.square_mirror(mv.to_square), mv.promotion)
                before = board.fen()
                self.assertEqual(oracle.exchange(board, mirrored), expected)
                self.assertEqual(board.fen(), before)
                self.assertEqual(board.move_stack, [])

    def test_created_and_released_pins_are_real(self):
        created = chess.Board("2k5/2n5/2B5/3p4/8/8/8/2R1K3 w - - 0 1")
        self.assertFalse(created.is_pinned(chess.BLACK, chess.C7))
        created.push_uci("c6d5")
        self.assertTrue(created.is_pinned(chess.BLACK, chess.C7))
        self.assertNotIn(chess.Move.from_uci("c7d5"), created.legal_moves)
        released = chess.Board("k7/1r1p4/2B5/8/8/8/8/4K3 w - - 0 1")
        self.assertTrue(released.is_pinned(chess.BLACK, chess.B7))
        released.push_uci("c6d7")
        self.assertFalse(released.is_pinned(chess.BLACK, chess.B7))
        self.assertIn(chess.Move.from_uci("b7d7"), released.legal_moves)

    def test_all_promotion_replies_and_best_choice(self):
        board = chess.Board("7k/8/8/8/8/7K/pR6/1r6 w - - 0 1")
        board.push_uci("b2b1")
        replies = [m for m in board.legal_moves if m.to_square == chess.B1 and board.is_capture(m)]
        self.assertEqual({m.promotion for m in replies},
                         {chess.QUEEN, chess.ROOK, chess.BISHOP, chess.KNIGHT})
        self.assertEqual(oracle.recaptures(board, chess.B1), 1300)

    def test_illegal_initial_move_is_rejected(self):
        with self.assertRaises(ValueError):
            oracle.exchange(chess.Board(), chess.Move.from_uci("e2e5"))

    def test_promoted_piece_is_recaptured_at_its_new_value(self):
        board = chess.Board(oracle.REPAIR_CASES[0][2])
        board.push_uci("b2b1")
        replies = [m for m in board.legal_moves if m.to_square == chess.B1 and m.promotion]
        self.assertEqual(len(replies), 4)
        # All promotion gains cancel their new victim value on Rc1xb1.
        self.assertEqual([oracle.exchange(board, m) for m in replies], [400] * 4)

    def test_cross_engine_values_match_independently_scored_contract(self):
        expected = {
            "free-pawn": 100, "defended-pawn": -400, "king-after-pawn": -300,
            "legal-king-recapture": 0, "defended-king-destination": 900,
            "pinned-pawn": 100, "pin-created": 100, "pin-released": -200,
            "xray": 500, "ep-opens-rook": 0, "quiet-hanging": -500,
            "quiet-promotion-hanging": -100, "quiet-underpromotion": 200,
            "capture-promotion": 1300, "capture-underpromotion": 700,
            "promotion-recapture": -800, "castle-king": 0, "castle-queen": 0,
            "promoted-piece-recaptured": 100, "pin-created-later": -300,
            "skip-pinned-choose-rook": -200, "quiet-allows-promotion": -1300,
            "initial-king-capture": 300,
        }
        checked = 0
        for name, _, fen, uci, _ in oracle.CASES + oracle.REPAIR_CASES:
            base = name.removeprefix("mirror-")
            board = chess.Board(fen)
            self.assertEqual(oracle.exchange(board, chess.Move.from_uci(uci),
                                             oracle.CROSS_ENGINE_VALUES), expected[base])
            checked += 1
        self.assertEqual(checked, 41)


if __name__ == "__main__":
    unittest.main()
