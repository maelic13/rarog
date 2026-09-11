# Preserved arm patches

Six experiment arms that `EXPERIMENTS.md` cites by SHA and that live only on
deleted branches, captured as diffs against a baseline that is reachable from
`dev`. A ledger row must reproduce its artifact without the branch it came
from; for these arms the recipe is the diff, and this is where the diff lives.

Captured 2026-09-10. Each was generated with `git diff <base> <arm>` and each
was verified to apply cleanly to `<base>` through a temporary index, and to
depend only on blobs reachable from a ref — so they stay applicable after any
prune.

| Patch | Row | Applies to | Content |
|---|---|---|---|
| `1e2a30d` | RAR-S65 | `1155ec3` | `KillerClearGrandchild` default 0 → 1, plus the 4.10 obligations note |
| `46fa4c4` | RAR-S62 | `db19aef` | ablation arm: ProbCut writes `mv` without its piece, reintroducing the desync |
| `6407061` | RAR-S58 | `dfa965e` | 4.7c-only arm: 4.7a reverted out of the accepted bundle |
| `76e72bb` | RAR-S56 | `090dedc` | 4.7a: hard `nmp_eval >= beta` entry, old margin re-homed onto `static_eval` |
| `7ea0620` | RAR-S63 | `05ba633` | 4.5.3 variant: ProbCut does not record its speculative move |
| `b517991` | RAR-S66 | `e2fd4e0` | `ImprovingPly4Fallback` default 0 → 1 |

## What was deliberately NOT preserved, and why

**Thirteen cited commits touched no `src/` file.** They are registration,
record and tooling commits whose content is already in the tracked documents
they edited. The citation is a date-stamp on a record, not a pointer to an
artifact, so there is nothing to reconstruct.

**Nine further cited commits sit deep on a 113-commit development line that
forked from `a5fd288` (Version 2.3.1) and was never merged** — the Phase-4.2,
4.3a and 10.x era. A diff from a reachable base to any of them runs 4,500 to
14,000 lines and is a whole-branch snapshot, not a recipe. Their rows are
closed results whose value is the finding; several say in terms not to retry
the mechanism. `ba3170b` (RAR-S20) is the one with a parameter recipe, and it
already carries all seven values and both bench fingerprints inline in its row.

That line is **not** removed by `git gc`: it is unreachable from any ref but
still held by the reflog, and only expiring the reflog would drop it. Nothing
in this clean-up expired a reflog.
