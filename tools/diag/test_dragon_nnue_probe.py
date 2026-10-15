import unittest

from dragon_nnue_probe import compare,one_eval,one_numeric

class ProbeTests(unittest.TestCase):
    def test_static_record_is_unique_and_perspectives_are_explicit(self):
        self.assertEqual(one_eval(['info string eval backend=DragonNNUE stm=52 white=-52','readyok']),{'backend':'DragonNNUE','stm':52,'white':-52})
        for lines in [[],['info string eval backend=DragonNNUE stm=x white=5'],['info string eval backend=Classical stm=1 white=1']*2]:
            with self.assertRaises(AssertionError):one_eval(lines)

    def test_native_record_requires_one_integer(self):
        self.assertEqual(one_numeric(['-52','readyok']),-52)
        for lines in [[],['52','53']]:
            with self.assertRaises(AssertionError):one_numeric(lines)

    def test_comparison_rejects_missing_extra_and_changed_scores(self):
        compare([1,2],[1,2])
        for actual in [[1],[1,2,3],[1,3]]:
            with self.assertRaises(AssertionError):compare([1,2],actual)

if __name__=='__main__':unittest.main()
