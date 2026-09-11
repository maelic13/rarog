#!/usr/bin/env python3
"""Export one Colosseum tournament to a self-contained PGN archive.

WHY THIS EXISTS. The games that back a measurement live in Colosseum's SQLite
database, which is large, this-machine-only, and not guaranteed to outlive the
measurement it justifies. A result whose evidence can only be re-derived from
that database is a promise that someone else is still storing your evidence.

This tool breaks that dependency. It is the ONLY tool here that knows anything
about Colosseum's schema; everything downstream reads PGN, which is portable,
harness-neutral, and readable by anything in ten years. Export once, hash the
result, and cite the hash.

WHAT GOES WHERE. The PGN is raw evidence: it belongs in ignored local storage
(`analysis/artifacts/`, `tools/results/`) or your own backups, NEVER in Git -
expect roughly 5 KB per game. What belongs in Git is this tool, the manifest's
hash, and whatever small frozen summary a downstream tool produces.

Usage:
  python tools/diag/export_tournament_pgn.py \\
      --tournament 41768fe9-fa35-43fc-99c8-ab42360e7361 \\
      --out analysis/artifacts/conversion-41768fe9.pgn

The database is opened READ-ONLY. `--db` defaults to Colosseum's standard
location on this machine and can point anywhere.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path

try:
    import chess.pgn
except ImportError:  # pragma: no cover - environment problem, not logic
    sys.exit("python-chess is required: pip install chess")

DEFAULT_DB = os.path.expandvars(r"%APPDATA%/Colosseum/data/colosseum.sqlite")


def engine_names(conn: sqlite3.Connection, tournament: str) -> dict[str, str]:
    """Map engine id -> "Name Version".

    Colosseum's `engines` table is empty; the names live inside each row's
    `engine_config_json`, which is why this reads `tournament_engines`.
    """
    rows = conn.execute(
        "select engine_id, "
        "json_extract(engine_config_json,'$.meta.name'), "
        "json_extract(engine_config_json,'$.meta.version') "
        "from tournament_engines where tournament_id=?",
        (tournament,),
    )
    names = {}
    for engine_id, name, version in rows:
        if name is None:
            raise SystemExit(f"engine {engine_id} has no meta.name in its config")
        names[engine_id] = f"{name} {version}".strip()
    if not names:
        raise SystemExit(f"no engines found for tournament {tournament}")
    return names


def export(db: Path, tournament: str, out: Path) -> dict:
    conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    try:
        names = engine_names(conn, tournament)
        rows = conn.execute(
            "select white_id, black_id, result, termination, pgn from games "
            "where tournament_id=? and status='finished'",
            (tournament,),
        )
        written = 0
        skipped = 0
        out.parent.mkdir(parents=True, exist_ok=True)
        with out.open("w", encoding="utf-8", newline="\n") as handle:
            for white_id, black_id, result, termination, pgn_text in rows:
                game = chess.pgn.read_game(io.StringIO(pgn_text))
                if game is None:
                    skipped += 1
                    continue
                # Rewrite the headers from the tournament's own engine table so
                # the archive is self-describing: nothing downstream should ever
                # need the database again to know who played.
                game.headers["White"] = names.get(white_id, white_id)
                game.headers["Black"] = names.get(black_id, black_id)
                game.headers["Result"] = _pgn_result(result.strip('"'))
                game.headers["Termination"] = termination.strip('"')
                game.headers["Event"] = f"Colosseum {tournament}"
                print(game, file=handle, end="\n\n")
                written += 1
    finally:
        conn.close()

    if written == 0:
        raise SystemExit(f"tournament {tournament} produced no finished games")

    digest = hashlib.sha256(out.read_bytes()).hexdigest()
    return {
        "tournament": tournament,
        "games": written,
        "unparsable_games_skipped": skipped,
        "engines": sorted(names.values()),
        "pgn": str(out),
        "pgn_sha256": digest,
        "pgn_bytes": out.stat().st_size,
        "exported_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "source_database": str(db),
    }


def _pgn_result(result: str) -> str:
    return {"WhiteWin": "1-0", "BlackWin": "0-1", "Draw": "1/2-1/2"}.get(result, "*")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--tournament", required=True, help="Colosseum tournament id")
    parser.add_argument("--out", required=True, type=Path, help="PGN file to write")
    parser.add_argument("--db", type=Path, default=Path(DEFAULT_DB))
    args = parser.parse_args()

    if not args.db.exists():
        raise SystemExit(f"database not found: {args.db}")

    manifest = export(args.db, args.tournament, args.out)
    manifest_path = args.out.with_suffix(".manifest.json")
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    print(f"wrote {manifest['games']} games to {manifest['pgn']}")
    print(f"  {manifest['pgn_bytes'] / 1_048_576:.1f} MB")
    print(f"  sha256 {manifest['pgn_sha256']}")
    if manifest["unparsable_games_skipped"]:
        print(f"  WARNING: skipped {manifest['unparsable_games_skipped']} unparsable game(s)")
    print(f"manifest: {manifest_path}")
    print("\nThe PGN is raw evidence: keep it out of Git. Cite the sha256 above.")


if __name__ == "__main__":
    main()
