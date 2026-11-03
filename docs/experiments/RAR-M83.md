# RAR-M83 — Self-relative scaling in the sibling-free slot: Rarog, Reckless and Stockfish, each one thread against four, RAR-M78's time-odds legs as G2 — REGISTERED 2026-11-03, before any game

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-03 by the agent; the maintainer runs
  it (one idle night, about nine hours).
- **Leaf:** PLAN D.2, the four-thread card's decision read, re-registered
  by [RAR-M82](RAR-M82.md)'s RESTORED.
- **Question.** [RAR-M78](RAR-M78.md) and [RAR-M81](RAR-M81.md) read Rarog's
  four threads as worth less of its own time than Reckless's and
  Stockfish's, but every four-thread leg of both ran in Colosseum's auto
  slot, where Windows packed each four-thread side on two or three cores'
  SMT siblings. With one logical CPU per physical core, are four threads
  worth fewer of Rarog's own doublings of time than of Reckless's and
  Stockfish's?
- **Kind:** fixed-length matches with intervals; they decide no gate and no
  default. Their outcome decides whether D.2 has a four-thread SMP target.

## Design

- **Legs**, each a 1,000-game `match` of the engine against itself (one
  binary on both sides), one thread (arm A) against four (arm B), `10+0.1`
  with the 20 ms margin, Hash 256 on both sides, `match-fixed-ltc.toml`
  (the UHO book, random order, no adjudication), three games at a time,
  `--cores-per-game 4 --placement 2,4,6,8,16,18,20,22,24,26,28,30` (the
  policy for rated multi-thread reads, PROCESS, *Harness*), Colosseum CLI
  0.5.0 (pinned), in this order:

  | Leg | Engine | Binary (SHA-256) | Options | Seed |
  |---|---|---|---|---|
  | `rarog-threads` | Rarog, `e057d84` | `tools/test_engines/rarog-d2dyn-pext-pgo.exe` (`a545b6b8…`) | defaults (`Move Overhead` 10) | 131 |
  | `reckless-threads` | Reckless 0.9.0 | `D:/chess/engines/reckless-windows-avx2.exe` (`b74ead56…`) | `MoveOverhead` 10 | 132 |
  | `stockfish-threads` | Stockfish 18 bmi2 | `D:/chess/engines/stockfish/stockfish-18-windows-x86-64-bmi2.exe` (`bf2d8bf6…`) | defaults (`Move Overhead` already 10) | 133 |

  - **Rarog's binary is RAR-M82's**, not the d2 head's. Since `e057d84`
    the head has taken D.5.15 (a draw-verdict repair that moved the
    fingerprint) and D.3.1–D.3.8 (protocol and shell), neither of which
    touches the threads. Keeping the binary keeps this card's validity
    floor calibrated on the binary that read R 4.03, and G2's reuse on the
    argument RAR-M81 made for it.
  - **Fresh seeds.** RAR-M82's 200 games per engine played seeds 101 and
    102, the night's first openings; 131 to 133 play none of them again.
- **G2 is reused from RAR-M78's time-odds legs** (`20+0.2` against
  `10+0.1`, one thread on both sides, 1,000 games each): Rarog +101.8 ±
  11.5, Reckless +86.2 ± 9.8, Stockfish +86.9 ± 9.9. At one thread on both
  sides the two engines of a game take turns on one physical core, so the
  packing cannot reach them, and RAR-M82 keeps them as G2. Rarog's was
  played on `ebbbc917…`, the head before `e057d84`, which is
  behaviour-neutral (the same fingerprint) and changes only how the
  multi-thread copies of the kernel are laid out; one-thread NPS read
  2.79 M there and 2.73 M in RAR-M82.
- **The host sampler** of RAR-M81 (`d2-night/sampler.ps1`, `03b6e9d3…`)
  rides from before the first game to after the last. It samples the
  processor counters for every logical CPU, which is all the instrument
  rule reads; its thread counters cover `rarog*` and `reckless*` only, so
  Stockfish's leg has no per-thread columns, which nothing here reads.
- **Interacting mechanisms.** The one-thread side runs on the same four
  cores as the four-thread side, as in every earlier threads leg; the slot
  holds cores 10, 12 and 14 free as headroom. G4 and G2 are each engine's
  own, so strength, evaluation and node accounting cancel within an
  engine, and D compares engines only through their own doublings.
- **Runtime.** RAR-M78's Rarog and Reckless threads legs took 2 h 41 min
  and 3 h 02 min at this control and concurrency; Stockfish's about as
  long; about nine hours with the idle waits.

## Readings and judgement

