# RAR-S99 — C.12 speed pass in games: the C.12 head against the C.13 head

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-17, before any game. Asked for by the maintainer when C.12 was delegated (2026-10-16) and confirmed with this design on 2026-10-17. The run is the maintainer's command.
- **Kind:** a speed gate. Every C.12 change is behaviour-neutral (both arms bench **10,521,141 / EBF 2.521**), so this prices the speed in Elo; it accepts no behaviour. RAR-S98 did the same for B.7's speed pass.
- **Arms:**
  - *C.12 head:* `tools/test_engines/rarog-c12speed-pext-pgo.exe` (`C689367B…`, `a829601`, clean): the C.13 head's engine plus C.12's 21 kept speed commits (`1781b34` to `b52520d`; `c8934ec` was reverted in `1108805`) and two exact cleanups (`91d0b0f`, `e55dc9e`).
  - *C.13 head:* `tools/test_engines/rarog-c13best-pext-pgo.exe` (`3C92DC47…`, `931dcca`, clean), the engine C.12 started from (`f86a657`'s).
- **What can differ:** speed only. The C.12 head's engine reads **+22.47%** pooled-PGO NPS over the C.13 head's (95% +21.60 .. +23.34; `bench 13`, four builds an arm, quiet host; `analysis/c12_speed_pass_2026-10-17.md`), and the two gate binaries read +22.80% apart directly. Each arm is one PGO build, so a build's own offset of about ±1% rides on the reading.
- **Gate (frozen):** Colosseum `sprt`, `sprt-default`: **`[0,3]` nElo**, cap **5,000 pairs**, `3+0.03`, Hash 64, one thread, the UHO book in random order, no adjudication, seed **20261017**, `tools/results/rar-s99`, `-ExpectBench 10521141,10521141`.
- **Rule (frozen):** H1 confirms the pass in games and nothing more is played (one gate, one read); C.12 then closes. H0 or the cap contradicts the NPS evidence: C.12 does not close until the cause is found (the clock, behaviour the fingerprint does not reach, the binaries). A time loss or fault by the C.12 head is investigated whatever the SPRT says.
- **Diagnostic, not the verdict:** each arm's nodes a second inside the games, read from the PGN with the mover taken from the position (RAR-S98's correction).
- **PRE-REGISTERED PREDICTION (freeze before exposure):** **+42 Elo** (about +70 nElo), 80% band **[+30, +55]**. Basis: the two earlier exact speed passes at this control give about 145 Elo a doubling (+10.35% for +20.3 ± 7.1; RAR-S98's +18.4% for +35.5 ± 9.0), and +22.47% is 0.292 doublings; in-game speed ran at about 0.9 of bench in RAR-S98, which that rate already carries. H1 in 500 to 1,300 pairs; probability of H0 or the cap 0.01. Most likely miss: magnitude, from the single-build offsets and the SPRT's high lean at an early stop.
- **Result:** not played.
