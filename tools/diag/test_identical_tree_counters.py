"""The ETW dump reader of identical_tree_counters.py on hand-made events."""
import unittest

from identical_tree_counters import spread, thread_counters

TID, OTHER, CPU = 100, 200, 6


def pmc(stamp, new, cycles, instructions, misses):
    return f"  Pmc, {stamp}, {new}, {cycles}, {instructions}, {misses}"


def switch(stamp, new, old, cpu=CPU):
    fields = ["CSwitch", stamp, "a.exe (1)", new, 8, -1, 0, 0, "b.exe (2)", old, 8, -1, "Waiting", "Executive",
              "NonSwap", 0, cpu, 0, 0, 0, 0, "Utility", "Important"]
    return "  " + ", ".join(str(f) for f in fields)


class ThreadCountersTest(unittest.TestCase):
    def test_sums_only_the_threads_stretches(self):
        lines = [
            pmc(10, TID, 100, 1000, 1), switch(10, TID, OTHER),
            pmc(20, OTHER, 150, 1500, 3), switch(20, OTHER, TID),
            pmc(30, TID, 900, 9000, 90), switch(30, TID, OTHER),
            pmc(50, 0, 1000, 10000, 100), switch(50, 0, TID),
        ]
        c = thread_counters(lines, TID, CPU)
        self.assertEqual((c.cycles, c.instructions, c.misses), (150, 1500, 12))
        self.assertEqual((c.micros, c.stretches), (30, 2))

    def test_other_cpus_and_headers_are_ignored(self):
        lines = [
            "  Pmc,  TimeStamp,   ThreadID, TotalCycles, InstructionRetired, BranchMispredictions",
            pmc(10, TID, 100, 1000, 1), switch(10, TID, OTHER),
            pmc(15, OTHER, 7, 7, 7), switch(15, OTHER, 0, cpu=3),
            pmc(20, 0, 300, 3000, 3), switch(20, 0, TID),
        ]
        self.assertEqual(thread_counters(lines, TID, CPU).cycles, 200)

    def test_a_switch_without_its_pmc_is_refused(self):
        lines = [pmc(10, TID, 100, 1000, 1), switch(10, TID, OTHER), switch(20, 0, TID)]
        with self.assertRaisesRegex(AssertionError, "no Pmc"):
            thread_counters(lines, TID, CPU)

    def test_a_pmc_of_another_thread_is_refused(self):
        lines = [pmc(10, OTHER, 100, 1000, 1), switch(10, TID, OTHER)]
        with self.assertRaisesRegex(AssertionError, "no Pmc"):
            thread_counters(lines, TID, CPU)

    def test_a_thread_on_another_cpu_is_refused(self):
        lines = [pmc(10, TID, 100, 1000, 1), switch(10, TID, OTHER, cpu=3)]
        with self.assertRaisesRegex(AssertionError, "on CPU 3"):
            thread_counters(lines, TID, CPU)

    def test_a_thread_still_running_at_the_end_is_refused(self):
        lines = [pmc(10, TID, 100, 1000, 1), switch(10, TID, OTHER)]
        with self.assertRaisesRegex(AssertionError, "still on the CPU"):
            thread_counters(lines, TID, CPU)

    def test_a_first_switch_out_is_refused(self):
        lines = [pmc(10, 0, 100, 1000, 1), switch(10, 0, TID)]
        with self.assertRaisesRegex(AssertionError, "without entering"):
            thread_counters(lines, TID, CPU)


class SpreadTest(unittest.TestCase):
    def test_spread_of_medians(self):
        values = [100, 100, 100, 110, 110, 110]
        labels = ["a", "a", "a", "b", "b", "b"]
        self.assertAlmostEqual(spread(values, labels), 10 / 105)


if __name__ == "__main__":
    unittest.main()
