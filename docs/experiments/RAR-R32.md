# RAR-R32 — Which member of the RAR-R03 parallel-search bundle holds the main thread back: four switches on RAR-R29's time-to-depth harness — REGISTERED 2026-11-02, before the read; READ 2026-11-02

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-02 by the agent; the maintainer runs
  it (about 45 minutes).
- **Leaf:** PLAN D.2, the four-thread card's read 6 (H3), by
  [RAR-R29](RAR-R29.md)'s rule for X under 1.2.
- **Question.** Four threads reach a fixed depth more slowly than one
  (RAR-R29: X 0.90 for the head with `e057d84`) at about four times one
  thread's nodes, so the helpers' work buys the main thread no depth. RAR-R03
  accepted the parallel-search bundle whole (+102.78 ± 16.38 at 4T) and never
  decomposed it. Does any of its members acting on the search tree hold the
  main thread's progress to depth back?
- **Kind:** zero-game screen. A member it names is a candidate for a game
  read; nothing about strength follows, and a member it clears keeps its
  place.

## The members and the switches

`RAROG_ABLATE` names the members to switch off (instrument patch
`analysis/arm_patches/e057d84-rar-r32-smp-ablation.patch`, `cb908535…`,
never merged; built by `analysis/artifacts/d2-ablate/build.sh` with
`make_patch.py`, `2348045b…`):

| Switch | Member | Where |
|---|---|---|
| `order` | every thread, the main one included, ranks root moves another thread has scored above its own order, by the pool's depth and score | `apply_shared_root_scores` |
| `center` | a thread centres its aspiration window on the pool's exact root score when the pool is deeper than the thread | the iterative-deepening loop |
| `rotate` | each helper starts its root list at its own offset | `search_parallel`'s jobs |
| `jitter` | the reduction jitter is offset by the thread's index | `negamax`'s reduction |

The fifth member, the soft-stop vote, acts only under a clock and is not
read by `go depth`. Every switch acts only inside a pool, so a one-thread
search is unchanged.

- **Build:** `tools/test_engines/rarog-d2abl-pext-pgo.exe` (`d2afbd8b…`),
  `e057d84` with the patch, `cargo xtask build --arch pext --pgo`, rustc
  1.99.1, 2026-11-02 (`build.log`). `cargo fmt --check` and `cargo clippy
  --release --all-targets` clean on the patched tree.
- **Wire** (`build.log`): `bench 13` reads **13,987,223** with no switch and
  with every switch. Each member writes one stderr line when first consulted;
  a four-thread `go depth 18` reports all four consulted, off exactly as set,
  with no switch and with every one.

## Design

- `analysis/artifacts/d2-ablate/ablate_read.py` (`9c086af1…`, ignored):
  RAR-R29's `ttd_read.py` unchanged but for the arms and the judgement. One
  process per (arm, position), Threads 1 and 4 alternating inside it, the
  order swapped every pair, `ucinewgame` before every search, CPUs 2, 4, 6, 8,
  Hash 256, RAR-R29's eight positions at its frozen depths, **twelve pairs**
  each, the arms' order rotating by position.
- **Arms:** `none`, `order`, `center`, `rotate`, `jitter` and `all` (the
  four together), each the `RAROG_ABLATE` value its processes start with.
- Every process's stderr is read at its close: each member consulted must be
  off exactly when its arm names it, and the root order, the rotation and the
  jitter must be consulted, or the read stops. The window centre is consulted
  only when the pool is deeper than the thread, and its absence is recorded.

## Readings and judgement

- Per pair r = ln(T1 / T4) at the position's depth; per arm and position the
  median; per arm **X = exp(mean over positions)**. A 90% bootstrap over
  pairs within positions for each arm against `none`.
- Per member, and for `all`, d = ln X_arm − ln X_none:
  - **HOLDS DEPTH BACK:** d at ln 1.10 or more with the interval above zero;
  - **HELPS DEPTH:** d at −ln 1.10 or less with the interval below zero;
  - **NO RESOLVED EFFECT** otherwise.
- **Also read:** X_none against RAR-R29's X_dyn (0.897), the same source with
  inert switches.
- Invalid when any pair lacks a depth-D line; an interrupted read is rerun
  whole.

## Exposure before registration

- The build, the benches and the wire above, which print no time-to-depth.
- One smoke pair per arm and position at D − 6 (`smoke.log`): 48 of 48
  pairs with a depth line; the switch reports held in all 48 processes, the
  window centre consulted in all 48. It prints no reading; its JSON was not
  opened.
- The judgement's self-test on synthetic pairs (`judge_selftest.py`).

## Prediction, frozen before the read

| Arm | HOLDS DEPTH BACK | NO RESOLVED EFFECT | HELPS DEPTH |
|---|---:|---:|---:|
| `rotate` | 0.45 | 0.45 | 0.10 |
| `order` | 0.25 | 0.65 | 0.10 |
| `center` | 0.10 | 0.85 | 0.05 |
| `jitter` | 0.10 | 0.80 | 0.10 |
| `all` | 0.55 | 0.35 | 0.10 |

