"""Tests for unstoppable_passers.py: the engine's own cases, the rule of the
square in both move orders, the preconditions and the row-by-row check."""

import argparse
import contextlib
import io
import tempfile
import unittest
from pathlib import Path

import chess

import unstoppable_passers as up


def count(fen, rule, color=chess.WHITE):
    return up.counted(chess.Board(fen), color, rule)


class EngineCases(unittest.TestCase):
    def test_the_trace_tests_positions(self):
        # src/eval/trace.rs, phase_7_4_semantic_fixes (iii).
        self.assertEqual(count("8/P7/1K6/8/8/8/8/7k w - - 0 1", "engine"), 1)
        self.assertEqual(count("8/P7/1K6/8/8/8/6n1/7k w - - 0 1", "engine"), 0)

    def test_net_is_white_minus_black(self):
        board = chess.Board("7K/8/8/8/8/1k6/p7/8 b - - 0 1")
        self.assertEqual(up.net(board, "engine"), -1)


class RuleOfTheSquare(unittest.TestCase):
    # White pawn a5 (three moves to promote), black king four squares from a8.
    def test_defender_to_move_one_square_outside_still_catches(self):
        fen = "8/8/8/P3k3/8/8/8/K7 b - - 0 1"  # 1...Kd6 2.a6 Kc7 3.a7 Kb7 4.a8=Q+ Kxa8
        self.assertEqual(count(fen, "engine"), 1)  # the engine's generous tempo
        self.assertEqual(count(fen, "square"), 0)

    def test_pawn_side_to_move_from_the_same_squares_queens(self):
        fen = "8/8/8/P3k3/8/8/8/K7 w - - 0 1"
        self.assertEqual(count(fen, "engine"), 1)
        self.assertEqual(count(fen, "square"), 1)

    def test_one_square_further_queens_with_the_defender_to_move(self):
        fen = "8/8/8/P4k2/8/8/8/K7 b - - 0 1"
        self.assertEqual(count(fen, "square"), 1)

    def test_pawn_side_to_move_inside_the_square_is_caught(self):
        fen = "8/8/8/P2k4/8/8/8/K7 w - - 0 1"  # king three from a8
        self.assertEqual(count(fen, "engine"), 1)
        self.assertEqual(count(fen, "square"), 0)

    def test_second_rank_pawn_counts_its_double_step(self):
        # a2 promotes in five moves; the king on h4 is seven from a8.
        self.assertEqual(count("8/8/8/8/7k/8/P7/K7 b - - 0 1", "square"), 1)
        # On g4 it is six away and catches it (threshold 5 + 1).
        self.assertEqual(count("8/8/8/8/6k1/8/P7/K7 b - - 0 1", "square"), 0)


class Preconditions(unittest.TestCase):
    def test_a_defending_piece_or_a_blocked_path_or_an_enemy_pawn_stops_it(self):
        self.assertEqual(count("8/8/8/P6k/8/8/8/K5n1 w - - 0 1", "square"), 0)
        self.assertEqual(count("K7/8/8/P6k/8/8/8/8 w - - 0 1", "square"), 0)
        self.assertEqual(count("8/1p6/8/P6k/8/8/8/K7 w - - 0 1", "square"), 0)
        self.assertTrue(up.admitted(chess.Board("8/8/8/P6k/8/8/8/K7 w - - 0 1")))
        self.assertFalse(up.admitted(chess.Board("K7/8/8/P6k/8/8/8/8 w - - 0 1")))


class Verify(unittest.TestCase):
    def run_verify(self, values):
        fens = ["8/P7/1K6/8/8/8/8/7k w - - 0 1", "8/P7/1K6/8/8/8/6n1/7k w - - 0 1"]
        with tempfile.TemporaryDirectory() as tmp:
            pos = Path(tmp) / "p.csv"
            dump = Path(tmp) / "d.csv"
            pos.write_text("".join(f"{f};0.5\n" for f in fens))
            dump.write_text("".join(f"{f};{v}\n" for f, v in zip(fens, values)))
            with contextlib.redirect_stdout(io.StringIO()):
                return up.cmd_verify(argparse.Namespace(positions=pos, dump=dump))

    def test_the_engines_counts_pass_and_a_wrong_one_fails(self):
        self.assertEqual(self.run_verify(["1", "0"]), 0)
        self.assertEqual(self.run_verify(["0", "0"]), 1)


if __name__ == "__main__":
    unittest.main()
