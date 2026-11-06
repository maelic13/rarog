# RAR-S106 — D.5.5: `Hash` at any size, the repair's speed and harm reads

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-11-06 before any game and any speed
  read. The builds and the speed read need an idle host; the match is the
  maintainer's command.
- **Kind:** AGENTS' repair case 2, with no Elo claim: a correction that moves
  the fingerprint, expected near zero, where a small loss is acceptable for
  correctness. D.5.5 replaced its notice with this repair by maintainer
  decision 2026-11-06, moving the item up from G.2.
- **Baseline / candidate:** `9b8538d` (bench 14,749,043 / EBF 2.564) and
  `1450340` (bench 15,625,852 / EBF 2.563). They differ in `src/tt.rs` and
  two test files only. Each arm builds from a clean worktree at its commit;
  the binaries' hashes are appended here once built, before any game.
- **Research question:** does exact sizing, with a multiply-high index and
  the tag in the key's low 16 bits, cost speed or strength at the size every
  gate uses?
- **Hypothesis / mechanism:** none for strength. `Hash` was floored to a power
  of two (1000 gave 512 MiB). Reckless and Stockfish 19 both size exactly and
  index by the product's high half; Rarog now does the same. At a power-of-two
  `Hash` the table keeps its size and only the slots move.
- **Competing hypotheses:**
  - The low-bit tag weakens verification. Zero-game read against it: six valid
    tag and index choices bench 11.9M to 15.6M around the old 14.75M, 18
    positions up and 17 down with a median ratio of 1.000, while an index
    overlapping the tag reads 53.0M at EBF 2.80. A broken tag is that visible,
    and this change is not.
  - The extra multiply in `probe`, `store` and `prefetch` costs speed. Both
    donors pay it; this read measures it.
- **Interacting mechanisms / consumers:** the table's replacement and
  `hashfull` (unchanged), the shared table's `key16 ^ fold16` verification (the
  same 16 bits of strength), `prefetch` and `key_after_hint` (the same index),
  and every signal that reads a table hit, through collisions only.
- **Exposure:** every table access in every game, so all of them, by
  construction. Gates run at a power-of-two `Hash`, so the games read the
  remapping alone; the larger table a size such as 1000 now gets cannot be read
  from them, and is read by allocation: `Hash 1000` holds 1,042 MiB against 554
  at one thread, 1,163 against 675 at four.

## The reads (frozen)

- **`nps`:** four pext PGO builds an arm, `tools/nps_read.py --no-regression`,
  PROCESS's method. Step 1 passes at −0.1% or above and fails at −0.9% or below;
  in between, step 2. The arms search different trees (+5.9% nodes), so this read
  also carries that tree's mix, and it cannot be separated here.
- **`stc`:** Colosseum `match`, 2,000 games at `3+0.03`, Hash 64, one thread,
  the wrapper's slot count, the UHO book in random order, no adjudication, seed
  20261201, `tools/results/rar-s106-stc`.
- **Commands:** `run_s106.ps1 -Step build|nps|stc` in
  `analysis/artifacts/d55-hash-2026-11-06/`, in that order.
- **Harm rule** (AGENTS' case 2, from the manifest's nElo): below **−13 nElo**
  (2,000 games, standard error about 7.8). This catches a true −10 with
  probability 0.36 and −20 with 0.82, so "kept" means no harm beyond roughly −20
  nElo.

## Dispositions

- **Neither read fails:** D.5.5 closes, the fingerprint stands, and the
  changelog gets its entry.
- **The harm rule fires:** the repair leaves `d5` by the gate, the defect stays
  open in D.5.5, and a rework is a new registration.
- **The speed read fails:** the read goes to the maintainer with the game read
  beside it. The repair is a correctness fix for users, so a small cost is
  theirs to weigh; neither outcome is decided here in advance.

## PRE-REGISTERED PREDICTION (frozen before exposure)

- **`stc`:** 0 ± 15 nElo. P(no harm) 0.95, which is the rule's own false-alarm
  rate at a true zero.
- **`nps`:** −0.2% ± 0.4%, P(pass) 0.75. One multiply replaces one mask, off the
  memory latency that dominates a probe.
- **Most likely failure mode:** the speed read landing between its bounds on
  code placement or the trees' mix, leaving step 2 unresolved rather than failed.

## Binaries (appended before any game)

Built 2026-11-06 on an idle host (guard: 1.7% CPU, no other process), each from
a clean worktree at its commit, rustc 1.99.1, `cargo xtask build --tier pext`.

| Arm | Commit | `id name` | Bench | SHA-256 |
| --- | --- | --- | --- | --- |
| `d55base` | `9b8538d` | `dev-20261106-9b8538d2` | 14,749,043 | `6dcb982fabca08802614f3aa0eb749aa8c85a8b6ea3f0578d6ef03666c5cb786` |
| `d55cand` | `1450340` | `dev-20261106-14503405` | 15,625,852 | `66fd5e0dd28710f1544daea8abea73c12551f67aadb5ed27e5b8b781c1ecac8f` |

Each arm's four pool builds are distinct bytes at its fingerprint. The match's
dry run passed every preflight: runner pin, both fingerprints, build flavour,
compiler and policy.

## Result: `nps` (2026-11-06)

**Pass by step 1: +1.07%, 95% interval −0.01% to +2.16%.** Levels 2,942,532
against 2,974,112, two cycles, CPU 1.3% and 0.9% before them, 1.0% after, no
disturbed cycle; `read/read.txt` in the artifact folder.

The tool warns that arms at different fingerprints do not make an NPS speed
read, as this registration said they would not. Rule 8's alternative does not
do better here: bench nodes over pooled NPS gives 5.01 s against 5.25 s, +4.8%
of time, which is the +5.9% of nodes, and valid mappings alone span 11.9M to
15.6M. So this read rules out a large cost and says nothing finer about the
multiply; the games carry the speed that matters. A same-tree arm could
isolate it, a shift for the multiply at a power-of-two `Hash`, contrary to
"cannot be separated" above. It is not run: no result would change the
disposition, since both donors pay the multiply and the repair is for users.

**Calibration (speed):** a miss in magnitude and sign. The prediction was
−0.2% ± 0.4%, and the read is above that range. The failed assumption is that
the multiply would set the size of the difference; the trees' mix, named only
as a failure mode, was larger than the range allowed. Nothing here is a speed
gain: the interval touches zero and the two arms search different trees.

## Result: `stc` (2026-11-06)

**The harm rule did not fire: nElo +2.1 ± 15.2, against the rule's −13.** Elo
+1.2 ± 8.6, candidate 1003.5 of 2,000, pentanomial 22, 228, 494, 233, 23, with
no time losses and no engine or infrastructure faults. Host 0.3% busy at start,
no waiver; both binaries' hashes in the manifest match the table above.
Manifest: `tools/results/colosseum_match_rar-s106-stc_20261106_211745.manifest.txt`.

This keeps the repair and claims no strength. By the registered power, "kept"
means no harm beyond roughly −20 nElo.

In these games the candidate ran 2.38M nodes a second against 2.34M at 66.5 and
66.7 ms a move. That is a description, not a speed read: the trees still differ.

**Calibration (games):** a hit. The prediction was 0 ± 15 nElo with P(no harm)
0.95, and the read is +2.1. It cannot say more than that: an interval of ±15
would have held any effect this repair was likely to have.

## Disposition (2026-11-06)

**Neither read failed: D.5.5 closes.** The fingerprint 15,625,852 / EBF 2.563
stands, and the changelog has its entry.
