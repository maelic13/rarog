"""Tests for threat_subterms.py: the rebuilt threats components and the
donor-area mobility against rows of Stockfish 9587eeeb's printed trace, the
definitions on crafted positions, and the analysis on a planted signal."""

import unittest

import chess
import numpy as np

import threat_subterms as ts

# `idx;mobility_mg;mobility_eg;threats_mg;threats_eg;fen` rows of
# RAR-E23's `scores-9587.csv` (hce-v4-tb validation, 2026-10-07), the
# printed Total column in pawns.
PRINTED = [
    ("0.10", "0.62", "0.49", "0.50", "2b1kr2/p1R2p2/4p3/3pP3/q2P2Qp/P7/1P3PPP/6K1 b - - 1 9"),
    ("0.42", "0.55", "0.24", "0.17", "1k5r/1p6/p7/4p1P1/1r1n3p/2RBB2P/5PK1/2R5 w - - 0 22"),
    ("0.26", "0.73", "0.02", "0.09", "5rk1/1bq3p1/p1r1pbnp/1pP1p3/1P2P3/P1B1QN2/6PP/1B1R1RK1 w - - 7 8"),
    ("0.18", "0.06", "-0.33", "-0.12", "3r3k/1p1q2p1/1np1p2p/4P3/pP3P2/P1Br2P1/2Q4P/2R1R1K1 w - - 2 12"),
    ("-0.54", "-0.62", "-0.59", "-0.72", "8/6k1/R7/8/2r3BK/8/2Pb4/8 w - - 2 4"),
    ("0.00", "0.00", "0.03", "0.03", "7k/8/1p4KP/1P6/8/8/8/8 b - - 4 10"),
    ("0.39", "0.40", "-0.45", "-0.56", "1qr3k1/1r1b1ppp/2n2n2/bp2p3/p3P3/3BBNNP/PP1Q1PP1/2RR2K1 w - - 2 2"),
]


def summed(comps, names):
    mg = sum(comps[n][0] for n in names)
    eg = sum(comps[n][1] for n in names)
    return mg, eg


class PrintedRows(unittest.TestCase):
    def test_threats_components_sum_to_the_printed_row(self):
        for _mmg, _meg, tmg, teg, fen in PRINTED:
            comps = ts.components_for(fen)
            mg, eg = ts.printed(*summed(comps, ts.THREAT_COMPONENTS))
            self.assertEqual((f"{mg:.2f}", f"{eg:.2f}"), (tmg, teg), fen)

    def test_donor_area_mobility_equals_the_printed_row(self):
        for mmg, meg, _tmg, _teg, fen in PRINTED:
            comps = ts.components_for(fen)
            mg, eg = ts.printed(*comps["mob_v0"])
            self.assertEqual((f"{mg:.2f}", f"{eg:.2f}"), (mmg, meg), fen)


