"""Apply PLAN rule 7c's stop rule to a finished tune block.

    python tools/spsa_block_rule.py <group> <run dir> [--min-moved 3]

Reads the block's `result.json` and the group's `config_<group>.json` (the
registered steps), counts the coordinates whose rounded tuned value lies at
least one step from the block's own seeds (`original`), and prints the count
with the movers. Exit 0 means the next block runs (count >= --min-moved);
exit 1 means the tune stops with this block's values as theta; exit 2 is a
usage or artifact error. The decision is written beside the run as
`block-rule.txt`, so a chained launch leaves a record.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("group")
    parser.add_argument("run_dir")
    parser.add_argument("--min-moved", type=int, default=3)
    args = parser.parse_args()

    run = pathlib.Path(args.run_dir)
    result_path = run / "result.json"
    config_path = ROOT / "spsa_configs" / f"config_{args.group}.json"
    for path in (result_path, config_path):
        if not path.is_file():
            print(f"missing: {path}", file=sys.stderr)
            return 2
    result = json.loads(result_path.read_text(encoding="utf-8"))
    config = json.loads(config_path.read_text(encoding="utf-8"))

    driver = result.get("driver", {})
    tuned = result.get("tuned_result", {})
    horizon = int(tuned.get("settings", {}).get("iterations", 0))
    done = int(tuned.get("completed_iterations", 0))
    if driver.get("status") != "completed" or done != horizon or horizon == 0:
        print(f"not a completed block: status {driver.get('status')!r}, {done} of {horizon} iterations", file=sys.stderr)
        return 2

    parameters = tuned.get("parameters", [])
    names = [p["name"] for p in parameters]
    if names != list(config.keys()):
        print(f"surface mismatch: run tuned {len(names)} coordinates, config_{args.group}.json lists {len(config)}", file=sys.stderr)
        return 2

    movers = []
    for p in parameters:
        step = float(config[p["name"]]["step"])
        moved = (int(p["tuned"]) - int(p["original"])) / step
        if abs(moved) >= 1.0:
            movers.append((p["name"], int(p["original"]), int(p["tuned"]), moved))
    movers.sort(key=lambda m: -abs(m[3]))

    verdict = "CONTINUE" if len(movers) >= args.min_moved else "STOP"
    lines = [
        f"block {run} ({done} iterations): {len(movers)} of {len(parameters)} coordinates moved >= 1 step "
        f"from the block's seeds; rule needs {args.min_moved}: {verdict}",
    ]
    lines += [f"  {n:32} {o:>8} -> {t:<8} {m:+.2f} steps" for n, o, t, m in movers]
    text = "\n".join(lines)
    print(text)
    (run / "block-rule.txt").write_text(text + "\n", encoding="utf-8")
    return 0 if verdict == "CONTINUE" else 1


if __name__ == "__main__":
    sys.exit(main())
