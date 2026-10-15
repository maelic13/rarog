#!/usr/bin/env python3
"""One informational bench 13 per backend; no repeats, warm-up or strength claim."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import time

from dragon_nnue_probe import Session,digest,ready,dragon_uci_options

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
from nps_read import cpu_load

ROW=re.compile(r'^bench (\d+)/40\s+depth (\d+)\s+score (-?\d+)\s+nodes (\d+)\s+ebf ([\d.]+)\s+time (\d+)ms\s+nps (\d+)$',re.M)


def parse_bench(text):
    rows=[{'position':int(a),'depth':int(b),'score':int(c),'nodes':int(d),'ebf':float(e),'ms':int(f),'nps':int(g)} for a,b,c,d,e,f,g in ROW.findall(text)]
    if [row['position'] for row in rows]!=list(range(1,41)):
        raise AssertionError(f'expected all 40 positions, received {len(rows)}')
    result={'positions':rows}
    for key,pattern in [('nodes',r'^Nodes searched\s*:\s*(\d+)$'),('ms',r'^Total time \(ms\)\s*:\s*(\d+)$'),('nps',r'^Nodes/second\s*:\s*(\d+)$')]:
        values=re.findall(pattern,text,re.M)
        if len(values)!=1:raise AssertionError(f'expected one {key} aggregate')
        result[key]=int(values[0])
    ebf=re.findall(r'^Geomean EBF\s*:\s*([\d.]+)$',text,re.M)
    if len(ebf)!=1:raise AssertionError('expected one EBF aggregate')
    result['ebf']=float(ebf[0])
    if sum(row['nodes'] for row in rows)!=result['nodes']:raise AssertionError('node sum mismatch')
    if sum(row['ms'] for row in rows)!=result['ms']:raise AssertionError('time sum mismatch')
    if result['nps']!=result['nodes']*1000//max(result['ms'],1):raise AssertionError('NPS aggregate mismatch')
    return result


def idle_host(allow_busy=False):
    load=cpu_load()
    if load is None:raise RuntimeError('host CPU load is unavailable')
    command="Get-Process | Where-Object { $_.ProcessName -match '^(rarog.*|dragon.*|stockfish.*|basilisk.*|fastchess.*|colosseum.*|cargo|rustc)$' } | Select-Object ProcessName,Id | ConvertTo-Json -Compress"
    p=subprocess.run(['powershell','-NoProfile','-Command',command],capture_output=True,text=True,timeout=20,check=True)
    processes=json.loads(p.stdout) if p.stdout.strip() else []
    if isinstance(processes,dict):processes=[processes]
    if (load>10 or processes) and not allow_busy:raise RuntimeError(f'host is busy: CPU {load}%, processes {processes}')
    return {'cpu_percent':load,'engine_or_build_processes':processes}


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--engine',required=True,type=Path)
    p.add_argument('--model',required=True,type=Path)
    p.add_argument('--out',required=True,type=Path)
    p.add_argument('--allow-busy',action='store_true',help='explicit maintainer waiver for a rough run under background load')
    args=p.parse_args()
    out=args.out.resolve()
    out.mkdir(parents=True,exist_ok=False)
    results={'kind':'single informational bench per backend','depth':13,'threads':1,'hash_mb':64,'scale':72,'repeats':1,'engine_sha256':digest(args.engine),'model_sha256':digest(args.model),'arms':{},'busy_host_waiver':args.allow_busy}
    try:
        for mode in ['Classical','DragonNNUE']:
            idle=idle_host(args.allow_busy)
            session=Session(args.engine.resolve(),out)
            started=time.monotonic()
            try:
                session.send('uci')
                names=dragon_uci_options(session.until('uciok'))
                for name,value in [('Hash',64),('Threads',1),('Ponder','false'),(names['file'],str(args.model.resolve())),('Evaluation',names['value'] if mode=='DragonNNUE' else mode),(names['scale'],72)]:
                    session.send(f'setoption name {name} value {value}')
                ready(session)
                session.send('ucinewgame')
                ready(session)
                session.send('bench 13')
                output=session.until('Nodes/second',timeout=900)
                text='\n'.join(output)+'\n'
                record=parse_bench(text)
                if mode=='Classical' and (record['nodes'],record['ebf'])!=(14_197_472,2.557):
                    raise AssertionError('default HCE fingerprint changed')
                record.update(idle_host=idle,wall_seconds=round(time.monotonic()-started,3))
                results['arms'][mode]=record
                print(f"{mode}: {record['nodes']:,} nodes, {record['ms']} ms, {record['nps']:,} NPS, 40/40 positions",flush=True)
            finally:
                session.close()
                (out/(mode.replace(' ','-')+'.log')).write_text('\n'.join(event['line'] for event in session.events if event['direction']=='stdout')+'\n',encoding='utf-8')
                (out/(mode.replace(' ','-')+'-transcript.json')).write_text(json.dumps(session.events,indent=2)+'\n',encoding='utf-8')
        if digest(args.engine)!=results['engine_sha256']:raise AssertionError('measured binary changed')
        hce=results['arms']['Classical']
        nnue=results['arms']['DragonNNUE']
        results.update(status='pass',nnue_to_hce_nps=nnue['nps']/hce['nps'],nnue_to_hce_time=nnue['ms']/hce['ms'])
    except Exception as exc:
        results.update(status='fail',error=repr(exc))
        raise
    finally:
        (out/'results.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')


if __name__=='__main__':main()
