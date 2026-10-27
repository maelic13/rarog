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
