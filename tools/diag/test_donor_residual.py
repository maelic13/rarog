"""Tests for donor_residual.py: the parsers against real engine output, the
phase arithmetic, and the model comparison against a planted signal."""

import unittest

import numpy as np

import donor_residual as dr

SF11_BLOCK = """\
     Term    |    White    |    Black    |    Total
             |   MG    EG  |   MG    EG  |   MG    EG
 ------------+-------------+-------------+------------
    Material |  ----  ---- |  ----  ---- | -0.29  0.42
   Imbalance |  ----  ---- |  ----  ---- |  0.00  0.00
       Pawns |  0.65 -0.04 |  0.67 -0.06 | -0.02  0.01
     Knights | -0.16 -0.19 | -0.20 -0.23 |  0.03  0.04
     Bishops | -0.21 -0.56 | -0.07 -0.63 | -0.15  0.08
       Rooks | -0.24 -0.05 |  0.00  0.00 | -0.24 -0.05
      Queens |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
    Mobility |  0.67  1.20 |  0.35  0.71 |  0.32  0.49
 King safety |  0.38 -0.13 |  0.36 -0.05 |  0.02 -0.08
     Threats |  0.44  0.36 |  0.63  0.51 | -0.19 -0.15
      Passed |  0.00  0.00 |  0.00  0.00 |  0.00  0.00
       Space |  0.69  0.00 |  0.46  0.00 |  0.23  0.00
  Initiative |  ----  ---- |  ----  ---- |  0.00  0.14
 ------------+-------------+-------------+------------
       Total |  ----  ---- |  ----  ---- | -0.29  0.91

Total evaluation: -0.15 (white side)
"""

ORACLE_RAROG = """\
Rarog 2.3.2 HCE through Stockfish 9587eeeb search
Final evaluation: -0.99 (white side)
"""

ORACLE_CONTROL = """\
Use Rarog HCE is disabled.

Original Stockfish HCE final evaluation: -0.24 (white side)
"""


class Parsers(unittest.TestCase):
    def test_sf11_trace_reads_every_term_and_the_total(self):
        terms, total = dr.parse_sf11_trace(SF11_BLOCK)
        self.assertEqual(total, -0.15)
        for term in dr.SF11_TERMS:
            self.assertIn(term, terms)
        self.assertEqual(terms["Material"], (-0.29, 0.42))
        self.assertEqual(terms["King safety"], (0.02, -0.08))
        self.assertEqual(terms["Initiative"], (0.0, 0.14))
        self.assertEqual(terms["Total"], (-0.29, 0.91))

    def test_sf11_in_check_has_no_total(self):
        terms, total = dr.parse_sf11_trace("Total evaluation: none (in check)\n")
        self.assertIsNone(total)
        self.assertEqual(terms, {})

    def test_oracle_totals_for_both_evaluators(self):
        self.assertEqual(dr.parse_oracle_total(ORACLE_RAROG), -0.99)
        self.assertEqual(dr.parse_oracle_total(ORACLE_CONTROL), -0.24)
        self.assertIsNone(dr.parse_oracle_total("Final evaluation: none (in check)"))

    def test_every_reported_family_term_is_a_trace_term(self):
        listed = [t for terms in dr.FAMILIES.values() for t in terms]
        self.assertEqual(sorted(listed), sorted(dr.SF11_TERMS))


class Phase(unittest.TestCase):
    def test_phase_limits(self):
        start = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        self.assertEqual(dr.sf11_phase(start), 128)
        self.assertEqual(dr.sf11_phase("8/8/4k3/8/8/4K3/4P3/8 w - - 0 1"), 0)
        # Two rooks and two minors a side: between the limits.
        mid = "r3k2r/1b6/8/8/8/8/1B6/R3K2R w - - 0 1"
        self.assertTrue(0 < dr.sf11_phase(mid) < 128)

    def test_men_count_includes_both_kings_and_ignores_other_fields(self):
        self.assertEqual(dr.men_count("4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1"), 2)
        start = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        self.assertEqual(dr.men_count(start), 32)

    def test_queen_detection_reads_the_board_field_only(self):
        self.assertTrue(dr.has_queen("3qk3/8/8/8/8/8/8/4K3 w - - 0 1"))
        self.assertFalse(dr.has_queen("4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1"))


