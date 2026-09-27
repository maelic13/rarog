# Ledger-cited commits outside every branch, preserved 2026-09-27

A record, not a roadmap. `EXPERIMENTS.md`, `PLAN.md`, `GUIDE.md` and
`HISTORY.md` cite 71 commits by hash that no branch or tag reaches: the
fine-grained development line before the 2.4.0 squash (dev descends from the
release squashes since 2026-09-13), commits of experiment branches deleted
earlier, and a few amended or rebased originals. They survived only in the
reflog or as dangling objects, which garbage collection removes. A ledger row
must reproduce its artifact without the branch it came from (AGENTS,
*Evidence*); this file is where those hashes now resolve.

## The bundle

Every commit in the repository on 2026-09-27, including all 499 unreachable
ones (312 held by the reflog, 187 dangling), is in one git bundle, kept with
the other raw evidence on the development machine (ignored, never in Git):

- Path: `analysis/artifacts/git-history-2026-09-27/rarog-full-history.bundle`
- Size: 19,753,138 bytes
- SHA-256: `21688CB958D3E9E4452B933437C4F3D2BBD1EF95E3D217FF3E8920F24531365E`
- Built: every unreachable commit given a temporary ref
  `refs/archive-tmp/<sha>` (reflog commits, then `git fsck --unreachable
  --no-reflogs`), `git bundle create <path> --all`, temporary refs deleted.
- Checked: `git bundle verify` passes; a `--mirror` clone of the bundle holds
  all 71 cited commits and all 499 archived ones, and its `git fsck
  --connectivity-only` passes.

To restore any of them into a working repository:

```bash
git fetch analysis/artifacts/git-history-2026-09-27/rarog-full-history.bundle "refs/archive-tmp/*:refs/archive/*"
```

Each commit then resolves by its hash; `git for-each-ref refs/archive` lists
them, and `git update-ref -d` removes the refs again.

## The cited commits

*Recipe in Git* says where a row's reproduction lives without the bundle:
an arm patch (`analysis/arm_patches/`, applicable to a base reachable from
the `arm/*` tags), a twin commit that is still on a ref, or, for documentation
commits, the tracked documents themselves, whose citation is a date stamp
(the policy `analysis/arm_patches/README.md` set on 2026-09-10). *Bundle*
means the diff itself exists only in the bundle; the citing row still holds
the finding, and where it names a fingerprint, that fingerprint checks a
rebuild from the restored commit.

