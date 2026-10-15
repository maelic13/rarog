#!/usr/bin/env python3
"""Exact Rarog static-evaluator checks against the immutable Dragon nnue command."""
import argparse
import hashlib
import json
from pathlib import Path
import queue
import re
import subprocess
import threading
import time

ROOT=Path(__file__).resolve().parents[2]
IDENTITIES={
    'base':'77d5042536785f337a2263679b722e4bb391d3bac44bc33fb50a310f7c0954ee',
    'avx2':'90ca3d6aac9c19f46b9b443ebbea468d2f50b3a487f038e6a964ee7f3cf6e69b',
    'model':'c6e25a7fde5971dc47e78e19764c82b8d18a438bbd5ee4da065d061115064417',
}
EVAL=re.compile(r'^info string eval backend=(\S+) stm=(-?\d+) white=(-?\d+)$')
START='rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class Session:
    def __init__(self, executable, cwd):
        self.events=[]
        self.lines=queue.Queue()
        self.process=subprocess.Popen([str(executable)],cwd=cwd,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,encoding='utf-8',errors='replace',bufsize=1)
        self.threads=[]
        for label,stream in [('stdout',self.process.stdout),('stderr',self.process.stderr)]:
            thread=threading.Thread(target=self.reader,args=(label,stream),daemon=True)
            thread.start()
            self.threads.append(thread)

    def reader(self,label,stream):
        for line in stream:
            line=line.rstrip('\r\n')
            self.events.append({'direction':label,'line':line})
            if label=='stdout':self.lines.put(line)
        if label=='stdout':self.lines.put(None)

    def send(self,line):
        self.events.append({'direction':'stdin','line':line})
        self.process.stdin.write(line+'\n')
        self.process.stdin.flush()

    def until(self,prefix,timeout=30):
        deadline=time.monotonic()+timeout
        result=[]
        while True:
            remaining=deadline-time.monotonic()
            if remaining<=0:raise TimeoutError(prefix)
            try:line=self.lines.get(timeout=remaining)
            except queue.Empty as exc:raise TimeoutError(prefix) from exc
            if line is None:raise RuntimeError('engine closed stdout')
            result.append(line)
            if line.startswith(prefix):return result

    def close(self):
        if self.process.poll() is None:
            try:
                self.send('quit')
                self.process.wait(timeout=5)
            except (OSError,subprocess.TimeoutExpired):
                self.process.kill()
                self.process.wait(timeout=5)
        for thread in self.threads:thread.join(timeout=2)
        for stream in [self.process.stdin,self.process.stdout,self.process.stderr]:stream.close()


def one_eval(lines):
    rows=[EVAL.fullmatch(line) for line in lines if line.startswith('info string eval ')]
    if len(rows)!=1 or rows[0] is None:raise AssertionError(f'expected exactly one eval record: {lines}')
    return {'backend':rows[0][1],'stm':int(rows[0][2]),'white':int(rows[0][3])}


def one_numeric(lines):
    rows=[int(line) for line in lines if re.fullmatch(r'-?\d+',line)]
    if len(rows)!=1:raise AssertionError(f'expected exactly one native diagnostic: {lines}')
    return rows[0]


def compare(expected,actual):
    if len(expected)!=len(actual):raise AssertionError('record count mismatch')
    mismatches=[i for i,(a,b) in enumerate(zip(expected,actual)) if a!=b]
    if mismatches:raise AssertionError(f'{len(mismatches)} mismatches, first {mismatches[:8]}')


def fixtures():
    cases=[]
    for name in ['dragon-nnue-reference.tsv','dragon-nnue-expanded.tsv']:
        for row in (ROOT/'tests/fixtures'/name).read_text(encoding='utf-8').splitlines():
            if row.startswith('#'):continue
            label,scale,expected,fen=row.split('\t')
            cases.append({'name':label,'scale':int(scale),'expected':int(expected),'position':'fen '+fen})
    if len(cases)!=2738:raise AssertionError('fixture row count changed')
    sequences=[
        ('opening',START,'e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6 c2c3 e8g8 h2h3 c6b8 d2d4 b8d7'),
        ('ep','4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1','e5d6 e8d7 e1d2 d7d6'),
    ]+[(f'promotion-{piece}','8/P7/7k/8/8/8/7p/4K3 w - - 0 1',f'a7a8{piece} h2h1{piece}') for piece in 'qnbr']
    for label,fen,moves in sequences:
        prefix=[]
        for move in moves.split():
            prefix.append(move)
            cases.append({'name':f'live-{label}-{len(prefix)}','scale':72,'position':'fen '+fen+' moves '+' '.join(prefix)})
    assert len(cases)==2770
    return sorted(cases,key=lambda c:c['scale'])


