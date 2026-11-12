# RAR-O08 — Search-shape oracles on the current HCE: Rarog's head, Stockfish classical, Stockfish dev and Reckless, each searching Rarog's evaluation, at measured time odds

Indexed under *3. Search and selectivity › Search-oracle observations* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-13, before any game; maintainer-run overnight (six fixed matches of 3,000 games, about four hours).
- **Revisions:** Rarog `d6` at `35e9d69` (engine source `5f9a4a2`'s, bench **11,477,040 / EBF 2.537**); the bridges from `oracle-reads` at `e40ddc3` (engine `ea89fe7` adds `Board::from_placement`, unused by the engine, so the evaluation the oracles call is the head's).
- **Arms** (`analysis/artifacts/oracle-reads-2026-11-12/`, ignored; SHA-256):
  - `rarog-head`: `rarog/rarog-dev-20261112-35e9d692-windows-x64-pext.exe` `9fc94ca8…7051624`, `cargo xtask build --tier pext` (PGO) of the clean `d6` head; `id name Rarog dev-20261112-35e9d692`.
  - `sf-classical-hce`: `sf-classical/rarog-stockfish-hce-hybrid.exe` `42abbdca…a41ed`, the executable of RAR-O03 to O07 (Stockfish `9587eeeb`, the last classical master), with `rarog_hce.dll` `7d5a45a5…8e7214` rebuilt from `e40ddc3`.
  - `sfdev-hce`: `sfdev/rarog-sfdev-hce.exe` `d2135512…d50f08`, Stockfish `def601cc` (2026-11-10, net `nn-68752b755c93` embedded and never evaluated) with `tools/oracle-hybrids/stockfish-def601cc-rarog-hce.patch` (`14f055ac…`), MinGW `profile-build ARCH=x86-64-bmi2`; the same DLL.
  - `reckless-hce`: `reckless/reckless-hce.exe` `cade1a19…a5ecd`, Reckless `0756957` (2026-11-07, net `v60-7f587dfb`) with `tools/oracle-hybrids/reckless-0756957-rarog-hce.patch` (`c546d2c4…`), `cargo pgo` with `target-cpu=native`, no Syzygy.
  - Each patch was checked with `git apply --check` against a fresh clone of its donor commit. The recipes are in `tools/oracle-hybrids/README.md`.
