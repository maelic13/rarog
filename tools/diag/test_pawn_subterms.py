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


class RarogDefinitions(unittest.TestCase):
    def test_a_lone_pawn(self):
        r = ps.rarog_terms_for("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1")
        self.assertEqual((r["r_isolated"], r["r_unopposed"], r["r_passed_3"], r["r_islands"]),
                         (1, 1, 1, 1))
        self.assertEqual((r["r_doubled"], r["r_backward"]), (0, 0))

    def test_rarog_counts_both_doubled_pawns_and_both_as_passed(self):
        # The donor penalises neither (no pawn directly behind another) and
        # passes only the front one.
        fen = "4k3/8/8/8/4P3/8/4P3/4K3 w - - 0 1"
        r = ps.rarog_terms_for(fen)
        self.assertEqual((r["r_doubled"], r["r_doubled_isolated"]), (2, 2))
        self.assertEqual((r["r_passed_1"], r["r_passed_3"]), (1, 1))
        d = ps.components_for(fen)
        self.assertEqual(d["w_doubled"], (0, 0))
        self.assertEqual(d["p_rank"], ps.PASSED_RANK[3])

    def test_a_pawn_with_a_level_neighbour_is_backward_for_rarog_only(self):
        # e4 beside d4; f6 attacks e5. Rarog: no own pawn strictly behind on an
        # adjacent file, so backward. The donor: d4 is level, so not.
        fen = "4k3/6p1/5p2/8/3PP3/8/8/4K3 w - - 0 1"
        self.assertEqual(ps.rarog_terms_for(fen)["r_backward"], 1)
        self.assertEqual(ps.components_for(fen)["w_backward"], (0, 0))

    def test_support_and_phalanx_by_rank(self):
        r = ps.rarog_terms_for("4k3/8/8/8/3PP3/2P5/8/4K3 w - - 0 1")
        # d4 is supported by c3 (rank index 3); d4 and e4 stand in a phalanx.
        self.assertEqual(r["r_support_3"], 1)
        self.assertEqual(r["r_phalanx_3"], 2)


class VerifyRarog(unittest.TestCase):
    def test_engine_counts_pass_and_a_wrong_one_fails(self):
        fen = "4k3/8/8/8/4P3/8/4P3/4K3 w - - 0 1"
        own = ps.rarog_terms_for(fen)
        comps = ps.components_for(fen)
        cells = ["0", "0.5", "0.1", "0", "0", "0", "0"]
        for name in ps.COLUMNS:
            cells += [str(comps[name][0]), str(comps[name][1])]
        cells += [str(own[name]) for name in ps.RAROG_COLUMNS]
        cells += [str(own["pr_total_mg"]), str(own["pr_total_eg"]), fen]

        def run(tamper):
            with tempfile.TemporaryDirectory() as tmp:
                dump = Path(tmp) / "d.csv"
                dump.write_text(ps.header() + "\n" + ";".join(cells) + "\n", encoding="utf-8")
                engine = Path(tmp) / "engine"
                engine.mkdir()
                for field, (names, sign) in ps.ENGINE_FIELDS.items():
                    # Phase 6: each count shows as count * 18 / 24.
                    values = [sign * own[n] * 18 / 24 for n in names]
                    if tamper and field == "pawn_doubled_eg":
                        values[0] += 0.75
                    (engine / f"{field}.csv").write_text(
                        f"{fen};{','.join(str(v) for v in values)};6\n", encoding="utf-8")
                args = argparse.Namespace(dump=str(dump), engine_dumps=str(engine))
                with contextlib.redirect_stdout(io.StringIO()):
                    return ps.verify_rarog(args)

        self.assertEqual(run(False), 0)
        self.assertEqual(run(True), 1)


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
            own = ps.rarog_terms_for(fen)
            cells += [str(own[name]) for name in ps.RAROG_COLUMNS]
            cells += [str(own["pr_total_mg"]), str(own["pr_total_eg"])]
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