- **Levels:** X_none 0.90 (0.80 to 1.00); X_all 1.1 (0.85 to 1.6).
- **Reasoning.** Lazy SMP speeds the main thread to depth when helpers search
  near its tree a little ahead of it and leave entries it can use. The
  rotation sends each helper first into root moves the main thread searches
  late or with a null window, so the entries the helpers write serve their
  own trees; switching it off should help the main thread most. The pool's
  root order acts on the main thread directly: a move a helper searched
  deeper rises in the main thread's order whatever its score, which can cost
  the main thread its own best ordering. The window centre and the jitter
  offset change little the main thread sees.

## Decision and stop rule

- **A member HOLDS DEPTH BACK:** a candidate for removal or rework. Only
  games price it: a removal takes the removal bracket `[-1.75, 0.25]` at four
  threads, registered under this leaf after [RAR-R31](RAR-R31.md) has said
  whether the engine's own depth deficit is real. Time-to-depth is not Elo:
  the bundle earned +103 at four threads with these members in it.
- **Only `all` resolves:** the members act together; a cluster removal is
  registered as one unit.
- **Nothing resolves:** the RAR-R03 members do not hold the main thread
  back; H3's next question is what the helpers store and how the main
  thread reads it (the shared table's replacement and depth policy), a new
  card entry.
- **HELPS DEPTH:** the member stays and is recorded as earning its place in
  depth, not only in games.
- One read; nothing in the arms, positions, depths, pairs or thresholds
  changes after it starts.

## How to run

From PowerShell 7 on an idle host, nothing else running, about 45 minutes:

```powershell
python D:\code\rarog\analysis\artifacts\d2-ablate\ablate_read.py D:\code\rarog\analysis\artifacts\d2-ablate\ablate_read.json 2>&1 | Tee-Object -FilePath D:\code\rarog\analysis\artifacts\d2-ablate\ablate_read.log
```

If it refuses with `HOST NOT IDLE`, run it again. The summary ends with an
`OUTCOME:` line.

## Result — 2026-11-02, appended after exposure

Run by the maintainer on 2026-11-02 from about 11:05 to 11:50, host busy
0.5% before and after; 576 of 576 pairs with a depth-D line; the switch
reports held in all 48 processes, the window centre consulted in all 48
(`ablate_read.log`; `ablate_read.json` `518fab64…`).

| Arm | X | Per position | ln X − ln X_none (90%) | Verdict |
|---|---:|---|---|---|
| `none` | 0.814 | 0.57 1.13 0.91 0.52 0.99 0.78 0.84 1.00 | — | — |
| `order` | 0.897 | 0.61 1.11 1.10 0.42 1.21 0.86 0.99 1.31 | +0.098 (−0.031 to +0.168) | NO RESOLVED EFFECT |
| `center` | 0.928 | 0.75 1.10 1.03 0.51 1.17 0.81 0.89 1.51 | +0.131 (−0.013 to +0.195) | NO RESOLVED EFFECT |
| `rotate` | 0.878 | 0.87 0.85 0.94 0.42 1.12 0.88 0.91 1.36 | +0.077 (−0.043 to +0.148) | NO RESOLVED EFFECT |
| `jitter` | 0.868 | 0.63 1.22 0.89 0.42 1.19 0.91 1.03 1.01 | +0.065 (−0.022 to +0.167) | NO RESOLVED EFFECT |
| `all` | 0.847 | 0.67 0.82 0.94 0.48 1.08 0.93 0.86 1.22 | +0.040 (−0.069 to +0.122) | NO RESOLVED EFFECT |

- **Outcome:** every member and the four together, **NO RESOLVED EFFECT**.
- X_none 0.814 against RAR-R29's X_dyn 0.897 for the same source: the read's
  own run-to-run spread is of the size of the effects it looks for.
- Every single member reads above `none` and the four together read least,
  which is not how independent effects add: the pattern is noise around a
  low `none`, not four small harms.

## Disposition

- **Nothing resolves:** the RAR-R03 members do not hold the main thread
  back at the read's resolution (about ±10% in X). They keep their place.
- The rule's next question for H3, what the helpers store and how the main
  thread reads it, is **not opened now**: [RAR-R31](RAR-R31.md) shows the
  games' depth deficit is not the engine's own, so H3 no longer explains the
  four-thread deficit, and it waits behind H7.

## Calibration

- Predicted HOLDS DEPTH BACK: `rotate` 0.45, `order` 0.25, `center` 0.10,
  `jitter` 0.10, `all` 0.55; none came. The reasoning that helpers starting
  elsewhere in the root list deny the main thread usable entries is not
  supported at this resolution: a miss in mechanism, or an effect under 10%.
- X_none, predicted 0.90 (0.80 to 1.00): 0.814, inside at the low end;
  X_all, predicted 1.1 (0.85 to 1.6): 0.847, just below.
- Instrument: twelve pairs resolve about ±0.1 in ln X here; a smaller effect
  needs a larger read, and nothing now asks for one.
