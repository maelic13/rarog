"""Fast unit tests for the Texel datagen sampling contracts."""

import random
import sys
import unittest
from pathlib import Path

import chess
import chess.pgn

sys.path.insert(0, str(Path(__file__).resolve().parent))
import extract
import sample_fens


class ExtractTests(unittest.TestCase):
    def test_allocate_is_exact(self):
        self.assertEqual(extract.allocate(3_000_003, [1, 1, 1, 1, 1]),
                         [600_001, 600_001, 600_001, 600_000, 600_000])

    def test_five_phase_boundaries(self):
        expected = {
            24: 0, 20: 0,
            19: 1, 14: 1,
            13: 2, 8: 2,
            7: 3, 3: 3,
            2: 4, 0: 4,
        }
        for phase, bucket in expected.items():
            with self.subTest(phase=phase):
                self.assertEqual(extract.phase_bucket(phase), bucket)

    def test_reservoir_never_exceeds_capacity(self):
        reservoir = extract.Reservoir(7, random.Random(4))
        for i in range(100):
            reservoir.offer((str(i), 0.5, None))
        self.assertEqual(reservoir.seen, 100)
        self.assertEqual(len(reservoir.items), 7)

    def test_per_game_sampling_keeps_scarce_phase(self):
        game = chess.pgn.Game()
        game.headers["Result"] = "1/2-1/2"
        node = game.add_variation(chess.Move.from_uci("e2e4"))
        node.comment = "+0.10/8 0.01s"
        rows, rejected = extract.process_game(
            game, 0, 0, 1, 0, False, random.Random(1)
        )
        self.assertEqual(rejected, 0)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0][1], 0)  # full-material opening

    def test_process_game_classifies_all_five_phases(self):
        cases = (
            (chess.STARTING_FEN, "e2e4", 0),
            ("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1", "e2e4", 1),
            ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "a1a2", 2),
            ("4k2r/8/8/8/8/8/8/R3K3 w - - 0 1", "a1a2", 3),
            ("4k3/8/8/8/8/8/P7/4K3 w - - 0 1", "a2a3", 4),
        )
        for fen, move_uci, expected_bucket in cases:
            with self.subTest(bucket=expected_bucket):
                board = chess.Board(fen)
                game = chess.pgn.Game()
                game.setup(board)
                game.headers["Result"] = "1/2-1/2"
                game.add_variation(chess.Move.from_uci(move_uci))
                rows, _ = extract.process_game(
                    game, 0, 0, 1, 0, False, random.Random(1)
                )
                self.assertEqual(rows[0][1], expected_bucket)


class SeedSamplerTests(unittest.TestCase):
    def test_bucket_targets_are_exact(self):
        self.assertEqual(sample_fens.bucket_targets(12), [3, 3, 2, 2, 2])

    def test_phase_classifier(self):
        self.assertEqual(sample_fens.phase_bucket(chess.STARTING_FEN), 0)
        kings_and_pawn = "4k3/8/8/8/8/8/P7/4K3 w - -"
        self.assertEqual(sample_fens.phase_bucket(kings_and_pawn), 4)


if __name__ == "__main__":
    unittest.main()
