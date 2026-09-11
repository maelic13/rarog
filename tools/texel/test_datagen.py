"""Fast unit tests for the Texel datagen sampling contracts."""

import json
import random
import sys
import tempfile
import unittest
from pathlib import Path

import chess
import chess.pgn

sys.path.insert(0, str(Path(__file__).resolve().parent))
import extract
import extract_parallel
import sample_fens


class ExtractTests(unittest.TestCase):
    def test_parallel_provenance_accepts_and_verifies_v2(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "games.pgn"
            path.write_text("test pgn\n", encoding="utf-8")
            manifest = {
                "schema": "rarog-fastchess-datagen-v2",
                "output": {
                    "path": str(path.resolve()),
                    "bytes": path.stat().st_size,
                    "sha256": extract.sha256_file(path),
                },
                "engine": {"sha256": "engine", "git_sha": "commit", "git_dirty": False},
                "book": {"sha256": "book", "seed": 42, "start": 1, "end": 1},
                "games": 1,
                "nodes_per_move": 8000,
                "adjudication": {"Name": "datagen-v1"},
            }
            path.with_suffix(".manifest.json").write_text(
                json.dumps(manifest), encoding="utf-8"
            )
            records, contract = extract_parallel.load_datagen_provenance([path])
            self.assertEqual(records[0]["games"], 1)
            self.assertEqual(contract["engine_sha256"], "engine")

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

    def test_static_identity_keeps_rule_fifty_clock(self):
        first = "8/8/8/8/8/8/P6p/4K2k w - - 17 42"
        second = "8/8/8/8/8/8/P6p/4K2k w - - 18 42"
        third = "8/8/8/8/8/8/P6p/4K2k w - - 17 99"
        self.assertNotEqual(extract.fen_key(first), extract.fen_key(second))
        self.assertEqual(extract.fen_key(first), extract.fen_key(third))

    def test_whole_start_determines_split(self):
        game = chess.pgn.Game()
        game.headers["Result"] = "1/2-1/2"
        game.add_variation(chess.Move.from_uci("e2e4"))
        clone = chess.pgn.Game()
        clone.headers["Result"] = "1-0"
        clone.add_variation(chess.Move.from_uci("d2d4"))
        self.assertEqual(extract.start_digest(game), extract.start_digest(clone))
        self.assertEqual(extract.split_for(game, 5, 5), extract.split_for(clone, 5, 5))

    def test_three_split_counts_are_relative_to_fixed_train(self):
        self.assertEqual(extract.split_counts(900, 5, 5),
                         {"train": 900, "validation": 50, "test": 50})

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

    def test_process_game_audit_reuses_walked_final_position(self):
        game = chess.pgn.Game()
        game.headers["Result"] = "1-0"
        node = game
        for move_uci in ("f2f3", "e7e5", "g2g4", "d8h4"):
            node = node.add_variation(chess.Move.from_uci(move_uci))
        audit = {}
        extract.process_game(game, 0, 0, 1, 0, False, random.Random(1), audit)
        self.assertEqual(audit, {"plies": 4, "natural_mate": True})

    def test_material_audit_has_exact_signature_and_classes(self):
        signature = extract_parallel.material_signature(chess.STARTING_FEN)
        self.assertEqual(signature, (8, 2, 2, 2, 1, 8, 2, 2, 2, 1))
        self.assertEqual(extract_parallel.material_class(signature), "both_queens")
        self.assertEqual(
            extract_parallel.material_class((1, 0, 0, 0, 0, 2, 0, 0, 0, 0)),
            "pawn_only",
        )

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

    def test_parallel_worker_counts_empty_game_start(self):
        import tempfile
        game = chess.pgn.Game()
        game.headers["Result"] = "1/2-1/2"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "empty.pgn"
            path.write_text(str(game) + "\n\n", encoding="utf-8")
            opts = {
                "validation_pct": 5.0, "test_pct": 5.0,
                "skip_start": 0, "skip_end": 6,
                "max_per_phase_per_game": 1, "max_per_game": 1,
                "quiet_filter": False, "seed": 1, "blend": 1.0,
            }
            output, stats = extract_parallel.worker((str(path), 0, path.stat().st_size, opts))
            self.assertEqual(stats["recorded_games"], 1)
            self.assertEqual(stats["skipped"], 1)
            self.assertEqual(len(output), 1)
            self.assertEqual(output[0][2], [])


class SeedSamplerTests(unittest.TestCase):
    def test_bucket_targets_are_exact(self):
        self.assertEqual(sample_fens.bucket_targets(12), [3, 3, 2, 2, 2])

    def test_phase_classifier(self):
        self.assertEqual(sample_fens.phase_bucket(chess.STARTING_FEN), 0)
        kings_and_pawn = "4k3/8/8/8/8/8/P7/4K3 w - -"
        self.assertEqual(sample_fens.phase_bucket(kings_and_pawn), 4)


if __name__ == "__main__":
    unittest.main()