class PlantedSignal(unittest.TestCase):
    def test_a_real_signal_is_reported_and_a_null_one_is_not(self):
        rng = np.random.default_rng(7)
        n = 40_000
        base = rng.normal(0.0, 150.0, n)
        real = rng.normal(0.0, 60.0, n)
        null = rng.normal(0.0, 60.0, n)
        p = 1.0 / (1.0 + np.exp(-(base + real) / 170.0))
        y = (rng.random(n) < p).astype(float)

        def error(columns):
            return (y - dr.held_out_predictions(np.column_stack(columns), y)) ** 2

        e_base = error([base])
        gain_real = e_base - error([base, real])
        gain_null = e_base - error([base, null])
        se_real = gain_real.std(ddof=1) / np.sqrt(n)
        se_null = gain_null.std(ddof=1) / np.sqrt(n)
        self.assertGreater(gain_real.mean(), 10 * se_real)
        self.assertLess(abs(gain_null.mean()), 4 * se_null)

    def test_a_queens_row_signal_is_read_as_queens_not_pieces(self):
        import argparse
        import contextlib
        import io
        import json
        import tempfile
        from pathlib import Path

        rng = np.random.default_rng(13)
        n = 20_000
        rarog = rng.normal(0.0, 1.5, n)
        queens = rng.normal(0.0, 0.6, n)
        p = 1.0 / (1.0 + np.exp(-(rarog + queens) / 1.7))
        y = (rng.random(n) < p).astype(float)
        names = dr.header().split(";")
        lines = [";".join(names)]
        for i in range(n):
            cells = {name: 0.0 for name in names}
            cells.update(idx=i, label=y[i], rarog=rarog[i], sf_control=rarog[i],
                         sf11_total=rarog[i], queens_mg=queens[i], queens_eg=queens[i],
                         knights_mg=rng.normal(0.0, 0.6), knights_eg=0.0)
            cells["fen"] = "4k3/8/8/8/8/8/8/4K3 w - - 0 1"
            lines.append(";".join(str(cells[name]) for name in names))
        with tempfile.TemporaryDirectory() as tmp:
            scores = Path(tmp) / "scores.csv"
            scores.write_text("\n".join(lines) + "\n", encoding="utf-8")
            out = Path(tmp) / "report.json"
            args = argparse.Namespace(scores=str(scores), out=str(out), within=False)
            with contextlib.redirect_stdout(io.StringIO()):
                dr.analyse(args)
            models = json.loads(out.read_text())["cohorts"]["all"]["models"]
        gain = {name: models[name]["gain_vs_rarog_pct"] for name in models}
        se = {name: models[name]["gain_se_pct"] for name in models}
        self.assertGreater(gain["rarog+queens"], 10 * se["rarog+queens"])
        self.assertLess(abs(gain["rarog+pieces"]), 4 * se["rarog+pieces"])
        self.assertGreater(gain["rarog+pieces+queens"], 10 * se["rarog+pieces+queens"])

    def test_the_joint_model_keeps_the_four_piece_rows_in_one_column(self):
        family = {name: np.full(3, float(i + 1)) for i, name in enumerate(dr.FAMILIES)}
        columns = dr.joint_columns(family)
        self.assertEqual(len(columns), len(dr.FAMILIES) - 1)
        joined = family["pieces"] + family["queens"]
        self.assertTrue(any(np.array_equal(c, joined) for c in columns))
        self.assertEqual(dr.FAMILIES["queens"], ["Queens"])
        self.assertNotIn("Queens", dr.FAMILIES["pieces"])

    def test_each_row_is_one_trace_term_outside_the_families(self):
        for name, terms in dr.ROWS.items():
            self.assertEqual(len(terms), 1)
            self.assertIn(terms[0], dr.SF11_TERMS)
            self.assertNotIn(name, dr.FAMILIES)

    def test_a_knights_row_signal_is_read_by_its_row_and_cohorts_split_by_men(self):
        import argparse
        import contextlib
        import io
        import json
        import tempfile
        from pathlib import Path

        fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",  # queens, phase 128
            "r3k2r/1b6/8/8/8/8/1B6/R3K2R w - - 0 1",  # no queens, phase 32, 8 men
            "4k3/pppp4/8/8/8/8/PPP5/4K3 w - - 0 1",  # no queens, phase 0, 9 men
            "4k3/8/8/8/8/8/8/4K3 w - - 0 1",  # no queens, phase 0, 2 men
        ]
        rng = np.random.default_rng(17)
        n = 16_000
        rarog = rng.normal(0.0, 2.5, n)
        knights = rng.normal(0.0, 0.8, n)
        p = 1.0 / (1.0 + np.exp(-(rarog + knights) / 1.7))
        y = (rng.random(n) < p).astype(float)
        names = dr.header().split(";")
        lines = [";".join(names)]
        for i in range(n):
            cells = {name: 0.0 for name in names}
            noise = rng.normal(0.0, 0.8)
            cells.update(idx=i, label=y[i], rarog=rarog[i], sf_control=rarog[i],
                         sf11_total=rarog[i], knights_mg=knights[i], knights_eg=knights[i],
                         bishops_mg=noise, bishops_eg=noise)
            cells["fen"] = fens[i % len(fens)]
            lines.append(";".join(str(cells[name]) for name in names))
        with tempfile.TemporaryDirectory() as tmp:
            scores = Path(tmp) / "scores.csv"
            scores.write_text("\n".join(lines) + "\n", encoding="utf-8")
            out = Path(tmp) / "report.json"
            args = argparse.Namespace(scores=str(scores), out=str(out), within=True)
            with contextlib.redirect_stdout(io.StringIO()):
                dr.analyse(args)
            cohorts = json.loads(out.read_text())["cohorts"]
        self.assertEqual(cohorts["phase<32 men>=7"]["rows"], n // 4)
        self.assertEqual(cohorts["no queens men>=7"]["rows"], n // 2)
        models = cohorts["all"]["models"]
        gain = {name: models[name]["gain_vs_rarog_pct"] for name in models}
        se = {name: models[name]["gain_se_pct"] for name in models}
        self.assertGreater(gain["rarog+knights"], 10 * se["rarog+knights"])
        self.assertLess(abs(gain["rarog+bishops"]), 4 * se["rarog+bishops"])
        self.assertLess(abs(gain["rarog+rooks"]), 4 * se["rarog+rooks"] + 1e-12)

    def test_fractional_labels_fit(self):
        rng = np.random.default_rng(11)
        x = rng.normal(0.0, 1.0, (5_000, 1))
        y = np.where(x[:, 0] > 0.5, 1.0, np.where(x[:, 0] < -0.5, 0.0, 0.5))
        w = dr.fit_logistic(x, y)
        self.assertGreater(w[1], 0.0)
        self.assertAlmostEqual(float(dr.predict(w, np.zeros((1, 1)))[0]), 0.5, delta=0.05)


if __name__ == "__main__":
    unittest.main()
