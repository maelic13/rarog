"""Tests for the occurrence split (PLAN 4.11.5).

The measurement's whole risk is the threshold: at 7 men the bench suite looks
uncontaminated and at 10 it is 94% contaminated, on identical data. So the
parser that classifies roots must be right, and the suite it reads must not be
allowed to drift away from it silently.
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import endgame_occurrence as eo


class MenTests(unittest.TestCase):
    def test_the_start_position_has_32_men(self):
        self.assertEqual(
            eo.men("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), 32)

    def test_a_bare_king_ending_has_three(self):
        self.assertEqual(eo.men("8/8/8/8/8/8/4K1N1/7k w - - 0 1"), 3)

    def test_only_the_board_field_is_counted(self):
        """Castling rights and the side to move are letters too."""
        self.assertEqual(eo.men("4k3/8/8/8/8/8/8/4K2R w K - 0 1"), 3)


class BenchSuiteTests(unittest.TestCase):
    def test_the_suite_parses_and_matches_its_declared_length(self):
        fens = eo.bench_fens()
        self.assertEqual(len(fens), 40)

    def test_every_parsed_entry_is_a_plausible_fen(self):
        for fen in eo.bench_fens():
            with self.subTest(fen[:24]):
                self.assertIn(" w ", f" {fen} ".replace(" b ", " w "))
                self.assertGreaterEqual(eo.men(fen), 3)

    def test_the_declared_count_guard_fires_on_a_mismatch(self):
        """A suite that grew without the parser noticing must fail loudly."""
        text = eo.BENCH_RS.read_text(encoding="utf-8")
        m = re.search(r"pub const BENCH_FENS: \[&str; (\d+)\]", text)
        self.assertIsNotNone(m)
        self.assertEqual(int(m.group(1)), len(eo.bench_fens()))

    def test_the_suite_has_no_roots_below_the_tablebase_line(self):
        """Recorded because it is the reassuring half of the finding.

        Zero roots at <= 7 men is true and is NOT the same as "the census is
        uncontaminated": three roots at <= 8 men produce 56% of every
        reference-family evaluation. See
        `analysis/endgame_occurrence_split_2026-09-05.md`.
        """
        counts = [eo.men(f) for f in eo.bench_fens()]
        self.assertEqual([c for c in counts if c <= 7], [])
        self.assertEqual(len([c for c in counts if c <= 8]), 3)


class SweepTests(unittest.TestCase):
    def test_the_sweep_is_ordered_and_covers_the_disagreement(self):
        self.assertEqual(list(eo.SWEEP), sorted(eo.SWEEP))
        # 7 is where the suite looks clean and 10 is where it does not; a sweep
        # that skipped either would hide the thing it exists to show.
        self.assertIn(7, eo.SWEEP)
        self.assertIn(10, eo.SWEEP)

    def test_more_men_never_means_fewer_endgame_roots(self):
        counts = [eo.men(f) for f in eo.bench_fens()]
        sizes = [len([c for c in counts if c <= t]) for t in eo.SWEEP]
        self.assertEqual(sizes, sorted(sizes))


if __name__ == "__main__":
    unittest.main()
