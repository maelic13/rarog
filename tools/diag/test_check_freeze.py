"""Tests for the freeze check: which lines a change adds, when a ledger entry
is born resolved, and when an analysis gains a registration and its result
together."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import check_freeze as cf  # noqa: E402

REGISTERED = ("# RAR-S1 — a read — REGISTERED 2026-11-23, before any game\n\n"
              "- Disposition: open\n\n## Prediction\n\n- It finishes, 0.6.\n")


class Added(unittest.TestCase):
    def test_a_new_marker_line_is_added(self):
        old = "intro\n"
        new = "intro\n### The read, registered 2026-11-26 before it ran\n"
        self.assertEqual(cf.added(cf.REGISTRATION, old, new), [2])

    def test_editing_a_line_that_carried_the_marker_adds_none(self):
        # The result commit appends to the heading that holds the registration.
        old = "# RAR-S1 — a read — REGISTERED 2026-11-23, before any game\n"
        new = "# RAR-S1 — a read — REGISTERED 2026-11-23, before any game; READ 2026-11-24\n"
        self.assertEqual(cf.added(cf.REGISTRATION, old, new), [])

    def test_a_second_marker_of_the_same_text_is_added(self):
        old = "Frozen predictions:\n- a\n"
        new = "Frozen predictions:\n- a\nFrozen predictions:\n- b\n"
        self.assertEqual(cf.added(cf.REGISTRATION, old, new), [3])

    def test_an_unchanged_line_is_never_added(self):
        text = "**Calibration.** held.\n"
        self.assertEqual(cf.added(cf.RESULT, text, "above\n" + text), [])


class Entry(unittest.TestCase):
    def test_an_entry_registered_open_passes(self):
        self.assertIsNone(cf.entry_problem(None, REGISTERED))

    def test_an_entry_born_resolved_with_a_prediction_fails(self):
        born = REGISTERED.replace("Disposition: open", "Disposition: accepted — +3%")
        self.assertIn("already `accepted`", cf.entry_problem(None, born))

    def test_a_bold_prediction_lead_counts(self):
        born = ("# RAR-M1 — speed\n\n- Disposition: observation\n\n"
                "**Predictions, frozen before the timed series:** faster.\n")
        self.assertIsNotNone(cf.entry_problem(None, born))

    def test_an_observation_with_no_prediction_passes(self):
        observed = ("# RAR-E1 — candidates\n\n- **Date / owner:** observed 2026-11-26: an "
                    "observed record with no frozen prediction.\n- Disposition: observation\n")
        self.assertIsNone(cf.entry_problem(None, observed))

    def test_resolving_an_existing_entry_passes(self):
        resolved = REGISTERED.replace("Disposition: open", "Disposition: refuted")
        self.assertIsNone(cf.entry_problem(REGISTERED, resolved))


class Analysis(unittest.TestCase):
    OLD = "## Diagnosis\n\ntext\n"

    def test_a_registration_and_its_result_together_fail(self):
        new = (self.OLD + "### The read, registered 2026-11-26 before it ran\n\nFrozen predictions:\n"
               "- a\n\n### The read: the result\n\n**Calibration**, appended.\n")
        self.assertIn("line 4 registers and line 11 records", cf.analysis_problem(self.OLD, new))

    def test_a_registration_alone_passes(self):
        new = self.OLD + "### The read, registered 2026-11-26 before it ran\n"
        self.assertIsNone(cf.analysis_problem(self.OLD, new))

    def test_a_result_above_the_next_registration_passes(self):
        old = self.OLD + "### The read, registered 2026-11-25 before it ran\n"
        new = old + "**Calibration**, appended.\n\n### One more, registered 2026-11-26 before it ran\n"
        self.assertIsNone(cf.analysis_problem(old, new))

    def test_a_result_for_an_earlier_registration_passes(self):
        old = self.OLD + "### The read, registered 2026-11-25 before it ran\n"
        self.assertIsNone(cf.analysis_problem(old, old + "**Calibration**, appended.\n"))

    def test_a_new_file_is_read_against_nothing(self):
        new = "The read, registered 2026-11-26 before it ran\n\n**Calibration.** held.\n"
        self.assertIsNotNone(cf.analysis_problem(None, new))


class Problem(unittest.TestCase):
    BOTH = "registered 2026-11-26 before it ran\n**Calibration.**\n"

    def test_the_check_follows_the_path(self):
        self.assertIsNotNone(cf.problem("analysis/a.md", None, self.BOTH))
        self.assertIsNone(cf.problem("analysis/artifacts/a.md", None, self.BOTH))
        self.assertIsNone(cf.problem("analysis/a.txt", None, self.BOTH))
        self.assertIsNone(cf.problem("docs/PLAN.md", None, self.BOTH))
        born = REGISTERED.replace("Disposition: open", "Disposition: rejected")
        self.assertIsNotNone(cf.problem("docs/experiments/RAR-S1.md", None, born))

    def test_known_commits_are_full_hashes_with_a_reason(self):
        for rev, reason in cf.KNOWN.items():
            self.assertRegex(rev, r"^[0-9a-f]{40}$")
            self.assertTrue(reason.strip())


if __name__ == "__main__":
    unittest.main()