class Definitions(unittest.TestCase):
    def test_blockers_for_king_takes_either_colour_and_only_a_lone_piece(self):
        # White king e1, black rook e8; a white knight on e4 alone is a
        # blocker, a black pawn on e5 alone is one too, both together none.
        self.assertEqual(ts.blockers_for_king(chess.Board("4r2k/8/8/8/4N3/8/8/4K3 w - - 0 1"), chess.WHITE),
                         chess.BB_E4)
        self.assertEqual(ts.blockers_for_king(chess.Board("4r2k/8/8/4p3/8/8/8/4K3 w - - 0 1"), chess.WHITE),
                         chess.BB_E5)
        self.assertEqual(ts.blockers_for_king(chess.Board("4r2k/8/8/4p3/4N3/8/8/4K3 w - - 0 1"), chess.WHITE),
                         0)

    def test_pinned_piece_attacks_are_cut_to_the_pin_line(self):
        # White knight e4 pinned by the rook e8: its attacks leave the e-file,
        # so its donor mobility is 0 while its plain mobility is not.
        board = chess.Board("4r2k/8/8/8/4N3/8/8/4K3 w - - 0 1")
        side = ts.Side(board, chess.WHITE)
        self.assertEqual(side.attacks_of[chess.E4], 0)
        self.assertEqual(side.mobility["v0"], ts.MOBILITY_BONUS[chess.KNIGHT][0])
        self.assertEqual(side.mobility["no_pin"], ts.MOBILITY_BONUS[chess.KNIGHT][8])

    def test_mobility_area_excludes_blocked_and_low_pawns_king_and_queen(self):
        # White: Ka1, Qd1, pawns a2 (low), e4 blocked by e5. The queen on d1
        # and a2 and e4 leave the area; h6 (a free high pawn) stays in it.
        board = chess.Board("7k/8/7P/4p3/4P3/8/P7/K2Q4 w - - 0 1")
        side = ts.Side(board, chess.WHITE)
        for sq in (chess.A1, chess.D1, chess.A2, chess.E4):
            self.assertFalse(side.area & chess.BB_SQUARES[sq], chess.square_name(sq))
        self.assertTrue(side.area & chess.BB_H6)
        # Squares the black pawn attacks are out of White's area.
        self.assertFalse(side.area & chess.BB_D4)

    def test_rook_xrays_through_queens_and_own_rooks(self):
        # Rooks a1 and a8 (white) with a white queen on a4: each rook sees
        # through the queen and the other rook under the donor's rule.
        board = chess.Board("R6k/8/8/8/Q7/8/8/R3K3 w - - 0 1")
        side = ts.Side(board, chess.WHITE)
        self.assertTrue(side.attacks_of[chess.A1] & chess.BB_A8)
        self.assertTrue(side.attacks_of[chess.A8] & chess.BB_A1)
        self.assertEqual(side.mobility["v0"] != side.mobility["no_xray"], True)

    def test_safe_pawn_threat_needs_a_safe_pawn_and_a_non_pawn_victim(self):
        # White pawn d4 attacks the black knight on e5; the pawn stands on a
        # square the black bishop (g1) attacks and no white piece defends, so
        # it is not safe: the gated term is 0 and the ungated one is not.
        board = chess.Board("7k/8/8/4n3/3P4/8/8/K5b1 w - - 0 1")
        white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
        comps = ts.threats(board, white, black)
        self.assertEqual(comps["safe_pawn"], (0, 0))
        self.assertEqual(comps["pawn_ungated"], ts.THREAT_BY_SAFE_PAWN)

    def test_minor_threat_is_gated_by_strong_protection(self):
        # A white knight on c3 attacks the black pawn on d5, which the pawn
        # on e6 defends: strongly protected and a pawn, so neither weak nor a
        # defended non-pawn target; the gated term is 0, the ungated one is
        # the pawn entry.
        board = chess.Board("7k/8/4p3/3p4/8/2N5/8/K7 w - - 0 1")
        white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
        comps = ts.threats(board, white, black)
        self.assertEqual(comps["by_minor"], (0, 0))
        self.assertEqual(comps["by_minor_ungated"], ts.THREAT_BY_MINOR[chess.PAWN])

    def test_king_threat_fires_on_a_weak_piece_the_king_attacks(self):
        board = chess.Board("7k/8/8/8/8/8/3n4/2K5 w - - 0 1")
        white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
        comps = ts.threats(board, white, black)
        self.assertEqual(comps["by_king"], ts.THREAT_BY_KING)
        self.assertEqual(comps["hanging"], ts.HANGING)

    def test_queen_threats_double_when_the_enemy_has_the_only_queen(self):
        # White knight c3 attacks e4, next to the black queen on d6? The
        # knight attack must land on a square a knight move from the queen:
        # from c3 the knight attacks e4, b5, d5, e2, a4, a2, b1, d1; the
        # queen on d6 is a knight move from e4 and b5. Both are safe squares.
        board = chess.Board("7k/8/3q4/8/8/2N5/8/K7 w - - 0 1")
        white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
        comps = ts.threats(board, white, black)
        self.assertEqual(comps["knight_on_queen"], (2 * 2 * ts.KNIGHT_ON_QUEEN[0], 2 * 2 * ts.KNIGHT_ON_QUEEN[1]))
        board = chess.Board("7k/8/3q4/8/8/2N5/8/K2Q4 w - - 0 1")
        white, black = ts.Side(board, chess.WHITE), ts.Side(board, chess.BLACK)
        comps = ts.threats(board, white, black)
        self.assertEqual(comps["knight_on_queen"], (2 * ts.KNIGHT_ON_QUEEN[0], 2 * ts.KNIGHT_ON_QUEEN[1]))


class Analysis(unittest.TestCase):
    def test_a_planted_component_is_found_by_leave_one_out(self):
        import argparse
        import contextlib
        import io
        import json
        import tempfile
        import os

        rng = np.random.default_rng(3)
        n = 6000
        fens = ["4k3/8/8/8/8/8/8/4K3 w - - 0 1"] * n
        rarog = rng.normal(0, 2.0, n)
        hanging = rng.normal(0, 60.0, n)
        other = rng.normal(0, 60.0, n)
        p = 1 / (1 + np.exp(-(rarog * 100 + 3.0 * hanging * 100 / ts.PAWN_EG) / 400 * 1.5))
        label = (rng.random(n) < p).astype(float)
        with tempfile.TemporaryDirectory() as tmp:
            dump = os.path.join(tmp, "d.csv")
            with open(dump, "w", encoding="utf-8") as handle:
                handle.write(ts.header() + "\n")
                for i in range(n):
                    cells = [str(i), str(label[i]), f"{rarog[i]:.2f}", "0.10", "0.10", "0.00", "0.00"]
                    for name in ts.COLUMNS:
                        v = hanging[i] if name == "hanging" else other[i] if name == "by_minor" else 0.0
                        cells += [f"{v:.0f}", f"{v:.0f}"]
                    cells.append(fens[i])
                    handle.write(";".join(cells) + "\n")
            out = os.path.join(tmp, "r.json")
            with contextlib.redirect_stdout(io.StringIO()):
                ts.analyse(argparse.Namespace(dump=dump, out=out))
            report = json.load(open(out, encoding="utf-8"))
        rows = report["cohorts"]["all"]
        without_hanging = rows["paired"]["rarog+threats-hanging vs rarog+threats"]["gain_pct"]
        without_minor = rows["paired"]["rarog+threats-by_minor vs rarog+threats"]["gain_pct"]
        self.assertLess(without_hanging, -1.0)
        self.assertGreater(without_minor, without_hanging + 1.0)
        self.assertLess(abs(rows["models"]["rarog+shuffled_threats"]["gain_vs_rarog_pct"]), 0.2)


if __name__ == "__main__":
    unittest.main()
