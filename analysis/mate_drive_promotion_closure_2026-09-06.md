# 4.11.9 mate-drive promotion closure -- RAR-M23

Completed 2026-09-06. This resolves the scope of the accepted 4.9a.4
minor-piece mate drive. It is an analysis of existing paired reports, not a
new engine test, current conversion baseline, strength result or rollback.

## Inputs and method

The pre-drive report is `tools/results/v2-accepted-pgo/endgame-truth.json`
(SHA-256 `43745A25CF14AEED796F65EE9D1CB682C69739DB3E210466E70EDE33C0189851`).
The accepted-drive report is `tools/results/mopup-diag/endgame-truth.json`
(SHA-256 `35A5447F2721E80D64AC2F54CB0EB84D89E2BD5AF68421CBCD91CD909893A8CC`).
They intentionally use different engine binaries. The derivation verifies that
their schema, Syzygy path, positions per family, node budget (60,000), ply
limit, seed (6200600), hash size, persistent-TT setting, family set and every
position's index/FEN/Syzygy WDL/DTZ agree before it compares results.

Conversion here means a theoretically won root whose recorded outcome is
`mated`. This is the historical report pair that contains the material-shed
abort later repaired by 4.10. Therefore the comparison identifies the
mate-drive's historical paired direction and closure; it does **not** set a
current conversion rate, floor, or implementation target.

## Measured closure

| Family | Before | After | Net | Paired gains | Paired losses | Interpretation |
|---|---:|---:|---:|---:|---:|---|
| KBB-K | 78/100 | 100/100 | +22 | 22 | 0 | Direct minor-mate activation |
| KBN-K | 19/98 | 95/98 | +76 | 76 | 0 | Direct minor-mate activation |
| KPP-K | 75/98 | 75/98 | 0 | 0 | 0 | Promotion-reached; route/DTZ fields change only |
| KBP-K | 90/94 | 92/94 | +2 | 3 | 1 | Promotion-reached guard, not debt |
| KBP-KB | 18/26 | 17/26 | -1 | 2 | 3 | Promotion-closure debt: 4.12.7 |
| KBP-KN | 45/57 | 44/57 | -1 | 1 | 2 | Promotion-closure debt: 4.12.9 |

The drive's runtime branch requires a bare losing king and no winning pawn,
rook or queen. A pawn family can reach that material shape after a
truth-preserving underpromotion to a knight followed by the necessary material
exchange. The archived earlier audit traced that route for these pawn families.
Root material alone therefore understates the branch's scope.

## Decision and reusable rule

KBP-KB and KBP-KN each count as debt: a shipped mechanism loses one net
conversion on the paired cohort. The larger gains in KBB-K/KBN-K, and the
offsetting gains inside either family, do not erase a family-specific loss.
KPP-K and KBP-K remain future regression guards because their net result is not
negative. This result does not justify reverting the mate drive, whose direct
benefit remains large; it assigns the two losses to the family leaves that can
measure and repair them on the corrected instrument.

For every future recognizer or guidance term, derive its promotion closure from
the complete runtime condition. State direct root-material matches and every
family that can reach the condition by legal truth-preserving promotion,
underpromotion and material-shed paths, or measure every family. Report paired
gains and losses as well as net conversion. A net loss is debt; a nonnegative
net family remains a regression guard. Historical contaminated reports may
establish scope and ownership, but only a corrected run may supply current
rates or floors.

Reports, derivation scripts and machine-readable matrix are byte-preserved in
`analysis/artifacts/mate-drive-promotion-closure-20260906.zip` (SHA-256
`BA1EA71536D6C47B13B7A901B294AEA475CA5A24787EC5F5925639D80FD9EB03`).
