# Preserved arm patches

Six experiment arms that `docs/EXPERIMENTS.md` cites by SHA and that live only on
deleted branches, captured as diffs against a base commit (where each base
lives now: *Updated 2026-10-05*, below). A ledger row must reproduce its
artifact without the branch it came from; for these arms the recipe is the diff, and this is where the diff lives.

Captured 2026-09-10. Each was generated with `git diff <base> <arm>` and each
was verified to apply cleanly to `<base>` through a temporary index, and to
depend only on blobs reachable from a ref — so they stayed applicable after
any prune while those bases were on a ref.

| Patch | Row | Applies to | Content |
|---|---|---|---|
| `1e2a30d` | RAR-S65 | `1155ec3` | `KillerClearGrandchild` default 0 → 1, plus the 4.10 obligations note |
| `46fa4c4` | RAR-S62 | `db19aef` | ablation arm: ProbCut writes `mv` without its piece, reintroducing the desync |
| `6407061` | RAR-S58 | `dfa965e` | 4.7c-only arm: 4.7a reverted out of the accepted bundle |
| `76e72bb` | RAR-S56 | `090dedc` | 4.7a: hard `nmp_eval >= beta` entry, old margin re-homed onto `static_eval` |
| `7ea0620` | RAR-S63 | `05ba633` | 4.5.3 variant: ProbCut does not record its speculative move |
| `b517991` | RAR-S66 | `e2fd4e0` | `ImprovingPly4Fallback` default 0 → 1 |
| `23b8a7a` | RAR-S76 | `df6308e` | B.2.3 theta at iteration 3,900 baked into the 82 `CoreParams` defaults (diagnostic peek, never merged) |
| `883666d` | RAR-S82 | `7ba3a1b` | RAR-S82's block-1 theta baked into `ProofParams` (diagnostic probe) |
| `11e7145` | RAR-S84 | `f5d16d8` | RAR-S83's block-2 theta baked into all 36 coordinates (the B.3.4 gate candidate; `f53ca7d` baked the same values on `dev`) |

**Added 2026-10-04 (B.10's ref review):** the last three rows came from the
throwaway branches `diag/b23-theta3900`, `b33-block1-probe` and `b33-gate`,
each one diagnostic commit on a parent `dev` contains. Each was captured with
`git diff <base> <arm>` and proved, through a temporary index, to apply to its
base and reproduce the commit's tree exactly
(`analysis/artifacts/b10-release/save_branch_patches.sh`), so the branches
are no longer the only carriers. Their bases were then reachable from `dev`.

**Updated 2026-10-05 (PLAN B.10, *Ref review after the release*):** the
`arm/*` tags are retired. The first six rows' bases (`1155ec3`, `db19aef`,
`dfa965e`, `090dedc`, `05ba633`, `e2fd4e0`) are on no ref and restore from
the 2026-09-27 bundle (`../ledger_commits_2026-09-27.md`), where they sit
under `refs/tags/arm/*`:
`git fetch analysis/artifacts/git-history-2026-09-27/rarog-full-history.bundle "refs/tags/arm/*:refs/archive/arm/*"`
(proved 2026-10-05 into an empty repository: all six patches apply). A
prune may drop them from the working repository, so fetch them before
applying one of those patches. Those arms change the
2.4.0 search, which B.8 deleted. The last three rows' bases left `dev`
when it was reset to `master` after the 2.5.0 squash merge;
`archive/pr2-version-2.5.0` holds them, and `master` will once the next
merge commit lands.

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

**Superseded 2026-09-27:** the reflog is no longer the safeguard. That line
and every other unreachable commit are preserved in the bundle recorded in
`../ledger_commits_2026-09-27.md`, which also lists every cited commit outside
the refs, so the reflog may be expired without losing anything the bundle
holds. The six patches here still apply: their bases are in the bundle
(the `arm/*` tags that also held them were retired on 2026-10-05).