- **Research question:** how much is a search shape worth on Rarog's own evaluation, measured with no bridge penalty? Three parts, one per decision PLAN must take: (a) is Rarog's search behind the best HCE search; (b) does Stockfish's or Reckless's current search transfer better to this HCE; (c) does Stockfish's NNUE-era search lose to its own classical-era search on an HCE.
- **Why now:** the Basilisk comparison (Rarog 2.4.0 and Basilisk 1.10.2 level, Basilisk's `sf_19`-shaped Phase B +318.1 ± 24.5 against Rarog's Reckless-shaped +252.6 ± 20.4, Basilisk's B.4 head +71.3 ± 18.0 over 2.5.0, all RAR-M84), and the finding that RAR-O03/O04's oracle evaluated through a FEN round trip, at 0.56 of Rarog's own speed where Basilisk's ran at 0.83 of its own. RAR-O04's "+24.24 ± 8.01, the deficit closed" was therefore read against a handicapped oracle.
- **Instrument, measured before any game:**
  - *Bridge.* `Board::from_placement` replaces the FEN round trip. The adapter's timing test: 260.7 ns an evaluation, 98 ns the bridge, on 50,009 positions. The classical oracle benches at 1.91 Mnps where RAR-O03 recorded 1.64 (different evaluation revisions; not a controlled comparison).
  - *Bridge price in each search* (`tools/oracle-hybrids/bridge_price.py`, three cycles each of `RAROG_HCE_BRIDGE_REPEAT` 1 and 3, identical node counts in all six runs, so the wire is live and the trees equal; host idle at 1.5% CPU before): Stockfish classical 43.9 ns of 518.3 per node, **8.5%**; Stockfish dev 40.1 of 504.9, **7.9%**; Reckless 48.8 of 433.4, **11.3%**. Raw outputs are `bridge_price_*.json` in the artifact folder.
  - *Time odds*, which hand that share back: ×1.093, ×1.086, ×1.127. As clocks, rounded to the millisecond: `sf-classical-hce` **3.279+0.033**, `sfdev-hce` **3.259+0.033**, `reckless-hce` **3.380+0.034**, `rarog-head` 3+0.03. Odds equalise the bridge only; each engine keeps its own speed otherwise, which is part of what a search shape is.
  - *Scale.* Mean |raw network| over mean |Rarog HCE| on B.0's corpus (`tools/results/b0-20260913/scale_corpus.txt`, 10,040 positions), by each hybrid's `hcescale` command: Reckless 959.53 against 387.48, **0.4038**, scale 2536/1024 (B.0 measured 0.406 on the same net and corpus); Stockfish dev 1006.64 against 387.48, **0.3849**, scale 2660/1024. Both hybrids report the same HCE mean, a check that both bridges feed the same evaluation. Each donor's own wrapper (`correct_eval`, `scale_evaluation`) then applies as it does to its network. The classical oracle keeps its tag's `PawnValueEg/100` mapping, for continuity.
  - *Harness wire.* `sprt.ps1` gained per-engine clocks (`-TCA`, `-TCB`). A 20-game smoke at 0.5+0.005 against 5+0.05 wrote `WhiteTimeControl "5+0.05"` and `BlackTimeControl "0.5+0.005"` into the PGN, the odds into the manifest, and scored 7.5% for the short clock. `-Nodes` with a clock and a malformed clock are both refused. The smoke's files are in the artifact folder.
- **Interacting mechanisms / confounds:**
  - *Unfitted donors.* Stockfish dev's and Reckless's constants are fitted to their networks. An unfitted read measures transfer, not what a fitted rebuild would reach. Rarog's B.2 fit added +138.6 over its unfitted donor shape (RAR-S73), Basilisk's +58.7 (BAS-S23), so a small difference between donors here does not rank fitted potential.
  - *Stockfish classical* is fitted to Stockfish's classical evaluation, not to Rarog's. It is the strongest HCE search known here; it is not Rarog's ceiling.
  - *Time management* belongs to each engine and is part of every read.
  - *The scale rule* (raw over raw) is one choice; B.0 recorded its spread (per-position median 0.350).
- **PRE-REGISTERED PREDICTION (frozen 2026-11-13, before any game).** Elo of the first-named arm, 80% bands:
  1. `sfdev-hce` over `reckless-hce`: **+40** [−10, +90]. Basis: Stockfish's shape needed a smaller fit on an HCE (BAS-S23's +58.7 against RAR-S73's +138.6).
  2. `rarog-head` over `sf-classical-hce`: **+10** [−30, +50]. Basis: RAR-O04's +24 less about 50 for the oracle's speed (the faster bridge and the odds, about 0.35 of a doubling), plus about 55 Rarog gained since (C.12's speed pass +42.3, C.10 +4.5, D.6's search repairs).
  3. `sfdev-hce` over `sf-classical-hce`: **−60** [−110, −10]. Basis: an unfitted NNUE-era search against a classical search fitted to a classical evaluation.
  4. `rarog-head` over `sfdev-hce`: **+70** [+20, +120].
  5. `rarog-head` over `reckless-hce`: **+110** [+60, +160].
  6. `reckless-hce` over `sf-classical-hce`: **−100** [−150, −50].
  - Probability (1) is above zero: 0.75. Probability (2) is −30 or below: 0.15.
  - Expected Elo for Rarog: none. These are instruments, not strength claims.
  - Most likely failure mode: reading an unfitted donor's deficit as the shape's worth.
- **Use of the reading, frozen:** these feed the PLAN rework the maintainer scheduled for 2026-11-13. They accept and reject nothing by themselves.
  - *(a) Search rework.* Indicated if (2) is −30 or below with its 95% interval excluding zero. At −10 or above, this evidence does not indicate one. Between, it is unresolved here.
  - *(b) Donor shape for a rework.* Stockfish if (1) is +20 or above with its interval excluding zero; Reckless if −20 or below with its interval excluding zero; otherwise not separated by this read, and the choice rests on cost and mechanism.
  - *(c) An HCE shape apart from the NNUE shape.* Indicated if (3) is −20 or below with its interval excluding zero. This is the case in which a rework fitted to the HCE would be partly redone for the network.
  - Matches (4) to (6) close the round robin and check (1) to (3) for consistency through common opponents.
- **Falsification criteria:** each band is judged as stated. A miss in (2) by more than its band is a miss in the instrument's assumptions (the speed and gains estimates), not in the search.
- **Registered design and stop rule:** no gate. Six fixed matches of 3,000 games each, in the order above (the decision reads first), no early stop, no extension, no re-run on the same binaries. `tools/sprt.ps1 -Mode fixed -Games 3000 -NoAdjudication -Seed 20261113`, UHO_Lichess_4852_v1 in random order, Hash 64, 1T, the runner's default concurrency (14 with explicit affinity), timemargin 20 ms, `MoveOverhead=10` for `reckless-hce` (its default is 100; every other arm defaults to 10). The runner is `sprt.ps1` and not Colosseum because the oracle reads have always used it (RAR-O03 to O07) and Colosseum's guards require Rarog sidecars and a shared compiler, which foreign engines lack.
- Disposition: open — registered 2026-11-13, before any game; the maintainer plays the six matches.
