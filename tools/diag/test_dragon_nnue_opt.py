import hashlib
from pathlib import Path
import tempfile
import unittest
from dragon_nnue_opt import parse_micro, tree_signature, verify_component_sources


def sample():
    return ''.join(f'{name}\t{round_}\t65536\t{ns}\t123\n'
                   for name, ns in [('full-scalar',655360),('portable',327680)]
                   for round_ in range(3))


class OptimizationParserTests(unittest.TestCase):
    def test_reuse_requires_same_binary_and_source_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            (root/'lib.rs').write_bytes(b'known\r\n')
            identity={'binary_sha256':'binary','sources_sha256_lf':{'lib.rs':hashlib.sha256(b'known\n').hexdigest()}}
            verify_component_sources(identity,'binary',root)
            with self.assertRaises(AssertionError):verify_component_sources(identity,'changed',root)
            (root/'lib.rs').write_bytes(b'changed\n')
            with self.assertRaises(AssertionError):verify_component_sources(identity,'binary',root)

    def test_exact_round_counts_and_component_ratio(self):
        result=parse_micro(sample())
        self.assertEqual(result['median_ns_per_eval']['portable'],5)
        self.assertEqual(result['speedup_vs_reference']['portable'],2)

    def test_rejects_missing_extra_wrong_count_and_changed_checksum(self):
        for text in [sample().split('\n',1)[1],sample()+sample(),
                     sample().replace('65536','65535',1),sample().replace('\t123\n','\t124\n',1)]:
            with self.assertRaises(AssertionError):parse_micro(text)

    def test_tree_signature_preserves_deterministic_fields(self):
        row=dict(position=1,depth=13,score=-52,nodes=100,ebf=1.5,ms=2,nps=50000)
        a=tree_signature({'positions':[row]})
        self.assertEqual(a,tree_signature({'positions':[row|{'ms':4,'nps':25000}]}))
        self.assertNotEqual(a,tree_signature({'positions':[row|{'nodes':101}]}))


if __name__=='__main__':unittest.main()
