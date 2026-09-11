# 4.11.8 datagen label audit — RAR-M22

Completed 2026-09-06. This is a corpus-label audit, not a conversion test or
a strength result. It checks the first 3–6-man Syzygy clean win reached by a
recorded game against that game's final result. Cursed wins are excluded.

## Frozen inputs and method

`hce-v2` consists of two immutable 8,000-node game segments, 20,000 and
580,000 games. Their SHA-256 values match `hce-v2/manifest.json`:
`054CF7C66FC2B1F888B4E7881DDDD694AB21425515D07CE2F3A6ECA41CBFDFAB` and
`4600D8DFB632FD1414EA5622E155079A9383DEE334987225A547BE4BA2EB9F20`.
`hce-v3-tb` derives from the 602,619-game, 8,000-node `hce-v3` source PGN;
its SHA-256 matches the derived corpus manifest:
`C278BEE27DD5AC2925F75A9026E60311FF1908B9EF235889D0E993BFAA776181`.

The existing `tools/diag/datagen_label_audit.py` was used unchanged with local
3–6-man Syzygy, `--max-men 6` and 30 workers. Its SHA-256 is
`B4BAE10DD08B853A77AE67B578232BAEBE6D6B137FCE40E3F3B4D2D37C24AC81`.
The 19 focused tool tests passed. Raw commands, logs, reports, manifests and
the exact tool/test sources are byte-preserved in
`analysis/artifacts/datagen-label-audit-20260906.zip` (SHA-256
`610BC366C6088168D8BDA7CACA48590EC705D369159794CFDD4D3F3033525EBF`).

```powershell
python tools/diag/datagen_label_audit.py --pgn <published-source.pgn> --syzygy D:/chess/tablebases/syzygy3456 --max-men 6 --workers 30 --output <report.json>
```

## Results

| Source game corpus | Games | First clean win reached | Clean wins not won | Of clean wins | Of all games |
|---|---:|---:|---:|---:|---:|
| `hce-v2` (both segments combined) | 600,000 | 134,948 | 26,316 | 19.50% | 4.39% |
| `hce-v3` source of `hce-v3-tb` | 602,619 | 266,490 | 54,186 | 20.33% | 8.99% |

The `hce-v3` source reaches a clean win in 44.22% of games, nearly twice the
22.49% for `hce-v2`; that explains much of the higher all-game rate. The
conversion failure conditional on reaching a clean win is also slightly higher,
not lower. The two sources use the **same 8,000-node budget**, so this is not a
node-budget comparison.

High-volume affected families confirm the expected rook/pawn concentration.
For `hce-v2`, the largest are KRPP-KR 2,535/16,712 (15.17%), KRP-KRP
3,457/11,445 (30.21%), KPP-KPP 811/9,087 (8.92%), and KRP-KR
1,524/5,210 (29.25%). For the `hce-v3` source, they include KRPP-KR
5,443/17,652 (30.84%), KPP-KPP 2,789/9,181 (30.38%), KRP-KRP
4,205/8,026 (52.39%), KRBP-KR 1,485/5,560 (26.71%), and KRNP-KR
1,457/4,660 (31.27%). These are first-clean-win game counts, not independent
samples of every position in a long ending.

## What this says about `hce-v3-tb`

The second row is deliberately named **source of `hce-v3-tb`**. The audit
reads final PGN game results; it cannot inspect the post-hoc correction applied
to each extracted CSV row. `hce-v3-tb` has already corrected 125,643 <=6-man
rows (113,046 train, 6,267 validation, 6,330 test) by Syzygy, with zero probe
failures. Those corrections are real, but they do not make the 8.99% figure a
remaining row-error rate, and they do not correct rows above six men.

Therefore do not claim either that `hce-v3-tb` has 8.99% mislabeled rows or
that its post-hoc relabeling removes every label consequence of a missed
conversion. The unresolved quantity is row-level impact after extraction,
sampling and the <=6-man correction. It belongs to 4.13.1, together with the
game-to-row lineage audit. Whole-game tablebase adjudication remains a distinct,
unmeasured datagen-v3 arm.

## Disposition

4.11.8 is complete. It establishes a material one-directional raw game-label
bias toward draws in both 8,000-node source corpora; no HCE change, refit,
SPSA or strength claim follows from it. 4.13.1 owns the row-level measurement
and decides whether to register separate post-hoc relabel and whole-game
adjudication arms. Next leaf: 4.11.9, mate-drive promotion-closure accounting.
