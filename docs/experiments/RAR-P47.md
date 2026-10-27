# RAR-P47 — `LargePages` on against off under the games' conditions: does the C head's slow state belong to large pages? — REGISTERED 2026-10-27, before the read

Indexed under *6. Throughput, build and platforms* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** 2026-10-27, the agent, D.2's second read after
  [RAR-P46](RAR-P46.md).
- **Question:** RAR-P46 found that under the games' conditions the C head
  alone enters a slow state shared by processes started together (about
  5.5 M nps against 7.4 M, 7–10% under the B.9 head), in 5 of 12 passes.
  Large pages are the one system resource the C head uses and the B.9 head
  does not. Does the state go when they are off?
- **Kind:** zero-game diagnostic, registered before the read. It decides
  what D.2 investigates next, not strength or a default.
- **Binary:** `tools/test_engines/rarog-d2lp-pext-pgo.exe` (`ebbbc917…7032`,
  `2b7bca9` on `d2`, `cargo xtask build --arch pext --pgo`, rustc 1.99.1,
  bench **13,987,223**), the `d2` head with the `LargePages` option.
  - Its live wire is proved: Hash 512 at 4T, the working set after
    `isready` goes 6.8 → 648 → 6.8 → 648 MiB as the option goes off, on, off.
    `bench` then reports `disabled`.
  - Check: `analysis/artifacts/d2-scaling/wire_check.py` (`87ff1541…`).
- **Arms:**
  - `lp-on`: that binary with `LargePages true`;
  - `lp-off`: the same binary with `LargePages false`;
  - `b9head`: the B.9 head (`aac92114…01ee`), the stable reference.
- **Instrument:** `analysis/artifacts/d2-scaling/lp_read.py` (`66a7491b…`)
  over `conditions.py` (`1139796d…`). The settings are RAR-P46's, with one
  reader process per engine:
  - Hash 512, 40 searches per process at `go movetime 100` on RAR-P46's two
    FENs, no `ucinewgame` between them;
  - engine-reported NPS; working set recorded after `isready`.
  - **Each pass runs one arm through three cells:** 1T alone, 4T alone, and
    three concurrent 4T processes pinned to CPUs 2–9, 16–23 and 24–31.
  - **Eight rounds, 24 passes,** each round in the rotated order
    (on, off, B.9), (off, B.9, on), (B.9, on, off), so each arm holds each
    place in the order about equally often.
  - Host: the 5950X match host, idle checked before the run.
- **Slow-pass rule, fixed now:** a pass is slow when its concurrent cell's
  median NPS is below 0.85 × the same pass's 4T-alone NPS. In RAR-P46 the C
  head's fast passes sat near 0.94 of the alone figure, its slow passes near
  0.70, and the B.9 head near 0.94.

## Prediction, frozen before the read

- **H1, large pages own the state:** `lp-off` has no slow pass in 8, and
  `lp-on` has at least 2 in 8. **Probability 0.4.**
  - Against it: in RAR-P46 every slow process held its tables in large
    pages, so the state was not a failure to get them.
  - For it: large pages are the only system-level difference.
- **H0, the state persists with large pages off:** at least one slow pass in
  `lp-off`. **Probability 0.45.**
- **Not reproduced:** no slow pass in either C arm. **Probability 0.15.**
- **B.9:** no slow pass (0.9).
- **Price of large pages off** in the 4T-alone cell: `lp-off` 2–8% under
  `lp-on`, from the TLB.

## Decision and stop rule

- **H1:** the allocator owns the slow state. D.2 then researches its
  mechanism: why 2 MB pages slow three concurrent processes in some starts.
  Also open, as a correctness and product question for the maintainer, not a
  conclusion of this read: whether `LargePages` should default off under
  concurrency.
- **H0:** not large pages. The next read separates the slots that share
  CCD1's L3 (two of the three) from a split placement, then frequency.
- **Not reproduced:** one pre-registered extension of eight more rounds;
  then stop and record "unresolved".
- No other change to the read after it starts.

