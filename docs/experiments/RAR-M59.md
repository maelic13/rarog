# RAR-M59 — First-search stalls: the Colosseum confirmation — REGISTERED 2026-09-19, before any game (PLAN B.2.8)

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

## Experiment and conditions

**First-search stalls: the Colosseum confirmation — REGISTERED 2026-09-19, before any game (PLAN B.2.8).** **Defect, measured here:** the KPK bitbase was built on its first probe, inside the first search that reached KPK (about 34 ms and 130 page faults on that search's clock, both arms, 8 fresh processes each); with `Threads > 1` the first search also converted the hash table to its shared form (128 MiB: +20 ms, 32,917 page faults); helpers held and then freed an unused 64 MiB table each, and per-thread tables were first touched by the first search when no `ucinewgame` came. The maintainer's lead: about 1 time loss in 1,000–2,000 games under Colosseum CLI, fresh engine processes per game, every one late in the game and 42–64 ms past Rarog's hard cap. The B.2.3 SPSA, which restarts engines every iteration, shows 314 time forfeits in 125,710 games, all past ply 78 (not individually attributed). **Fix:** engine `9a7b663` builds every lookup table in `main` before input is read and in `Engine::new`, and converts the table at `configure`; `c0e6ef7` gives helpers a 1 MiB placeholder table and clears per-thread tables at start-up and helper creation, as Stockfish and Reckless do. Output-neutral: bench 7,601,220 / EBF 2.474 and 4,706,910 unchanged. On the final binaries a fresh process's first KPK search takes 0.16 ms against 0.12 warm (was 34), the 4T first search 7.3 ms against 6.8 warm (was 34), and start-up to `readyok` about 55 ms. **Binaries** (`c0e6ef7`, clean): `rarog-startupfix-core-pext-pgo.exe` (4,706,910, sha256 `84225A57…`) and `rarog-startupfix-base-pext-pgo.exe` (7,601,220, sha256 `2CFC2DFE…`). **Run:** Colosseum CLI 0.1.0 `match`, the core against the base arm, **10,000 games**, `3+0.03` with a 20 ms margin, Hash 128, one thread, fresh engine processes per game, concurrency 14 with placement auto, UHO_Lichess_4852_v1 in random order with a generated master seed, no adjudication, fault limit 100 (dry run resolved 2026-09-19). **Prediction, frozen here:** 0 time losses (the lead's rate predicts 5 to 10 without the fix). **Falsifier:** any time loss, read with Colosseum's per-search held time and page faults before anything else is concluded. An observation of the harness contract, not a strength gate; the Elo it reports is not a gate. **Handed to Colosseum development, maintainer decision 2026-09-19:** the confirmation is the harness's qualification and runs there, not in Rarog; B.2.8 closed on Rarog's own evidence (bench unchanged on both arms, fresh-process timings, tests that fail with the fix disabled). The result is appended here when Colosseum reports it; nothing in Rarog waits for it.

## Result / disposition

**Not run, and it will not be:** Colosseum closed its step 10.9m on 2026-09-19 without pursuing the 10,000-game target, and every binary in its qualification runs predates `c0e6ef7` (86,450 games, no time loss, but no test of this fix). Substitute evidence, fastchess with engines kept across games: RAR-S77 and B.2.4b played 21,055 games on post-fix binaries, every termination `normal` (`analysis/b2_audit_2026-09-21.md`).

## Conditional lesson and retry trigger

**Not scored:** the prediction named fresh engine processes per game under Colosseum, and no such run on a post-fix binary exists. The fix rests on Rarog's own evidence (fresh-process timings, tests that fail with it disabled); a fresh-process run is owed only if time losses reappear.

## Source

PLAN B.2.8; engine `9a7b663`, `c0e6ef7`
