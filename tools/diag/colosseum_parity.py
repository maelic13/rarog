"""Compare a Colosseum dry run against a recorded `sprt.ps1` manifest, field by field.

The two harnesses are only interchangeable if they resolve the same conditions.
"They look the same" is not a check, so this reads both records mechanically and
reports every field that differs.

    python tools/diag/colosseum_parity.py \
        --manifest tools/results/sprt_<a>_vs_<b>_<stamp>.manifest.txt \
        --dry-run  tools/results/colosseum_sprt_<name>_<stamp>.dry-run.json

Exit status 0 means every comparable field agreed; 1 means at least one did not,
and each one is printed. `--json` emits the full comparison instead.

Not comparable, by design rather than by omission, and reported as `differs by
design`: fastchess starts two engine processes per game while Colosseum keeps
them per slot (Colosseum 10.9u), and fastchess pins one logical CPU per game
while Colosseum allocates a whole physical core. The CORE SET is compared, which
is the property RAR-M48 made mandatory; the sibling logical CPU is not.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import sys

MANIFEST_LINE = re.compile(r"^(?P<key>[A-Za-z0-9_]+):\s*(?P<value>.*)$")
DESIGN = re.compile(
    r"SPRT\s+elo0=(?P<elo0>-?[\d.]+)\s+elo1=(?P<elo1>-?[\d.]+)\s+"
    r"alpha=(?P<alpha>[\d.]+)\s+beta=(?P<beta>[\d.]+)\s+model=(?P<model>\w+)"
)
CLOCK = re.compile(r"tc=(?P<base>[\d.]+)\+(?P<inc>[\d.]+)")
MARGIN = re.compile(r"timemargin=(?P<margin>\d+)ms")


def read_manifest(path: pathlib.Path) -> dict[str, str]:
    fields: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8-sig").splitlines():
        match = MANIFEST_LINE.match(line.strip())
        if match:
            fields[match.group("key")] = match.group("value").strip()
    if "engineA" not in fields:
        raise SystemExit(f"{path} does not look like an sprt.ps1 manifest (no engineA line)")
    return fields


def sha256(path: pathlib.Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest().upper()


def slot_physical_cpus(resolved: dict) -> list[int]:
    """The lowest logical CPU of every core a slot is allocated.

    fastchess records one logical CPU per game and Colosseum records both SMT
    siblings of a physical core, so the comparable quantity is the set of cores,
    named as fastchess names them: the even (first) logical CPU of each.
    """
    cpus: set[int] = set()
    for slot in resolved.get("execution", {}).get("slots", []):
        for side in ("engine_a", "engine_b", "engine"):
            allocation = slot.get(side, {}).get("allocation", {})
            for cpu in allocation.get("cpus", []) or []:
                cpus.add(int(cpu["number"]))
    return sorted(cpu for cpu in cpus if cpu % 2 == 0)


def engine_options(engine: dict) -> dict[str, str]:
    return {name: str(spec["value"]) for name, spec in (engine.get("options") or {}).items()}


def compare(manifest: dict[str, str], dry: dict) -> list[dict]:
    resolved = dry["resolved_configuration"]
    rows: list[dict] = []

    def check(field: str, fastchess, colosseum, ok: bool | None = None, note: str = "") -> None:
        rows.append(
            {
                "field": field,
                "fastchess": fastchess,
                "colosseum": colosseum,
                "equal": (fastchess == colosseum) if ok is None else ok,
                "note": note,
            }
        )

    design_match = DESIGN.search(manifest.get("test_design", ""))
    parameters = resolved.get("design", {}).get("parameters", {})
    if design_match:
        check("sprt.model", design_match.group("model"), str(parameters.get("model")))
        for bound in ("elo0", "elo1", "alpha", "beta"):
            check(f"sprt.{bound}", float(design_match.group(bound)), float(parameters.get(bound, "nan")))
        budget = int(manifest.get("game_budget", "0"))
        check("game_budget", budget, int(resolved.get("design", {}).get("max_pairs", 0)) * 2,
              note="fastchess counts games, Colosseum caps pairs")
    else:
        check("sprt.design", manifest.get("test_design"), "(no SPRT design in the dry run)", ok=False,
              note="the manifest does not record an SPRT; compare a gate against a gate")

    clock = CLOCK.search(manifest.get("time_control", ""))
    margin = MARGIN.search(manifest.get("time_control", ""))
    controls = {
        name: resolved[name]
        for name in ("engine_a_time_control", "engine_b_time_control", "engine_time_control")
        if name in resolved
    }
    if clock:
        base_ms = round(float(clock.group("base")) * 1000)
        inc_ms = round(float(clock.group("inc")) * 1000)
        for name, control in controls.items():
            increment = control.get("control", {}).get("Increment", {})
            check(f"{name}.base_ms", base_ms, int(increment.get("base_ms", -1)))
            check(f"{name}.inc_ms", inc_ms, int(increment.get("inc_ms", -1)))
            if margin:
                check(f"{name}.margin_ms", int(margin.group("margin")), int(control.get("margin_ms", -1)))
    else:
        check("time_control", manifest.get("time_control"), "(clock only)", ok=False,
              note="a fixed-movetime or fixed-nodes manifest is not comparable with a clock run")

    adjudication = resolved.get("adjudication", {})
    manifest_adjudicated = not manifest.get("adjudication", "").startswith("none")
    colosseum_adjudicated = any(adjudication.get(rule) is not None for rule in ("draw", "resign", "max_moves"))
    check("adjudication", "on" if manifest_adjudicated else "none",
          "on" if colosseum_adjudicated else "none")

    engines = [("engineA", "engine_a"), ("engineB", "engine_b")]
    for manifest_key, resolved_key in engines:
        if resolved_key not in resolved:
            check(manifest_key, manifest.get(manifest_key), "(absent)", ok=False)
            continue
        wanted = manifest.get(manifest_key, "")
        _, _, path = wanted.partition("=")
        manifest_path = pathlib.Path(path.strip() or wanted)
        colosseum_path = pathlib.Path(resolved[resolved_key]["executable"])
        check(f"{manifest_key}.executable", str(manifest_path).lower(), str(colosseum_path).lower())
        recorded = manifest.get(f"{manifest_key}_sha256", "").upper()
        actual = sha256(colosseum_path)
        check(f"{manifest_key}.sha256", recorded, actual or "(binary missing)",
              note="hashed from the executable the dry run resolved")

        options = engine_options(resolved[resolved_key])
        check(f"{manifest_key}.Hash", manifest.get("hash_mb"), options.get("Hash"))
        threads = manifest.get("threads", "")
        if "=" not in threads:
            check(f"{manifest_key}.Threads", threads, options.get("Threads"))
        extra = sorted(set(options) - {"Hash", "Threads"})
        recorded_options = manifest.get("optionsA" if manifest_key == "engineA" else "optionsB", "(none)")
        wanted_extra = [] if recorded_options == "(none)" else sorted(
            option.split("=", 1)[0] for option in recorded_options.split()
        )
        check(f"{manifest_key}.extra_options", wanted_extra, extra)

    openings = resolved.get("openings", {})
    book = pathlib.Path(manifest.get("book", ""))
    check("book", str(book).lower(), str(pathlib.Path(openings.get("path", ""))).lower())
    check("book_sha256", manifest.get("book_sha256", "").upper(),
          sha256(pathlib.Path(openings.get("path", ""))) or "(book missing)")
    check("opening_order", manifest.get("opening_order", "").lower(), str(openings.get("order", "")).lower())
    check("opening_seed", int(manifest.get("opening_seed", "-1")), int(resolved.get("master_seed", -2)))

    check("concurrency", int(manifest.get("concurrency", "-1")),
          int(resolved.get("execution", {}).get("concurrency", -2)))

    affinity = manifest.get("affinity_cpus", "")
    if affinity.startswith("("):
        check("affinity_cpus", affinity, "(placement auto)", ok=False,
              note="the fastchess run dropped affinity; a Colosseum run always places")
    else:
        recorded_cpus = [int(cpu) for cpu in affinity.split(",") if cpu.strip()]
        check("affinity_cpus", recorded_cpus, slot_physical_cpus(resolved),
              note="fastchess pins one logical CPU per game; Colosseum allocates the whole core")

    rows.append(
        {
            "field": "engine_processes",
            "fastchess": "per-game",
            "colosseum": resolved.get("engine_processes"),
            "equal": True,
            "note": "differs by design (Colosseum 10.9u): start-up work is paid once per slot",
        }
    )
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--manifest", required=True, type=pathlib.Path)
    parser.add_argument("--dry-run", required=True, type=pathlib.Path, dest="dry_run")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    manifest = read_manifest(args.manifest)
    dry = json.loads(args.dry_run.read_text(encoding="utf-8-sig"))
    if dry.get("type") != "dry-run":
        raise SystemExit(f"{args.dry_run} is not a colosseum-cli dry run")

    rows = compare(manifest, dry)
    mismatches = [row for row in rows if not row["equal"]]

    if args.json:
        print(json.dumps({"rows": rows, "mismatches": len(mismatches)}, indent=2, default=str))
    else:
        width = max(len(row["field"]) for row in rows)
        for row in rows:
            mark = "ok  " if row["equal"] else "DIFF"
            print(f"{mark} {row['field']:<{width}}  fastchess={row['fastchess']!r}  colosseum={row['colosseum']!r}"
                  + (f"  [{row['note']}]" if row["note"] else ""))
        print(f"\n{len(rows) - len(mismatches)}/{len(rows)} fields agree", file=sys.stderr)
    return 1 if mismatches else 0


if __name__ == "__main__":
    raise SystemExit(main())