| Commit | Author date | Subject | Cited by | Kind | Recipe in Git |
|---|---|---|---|---|---|
| `36bced4` | 2026-07-29 23:26 | 10.0(a) RESULT: ordering is not the defect; over-reduction is 1.80% | RAR-S52 | docs | content in the tracked documents; the citation dates it |
| `1696028` | 2026-07-29 23:40 | 10.0(b)/(c): binaries built, designs registered, both matches handed over | RAR-S53 | docs | content in the tracked documents; the citation dates it |
| `d472f6c` | 2026-07-30 11:22 | 10.0(c) POSITIVE +4.06: Rarog over-prunes. 10.4.6 promoted to cycle headline | RAR-S54 | docs | content in the tracked documents; the citation dates it |
| `eaf0965` | 2026-07-30 12:37 | 10.0 COMPLETE: the gap survives at equal nodes, and Rarog is 2.5 plies DEEPER | RAR-S53 | docs | content in the tracked documents; the citation dates it |
| `ba3170b` | 2026-08-05 09:20 | strength_test: bake the mid-tune 10.2(a) aspiration snapshot (UNGATED) | RAR-S20 | code | bundle |
| `f35bc09` | 2026-08-05 16:38 | Finalize Phase 4.0 experiment cleanup | RAR-S22 | docs | content in the tracked documents; the citation dates it |
| `1cf9c51` | 2026-08-05 23:05 | Audit the Phase 4.2 TT evidence graph | RAR-S28 | docs | content in the tracked documents; the citation dates it |
| `47f3ac6` | 2026-08-05 23:21 | Implement Phase 4.2 typed result evidence | RAR-S23, RAR-S28 | code | bundle |
| `7815054` | 2026-08-05 23:35 | Shadow-test contradicting inexact TT bounds | RAR-S24 | code | bundle |
| `d354d02` | 2026-08-06 08:00 | Add Phase 4.3a evidence-hygiene arms, inert | RAR-S25 | code | bundle |
| `8acfd22` | 2026-08-06 08:00 | Report the 4.3 provenance hazards in the diag tool | RAR-S26 | docs | content in the tracked documents; the citation dates it |
| `040b49e` | 2026-08-06 08:05 | Register the Phase 4.3a arms and their sizing | RAR-S27 | docs | content in the tracked documents; the citation dates it |
| `adf3f22` | 2026-08-06 13:56 | Record the arm-A verdict and close 4.2 throughput | RAR-S29 | docs | content in the tracked documents; the citation dates it |
| `8822cf2` | 2026-08-06 17:18 | Shadow whether TT eval refinement self-cancels | RAR-S30 | code | bundle |
| `3eeea89` | 2026-08-06 17:20 | Record the arm-A refutation and the refinement shadow | RAR-S31 | docs | content in the tracked documents; the citation dates it |
| `d00e1ac` | 2026-08-07 10:30 | Park arm B and preserve the accepted baseline | RAR-S33, RAR-S34 | code | bundle |
| `1dc4bc6` | 2026-08-07 10:42 | Persist speculative TT evidence for ProbCut | RAR-S33, RAR-S34 | code | bundle |
| `2b6d2c0` | 2026-08-10 16:10 | Let verify-isa find either PGO flavour of a tier's artifact | RAR-P19, RAR-P14 | code | bundle |
| `f7f424a` | 2026-08-10 18:22 | Record the Windows ARM64 PGO retest as passing | RAR-P15 | docs | content in the tracked documents; the citation dates it |
| `d12d15d` | 2026-08-11 13:48 | Remove NmpDecisiveGuard: an efficiency guard worth 0.004% of nodes | RAR-S51 | code | bundle |
| `8557b18` | 2026-08-11 13:52 | Clamp unproven mates out of NMP cutoffs | RAR-S51 | code | bundle |
| `1358b19` | 2026-08-11 17:41 | Merge nmp-mate-clamp: clamp unproven mates out of NMP cutoffs | RAR-S51 | docs | content in the tracked documents; the citation dates it |
| `76e72bb` | 2026-08-13 14:48 | 4.7a candidate: null-move entry contract (NOT gated, NOT accepted) | RAR-S56 | code | `arm_patches/76e72bb-4-7a-candidate-null-move-entry-contr.patch` |
| `6407061` | 2026-08-18 16:19 | 4.7c-only ablation arm: revert 4.7a from the accepted bundle | RAR-S58 | code | `arm_patches/6407061-4-7c-only-ablation-arm-revert-4-7a-f.patch` |
| `46fa4c4` | 2026-08-19 06:35 | Ablation arm ONLY: reintroduce the ProbCut piece desync. NEVER MERGE. | RAR-S62 | code | `arm_patches/46fa4c4-ablation-arm-only-reintroduce-the-pr.patch` |
| `7ea0620` | 2026-08-19 07:37 | 4.5.3 variant: ProbCut does not record its speculative move | RAR-S63 | code | `arm_patches/7ea0620-4-5-3-variant-probcut-does-not-recor.patch` |
| `1e2a30d` | 2026-08-20 09:08 | Candidate arm: KillerClearGrandchild default 1 (audit finding 3) | RAR-S65 | code | `arm_patches/1e2a30d-candidate-arm-killercleargrandchild.patch` |
| `b517991` | 2026-08-20 11:38 | Candidate arm: ImprovingPly4Fallback default 1 (audit finding 2) | RAR-S66 | code | `arm_patches/b517991-candidate-arm-improvingply4fallback.patch` |
| `e438ced` | 2026-08-21 21:36 | Register the 4.6.4 run before its games | RAR-S72 | docs | content in the tracked documents; the citation dates it |
| `43d5174` | 2026-08-21 22:42 | Register RAR-S71: the rebuilt search node, before any games | RAR-S71 | docs | content in the tracked documents; the citation dates it |
| `c5e451d` | 2026-08-24 23:40 | Revert failed search core reimplementation | RAR-S71 | code | bundle |
| `b9cc252` | 2026-09-01 15:25 | Seed endgame_truth families by name, not list index | RAR-E14 | docs | content in the tracked documents; the citation dates it |
| `d1d95ab` | 2026-09-03 08:14 | Adopt the hce-v3 refit as the accepted head (RAR-E12) | RAR-E12 | code | bundle |
| `9281435` | 2026-09-04 14:01 | Audit the endgame-truth instrument; three confirmed defects | RAR-E14 | docs | content in the tracked documents; the citation dates it |
| `ca03a46` | 2026-09-05 12:30 | 4.11.5: three of forty roots produce 56% of the occurrence census | RAR-M20, RAR-M43, RAR-M44 | docs | content in the tracked documents; the citation dates it |
| `fd21612` | 2026-09-06 11:47 | Complete 4.11b.2 board-v2 regression corpus | RAR-E15, RAR-M42 | code | bundle |
| `fce0b44` | 2026-09-06 22:09 | Fix SEE exchange legality and recapture promotions | HISTORY | code | bundle |
| `952711f` | 2026-09-07 13:34 | Fix ETW profile symbol resolution | HISTORY | docs | content in the tracked documents; the citation dates it |
| `2ea279f` | 2026-09-07 17:02 | Reduce move-generation pin discovery work | HISTORY | code | bundle |
| `c44608a` | 2026-09-07 21:54 | Withdraw unqualified pin optimization and retain oracle | RAR-M31 | code | bundle |
| `af83abf` | 2026-09-07 21:56 | Close pin candidate disposition and advance board roadmap | HISTORY | docs | content in the tracked documents; the citation dates it |
| `86e39f8` | 2026-09-07 22:07 | Register fused relocation qualification | HISTORY | docs | content in the tracked documents; the citation dates it |
| `8a73cfd` | 2026-09-07 22:25 | Cover ordinary relocation for every piece class | HISTORY | code | bundle |
| `5c439da` | 2026-09-07 23:19 | Fuse ordinary quiet relocation in make and unmake | RAR-M33 | code | bundle |
| `f70ac19` | 2026-09-08 16:17 | Reserve search history headroom and pin the board contracts | RAR-M38 | code | bundle |
| `b33d3ad` | 2026-09-08 20:45 | Qualify the integrated board cluster at +1.421% | RAR-E15, RAR-M42 | docs | content in the tracked documents; the citation dates it |
| `c1a7713` | 2026-09-09 08:25 | Refresh endgame evidence and close section 4.11b | RAR-M43, RAR-M44 | docs | content in the tracked documents; the citation dates it |
| `021dc98` | 2026-09-09 11:25 | Deliver generated moves into caller-owned lists (4.11b.19(b)) | RAR-M44 | code | bundle |
| `55e228a` | 2026-09-09 11:28 | Give the board bench caller-owned move lists (4.11b.19(a)) | RAR-M44 | code | bundle |
| `be5c02a` | 2026-09-09 12:06 | Force-inline the three out-of-line generator helpers (4.11b.19(c) candidate 1) | RAR-M44 | code | bundle |
| `c969ccd` | 2026-09-09 12:13 | Const-generic colour through the generator (4.11b.19(c) candidate 2) | RAR-M44 | code | bundle |
| `39542b7` | 2026-09-09 13:05 | Revert the 4.11b.19(c) generator candidates (bundle NO_CHANGE) | RAR-M44 | code | bundle |
| `c80df74` | 2026-09-09 13:13 | Close 4.11b.19: (c) rejected in search, (d) re-measured, section 4.11b done | RAR-M45, RAR-O03, HISTORY | docs | content in the tracked documents; the citation dates it |
| `7d8b013` | 2026-09-09 16:01 | Phase A reordered: release before baselines; universal x86-64 binary as A.4 | RAR-P18 | docs | content in the tracked documents; the citation dates it |
| `2faa542` | 2026-09-09 16:07 | A.2.3: feature, option and parameter inventory | RAR-P18 | docs | content in the tracked documents; the citation dates it |
| `ca8988a` | 2026-09-09 16:24 | A.3.1: pin the toolchain to 1.98.1 | RAR-R11, RAR-P18 | docs | content in the tracked documents; the citation dates it |
| `a4c4f95` | 2026-09-09 16:29 | RAR-P18: correct the arm provenance to 2faa542 | RAR-E16, RAR-R11 | docs | content in the tracked documents; the citation dates it |
| `d93f808` | 2026-09-09 18:00 | A.3.3: throttle info lines so a lagging harness cannot block the search | RAR-R11 | code | bundle |
| `e3430d9` | 2026-09-09 18:14 | Revert "A.3.3: throttle info lines so a lagging harness cannot block the search" | RAR-R11 | code | bundle |
| `79d3974` | 2026-09-09 18:21 | A.3.3: start the search clock when `go` is parsed, as the harness does | RAR-R11, RAR-P19, A.3.3 | code | bundle |
| `5073e37` | 2026-09-09 18:22 | A.3.3 record corrected: clock-origin repair, throttle reverted, RAR-R11 re-pointed | RAR-R12 | docs | content in the tracked documents; the citation dates it |
| `ef9c6ae` | 2026-09-10 14:09 | Preserve six deleted-branch arms as applicable patches | RAR-P20, RAR-P19 | docs | content in the tracked documents; the citation dates it |
| `fc6e39d` | 2026-09-10 15:32 | Register RAR-P20 and add the PGO build-pool helper | RAR-P22, RAR-P20 | docs | content in the tracked documents; the citation dates it |
| `1bf8171` | 2026-09-10 16:43 | A.4.1: cc 1.3.0 -> 1.4.5, and repair the MSRV lockstep | RAR-P22, RAR-P21 | code | bundle |
| `e66bb51` | 2026-09-10 17:26 | A.4.2: tell the user when they are running the wrong asset | RAR-P22 | code | bundle |
| `208e06c` | 2026-09-10 18:47 | A.5: conversion instrument, reading PGN rather than Colosseum | RAR-M47 | docs | content in the tracked documents; the citation dates it |
| `c6a548f` | 2026-09-10 19:07 | A.7: version 2.4.0 | A.7, HISTORY | code | bundle |
| `a927b4d` | 2026-09-10 19:08 | A.7 closed: CHANGELOG opened for 2.4.0 | RAR-P23 | docs | content in the tracked documents; the citation dates it |
| `51d5636` | 2026-09-10 19:28 | RAR-M45 scoped to twelve engines, before any game | RAR-P23 | docs | content in the tracked documents; the citation dates it |
| `65b946e` | 2026-09-11 13:45 | A.8.3 preparation audit, and correct a PLAN path that points at nothing | RAR-O03 | docs | content in the tracked documents; the citation dates it |
| `5513573` | 2026-09-11 15:01 | A.8.3 closed: G(0) = -247.97 +/- 10.89, and the depth gap is 0.97 ply | RAR-M48 | docs | content in the tracked documents; the citation dates it |

`883666d` (RAR-S82's block-1 probe) is not in this table: its branch
`b33-block1-probe` was restored on 2026-09-27 after an accidental deletion.
