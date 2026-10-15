#!/usr/bin/env python3
"""Bounded exact-tree A/B/A and component attribution for Dragon inference."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import statistics
import subprocess
import time

from dragon_nnue_bench import idle_host, parse_bench
from dragon_nnue_probe import Session, digest, ready


def parse_micro(text):
    rows = []
    for name, round_, calls, ns, checksum in re.findall(
        r'^(full-scalar|portable|AVX2|AVX-VNNI)\t([0-2])\t(\d+)\t(\d+)\t(-?\d+)$', text, re.M
    ):
        rows.append(dict(kernel=name, round=int(round_), calls=int(calls), ns=int(ns), checksum=int(checksum)))
    names = {r['kernel'] for r in rows}
    if not {'full-scalar', 'portable'} <= names:
        raise AssertionError('missing scalar/reference records')
    keys = {(r['kernel'], r['round']) for r in rows}
    if len(rows) != len(names) * 3 or len(keys) != len(rows):
        raise AssertionError('expected three unique rounds per kernel')
    if any(r['calls'] != 65536 or r['ns'] <= 0 for r in rows):
        raise AssertionError('wrong evaluation count or timer')
    if len({r['checksum'] for r in rows}) != 1:
        raise AssertionError('kernel checksum mismatch')
    medians = {name: statistics.median(r['ns'] / r['calls'] for r in rows if r['kernel'] == name) for name in names}
    return {'rows': rows, 'median_ns_per_eval': medians,
            'speedup_vs_reference': {name: medians['full-scalar'] / value for name, value in medians.items()}}


def tree_signature(result):
    return [(r['position'], r['depth'], r['score'], r['nodes'], r['ebf']) for r in result['positions']]


def verify_component_sources(identity, binary_sha256, root):
    if identity['binary_sha256'] != binary_sha256:
        raise AssertionError('component binary identity changed')
    for name, expected in identity['sources_sha256_lf'].items():
        if hashlib.sha256((root/name).read_bytes().replace(b'\r\n',b'\n')).hexdigest() != expected:
            raise AssertionError('component source changed: '+name)


def bench(engine, mode, out, allow_busy):
    host = idle_host(allow_busy)
    session = Session(engine, out)
    try:
        session.send('uci')
        session.until('uciok')
        for name, value in [('Hash', 64), ('Threads', 1), ('Ponder', 'false'),
                            ('Dragon Eval File', '<embedded>'), ('Evaluation', mode), ('Dragon NNUE Scale', 72)]:
            session.send(f'setoption name {name} value {value}')
        ready(session)
        session.send('ucinewgame')
        ready(session)
        start = time.monotonic()
        session.send('bench 13')
        output = session.until('Nodes/second', timeout=900)
        result = parse_bench('\n'.join(output)+'\n')
        expected = (14_197_472, 2.557) if mode == 'Classical' else (17_728_094, 2.604)
        if (result['nodes'], result['ebf']) != expected:
            raise AssertionError(f'{mode} fingerprint changed')
        result.update(host=host, wall_seconds=time.monotonic()-start)
        return result
    finally:
        session.close()
        (out/'transcript.json').write_text(json.dumps(session.events, indent=2)+'\n', encoding='utf-8')
        (out/'stdout.log').write_text('\n'.join(e['line'] for e in session.events if e['direction']=='stdout')+'\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ['baseline', 'candidate', 'micro', 'out']:
        parser.add_argument('--'+option, required=True, type=Path)
    parser.add_argument('--reuse-component-screen',type=Path,
                        help='reuse an identical-source archived component run instead of repeating it')
    parser.add_argument('--allow-busy', action='store_true')
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    paths = {name: getattr(args, name).resolve() for name in ['baseline', 'candidate', 'micro']}
    hashes = {name: digest(path) for name, path in paths.items()}
    result = {'status':'running', 'paths':{k:str(v) for k,v in paths.items()},
              'sha256':hashes, 'busy_host_waiver':args.allow_busy, 'arms':{}}
    try:
        if args.reuse_component_screen:
            previous=args.reuse_component_screen.resolve()
            identity=json.loads((previous/'identity.json').read_text())
            verify_component_sources(identity,hashes['micro'],Path(__file__).resolve().parents[2])
            text=(previous/'micro.log').read_text()
            result['micro_host']=json.loads((previous/'host.json').read_text())
            result['reused_component_screen']=str(previous)
        else:
            result['micro_host'] = idle_host(args.allow_busy)
            micro = subprocess.run([str(paths['micro'])], cwd=out, capture_output=True,
                                   text=True, encoding='utf-8', timeout=120)
            micro.check_returncode()
            text=micro.stdout+micro.stderr
        (out/'micro.log').write_text(text, encoding='utf-8')
        result['micro'] = parse_micro(text)
        print('Component medians ns/eval:',result['micro']['median_ns_per_eval'],flush=True)
        for name, executable, mode in [('scalar-A1','baseline','Dragon NNUE'),
                                       ('optimized-B','candidate','Dragon NNUE'),
                                       ('scalar-A2','baseline','Dragon NNUE'),
                                       ('classical','candidate','Classical')]:
            directory = out/name
            directory.mkdir()
            record = bench(paths[executable], mode, directory, args.allow_busy)
            result['arms'][name] = record
            print(f"{name}: {record['nodes']:,} nodes / {record['ms']} ms / {record['nps']:,} NPS",flush=True)
            (out/'results.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
        arms = result['arms']
        if not tree_signature(arms['scalar-A1']) == tree_signature(arms['optimized-B']) == tree_signature(arms['scalar-A2']):
            raise AssertionError('per-position Dragon search tree changed')
        if hashes != {name:digest(path) for name,path in paths.items()}:
            raise AssertionError('binary changed during measurements')
        baseline = statistics.mean(arms[name]['nps'] for name in ['scalar-A1','scalar-A2'])
        result.update(status='pass', all_40_dragon_trees_identical=True,
                      engine_nps_speedup=arms['optimized-B']['nps']/baseline)
    except Exception as exc:
        result.update(status='fail', error=repr(exc))
        raise
    finally:
        (out/'results.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')


if __name__ == '__main__':
    main()