## Result (read 2026-10-27)

Host idle before each block (3–8% CPU, no engine, harness or build
process). Data in `analysis/artifacts/d2-scaling/`: `lp_read.json`
(`d41d0347…`), `lp_read.log` (`12a92984…`), `lp_read_ext.json`
(`8a43b999…`), `lp_read_ext.log` (`d3b40cb8…`).

**Registered eight rounds: not reproduced.** No slow pass in any arm: on
0/8, off 0/8, B.9 0/8.

**The pre-registered extension, eight more rounds, unchanged:** on 2/8,
off 0/8, B.9 0/8.
- On, pass 17: 5.51/5.46/5.61 M concurrent against 7.80 M alone (0.70),
  RAR-P46's signature.
- On, pass 24: 6.09/6.64/6.40 M against 7.90 M (0.81), a partial one.

| Arm, 16 rounds | Slow | Concurrent/alone, min / median / max | 1T alone | 4T alone | Concurrent |
|---|---|---|---|---|---|
| on | 2 | 0.706 / 0.922 / 0.954 | 2.27 M, 2.21 M | 7.91 M, 7.90 M | 7.39 M, 7.23 M |
| off | 0 | 0.895 / 0.932 / 1.001 | 2.17 M, 2.13 M | 7.86 M, 7.70 M | 7.25 M, 7.22 M |
| B.9 head | 0 | 0.889 / 0.948 / 1.014 | 1.73 M, 1.71 M | 6.30 M, 6.29 M | 6.02 M, 5.93 M |

NPS columns give the registered block, then the extension. Every `lp-on`
process held its tables in large pages (working set 6–7 MiB after
`isready`), every `lp-off` one in ordinary pages (549 MiB at 1T, 648 MiB at
4T).

## Disposition

- **Consistent with H1, not established.**
  - The slow state appeared only with large pages on: 2 of 16 passes,
    against 0 of 16 off and 0 of 16 for the B.9 head.
  - At these counts that is weak: one-sided Fisher p = 0.24 for 2/16
    against 0/16. RAR-P46's 5 of 12 were all large-page processes too, but
    adding them is post hoc and two of its runs used the threaded reader.
- **The price of turning large pages off is small in these cells.**
  - Alone at 4T: −0.6% in the registered block, −2.5% in the extension.
  - At 1T: −4%.
  - Concurrent: about −1%.
  - So in these reads large pages buy about 1–4% when they work and cost
    about 30% when the slow state strikes.
- **The games point the same way.** RAR-M74's in-game −9.2% against the
  B.9 head is the slow state's level, not the fast state's +23%. That would
  fit a long tournament, with processes started and stopped game after game
  and physical memory fragmenting, keeping the C head mostly slow. This is
  an inference from two instruments, not a measurement.
- **Not decided here:** `LargePages`'s default. That is a product and
  strength question for the maintainer. The decisive evidence is in-game
  NPS and Elo at four threads with the option off against on, which is a
  game run.

## Calibration

- **Prediction:** H1 0.4, H0 0.45, not reproduced 0.15, all per eight
  rounds.
- **The miss was in frequency.** I expected RAR-P46's 5 slow passes in 12
  to carry over; here it was 2 in 16 with large pages on.
  - Two of RAR-P46's three runs read the engines through threads of one
    Python process, which may have favoured the state.
  - A single process starting fresh each pass, as here, may rarely hit
    the memory condition that causes it.
- **Sign:** right (the state only with large pages on).
- **Magnitude of the price of large pages off:** inside the predicted 2–8%
  at 4T only in the extension (−2.5%), under it in the registered block
  (−0.6%).
- **Instrument:** too few slow events for the question. A rare event needs
  more passes or the games' own conditions, not a finer NPS figure.

## Retry trigger

Either of these, registered first:
- **A longer zero-game read with only the on and off arms,** many process
  starts per arm, under a memory load that resembles a tournament's.
- **The in-game read:** RAR-M74's four-thread conditions with `LargePages`
  off against on, the maintainer's run.
