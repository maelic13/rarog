#!/usr/bin/env python3
"""Write fastchess's shuffled opening order as an ordered EPD for Colosseum.

Every datagen corpus since ``hce-v3`` replays the same starts: one book in
fastchess's seeded random order, consumed from start 1. Colosseum shuffles with
its own generator, so the same seed there is a different sequence. fastchess's
``Round`` tag is the opening's offset in its order (round ``r`` of a segment
from ``start`` played opening ``start + r - 1``), so the order can be recovered
from the PGNs fastchess already wrote and played back by Colosseum in
sequential order.

Each source must carry its ``rarog-fastchess-datagen-v2`` manifest naming the
same book (by SHA-256) and seed. Sources must agree wherever they overlap; the
sequence is written from index 1 up to the first index no source covers, as
the book's own lines, with a sidecar that binds it to the book and sources.

    python tools/texel/opening_sequence.py --book tools/texel/data/phase_book_v1.epd \\
        --seed 10403 --out tools/texel/data/phase_book_v1-s10403.order.epd \\
        tools/texel/data/selfplay-b9head-n8000-s1-g612747.pgn
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

from extract import sha256_file

SCHEMA = "rarog-opening-sequence-v1"
SOURCE_SCHEMA = "rarog-fastchess-datagen-v2"


def position_key(fen: str) -> str:
    """The four position fields; the move counters are not part of an opening."""
    fields = fen.split()
    if len(fields) < 4:
        raise ValueError(f"not a FEN or four-field EPD: {fen!r}")
    return " ".join(fields[:4])


def read_book(path: Path) -> dict[str, str]:
    """Map each opening's position key to its exact book line."""
    lines: dict[str, str] = {}
    with path.open(encoding="utf-8") as handle:
        for number, raw in enumerate(handle, 1):
            line = raw.rstrip("\r\n")
            key = position_key(line)
            if key in lines:
                raise SystemExit(f"{path}:{number} repeats opening {key!r}")
            lines[key] = line
    return lines


def read_rounds(path: Path) -> dict[int, str]:
    """Round number -> start position key, from the tag lines of a fastchess PGN."""
    rounds: dict[int, str] = {}
    current: int | None = None
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            if line.startswith('[Round "'):
                current = int(line[8:line.index('"', 8)])
            elif line.startswith('[FEN "'):
                if current is None:
                    raise SystemExit(f"{path}: a FEN tag precedes its Round tag")
                if current in rounds:
                    raise SystemExit(f"{path}: round {current} appears twice")
                rounds[current] = position_key(line[6:line.rindex('"')])
                current = None
    return rounds


def load_source(path: Path, book_sha256: str, seed: int) -> tuple[dict[int, str], dict]:
    manifest_path = path.with_suffix(".manifest.json")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema") != SOURCE_SCHEMA:
        raise SystemExit(f"{manifest_path}: schema is not {SOURCE_SCHEMA}")
    output = manifest.get("output", {})
    if Path(output.get("path", "")).resolve() != path.resolve():
        raise SystemExit(f"{manifest_path}: names another output")
    if output.get("bytes") != path.stat().st_size or output.get("sha256") != sha256_file(path):
        raise SystemExit(f"{manifest_path}: the PGN is not the one it records")
    book = manifest.get("book", {})
    if book.get("sha256") != book_sha256:
        raise SystemExit(f"{manifest_path}: another book ({book.get('sha256')})")
    if book.get("seed") != seed:
        raise SystemExit(f"{manifest_path}: another seed ({book.get('seed')})")
    start, end = book.get("start"), book.get("end")
    rounds = read_rounds(path)
    if sorted(rounds) != list(range(1, end - start + 2)):
        raise SystemExit(f"{path}: rounds are not exactly 1..{end - start + 1}")
    by_index = {start + number - 1: key for number, key in rounds.items()}
    record = {
        "path": str(path.resolve()),
        "sha256": output["sha256"],
        "manifest_sha256": sha256_file(manifest_path),
        "start": start,
        "end": end,
    }
    return by_index, record


def build_sequence(sources: list[dict[int, str]], book: dict[str, str]) -> list[str]:
    """Merge the sources' index maps and return book lines from index 1 onward."""
    merged: dict[int, str] = {}
    for by_index in sources:
        for index, key in by_index.items():
            previous = merged.setdefault(index, key)
            if previous != key:
                raise SystemExit(f"sources disagree at opening {index}: {previous!r} / {key!r}")
    sequence = []
    index = 1
    while index in merged:
        key = merged[index]
        if key not in book:
            raise SystemExit(f"opening {index} ({key!r}) is not in the book")
        sequence.append(book[key])
        index += 1
    if not sequence:
        raise SystemExit("no source covers opening 1")
    if len(set(sequence)) != len(sequence):
        raise SystemExit("the recovered sequence repeats an opening")
    return sequence


def write_atomic(path: Path, text: str) -> None:
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(text, encoding="utf-8", newline="\n")
    os.replace(temporary, path)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n", 1)[0])
    parser.add_argument("sources", nargs="+", type=Path, help="fastchess datagen PGNs")
    parser.add_argument("--book", type=Path, required=True)
    parser.add_argument("--seed", type=int, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)

    sidecar = args.out.with_suffix(".json")
    for target in (args.out, sidecar):
        if target.exists():
            raise SystemExit(f"refusing to overwrite {target}")

    book_sha256 = sha256_file(args.book)
    book = read_book(args.book)
    loaded = [load_source(path, book_sha256, args.seed) for path in args.sources]
    sequence = build_sequence([by_index for by_index, _ in loaded], book)

    write_atomic(args.out, "".join(line + "\n" for line in sequence))
    record = {
        "schema": SCHEMA,
        "book": {
            "path": str(args.book.resolve()),
            "sha256": book_sha256,
            "openings": len(book),
        },
        "seed": args.seed,
        "openings": len(sequence),
        "sources": [source for _, source in loaded],
        "output": {
            "path": str(args.out.resolve()),
            "bytes": args.out.stat().st_size,
            "sha256": sha256_file(args.out),
        },
    }
    write_atomic(sidecar, json.dumps(record, indent=2) + "\n")
    print(f"{len(sequence)} openings in fastchess order (seed {args.seed}) -> {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
