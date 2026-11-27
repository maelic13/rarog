"""Tests for the PLAN reader: the current leaf is the board's Now, a step's
lines stop where the next step starts, and a closed phase's step is refused."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import guide_board  # noqa: E402
import plan_leaf as pl  # noqa: E402

PLAN = "\n".join([
    "# Plan",                                                   # 1
    "",
    "## 2. Operating rules",                                    # 3
    "",
    "1. A rule.",
    "",
    "### Standing contracts",                                   # 7
    "",
    "| Contract | Where |",
    "",
    "## Phase A — Old work — CLOSED 2026-01-01",                # 11
    "",
    "Everything was done.",
    "",
    "### Active workflow register",                             # 15
    "",
    "Rows are in the order of work.",
    "",
    "| Leaf | Workflow state | Class | Current decision |",
    "|---|---|---|---|",
    "| B.3.1 | RESEARCH | V | Runs first, by decision |",       # 21
    "| B.2 | RESEARCH | R3 | Held on B.3.1 |",                  # 22
    "",
    "## Phase B — New work",                                    # 24
    "",
    "- **B.1 First step — `I1`, DONE 2026-02-02.** Body.",      # 26
    "- **B.2 Second step, wrapped",                             # 27
    "  over two lines — `R3` investigation.** Body of B.2,",
    "  which cites **B.9** without defining it.",
    "",
    "- **B.3 Parent — `M`/`V`.** Body.",                        # 31
    "    - **B.3.1 Child — `V`.** Body of the child.",          # 32
    "      More of the child.",
    "    - **B.3.2 Second child — `V`, DONE 2026-02-03.** Done.",  # 34
    "",
    "## 3. Protocols",                                          # 36
    "- **B.7 Outside any phase — `V`.** Ignored.",
])


class Spans(unittest.TestCase):
    def setUp(self):
        self.lines = PLAN.split("\n")
        self.spans = pl.step_spans(self.lines)

    def test_a_step_ends_before_the_next_step(self):
        self.assertEqual(self.spans["B.1"], (26, 26))
        self.assertEqual(self.spans["B.2"], (27, 29))

    def test_a_parent_holds_its_sub_steps_and_ends_at_the_heading(self):
        self.assertEqual(self.spans["B.3"], (31, 34))
        self.assertEqual(self.spans["B.3.1"], (32, 33))
        self.assertEqual(self.spans["B.3.2"], (34, 34))

    def test_a_step_outside_an_open_phase_has_no_span(self):
        self.assertNotIn("B.7", self.spans)
        self.assertNotIn("B.9", self.spans)

    def test_a_section_ends_at_the_next_heading_of_its_level_or_above(self):
        self.assertEqual(pl.section_span(self.lines, "## 2. Operating rules"), (3, 10))
        self.assertEqual(pl.section_span(self.lines, "### Standing contracts"), (7, 10))
        self.assertEqual(pl.section_span(self.lines, pl.REGISTER), (15, 23))
        self.assertIsNone(pl.section_span(self.lines, "## No such section"))

    def test_size_counts_crlf_endings(self):
        self.assertEqual(pl.size(self.lines, (26, 26)), len(self.lines[25].encode("utf-8")) + 2)


class Register(unittest.TestCase):
    def test_rows_come_in_their_own_order_with_their_decisions(self):
        rows = pl.register_rows(PLAN.split("\n"))
        self.assertEqual([(r[0], r[1], r[2], r[4]) for r in rows],
                         [("B.3.1", "RESEARCH", "V", 21), ("B.2", "RESEARCH", "R3", 22)])
        self.assertEqual(rows[1][3], "Held on B.3.1")

    def test_the_current_leaf_is_the_boards_now(self):
        report = pl.overview(PLAN)
        now = guide_board.build_board(PLAN).split("\n", 1)[0]
        self.assertEqual(now, "**Now: B.3.1** (`V`): Child.")
        self.assertEqual(report[0], "Current leaf: B.3.1 (RESEARCH / V), Child")
        self.assertIn("docs/PLAN.md:32-33", report[1])
        self.assertIn("under B.3: docs/PLAN.md:31-34", report[2])

    def test_a_done_first_row_is_not_the_current_leaf(self):
        done = PLAN.replace("- **B.3.1 Child — `V`.**", "- **B.3.1 Child — `V`, DONE 2026-02-04.**")
        self.assertTrue(pl.overview(done)[0].startswith("Current leaf: B.2 (RESEARCH / R3)"))

    def test_the_overview_names_the_sections_to_read(self):
        text = "\n".join(pl.overview(PLAN))
        self.assertIn("Register, in the order of work: docs/PLAN.md:15-23", text)
        self.assertIn("2. Operating rules: docs/PLAN.md:3-10", text)
        self.assertIn("Standing contracts: docs/PLAN.md:7-10", text)


class OneStep(unittest.TestCase):
    def test_a_leaf_shows_its_row_its_parent_and_its_text(self):
        report = pl.one_step(PLAN, "B.3.1", True)
        self.assertEqual(report[0], "B.3.1 (RESEARCH / V), Child")
        self.assertIn("under B.3: docs/PLAN.md:31-34", report[2])
        self.assertIn("register row, docs/PLAN.md:21: Runs first, by decision", report[3])
        self.assertEqual(report[-2:], ["    - **B.3.1 Child — `V`.** Body of the child.",
                                       "      More of the child."])

    def test_a_parent_lists_its_sub_steps(self):
        report = pl.one_step(PLAN, "B.3", False)
        self.assertIn("  sub-steps: B.3.1, B.3.2 (done)", report)
        self.assertFalse(any("no register row" in line for line in report))

    def test_a_done_step_shows_its_status(self):
        self.assertEqual(pl.one_step(PLAN, "B.1", False)[0], "B.1 (DONE 2026-02-02), First step")

    def test_a_step_that_is_not_in_an_open_phase_is_refused(self):
        self.assertIsNone(pl.one_step(PLAN, "A.1", False))
        self.assertIsNone(pl.one_step(PLAN, "B.7", False))


if __name__ == "__main__":
    unittest.main()
