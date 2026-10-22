# RAR-M75 — The wider pool at 1T: a round robin of the C head, two held anchors and six stronger or new engines, with a CCRL-equivalent reading

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-22, before any game. The maintainer's run (requested 2026-10-22: the old pool "seems exceeded"; a field with engines stronger than the C head), through `analysis/artifacts/pool2/run.ps1`.
- **Kind:** an observation on the held scale, never a gate. It places the C head among stronger engines and gives the scale's link to CCRL.
- **Why a round robin.** In a gauntlet only the newcomer is fitted, and four of the new engines have no held rating; a round robin fits every engine but the two anchors, so the head and the new engines are placed on the held scale together.
- **Field** (inventory and CCRL lookup of 2026-10-22: every engine in `D:\chess\engines` handshaken with `uci` only, CCRL's full text lists of 2026-10-16/17; `C:\…\scratchpad\engines\` of that session, summarised here):

  | # | Engine | Binary (SHA-256 prefix) | Role | Prior / held | CCRL Blitz 1CPU | CCRL 40/15 1CPU |
  |---|---|---|---|---|---|---|
  | 1 | Rarog C.11 head (`2.6.0-dev+c11head`) | `rarog-c11head-pext-pgo.exe` (`9750D3DE`) | fitted | 3429 (RAR-M73) | — | — |
  | 2 | Houdini 3 Pro | `houdini\Houdini_3_Pro_x64.exe` (`b0e7a631`) | **anchor, fixed 3287** | RAR-M57 | not listed | 3186 |
  | 3 | Critter 1.6a | `Critter_1.6a_64bit.exe` (`a8988ac9`) | **anchor, fixed 3192** | RAR-M57 | 3164 | 3137 |
  | 4 | Houdini 4 Pro | `houdini\Houdini_4_Pro_x64B-v1x.exe` (`a7363131`) | fitted; a check of the scale | 3310 (RAR-M57's value; that binary's hash was not recorded, so it is not fixed) | not listed | 3210 |
  | 5 | Komodo 10.4 | `komodo\komodo-10.4-64bit.exe` (`63d8593e`) | fitted, about level | 3420 | not listed | 3359 |
  | 6 | Houdini 6.03 Pro pext | `houdini\Houdini_6.03_Pro_pext_x64.exe` (`e5c0fdc5`) | fitted, stronger | 3460 | 3468 (Houdini 6) | 3389 |
  | 7 | Komodo 13.3 bmi2 | `komodo\komodo-13.3-64bit-bmi2.exe` (`5cdebfc8`) | fitted, stronger | 3478 | 3471 | 3402 |
  | 8 | Stockfish 11 bmi2 | `stockfish\stockfish-11-x64-bmi2.exe` (`271b62dc`) | fitted, clearly stronger | 3600 | 3563 | 3493 |
  | 9 | Dragon 1 (avx2) | `dragon\dragon-64bit-avx2.exe` (`672e2ec4`) | fitted, clearly stronger | 3648 | 3613 | 3527 |

  Left out: Dragon 2.6.1 and 3.3, Reckless 0.9.0 and Stockfish 18/19 (head expected under 20%, little information; Dragon 3.3 also prints a foreign licence string), Lc0 (CUDA only), the WinBoard-only programs, and engines well below the head.
- **Conditions: RAR-M73's,** except the format and the field: `tools/colosseum.ps1 -Mode gauntlet -Seed 44 -Hash 128 -MarginMs 2000` with `--format round-robin --games-per-pair 2 --cycles 150` through `analysis/artifacts/pool2/round_robin.ps1`: 36 pairs, **300 games a pair, 10,800 games**, about 2 hours; `3+0.03`, margin 2,000 ms, Hash 128 for every engine, one thread each set explicitly under each engine's own option (`Threads=1`; Houdini 3/4 and Critter default to 32), UHO_Lichess_4852_v1 at seed 44, no adjudication, 14 concurrent games, runner the pinned Colosseum 0.3.0. Dry run 2026-10-22 09:27, exit 0, at policy: the plan's `fixed_ratings` hold exactly participants 2 (3287) and 3 (3192), every participant carries `Hash=128` and `Threads=1`, the format is a round robin of 150 cycles; `dry-run.json` SHA-256 **`D42B868428274CE15C687544718F12C1C3E831E4640102CE8EB3A7B7F283143B`** (stable over two dry runs), which the run's manifest must repeat. The wrapper's `-SelfTest` proved the idle wait, the live stream and both fault paths.
- **Reading:** Colosseum's standings (every engine's rating with its interval; trusted as reported) and crosstable; for the head, each pair's W-D-L, score and Elo with a trinomial interval (computed: a named gap in Colosseum 0.4.0); terminations and faults by engine (computed, gaps); in-game NPS and depth from Colosseum CLI 0.4.0's `stats` (trusted). **CCRL-equivalent:** a least-squares line held = a + b · CCRL over this run's fitted and fixed ratings of the engines CCRL lists, once for Blitz 1CPU (Critter, Houdini 6, Komodo 13.3, Stockfish 11, Dragon 1) and once for 40/15 1CPU (all eight), and the head's CCRL-equivalent by inverting it, with the residual spread as its error. Houdini 4's fitted rating against its 3310 says whether RAR-M57's held value still describes this binary.
- **Fault rule (RAR-M74's):** an opponent's faults are read when they are under 1% of its games, reported by engine and scored as the harness scored them; any fault of the C head voids the run (repeated, not read). The wrapper continues past `colosseum.ps1`'s zero-tolerance exit when `result.json` exists, and the faults are read here.
- **PRE-REGISTERED PREDICTION (frozen 2026-10-22, before any game):**
  - *The head:* **3,425**, 80% band **[3,385, 3,465]** (RAR-M73 read 3,428.8 against the old field; here part of the evidence runs through engines whose ratings are fitted in the same run).
  - *The new engines* (held scale, 80% bands): Houdini 4 3,310 [3,260, 3,360]; Komodo 10.4 3,420 [3,340, 3,500]; Houdini 6.03 3,460 [3,380, 3,540]; Komodo 13.3 3,478 [3,400, 3,560]; Stockfish 11 3,600 [3,500, 3,700]; Dragon 1 3,648 [3,540, 3,760]. Basis: the inventory's fit of held ratings against CCRL (Blitz slope 1.19, residual SD 48; 40/15 slope 1.32, residual SD 90), engine families differing by about ±100.
  - *Engines stronger than the head:* four (Houdini 6.03, Komodo 13.3, Stockfish 11, Dragon 1), Komodo 10.4 about level (probability it rates above the head 0.4).
  - *The head's CCRL-equivalent:* Blitz 1CPU **3,400**, 80% band [3,340, 3,460]; 40/15 1CPU **3,360** [3,300, 3,420].
  - Most likely failure mode: the new engines' family offsets (±100 against CCRL) make the line through five Blitz points loose; the CCRL-equivalent's error, not its centre, is then the finding.
- **Stop rule:** one run of 10,800 games, no early stop, no extension; a run voided by a fault of the C head is repeated, not read.
- **Use:** GUIDE's checkpoint gains a row "Pool position, wider field" and the head's CCRL-equivalent; the old pool rows stay as they are. It changes no default and accepts nothing.
- **Result:** not played.
- **Artifacts:** `analysis/artifacts/pool2/` (driver, wrapper, dry-run logs); results `tools/results/pool2-rr/`.