- **Instrument, per leg, read first.** A leg is **INSTRUMENT** and not read
  when either holds:
  - **placement** (RAR-M82's rule): over the leg's clock window in the
    sampler's CSV, on the seconds with at least three of a slot's four
    logical CPUs above 60% (a four-thread turn), the share on four distinct
    cores is under 90%, or the share with an excluded sibling above 60% is
    over 5%, in any slot;
  - **R** = NPS(4T) / NPS(1T) by RAR-M79's rule, over the leg and each
    fifth of its games in completion order, is under the engine's floor:

    | Engine | Leg floor | Every fifth | Why |
    |---|---|---|---|
    | Rarog | 3.5 | 3.3 | RAR-M82's RESTORED threshold; it read 4.03, fifths 3.94–4.11; packed 2.48–2.66 |
    | Reckless | 4.0 | 3.8 | it read 4.45, fifths 4.36–4.53, sibling-free; packed 3.36–3.48 (RAR-M78, RAR-M80, RAR-M81) |
    | Stockfish | 3.8 | 3.6 | no sibling-free read; packed 3.47 (RAR-M78), near 4.45 clean at Reckless's packing loss; the floor lies between |

- **Elo:** G4 = arm B over arm A from Colosseum's final line, σ its 95%
  half-width / 1.96; **D = G4 / G2**, four threads in doublings of the
  engine's own time, by the delta method (var(D) = σ4² / G2² + G4² σ2² /
  G2⁴); X = 2^D; a difference of two engines' D adds their variances.
- **Outcome**, on three valid legs:
  - **DEFICIENT:** Rarog's D minus Reckless's and minus Stockfish's both
    have their 95% upper bound below zero (RAR-M78's rule).
  - **NOT DEFICIENT:** anything else; Rarog is not shown below both. The
    reading prints both differences, so a gap to one engine only is
    visible and recorded.
  - A leg with over 0.5% losses on time is reported and not judged
    (**NOT JUDGED**); an INSTRUMENT or missing leg leaves the outcome
    **INSTRUMENT**.
- **Power, frozen with the rule** (simulated at the levels below, G2's
  error included): for a true gap of Rarog's D below the two references'
  of 0.2, 0.3, 0.4 and 0.5 doublings, DEFICIENT reads with probability
  0.15, 0.42, 0.75 and 0.94; at no gap, under 0.01. Each difference's 95%
  half-width is about 0.28, two-thirds of its variance from the two
  engines' G2s. NOT
  DEFICIENT therefore excludes a gap of half a doubling and does not
  exclude one of 0.2.
- **Also read, not judged:** per-side depth over non-mate moves; the
  four-thread side's holds; each engine's G4 against its packed legs.
- **Reader:** `analysis/artifacts/d2-selfrel/m83_read.py`, which takes R,
  G4 and the runner's lines through RAR-M81's `m81_read.py` and the
  placement shares through RAR-M82's `placement_read.py`.

## Exposure before registration

- RAR-M78's six legs, RAR-M81's two, RAR-M82's two 200-game legs (seeds
  101 and 102), read. No Stockfish game has been played in a sibling-free
  slot, and no game of this design.

## Prediction, frozen before any game

| Outcome | Probability |
|---|---|
| NOT DEFICIENT | 0.70 |
| DEFICIENT | 0.20 |
| INSTRUMENT | 0.08 |
| NOT JUDGED | 0.02 |

- **Levels** (80% intervals):

  | Engine | R | G4 (Elo) | D | X |
  |---|---|---|---|---|
  | Rarog | 4.0 (3.85 to 4.15) | +135 (+118 to +152) | 1.33 (1.16 to 1.49) | 2.5 |
  | Reckless | 4.45 (4.3 to 4.6) | +130 (+115 to +145) | 1.51 (1.33 to 1.68) | 2.85 |
  | Stockfish | 4.3 (3.9 to 4.7) | +135 (+118 to +152) | 1.55 (1.36 to 1.75) | 2.9 |

  Rarog's D minus Reckless's −0.18, minus Stockfish's −0.22, each ± 0.28.
- **Reasoning.**
  - RAR-M82's 200 games, on other openings, read Rarog's four threads over
    one +127.0 ± 23.7 and Reckless's +121.1 ± 19.6: orientation only.
  - Each packed leg moved to its sibling-free R at its own G2 per doubling
    of NPS gives Rarog +146 (+83.9 at R 2.65), Reckless +142 (+110.6 at
    3.46) and Stockfish about +141 (+114.1 at 3.47, taking R 4.3). That
    treats the four-thread side's extra speed as worth as much as time,
    which lazy SMP's width is not, so the levels sit between it and the
    200-game reads.
  - Rarog's helpers buy little depth ([RAR-R29](RAR-R29.md),
    [RAR-R33](RAR-R33.md)): in RAR-M82 its non-mate depth gained 0.37 plies
    at four threads against Reckless's 1.75. So Rarog's D is put about 0.2
    below the others', a gap this design reads as DEFICIENT with
    probability about 0.15; DEFICIENT's 0.20 adds the chance that the gap
    is larger than that.
  - INSTRUMENT is three legs' chance of an episode or a mask that does not
    take, and Stockfish's floor is set without a sibling-free read of it.
