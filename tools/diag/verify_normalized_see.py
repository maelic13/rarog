"""Verify a normalized SEE comparison bundle without rerunning benchmarks."""

import argparse
import hashlib
import json
from pathlib import Path
import statistics

import normalized_see_compare as compare


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    args = parser.parse_args()
    bundle = args.bundle.resolve()
    manifest = json.loads((bundle / "manifest.json").read_text(encoding="utf-8"))
    sources = json.loads((bundle / "source-manifest.json").read_text(encoding="utf-8"))
    preflights = manifest["preflights"]
    assert set(preflights) == {"rarog", "basilisk", "reckless"}
    assert {item["values"] for item in preflights.values()} == {compare.VALUES}
    assert len({item["verdicts"] for item in preflights.values()}) == 1
    assert len(next(iter(preflights.values()))["verdicts"].split(",")) == 10
    assert manifest["wire_proof"]["normal_probe"] == "false"
    assert manifest["wire_proof"]["absurd_probe"] == "true"
    assert manifest["wire_proof"]["exit"] == 0

    expected_runs = 3 * len(preflights)
    assert len(manifest["runs"]) == expected_runs
    for run in manifest["runs"]:
        assert run["host_busy_percent"] <= manifest["busy_limit_percent"]
        raw = (bundle / f"round{run['round']}-{run['engine']}.txt").read_text(encoding="utf-8")
        parsed = compare.parse_output(raw, True)
        assert parsed["values"] == compare.VALUES
        assert parsed["verdicts"] == preflights[run["engine"]]["verdicts"]
        assert parsed["rows"] == run["rows"]

    recomputed = {}
    for engine in preflights:
        rows = [next(row for row in run["rows"] if row["workload"] == "threshold SEE")
                for run in manifest["runs"] if run["engine"] == engine]
        rates = [row["ops_per_sec"] for row in rows]
        recomputed[engine] = {
            "median_ops_per_sec": statistics.median(rates), "round_medians": rates,
            "round_range_percent": 100 * (max(rates) - min(rates)) / statistics.median(rates),
            "max_within_run_mad_percent": max(row["mad_percent"] for row in rows),
        }
    assert recomputed == manifest["summary"]
    for name, expected in sources["artifacts_sha256"].items():
        assert digest(bundle / name) == expected, name
    print(f"normalized SEE bundle PASS: {expected_runs} timed runs, identical vector/verdicts, live injection wire")


if __name__ == "__main__":
    main()
