"""Structural and negative checks for normalized_see_compare.py."""

import unittest

import normalized_see_compare as compare


VERDICTS = ",".join(f"0:a{i}a{i + 1}={i % 2}" for i in range(10))


def output(values=compare.VALUES, verdicts=VERDICTS, rows=True):
    text = f"see-values: {values}\nsee-verdicts: {verdicts}\npreflight: PASS\n"
    if rows:
        for name, work in zip(
            ["legal moves", "legal captures", "make/unmake", "threshold SEE",
             "perft(4) startpos", "two-ply simulation"], compare.EXPECTED_WORK
        ):
            text += f"{name:<22} {123456:>15} {10:>15} {0.1:>9.2f}% {work:>12} {5:>12} moves\n"
    return text


class CompareTests(unittest.TestCase):
    def test_valid_output_parses_all_rows_and_contract(self):
        parsed = compare.parse_output(output(), True)
        self.assertEqual(parsed["values"], compare.VALUES)
        self.assertEqual(len(parsed["verdicts"].split(",")), 10)
        self.assertEqual([row["ops_per_iter"] for row in parsed["rows"]], compare.EXPECTED_WORK)

    def test_missing_or_duplicate_answers_are_rejected(self):
        with self.assertRaises(ValueError):
            compare.parse_output(output(verdicts="0:a1a2=1," * 10, rows=False), False)

    def test_wrong_work_quantum_is_rejected(self):
        with self.assertRaises(ValueError):
            compare.parse_output(output().replace("      197281", "      197280"), True)

    def test_missing_preflight_is_rejected(self):
        with self.assertRaises(ValueError):
            compare.parse_output(output().replace("preflight: PASS", "preflight: FAIL"), False)

    def test_wrong_value_vector_remains_visible_to_coordinator(self):
        parsed = compare.parse_output(output(values="100/1/1/1/1/20000", rows=False), False)
        self.assertNotEqual(parsed["values"], compare.VALUES)


if __name__ == "__main__":
    unittest.main()
