# RAR-P54 — Sibling loss: what a Rarog or Reckless thread keeps beside its SMT sibling on one core — REGISTERED 2026-11-03, before the read

- **Date / owner:** 2026-11-03, D.2 (the audit's section 10,
  `analysis/d2_audit_2026-11-02.md`).
- **Baseline SHA / binaries:** d2 head `38ef0d5`; Rarog
  `tools/test_engines/rarog-d2dyn-pext-pgo.exe` (`a545b6b8…`, the `e057d84`
  head of RAR-M79, RAR-M80 and RAR-M81); Reckless 0.9.0
  `D:/chess/engines/reckless-windows-avx2.exe` (`b74ead56…`).
- **Research question:** RAR-M81's four-thread ratios, 2.65 for Rarog and
  3.46 for Reckless, were both measured with Windows packing the threads on
  two or three cores' SMT siblings inside Colosseum's eight-logical-CPU slot
  (the audit, section 10: Rarog on two cores in 90–96% of its turns,
  Reckless on two in 47–66% and three in most of the rest). How much of the
  gap is what each engine's thread loses beside its sibling on one core,
  rather than where the threads were put?
- **Hypothesis:** working back from the packing shares, a Rarog thread beside
  its sibling keeps about two thirds of its solo-core work, a Reckless
  thread about five sixths. Candidate cause: per-thread working set (history
  tables, eval and pawn caches) that fits one core's L2 alone and not
  doubled, against Reckless's denser network inference.
- **Competing hypotheses:** (a) both lose the same beside a sibling, so the
  night's gap is placement alone; (b) Rarog loses even more than two thirds
  and the night's 2.65 also carries some placement luck.
- **Interacting mechanisms:** the shared table (Hash 256) and the two-thread
  search's own loss cancel in the ratio, since both two-thread cells run the
  same search; `movetime` is not a game clock, and the unit is one core's
  two siblings, not the four-thread slot.
