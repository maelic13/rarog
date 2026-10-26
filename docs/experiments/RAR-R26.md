# RAR-R26 — D.1.6: the LMR correction term capped at one ply, a global search change, read at equal speed at `3+0.03` and `10+0.1` before any gate — REGISTERED 2026-10-25, before any game; READ 2026-10-25: no harm, and the cap and the stop cover each other in the probe

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

## Results (2026-10-25, maintainer-run, idle host)

Both matches completed with 0 faults and 0 time losses. The figures are the
runner's final report lines (colosseum-cli 0.4.0). A is the cap at 1024, B
uncapped.

| Read | Elo | nElo | Harm threshold | W/D/L (A) | Pentanomial | Host at launch |
|---|---|---|---|---|---|---|
| `stc`, 2,000 games | **−1.4 ± 9.0** | **−2.4 ± 15.2** | −13 | 511/970/519 | 29, 237, 475, 231, 28 | 0.3% |
| `ltc`, 1,000 games | **+1.7 ± 11.5** | **+3.2 ± 21.5** | −18 | 242/521/237 | 4, 122, 246, 121, 7 | 4.8% |

- **Manifests:** `colosseum_match_rar-r26-stc_20261025_155523.manifest.txt`
  (`games.pgn` `88AB94DC…15F9729BC8`) and
  `colosseum_match_rar-r26-ltc_20261025_161536.manifest.txt` (`C5B831AF…EAB02DC847F`),
  in `tools/results/`.
- **Time and depth per arm** (`nps_by_arm_n.py`, `nps_r26_*.txt` beside the
  evidence). A against B:
  - `stc`: time a move −0.6%, mean completed depth 12.77 against 12.68
    (+0.09 ply);
  - `ltc`: −0.6%, 16.28 against 16.04 (+0.24 ply).

  The cap acted in the games: deeper at equal time.

**Rule applied.** −2.4 nElo is above −13, and +3.2 is above −18: **no harm
at either control; RAR-R26 passes.** This is a harm read, so neither
estimate is accepted as a gain.

**The timed probe, added after the matches and before any gate (agent-run,
2026-10-25).** `tm_rec_probe_d16.py` on the same tune build, idle check
2.9% CPU over 20 s, no engine process (`tm_rec_probe_r26.txt`; one earlier
attempt was refused by the idle check and measured nothing). As a multiple
of the probe's optimum:

| Clock, position | cap, stop off | uncapped, stop off | cap with the stop at twice the target |
|---|---|---|---|
| `60000+600` rec1 | 0.91 | 1.05 | 0.91 |
| `60000+600` rec3 | 0.61 | 0.46 | 0.61 |
| `180000+2000` rec1 | **MAX** (104.4 s; depth 26, 3,910 cp lower bound) | 2.92 | **0.78** (11.8 s) |
| `180000+2000` rec3 | 0.41 (5.9 s) | **MAX** (97.9 s) | **0.42** (6.0 s) |

- **The cap is not the fix on its own.**
  - It ends rec3's single expensive window at depth 21, the case the stop
    cannot reach because it is checked only when a window returns.
  - But it takes rec1 deeper sooner, into the window walk that comes when
    the win resolves (depth 26), and that walk runs to the maximum with no
    stop.
- **The cap and the stop cover each other.** Together they end every row
  at or under its optimum. The stop alone ended rec1 and rec3 at
  `180000+2000` at 1.30 × and 1.10 × (RAR-R25).

**Disposition, as frozen, with the probe's correction.**
- The cap goes to its gate: a PGO arm with the cap baked at 1024 against
  `d1`'s head (which has the stop at twice the target), `[-5,5]` with its
  cap, AGENTS' repair case 3.
- The frozen text's probe "with the stop off, to read the search half
  alone" is now read: it fails on rec1. The gate's probe acceptance is
  therefore the cluster as it would ship, the cap with the stop, which
  passes here.
- Registered before any game when the gate is prepared.

**Calibration.**
- `stc` −2.4 inside the predicted −3 ± 12; `ltc` +3.2 inside −1 ± 16; time
  within ±1%, as predicted.
- The research card's trace reading (a deeper rec1 means a shorter stall)
  missed in mechanism. Deeper search reaches the window walk at the win's
  resolution sooner, and only a stop bounds that walk.
