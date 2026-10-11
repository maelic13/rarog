"""Tests for pawn_subterms.py: the donor's arithmetic on a hand-worked
position, C++ division, the passed set, and verify passing and failing."""

import argparse
import contextlib
import io
import tempfile
import unittest
from pathlib import Path

import pawn_subterms as ps


class Arithmetic(unittest.TestCase):
    def test_division_truncates_toward_zero_like_cpp(self):
        self.assertEqual(ps.tdiv(-7, 4), -1)
        self.assertEqual(ps.tdiv(7, 4), 1)
        self.assertEqual(ps.tdiv(-8, 4), -2)

    def test_a_lone_e4_pawn_by_hand(self):
        # Isolated and unopposed; passed on its fourth rank (w = 2): the black
        # king three from e5 gives 19*3//4 = 14 per w, ours four from e5 and
        # five (capped) from e6 costs 2*4 + 5 per w; the black king attacks
        # e7 on the path but not e5, so k = 9; e-file edge distance 3.
        c = ps.components_for("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1")
        self.assertEqual(c["w_isolated"], (-5, -15))
        self.assertEqual(c["w_unopposed"], (-13, -27))
        self.assertEqual(c["p_rank"], (15, 41))
        self.assertEqual(c["p_king_them"], (0, 28))
        self.assertEqual(c["p_king_us"], (0, -26))
        self.assertEqual(c["p_path"], (18, 18))
        self.assertEqual(c["p_defended"], (0, 0))
        self.assertEqual(c["p_file"], (-33, -24))

    def test_colours_are_symmetric(self):
        white = ps.components_for("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1")
        black = ps.components_for("4k3/8/8/4p3/8/8/8/4K3 b - - 0 1")
        for name in ps.COLUMNS:
            self.assertEqual(white[name], (-black[name][0], -black[name][1]), name)

    def test_a_pawn_with_an_enemy_pawn_ahead_is_not_passed(self):
        c = ps.components_for("4k3/4p3/8/8/4P3/8/8/4K3 w - - 0 1")
        self.assertEqual(c["p_rank"], (0, 0))


class Verify(unittest.TestCase):
    def test_verify_passes_the_donors_rows_and_fails_a_tampered_one(self):
        fen = "4k3/8/8/8/4P3/8/8/4K3 w - - 0 1"
        c = ps.components_for(fen)
        passed = [sum(c[n][i] for n in ps.PASSED_COMPONENTS) for i in (0, 1)]
        pawns = [sum(c[n][i] for n in ps.PAWN_COMPONENTS) for i in (0, 1)]

        def run(passed_eg_printed):
            cells = ["0", "0.5", "0.1",
                     f"{round(passed[0] / ps.PAWN_EG, 2)}", f"{passed_eg_printed}",
                     f"{round(pawns[0] / ps.PAWN_EG, 2)}", f"{round(pawns[1] / ps.PAWN_EG, 2)}"]
            for name in ps.COLUMNS:
                cells += [str(c[name][0]), str(c[name][1])]
            cells.append(fen)
            with tempfile.TemporaryDirectory() as tmp:
                dump = Path(tmp) / "d.csv"
                dump.write_text(ps.header() + "\n" + ";".join(cells) + "\n", encoding="utf-8")
                with contextlib.redirect_stdout(io.StringIO()):
                    return ps.verify(argparse.Namespace(dump=str(dump)))

        self.assertEqual(run(round(passed[1] / ps.PAWN_EG, 2)), 0)
        self.assertEqual(run(round(passed[1] / ps.PAWN_EG, 2) + 0.05), 1)


if __name__ == "__main__":
    unittest.main()