- **Confidence:** moderate on the outcome, low on Stockfish's levels.

## Decision and stop rule

- **DEFICIENT:** D.2's four-thread SMP target is open on clean evidence.
  The card's first engine question is RAR-R33's next read, the depth and
  bound of the main thread's table hits against what its node needs, with
  helper depth staggering and the shared histories (Basilisk cross-check
  H27) as the competing mechanisms; expected value stated before any
  engine work; one candidate for the leaf's 4T SPRT.
- **NOT DEFICIENT:** D.2's four-thread SMP target closes `NO_CHANGE`; the
  engine's 2T–4T loss (RAR-P49, RAR-P50) and the four-thread holds stay
  recorded; the engine-side placement hint stays its own held leaf;
  scaling beyond four threads stays with G.1.
- **INSTRUMENT:** the failed leg is played again on another idle night, with
  the sampler's reading of its windows in hand; the valid legs stand and
  nothing else changes. **NOT JUDGED:** reported as such.
- Three legs, played to completion; an interrupted leg resumes at its seed
  by the same command; a leg abandoned for an engine fault is reported, not
  replaced. No leg's clock, threads, Hash, book, seed, placement or game
  count changes after any game, legs are not added, and the floors, the
  placement rule and the outcome rule do not move after a reading.

## How to run

From PowerShell 7 on an idle host, overnight, about nine hours:

```powershell
pwsh -File D:\code\rarog\analysis\artifacts\d2-selfrel\run.ps1
```

It dry-runs all three legs first, waits for an idle host before each,
resumes with the same command after an interruption, writes `m83_read.log`
beside itself and ends with `ALL DONE`.

## Preparation, 2026-11-03, before any game

- **Scripts** (`analysis/artifacts/d2-selfrel/`, SHA-256):

  | Script | SHA-256 | Role |
  |---|---|---|
  | `run.ps1` | `45591c82…` | the driver |
  | `leg.ps1` | `cfcca13e…` | one leg through `tools/colosseum.ps1` |
  | `m83_read.py` | `7fe2eb21…` | the reading |

  Full hashes: `run.ps1`
  `45591c823cfc0b54e064131de79204d15bac7fcb2935d322cce3cf455f94dc9e`,
  `leg.ps1` `cfcca13e51f95060d62a64c4653a60b24d4f9aca5afe0cabdaa86177f6991a89`,
  `m83_read.py`
  `7fe2eb217f00a8c2834b6b722baa37cb93fe42c49ebb4ce1c39491c3d79d687c`.
  `run.ps1` checks the three engines, its two scripts and the three it
  reuses (`d2-placement/placement_read.py` `fb2fabe3…`,
  `d2-night/sampler.ps1` `03b6e9d3…`, `d2-night/m81_read.py` `1f40d6aa…`)
  against their hashes before anything runs, dry-runs the three legs, then
  starts the sampler and waits for its first CSV row, plays the legs in the
  order above, each after the harness's idle wait, its output appended to
  `<leg>.log`, and reads. A leg that exits 0 with its `result.json` leaves
  `<leg>.done` and is skipped on a rerun; a half-played leg resumes at its
  seed. A resumed night starts a new `d2-selfrel-sampler-<stamp>`
  directory, and the reading takes all of them.
- **Dry run** (`run.ps1 -DryRun`, exit 0, `dryrun.log`): hashes matched;
  the three legs' manifests read 1,000 games, seeds 131, 132 and 133,
  Hash 256, threads A 1 and B 4, three at a time, `10000+100` with margin
  20, the UHO book in random order, no adjudication, Reckless's
  `MoveOverhead=10` on both sides and no option on Rarog's or Stockfish's,
  `placement: explicit (2,4,6,8,16,18,20,22,24,26,28,30), one logical CPU
  per physical core` and `slot_cpus` equal to that list. Nothing was
  played; no run directory exists.
- **Reading self-test** (`m83_read.py --self-test`, exit 0, 19 checks,
  `m83_read_selftest.log`): on RAR-M82's legs both valid, R 4.03 and 4.45,
  G4 +127.0 and +121.1, the lowest four-core shares 94.8% and 94.1% with
  no sibling busy, as recorded; a second sampler directory outside a leg's
  window adds nothing; on RAR-M81's packed legs both INSTRUMENT, on R (2.65
  under 3.5; 3.46 under 4.0) and on placement (0.0 to 1.1% of turns on four
  cores, a sibling busy in 85 to 100%), so the wire moves; the delta method
  on RAR-M78's figures gives DEFICIENT with upper bounds −0.26 and −0.36,
  as recorded; the judgement reaches every outcome. With no sampler
  directory the reader refuses (exit 2).
