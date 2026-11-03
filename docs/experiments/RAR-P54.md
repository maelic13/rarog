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
- **Result:** appended after the read.