def dragon_uci_options(handshake):
    """Negotiate old/new spellings so archived baselines remain measurable."""
    lines = handshake.splitlines() if isinstance(handshake, str) else handshake
    combo = [line for line in lines if line.startswith('option name Evaluation type combo ')]
    if len(combo) != 1:
        raise AssertionError('expected one Evaluation combo')
    values = combo[0].split(' var ')[1:]
    choices = [name for name in ['DragonNNUE', 'Dragon NNUE'] if name in values]
    if len(choices) != 1:
        raise AssertionError('ambiguous or missing Dragon evaluator')
    result = {'value': choices[0]}
    for key, names, kind in [('file', ['DragonEvalFile', 'Dragon Eval File'], 'string'),
                              ('scale', ['DragonNNUEScale', 'Dragon NNUE Scale'], 'spin')]:
        choices = [name for name in names if any(line.startswith(f'option name {name} type {kind} ') for line in lines)]
        if len(choices) != 1:
            raise AssertionError('ambiguous or missing Dragon '+key)
        result[key] = choices[0]
    return result


def ready(session):
    session.send('isready')
    return session.until('readyok')


def evaluate(session,position):
    session.send('position '+position)
    session.send('eval')
    return one_eval(ready(session))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--engine',type=Path,required=True)
    p.add_argument('--dragon-base',type=Path,required=True)
    p.add_argument('--dragon-avx2',type=Path,required=True)
    p.add_argument('--model',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    args=p.parse_args()
    out=args.out.resolve()
    out.mkdir(parents=True,exist_ok=False)
    runtime=out/'runtime'
    runtime.mkdir()
    cases=fixtures()
    (out/'inputs.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
    paths={'base':args.dragon_base.resolve(),'avx2':args.dragon_avx2.resolve(),'model':args.model.resolve()}
    for label,path in paths.items():
        if digest(path)!=IDENTITIES[label]:raise AssertionError(f'{label} identity mismatch')
    result={'status':'running','engine':str(args.engine.resolve()),'engine_sha256':digest(args.engine),'identities':IDENTITIES,'expected_pairs':5540,'records':[]}
    sessions={}
    try:
        host=Session(args.engine.resolve(),runtime)
        sessions['rarog']=host
        host.send('uci')
        handshake='\n'.join(host.until('uciok'))
        names=dragon_uci_options(handshake)
        for command in ['setoption name Hash value 64','setoption name Threads value 1','setoption name Ponder value false']:
            host.send(command)
        before=evaluate(host,'fen '+START)
        assert before['backend']=='Classical'
        # A GUI may send the mode before the file. Failed load stays Classical;
        # supplying the model must resolve the remembered selection.
        host.send('setoption name Evaluation value '+names['value'])
        ready(host)
        host.send('setoption name '+names['file']+' value '+str(paths['model']))
        ready(host)
        enabled=evaluate(host,'fen '+START)
        assert enabled=={'backend':'DragonNNUE','stm':52,'white':52}
        for variant in ['base','avx2']:
            native=Session(paths[variant],runtime)
            sessions[variant]=native
            native.send('uci')
            native.until('uciok')
            for name,value in [('Hash',64),('Threads',1),('Ponder','false'),('OwnBook','false'),('Use Syzygy','false'),('Use Regular Eval','false')]:
                native.send(f'setoption name {name} value {value}')
            ready(native)
            scale=None
            actual=[]
            expected=[]
            for case in cases:
                if case['scale']!=scale:
                    scale=case['scale']
                    host.send(f"setoption name {names['scale']} value {scale}")
                    native.send(f'setoption name NNUE_Scale value {scale}')
                    ready(host)
                    ready(native)
                value=evaluate(host,case['position'])
                assert value['backend']=='DragonNNUE'
                native.send('position '+case['position'])
                native.send('nnue')
                reference=one_numeric(ready(native))
                if 'expected' in case:assert reference==case['expected']
                actual.append(value['white'])
                expected.append(reference)
                result['records'].append({'variant':variant,'name':case['name'],'scale':scale,'rarog':value,'dragon':reference})
            compare(expected,actual)
            changed=actual.copy()
            changed[len(changed)//2]+=1
            try:compare(expected,changed)
            except AssertionError:pass
            else:raise AssertionError('changed-score control was accepted')
            native.close()
            print(f'{variant}: {len(actual)} exact static matches; changed-score control rejected',flush=True)
        host.send('setoption name Evaluation value Classical')
        restored=evaluate(host,'fen '+START)
        assert restored==before
        result.update(status='pass',exact_pairs=len(result['records']),classical_before=before,classical_restored=restored,negative_controls=2)
    except Exception as exc:
        result.update(status='fail',error=repr(exc))
        raise
    finally:
        for label,session in sessions.items():
            session.close()
            (out/f'{label}-transcript.json').write_text(json.dumps(session.events,indent=2)+'\n',encoding='utf-8')
        (out/'results.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')


if __name__=='__main__':main()
