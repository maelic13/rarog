"""Held-out loss of the full evaluation as a function of one king-danger input.

For each named input and each weight, write the tuner's complete default vector
with that one slot changed and read `rarog-texel --report-endgames`'s global
loss at a pinned K. The tuner evaluates the function the fits describe (lazy
shortcut off), so this is the surface the nonlinear pass saw when it left
`ks_weak_ring`, `ks_flank_attack` and `ks_shelter_storm` at zero (RAR-E21).

  python tools/diag/ks_weight_scan.py --tuner tools/texel-tuner/target/release/rarog-texel.exe \
      --dataset tools/texel/data/hce-v3-tb/validation.csv --k 1.34651 \
      --inputs ks_weak_ring,ks_flank_attack,ks_shelter_storm --weights 1,2,4 \
      --out tools/results/king-subterms-20261006/weight_scan.json
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

LOSS_RE = re.compile(r"global loss=([0-9.]+)")


def write_vector(base: list[str], name: str, value: int, path: str) -> None:
    hits = [i for i, line in enumerate(base) if line.split()[:2] == [name, "0"]]
    assert len(hits) == 1, f"{name} slot 0 found {len(hits)} times"
    lines = list(base)
    lines[hits[0]] = f"{name} 0 {value}"
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(lines) + "\n")


def global_loss(tuner: str, dataset: str, vector: str, k: str) -> float:
    proc = subprocess.run(
        [tuner, "--report-endgames", dataset, vector, "--fix-k", k],
        capture_output=True, text=True, check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"tuner exited {proc.returncode}: {proc.stderr[-400:]}")
    m = LOSS_RE.search(proc.stdout)
    if not m:
        raise RuntimeError(f"no global loss in tuner output: {proc.stdout[:200]}")
    return float(m.group(1))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--tuner", required=True)
    ap.add_argument("--dataset", required=True)
    ap.add_argument("--k", required=True)
    ap.add_argument("--inputs", required=True)
    ap.add_argument("--weights", default="1,2,4")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    tuner = os.path.abspath(args.tuner)
    dataset = os.path.abspath(args.dataset)
    with tempfile.TemporaryDirectory() as tmp:
        base_path = os.path.join(tmp, "base.txt")
        subprocess.run([tuner, "--write-defaults", base_path], check=True, capture_output=True)
        base = [l.rstrip("\n") for l in open(base_path, encoding="utf-8") if l.strip()]
        result = {"dataset": os.path.abspath(args.dataset), "k": args.k, "base_loss": None, "scan": {}}
        result["base_loss"] = global_loss(tuner, dataset, base_path, args.k)
        print(f"base loss {result['base_loss']:.8f}", flush=True)
        for name in args.inputs.split(","):
            result["scan"][name] = {}
            for w in args.weights.split(","):
                path = os.path.join(tmp, f"{name}_{w}.txt")
                write_vector(base, name, int(w), path)
                loss = global_loss(tuner, dataset, path, args.k)
                result["scan"][name][w] = {"loss": loss, "delta": loss - result["base_loss"]}
                print(f"  {name} = {w}: loss {loss:.8f} ({loss - result['base_loss']:+.8f})", flush=True)
    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(result, handle, indent=2)
    return 0


if __name__ == "__main__":
    sys.exit(main())
