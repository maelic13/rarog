# RAR-M72 — Dragon NNUE in Rarog's search against the current classical head at `3+0.03`

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**Maintainer request 2026-10-15: a fixed 2,000-game match between the
Dragon NNUE evaluator in Rarog's own search and the current best classical
head, under the usual gate conditions. PLAYED 2026-10-15: +327.9 ± 14.4 Elo.**
Registered retrospectively: the prediction below was stated in the agent's
handover of the prepared command, before the maintainer started the run
(11:08:32 UTC), but it was not committed before exposure. It is reproduced
unchanged.

- **Question:** how much stronger the imported Dragon 3.3 network
  ([DRAGON_NNUE](../DRAGON_NNUE.md), F.11–F.13) plays than the current
  hand-written evaluation when both run inside the same Rarog search, with
  nothing refitted for the network.
- **Arms.** Engine A, `dragon`: `dragon_nnue` at `e058052` (F.13's head,
  before F.14), `tools/test_engines/rarog-dragon-pext-pgo.exe`, SHA-256
  `2f87f030…53b1e7`, UCI `Evaluation=DragonNNUE`, `DragonNNUEScale` at its
  default 72, the embedded model `c6e25a7f…064417`. Engine B, `c13best`:
  `c7` at `931dcca` (the classical development head after C.13.2),
  `tools/test_engines/rarog-c13best-pext-pgo.exe`, SHA-256
  `3c92dc47…9b4b49`. Both `cargo xtask build --arch pext --pgo` from clean
  trees with `rustc 1.98.1`, built by `tools/build_test.ps1`.
- **Fingerprints.** Sidecar benches, Classical: A 14,197,472 (`52cc6de`'s
  evaluation, the branch base) and B 10,521,141, both as registered with
  `-ExpectBench`. A in Dragon mode, `tools/diag/dragon_nnue_bench.py`:
  17,728,094 / EBF 2.604, RAR-P38's Dragon fingerprint.
- **Search identity.** `git diff 52cc6de 931dcca -- src/search` is empty,
  so the arms share the search code apart from `dragon_nnue`'s evaluator
  plumbing; they differ in the evaluator.
- **Conditions.** `tools/colosseum/match-fixed.toml`: `3+0.03`, margin
  20 ms, Hash 64, 1T, concurrency 14, UHO_Lichess_4852_v1 in random order,
  seed 1015, no adjudication, 2,000 games; runner colosseum-cli 0.3.0, pinned
  `cb12cd3`. The dry run passed every guard on an idle host (2.5% CPU).
- **Command,** from PowerShell: `./tools/colosseum.ps1 -Mode match -EngineA tools/test_engines/rarog-dragon-pext-pgo.exe -EngineB tools/test_engines/rarog-c13best-pext-pgo.exe -NameA dragon -NameB c13best -OptionsA "Evaluation=DragonNNUE" -Games 2000 -Seed 1015 -ExpectBench 14197472,10521141 -Dir tools/results/dragon-vs-c13best`.
- **Live wire.** The resolved configuration sends `Evaluation=DragonNNUE` to
  engine A only. A 2-game smoke run (`tools/results/dragon-smoke`) showed
  engine A at 1.62 M in-game NPS, its Dragon bench speed, and engine B at
  2.18 M.
- **PGO.** xtask trains PGO on a Classical `bench 13`, so the Dragon path is
  untrained. One rough read per binary on an idle host, Dragon mode: the PGO
  arm 1,636,338 NPS, a plain `cargo build --release --locked` of the same
  tree (`b6214137…`) 1,605,660 NPS, both 17,728,094 nodes; Classical gains
  about 15% from PGO in the same pair. The repeat PGO run was refused as a
  busy host and was not retried. Under these conditions PGO neither helps nor
  visibly hurts the Dragon path.
- **Prediction, as stated before the run:** Dragon ahead by about
  **+150 Elo**, 80% band **[+60, +260]**. Basis: the network should
  dominate, but about a 25% speed deficit, search margins and a lazy gate
  fitted to the HCE, and an unrefitted score scale would hold it back.
- **Evidence layer:** a fixed match's Elo at one control between two Rarog
  builds. It decides nothing, accepts no strength claim for the shipped
  engine and changes no Phase F rule: the network stays comparison-only, is
  never a label source and never ships.

## Result / disposition

**Played 2026-10-15 11:08:32–11:29:23 UTC** (`tools/results/dragon-vs-c13best/`;
manifest `tools/results/colosseum_match_dragon-vs-c13best_20261015_130831.manifest.txt`
`f79b19af…`; `games.pgn` `0ef00546…`, `result.json` `e2f3a249…`):
**+327.9 ± 14.4 Elo** (nElo +594.3 ± 15.2) for Dragon over 1,000 complete
pairs, **1,524-50-426** (86.8%), pentanomial [0, 1, 58, 407, 534] (the
runner's recount of `games.pgn` agrees), 0 faults, 0 time losses, every game
a rules result.

In the games (`analysis/artifacts/rar-m72/read_match.py`, `fbc63e11…`,
over all 2,000 games' move comments): Dragon 1.49 M NPS, mean depth 14.80;
c13best 1.55 M NPS, mean depth 14.76; median depth 12 for both. The in-game
speed gap is about 4%, so the result is an evaluation difference at
practically equal speed and depth.

Disposition: recorded, informational. The maintainer kept the roadmap order
on this result: finish C, then D and E, then F (maintainer decision
2026-10-15).

## Calibration

Sign right, **magnitude wrong**: the whole 95% interval (+313.5 to +342.3)
lies above the 80% band's upper end.

- *Instrument:* the expected 25% speed deficit came from bench NPS. In the
  games the gap was 4%; the classical arm ran far below its bench speed
  (1.55 M against about 2.4 M) while Dragon stayed near its own. A bench
  ratio is not a proxy for the in-game ratio between evaluators.
- *Mechanism:* the expected cost of an HCE-fitted search (margins, lazy gate,
  score scale) is unmeasured. Without a refitted arm this match cannot say
  whether it was small or simply outweighed.
- *Reference class and confidence:* the estimate leaned on first own-data
  networks replacing a hand-written evaluation, not on a mature commercial
  network; an 80% band of ±100 was too narrow for a comparison with no
  precedent here.

## Artifacts

`tools/results/dragon-vs-c13best/`, its manifest and dry run;
`tools/results/dragon-smoke/`; `analysis/artifacts/rar-m72/` (the reader
and its output, the two Dragon bench reads `A1/` `4ff5ad38…` and `B/`
`c54f275d…`, their logs, the dry-run and smoke logs). Paths are on the
maintainer's development machine.
