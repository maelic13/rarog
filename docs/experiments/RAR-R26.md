# RAR-R26 — D.1.6: the LMR correction term capped at one ply, a global search change, read at equal speed at `3+0.03` and `10+0.1` before any gate — REGISTERED 2026-10-25, before any game

Indexed under *4. Root search, time management and SMP* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-25 before any game. The zero-game
  reads are the agent's (evidence in
  `analysis/artifacts/d16-cascade-2026-10-23/tree-growth-2026-10-25/`,
  `RESULTS.md` against the frozen `PREDICTIONS.md`); the two matches are
  the maintainer's.
- **Kind:** D.1.6's research card, its stop rule's second branch. The
  device that ends the won-ending tree's explosion changes most ordinary
  positions, so it is a global search change. It is read at equal speed
  first; only a pass registers the `[-5,5]` gate.
- **The device** (`c4da889`): `CoreLmrCorrectionCap`, tune-only. It bounds
  what the correction term takes off a late-move reduction, in 1024ths of a
  ply; 0 = uncapped, today. Arm A caps it at 1024, one ply.

## The evidence that made it the candidate (zero games, nodes)

- **The term at reductions.** It takes a ply or more off:
  - 19.2% of bench's reductions;
  - 73.5% of rec1's at depth 18;
  - 71.5% of rec3's at depth 20.

  It drives most deeper-than-nominal searches: 56.8% on bench, 91.4% on
  rec1. rec1 searches 31.4% of its reduced moves deeper, against bench's
  10.5%.
- **Traces at 200 M nodes.** Cap 1024 takes rec1 from depth 20 to 26 (the
  exact depth 25 at 16.6 M nodes, against depth 19 at 154.0 M) and rec3
  from 21 to 24. Caps of 2048 and 3072 do not help. The term at 0 reaches
  35 and 28.
- **The differential.** Cap 1024 changes 24 of 40 fresh-process bench
  positions, 32-piece openings among them; bench 13 reads 11,480,295
  against 13,987,223. Caps of 2048 and 3072 change 5–7 positions but leave
  the stalls.
- **Interactions.**
  - C.10 fitted `CoreLmrCorrection` (3330) and its neighbours with the
    term uncapped.
  - The correction also feeds the singular margins and the static
    evaluation; the cap changes neither.
  - The stop at twice the target (`a79efe4`) is in both arms, so the
    window walk at a win's resolution stays capped either way.
- **Competing explanations kept open:**
  - a large correction may mark a position the search should search deeper
    (the term's purpose), so the cap could cost decisions;
  - or the deeper searches are the line growth this card found, and the
    cap gains depth.

  Only games separate them.

## Build and arms

- Both sides run `tools/test_engines/rarog-d16inst4-tune.exe` (`c4da889`,
  SHA-256 `61738C60B477C12A…D378C65103`, `rustc 1.99.1`, bench 13,987,223).
- Arm A sets `CoreLmrCorrectionCap=1024`; arm B sets nothing.
- Both carry `d1`'s defaults, including the stop at twice the target.

## The reads (frozen)

- **`stc`:** Colosseum `match -CategoricalTuneBuild`, 2,000 games at
  `3+0.03`, 20 ms margin, Hash 64, one thread, the wrapper's slot count,
  the UHO book in random order, no adjudication, seed 20261101,
  `tools/results/rar-r26-stc`.
- **`ltc`:** `match-fixed-ltc`, 1,000 games at `10+0.1`, seed 20261102,
  `tools/results/rar-r26-ltc`.
- **Command:** `run_r26.ps1 -Read stc|ltc` beside the evidence. Both
  dry-ran clean on 2026-10-25 (host 5–6%, no engine process, the pinned
  colosseum-cli 0.4.0, both benches matched). The resolved configuration
  puts the cap on engine A only.
- **The harm rules**, from the manifest's nElo, with RAR-R23's power:
  `stc` below **−13 nElo**, `ltc` below **−18 nElo**.
- Time a move per arm from the PGN (`nps_by_arm_n.py`), reported, deciding
  nothing.

## Dispositions

- **Neither read below its threshold:** the cap goes to its gate,
  registered then before any game.
  - The gate: a PGO arm with the cap baked at 1024 against `d1`'s head,
    the symmetric `[-5,5]` with its cap (AGENTS' repair case 3; it changes
    shared search signals in the rated configuration), plus the probe with
    the stop off, to read the search half alone.
  - A pass there lands the cap, re-declares the fingerprint, and re-reads
    whether the stop at twice the target is still needed.
- **Either read below its threshold:** the cap is not pursued, and the
  won-ending tree's growth is accepted.
  - The stop at twice the target stays as the cap on the stalls.
  - D.1.6 closes on that stop, with this card recorded.
  - Retry trigger: a refit of the LMR terms (B-type cluster or F.6) re-reads
    the cap with the term's slope free.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **`stc`:** −3 ± 12 nElo, P(pass) 0.8. The term's slope was fitted with
  the term uncapped, and capping it at one ply undoes the fit for a fifth
  of the reductions; the deeper search it buys is worth something only
  where trees grow.
- **`ltc`:** −1 ± 16 nElo, P(pass) 0.8. Line growth costs more at a longer
  clock, so the cap's relative value rises.
- **Time a move:** within ±1%, P 0.8.
- **Most likely miss:** a small negative at `stc` from the fitted slope,
  which 2,000 games cannot separate from zero.
