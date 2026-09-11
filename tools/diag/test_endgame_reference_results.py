"""Tests for the attained-reference-result artifact (PLAN 4.11.3).

The generator's job is half arithmetic and half REFUSAL. An artifact that
silently combined two arms measured over different positions, or at different
node budgets, would be worse than no artifact -- it would look authoritative and
be wrong, which is RAR-E14's defect B. So every validation is exercised on a
known-bad input, per rule 15.
"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import endgame_reference_results as rr


def position(fen, wdl, outcome):
    return {"fen": fen, "theory_wdl": wdl, "outcome": outcome}


def arm(engine="e.exe", cohort="aa" * 32, families=None, **conditions):
    doc = {
        "schema": rr.TRUTH_SCHEMA,
        "engine": engine,
        "nodes_per_move": 60000, "max_plies": 100,
        "positions_per_family": 2, "seed": 1, "hash_mb": 16,
        "cohort": {"sha256": cohort},
        "families": families or {},
    }
    doc.update(conditions)
    return doc


def family(positions, cohort="ff" * 32):
    return {"cohort_sha256": cohort, "positions": positions}


class ArithmeticTests(unittest.TestCase):
    def test_the_paired_matrix_partitions_the_clean_wins(self):
        cand = family([position("a", 2, "mated"), position("b", 2, "mated"),
                       position("c", 2, "fifty_move"), position("d", 2, "ply_limit")])
        ref = family([position("a", 2, "mated"), position("b", 2, "ply_limit"),
                      position("c", 2, "mated"), position("d", 2, "ply_limit")])
        out = rr.family_result(cand, ref)
        self.assertEqual(out["paired"], {"both": 1, "candidate_only": 1,
                                         "reference_only": 1, "neither": 1})
        self.assertEqual(out["clean_wins"], 4)
        self.assertEqual(out["candidate_converted"], 2)
        self.assertEqual(out["attained_reference_result"], 2)
        self.assertEqual(out["paired_union"], 3)

    def test_only_clean_wins_are_counted(self):
        """A draw or a cursed win is not a conversion opportunity."""
        cand = family([position("a", 2, "mated"), position("b", 0, "fifty_move"),
                       position("c", 1, "fifty_move")])
        ref = family([position("a", 2, "mated"), position("b", 0, "fifty_move"),
                      position("c", 1, "fifty_move")])
        out = rr.family_result(cand, ref)
        self.assertEqual(out["clean_wins"], 1)

    def test_the_deficit_is_reference_minus_candidate(self):
        cand = family([position("a", 2, "ply_limit"), position("b", 2, "ply_limit")])
        ref = family([position("a", 2, "mated"), position("b", 2, "mated")])
        self.assertEqual(rr.family_result(cand, ref)["deficit_to_reference"], 2)

    def test_a_candidate_ahead_gives_a_negative_deficit(self):
        """Exceeding the reference is possible and must not be clamped."""
        cand = family([position("a", 2, "mated")])
        ref = family([position("a", 2, "ply_limit")])
        self.assertEqual(rr.family_result(cand, ref)["deficit_to_reference"], -1)


class RefusalTests(unittest.TestCase):
    """Each check exercised on the input it exists to reject."""

    def _pair(self, **ref_overrides):
        fam = {"KQ-K": family([position("a", 2, "mated")])}
        cand = arm(families={"KQ-K": family([position("a", 2, "mated")])})
        ref = arm(families=fam, **ref_overrides)
        return cand, ref

    def test_a_matched_pair_is_accepted(self):
        cand, ref = self._pair()
        rr.validate(cand, ref)

    def test_a_different_cohort_is_refused(self):
        cand, ref = self._pair()
        ref["cohort"]["sha256"] = "bb" * 32
        with self.assertRaises(SystemExit) as c:
            rr.validate(cand, ref)
        self.assertIn("different position sets", str(c.exception))

    def test_a_different_node_budget_is_refused(self):
        """A reference result at another budget is a different quantity."""
        cand, ref = self._pair(nodes_per_move=200000)
        with self.assertRaises(SystemExit) as c:
            rr.validate(cand, ref)
        self.assertIn("nodes_per_move", str(c.exception))

    def test_a_different_ply_limit_is_refused(self):
        cand, ref = self._pair(max_plies=200)
        with self.assertRaises(SystemExit) as c:
            rr.validate(cand, ref)
        self.assertIn("max_plies", str(c.exception))

    def test_a_different_seed_is_refused(self):
        cand, ref = self._pair(seed=999)
        with self.assertRaises(SystemExit):
            rr.validate(cand, ref)

    def test_a_different_family_set_is_refused(self):
        cand, ref = self._pair()
        ref["families"]["KR-K"] = family([position("z", 2, "mated")])
        with self.assertRaises(SystemExit) as c:
            rr.validate(cand, ref)
        self.assertIn("different families", str(c.exception))

    def test_a_per_family_digest_mismatch_is_refused(self):
        cand, ref = self._pair()
        ref["families"]["KQ-K"]["cohort_sha256"] = "cc" * 32
        with self.assertRaises(SystemExit) as c:
            rr.validate(cand, ref)
        self.assertIn("per-family cohort digests", str(c.exception))

    def test_missing_per_position_records_are_refused(self):
        cand, ref = self._pair()
        del ref["families"]["KQ-K"]["positions"]
        with self.assertRaises(SystemExit) as c:
            rr.validate(cand, ref)
        self.assertIn("--per-position", str(c.exception))

    def test_a_theory_disagreement_is_refused(self):
        """The verdict is a property of the position; arms cannot disagree."""
        cand, ref = self._pair()
        ref["families"]["KQ-K"]["positions"][0]["theory_wdl"] = 0
        with self.assertRaises(SystemExit) as c:
            rr.validate(cand, ref)
        self.assertIn("FEN/theory pairing", str(c.exception))

    def test_a_v1_report_is_refused(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "old.json"
            p.write_text(json.dumps({"schema": "rarog-endgame-truth-v1"}),
                         encoding="utf-8")
            with self.assertRaises(SystemExit):
                rr.load(p)


class FramingTests(unittest.TestCase):
    """The artifact must state its own limits so it cannot be over-read."""

    ARTIFACT = (Path(__file__).resolve().parent
                / "endgame_reference_results_v1.json")

    def test_the_frozen_artifact_exists(self):
        self.assertTrue(self.ARTIFACT.is_file())

    def test_no_field_is_named_a_ceiling(self):
        """BAS-E50: a field called `ceiling` was read as an acceptance target.

        Checks KEY names, not the serialized document -- the disclaimer text
        deliberately contains the phrase "not a ceiling", and a substring test
        over the whole JSON fails on the very sentence that prevents the
        misreading. (Written wrong first, exactly as in 4.10.3.)
        """
        doc = json.loads(self.ARTIFACT.read_text(encoding="utf-8"))
        self.assertIn("attained_reference_result", doc["totals"])

        def keys(node):
            if isinstance(node, dict):
                for k, v in node.items():
                    yield k
                    yield from keys(v)
            elif isinstance(node, list):
                for v in node:
                    yield from keys(v)

        offenders = [k for k in keys(doc) if "ceiling" in k.lower()]
        self.assertEqual(offenders, [])

    def test_it_carries_its_own_disclaimers(self):
        doc = json.loads(self.ARTIFACT.read_text(encoding="utf-8"))
        joined = " ".join(doc["what_this_is_not"]).lower()
        self.assertIn("not a ceiling", joined)
        self.assertIn("not an acceptance target", joined)
        self.assertIn("paired union", joined)

    def test_it_records_the_conditions_it_is_valid_at(self):
        doc = json.loads(self.ARTIFACT.read_text(encoding="utf-8"))
        for key in rr.MATCHED_CONDITIONS:
            self.assertIn(key, doc["conditions"])
        self.assertTrue(doc["cohort_sha256"])


if __name__ == "__main__":
    unittest.main()
