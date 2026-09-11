#!/usr/bin/env python3
"""Build RAR-E08 arm B: self-play labels with <=6-man positions set to Syzygy truth.

RAR-E08 asks which label a Texel fit should learn from in positions the
tablebase can adjudicate. The two arms differ in exactly one way and share
everything else -- same games, same positions, same splits, same row order --
so the comparison is paired by construction:

  arm A  the literal self-play WDL everywhere. This is `hce-v2`, and its fit is
         the accepted head, so arm A needs no new work.
  arm B  identical, except that every position with 6 men or fewer carries the
         tablebase's verdict instead of the game's result.

Why it is an open question rather than an obvious improvement: Texel fits the
value realizable by the CONSUMING SEARCH, and under that principle a KBN-K
position the engine converts 7% of the time really is a draw, so arm A is
right and arm B teaches it to steer into endings it cannot win. Against that,
arm A's labels are self-reinforcing -- cannot convert, so labelled a draw, so
the evaluator learns draw, so it never steers there. RAR-E09 found the
mechanism concretely: KR-K, a 100% theoretical win, is labelled a draw on 75%
of its `hce-v2` positions because Rarog scores a won rook ending below the
600 cp resign threshold and then fails to mate inside fifty moves.

**Cursed wins are labelled as draws.** Syzygy WDL 2 is a clean win and 1 is a
win the fifty-move rule takes away. The label must be the result the game would
really have had, so only 2 becomes 1.0; 1, 0 and -1 all become 0.5.

**All three splits are relabelled.** Relabelling only train would leave the
fit validating and testing against a different target than it optimises, which
is incoherent. Note that the arms' losses are still NOT comparable with each
other -- their targets differ, and a loss measured against different targets is
not a comparison. Only a head-to-head game result decides RAR-E08.

Usage:

  python tools/texel/relabel_tb.py \\
      --source tools/texel/data/hce-v2 \\
      --syzygy D:/chess/tablebases/syzygy3456 \\
      --out tools/texel/data/hce-v2-tb
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import io
import json
import sys
from pathlib import Path

import chess
import chess.syzygy

SPLITS = ("train", "validation", "test")
# Syzygy WDL -> white-perspective game result.
WDL_TO_LABEL = {2: "1", 1: "0.5", 0: "0.5", -1: "0.5", -2: "0"}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest().upper()


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--source", required=True, type=Path)
    ap.add_argument("--syzygy", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--max-men", type=int, default=6)
    args = ap.parse_args()

    if not args.source.is_dir():
        ap.error(f"source is not a directory: {args.source}")
    if not args.syzygy.is_dir():
        ap.error(f"syzygy path is not a directory: {args.syzygy}")
    if args.out.exists() and any(args.out.iterdir()):
        ap.error(f"refusing to overwrite a non-empty output directory: {args.out}")
    args.out.mkdir(parents=True, exist_ok=True)

    tb = chess.syzygy.open_tablebase(str(args.syzygy))
    report: dict[str, dict] = {}
    try:
        for split in SPLITS:
            src = args.source / f"{split}.csv"
            if not src.is_file():
                ap.error(f"missing {src}")
            dst = args.out / f"{split}.csv"
            rows = eligible = changed = probe_failed = 0
            moves = collections.Counter()
            with io.open(src, encoding="utf-8", errors="replace") as fin, io.open(
                dst, "w", encoding="utf-8", newline="\n"
            ) as fout:
                for line in fin:
                    stripped = line.rstrip("\n")
                    sep = stripped.rfind(";")
                    if sep < 0:
                        fout.write(line)
                        continue
                    rows += 1
                    fen, label = stripped[:sep], stripped[sep + 1 :]
                    board_field = fen.split(" ", 1)[0]
                    men = sum(1 for c in board_field if c.isalpha())
                    if men > args.max_men:
                        fout.write(stripped + "\n")
                        continue
                    eligible += 1
                    try:
                        board = chess.Board(fen)
                        wdl = tb.probe_wdl(board)
                    except (
                        ValueError,
                        KeyError,
                        chess.syzygy.MissingTableError,
                    ):
                        probe_failed += 1
                        fout.write(stripped + "\n")
                        continue
                    if board.turn == chess.BLACK:
                        wdl = -wdl
                    truth = WDL_TO_LABEL[wdl]
                    if truth != label:
                        changed += 1
                        moves[(label, truth)] += 1
                    fout.write(f"{fen};{truth}\n")
            report[split] = {
                "rows": rows,
                "eligible_le_max_men": eligible,
                "relabelled": changed,
                "probe_failures": probe_failed,
                "relabel_pct_of_rows": round(100.0 * changed / rows, 4) if rows else 0.0,
                "transitions": {f"{a}->{b}": n for (a, b), n in sorted(moves.items())},
                "source_sha256": sha256_file(src),
                "output_sha256": sha256_file(dst),
            }
            print(
                f"{split:<11} rows {rows:>9}  <= {args.max_men} men {eligible:>8}"
                f"  relabelled {changed:>7} ({report[split]['relabel_pct_of_rows']}%)"
                f"  probe failures {probe_failed}",
                flush=True,
            )
    finally:
        tb.close()

    manifest = {
        "schema": "rarog-tb-relabel-v1",
        "arm": "RAR-E08 arm B",
        "source": str(args.source.resolve()),
        "syzygy": str(args.syzygy.resolve()),
        "max_men": args.max_men,
        "wdl_to_label": {str(k): v for k, v in WDL_TO_LABEL.items()},
        "cursed_wins_are_draws": True,
        "splits": report,
    }
    (args.out / "relabel-manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    print(f"\nmanifest: {args.out / 'relabel-manifest.json'}")

    # Emit the CORPUS manifest too, not just the relabel report. Without it the
    # output is not a fittable corpus: `fit_complete.ps1` hash-verifies
    # `manifest.json` and checks the label string, so for RAR-E08 this file was
    # built BY HAND. That is a transcription step on the critical path of a
    # multi-hour fit, and the transformation is fully determined -- copy the
    # source manifest, retarget the label, replace the three output hashes, and
    # record provenance. Doing it here removes the chance to get it wrong.
    source_manifest = args.source / "manifest.json"
    if not source_manifest.is_file():
        print(f"WARNING: no {source_manifest}; wrote CSVs and the relabel report "
              f"only. The output is NOT a fittable corpus without a corpus "
              f"manifest.", file=sys.stderr)
        return 0

    corpus = json.loads(source_manifest.read_text(encoding="utf-8"))
    base_label = str(corpus.get("label", ""))
    suffix = f"<={args.max_men}-man Syzygy corrected"
    if suffix not in base_label:
        corpus["label"] = f"{base_label}, {suffix}" if base_label else suffix
    corpus["derived_from"] = {
        "tool": "tools/texel/relabel_tb.py",
        "source_dir": str(args.source.resolve()),
        "source_output_sha256": dict(corpus.get("output_sha256", {})),
        "cursed_wins_are_draws": True,
        # States only what this tool structurally guarantees. The hand-built
        # hce-v2-tb manifest also claimed "verified row-for-row"; that was a
        # separate check, and a tool must not assert an audit it did not run.
        "note": (
            "Rows, FENs, order and split membership are byte-identical to the "
            f"source; only labels on positions of {args.max_men} men or fewer "
            "differ, by construction -- every row is rewritten as its original "
            "FEN with either its Syzygy verdict or its original label."
        ),
        "relabel": {
            split: {
                "rows": report[split]["rows"],
                "eligible_le_max_men": report[split]["eligible_le_max_men"],
                "relabelled": report[split]["relabelled"],
                "probe_failures": report[split]["probe_failures"],
            }
            for split in SPLITS
        },
    }
    corpus["output_sha256"] = {
        split: report[split]["output_sha256"] for split in SPLITS
    }
    (args.out / "manifest.json").write_text(
        json.dumps(corpus, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    print(f"corpus manifest: {args.out / 'manifest.json'}")
    print(f"label: {corpus['label']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
