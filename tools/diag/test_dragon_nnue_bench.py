import unittest

from dragon_nnue_bench import parse_bench,idle_host
from unittest.mock import patch
from types import SimpleNamespace

def sample():
    rows=''.join(f'bench {i}/40  depth 13  score 0  nodes 10  ebf 1.00  time 2ms  nps 5000\n' for i in range(1,41))
    return rows+'Nodes searched  : 400\nGeomean EBF     : 1.000\nTotal time (ms) : 80\nNodes/second    : 5000\n'

class BenchTests(unittest.TestCase):
    def test_busy_host_requires_explicit_waiver(self):
        with patch('dragon_nnue_bench.cpu_load',return_value=36.0),patch('dragon_nnue_bench.subprocess.run',return_value=SimpleNamespace(stdout='')):
            with self.assertRaises(RuntimeError):idle_host()
            self.assertEqual(idle_host(True)['cpu_percent'],36.0)

    def test_counts_and_aggregates(self):
        result=parse_bench(sample())
        self.assertEqual((result['nodes'],result['ms'],result['nps']),(400,80,5000))

    def test_rejects_missing_duplicate_and_wrong_aggregates(self):
        for text in [sample().replace('bench 1/40','missing 1/40'),sample()+sample(),sample().replace('Nodes searched  : 400','Nodes searched  : 399'),sample().replace('Nodes/second    : 5000','Nodes/second    : 4999')]:
            with self.assertRaises(AssertionError):parse_bench(text)

if __name__=='__main__':unittest.main()