- **Instrument:** `analysis/artifacts/d2-placement/sibling_loss.py`
  (`1b32071f…`), run by `sibling_loss.ps1` (`3dfe3ba2…`), which first
  requires the idle host by the harness's measure and both binaries' hashes.
  Each engine standalone under a process affinity mask set before `uci` and
  read back: `1T-core` Threads 1 on logical CPU 4 (mask `0x10`);
  `2T-siblings` Threads 2 on CPUs 4 and 5, both siblings of core 2
  (`0x30`); `2T-cores` Threads 2 on CPUs 4 and 6, one per core (`0x50`).
  Hash 256, `ucinewgame` then `go movetime 8000` on four positions (the
  start position, Kiwipete, a queen's-gambit middlegame, a rook endgame); a
  cell's reading is the sum of the final `info` line's nodes over the sum
  of its time. Two cycles, each engine then each cell, the cell order
  rotated per cycle. **f = NPS(2T-siblings) / NPS(2T-cores)** is the share
  of its solo-core work a thread keeps beside its sibling;
  2T-cores / 1T-core is read beside it.
- **Wire:** a 300 ms, one-cycle smoke before registration proved the masks
  (read back as set), both engines' replies and the parser; its numbers
  (0.62 and 0.65) are ramp, not a reading, and were seen after the
  prediction below had been stated.
- **PRE-REGISTERED PREDICTION (frozen before the read):**
  - f_rarog 0.66 (0.60 to 0.75); f_reckless 0.83 (0.75 to 0.92).
  - P(f_reckless − f_rarog ≥ 0.08) = 0.70; P(within 0.04 of each other,
    hypothesis (a)) = 0.20; P(Rarog lower by 0.04 to 0.08) = 0.10.
  - 2T-cores / 1T-core: Rarog 1.90 to 1.98 (RAR-P49/P50's 6–7% a thread),
    Reckless 1.95 to 2.05.
  - Most likely failure mode: `movetime` cells at 8 s read a different
    working set than a 10+0.1 game's, so f is right in sign and off in
    magnitude.
- **Falsification:** f_reckless − f_rarog under 0.04 refutes the per-core
  hypothesis: the night's gap is placement alone, and the per-thread
  working-set question leaves D.2's card. Both f under 0.70 means sibling
  sharing costs any engine here about a third, which makes the sibling-free
  slot the only fair four-thread measurement and says nothing about Rarog.
- **Stop rule:** zero games; two cycles. A cell whose two cycles disagree by
  over 3% gets one third cycle for every cell; the mean of all cycles is
  read. Nothing else is tuned or repeated.
- **Conditions:** idle host (no engine, harness or build process, at most
  10% CPU, the harness's measure), the Ryzen 9 match host, Windows 11,
  masks above, Hash 256, Threads 1 or 2, `movetime 8000`, four positions,
  the two binaries by hash; raw readings archived in
  `tools/results/sibling-loss-<stamp>/raw.json`, the table in
  `analysis/artifacts/d2-placement/sibling_loss.log`.
## Result — 2026-11-03, appended after the read

- **Run:** `tools/results/sibling-loss-20261103_094151/raw.json`
  (`0489671a…`), the table in
  `analysis/artifacts/d2-placement/sibling_loss.log`; host idle at 3.0%
  CPU with no engine, harness or build process; every mask read back as
  set. Two cycles, then the stop rule's third for every cell (Rarog's
  2T-cores cell read 5.03 and 5.37, 6.6% apart): `--extend` was added to
  the script after exposure for that cycle (`ac90c1f0…`); the first two
  cycles' readings are untouched in `raw.json`.

| engine | cell | cycle 1 | cycle 2 | cycle 3 | mean (Mnps) |
|---|---|---|---|---|---|
| Rarog | 1T-core | 2.683 | 2.715 | 2.719 | 2.706 |
| Rarog | 2T-siblings | 3.272 | 3.291 | 3.289 | 3.284 |
| Rarog | 2T-cores | 5.028 | 5.371 | 5.245 | 5.215 |
| Reckless | 1T-core | 1.307 | 1.340 | 1.336 | 1.327 |
| Reckless | 2T-siblings | 1.769 | 1.777 | 1.804 | 1.783 |
| Reckless | 2T-cores | 2.724 | 2.755 | 2.795 | 2.758 |

- **f_rarog 0.630, f_reckless 0.647;** the difference 0.017. Two cores
  against one thread: Rarog 1.93, Reckless 2.08.
- **Disposition: REFUTED.** The per-core hypothesis is out: a Rarog thread
  and a Reckless thread keep the same share beside a sibling, within 0.02.
  Competing hypothesis (a) holds: the night's ratio gap (2.65 against
  3.46) is placement alone. Both f are under 0.70, so sibling sharing
  costs any engine on this host about 37% a thread, and a slot of one
  logical CPU per physical core is the only fair multi-thread
  measurement; nothing here is about Rarog's code.
- **Instrument note:** Rarog's final `info` line carries the cumulative
  time at its last report (26 to 29 s of the 32 s searched), Reckless's
  the full 32 s; each cell's rate is cumulative nodes over cumulative time
  at that line, the same measure within an engine, so f is unaffected.
  Rarog's one-thread node counts are identical across cycles (its last
  completed iteration), its times differ.

## Calibration

- **Original prediction (unchanged above):** f_rarog 0.66 (0.60 to
  0.75), f_reckless 0.83 (0.75 to 0.92), the gap 0.08 or more at 0.70.
- **Observed:** Rarog inside its range; Reckless far below it, 0.647
  against 0.75 at the lowest. The miss is in mechanism and magnitude. The
  five sixths was worked back from the night's per-second packing shares
  as if a share of seconds were a share of thread-time; threads migrate
  within a second, so the night's 3.46 under partial packing is consistent
  with the same per-sibling loss and more thread-time on three cores.
- **Confidence:** 0.70 was too high for a figure derived through an
  assumption the data could not check.
- **Conditional lesson:** a per-second placement share is not a per-thread
  state; read per-thread loss directly, as here, before attributing it.
- **Retry trigger:** `closed`. The per-thread working-set question leaves
  D.2's card; RAR-M82 decides the placement question.
