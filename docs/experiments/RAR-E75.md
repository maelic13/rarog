# RAR-E75 — Evaluation candidates beside the donor's form: a scale by ending type, the pawn race, a rook on the seventh, several pieces attacked at once

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** observed 2026-11-26, agent-run, zero games, minutes of CPU. Not registered before its numbers: an observed record with no frozen prediction. It accepts nothing.
- **Kind:** candidates for PLAN E.4.2 and for the families beside the donor's that E.4.4 leaves to their own read. Each is an idea to test, with the form it should take and the read that would kill it. Nothing here is built.
- Disposition: observation — four candidates carried to E.4.2 and E.4.4's follow-ups, two screened terms dropped.
- **Research question:** RAR-E74 found that results on `hce-v4-tb` cannot see the donor's gain with heavy material, yet below 25 pawns of piece material they order the evaluations as play does. Are the evaluations scaled evenly across ending types there, and do a few simple terms carry information that none of Rarog's, `sf_11`'s or `sf_16`'s evaluations holds?
- **Inputs (all existing):** `tools/texel/data/hce-v4-tb/validation.csv` (`3588865C…`; 194,444 rows, white's result, positions of six men or fewer relabelled by Syzygy; games by the B.9 head at 8,000 nodes), its games from `analysis/artifacts/c73/provenance.csv` (`6B1EA515…`), and RAR-E74's dumps of the three static evaluations for every row not in check (`dump-valid-sf11.tsv` `89803A29…`, `dump-valid-sf16.tsv` `C9D52E9A…`).
- **Instrument:** `analysis/artifacts/endgame-scale-candidates-2026-11-26/candidates.py`.
  - *Calibration.* For each evaluation, its logistic slope K by cross-entropy inside a class of positions over its K on all rows. Above 1 the class is under-scaled: an advantage of a given size decides more games there than the same number elsewhere. Game-clustered bootstrap, 400 resamples.
  - *Screen.* `p = sigmoid(k·s/400 + side-to-move terms + Σ w·f)`, fitted on the games with an even index (96,928 rows) and read on the others (97,516 rows, 14,884 games). A test's gain is the report half's cross-entropy with the term over without it, in percent, negative when the term adds information; bootstrap over the report half's games, 1,000 resamples. A term enters as `f·phase` and `f·(1 − phase)`. Coefficients are in Rarog centipawns (logit over Rarog's slope), with a game-clustered standard error.
- **Wires:** each K reproduces RAR-E74's to 1e-6 (1.7857, 1.1071, 1.1710). A random sign gains +0.001 [−0.002, +0.005]. A second evaluation added gains −1.192 [−1.339, −1.040] (`sf_16` to Rarog's) and −1.085 [−1.243, −0.936] (Rarog's to `sf_11`'s).
- **Limits:** cross-entropy on results is not Elo and has no exchange rate to it (RAR-X03). The rows are quiet positions from one engine's games at 8,000 nodes, so a term that acts on tactical positions is invisible here. `sf_11`'s numbers describe the bridge's evaluation, not the port E.4.4.1 will write: every seed is refitted on the port's own dump.

## Result

**Calibration, K in the class over K on all rows [95%]:**

| Class | Rows | Rarog | `sf_11` | `sf_16` |
|---|---|---|---|---|
| Pawn endings | 6,359 | 1.47 [1.40, 1.56] | 1.76 [1.65, 1.90] | 1.75 [1.64, 1.88] |
| One bishop each, opposite colours, nothing else | 4,121 | 0.71 [0.65, 0.79] | 1.90 [1.74, 2.06] | 1.68 [1.54, 1.83] |
| Opposite bishops with other pieces | 10,465 | 0.93 [0.87, 0.99] | 1.01 [0.95, 1.08] | 0.99 [0.93, 1.05] |
| One knight each or same-coloured bishops, nothing else | 6,605 | 0.99 [0.94, 1.04] | 1.34 [1.26, 1.43] | 1.35 [1.27, 1.43] |
| Rook endings | 14,742 | 1.14 [1.10, 1.17] | 1.27 [1.22, 1.33] | 1.30 [1.25, 1.37] |
| Queen endings | 7,791 | 1.05 [1.00, 1.10] | 0.83 [0.75, 0.91] | 1.02 [0.95, 1.08] |
| Other minor-piece endings | 17,742 | 1.02 [0.99, 1.05] | 1.37 [1.32, 1.43] | 1.45 [1.40, 1.50] |
| Non-pawn material 25 to 44 | 54,031 | 0.97 [0.94, 0.99] | 0.87 [0.85, 0.89] | 0.84 [0.82, 0.86] |
| Non-pawn material 45 or more | 43,915 | 0.78 [0.75, 0.82] | 0.66 [0.63, 0.69] | 0.63 [0.60, 0.66] |

Rarog's evaluation is even across ending types except three: pawn endings are under-scaled by about half, rook endings by about an eighth, and pure opposite-bishop endings are over-scaled by about 40%. The donors sit at 1.3 to 1.4 in every light-material class and at 0.6 to 0.9 with heavy material, a slope by phase that is E.4.5's question; against that level their pawn endings are under-scaled as well (1.76) and their pure opposite-bishop endings are damped too far (1.90, 1.68).

**Screen, held-out gain in percent [95%]:**

| Test | Share of rows | On Rarog's | On `sf_11`'s | On `sf_16`'s |
|---|---|---|---|---|
| A1 pawn endings scaled | 3.3% | −0.100 [−0.140, −0.054], ×1.52 ± 0.08 | −0.188 [−0.234, −0.136], ×1.78 ± 0.11 | −0.178 [−0.224, −0.128], ×1.74 ± 0.11 |
| A2 pure opposite bishops scaled | 2.1% | −0.023 [−0.056, +0.007], ×0.67 ± 0.05 | −0.082 [−0.114, −0.052], ×1.85 ± 0.11 | −0.054 [−0.082, −0.027], ×1.64 ± 0.11 |
| A3 rook endings scaled | 7.6% | −0.040 [−0.057, −0.022], ×1.13 ± 0.03 | −0.070 [−0.099, −0.040], ×1.27 ± 0.04 | −0.084 [−0.114, −0.049], ×1.28 ± 0.04 |
| A the three together | | −0.166 [−0.219, −0.108] | −0.359 [−0.429, −0.292] | −0.333 [−0.402, −0.264] |
| A beside a slope that varies with the phase | | −0.090 [−0.142, −0.039]; ×1.28 ± 0.07, ×0.60 ± 0.04, ×1.01 ± 0.03 | −0.058 [−0.082, −0.032]; ×1.19 ± 0.08, ×1.33 ± 0.08, ×0.97 ± 0.03 | −0.020 [−0.036, −0.003]; ×1.12 ± 0.07, ×1.14 ± 0.08, ×0.95 ± 0.03 |
| B pawn race lead | 1.1% | −0.076 [−0.118, −0.036], +102 ± 13 a rank | −0.088 [−0.126, −0.050], +100 ± 13 | −0.087 [−0.125, −0.050], +100 ± 13 |
| B beside A1 | | −0.030 [−0.068, +0.007], +87 ± 17 | −0.032 [−0.062, −0.002], +81 ± 17 | −0.034 [−0.064, −0.004], +83 ± 17 |
| C rook on the seventh when it bites, alone | 6.9% | −0.015 [−0.027, −0.004]; mg −39 ± 26, eg +31 ± 16 | −0.013 [−0.023, −0.003]; mg −34 ± 27, eg +36 ± 16 | −0.015 [−0.027, −0.004]; mg −38 ± 28, eg +39 ± 16 |
| C beside any rook on the seventh | | −0.006 [−0.013, +0.001] | −0.004 [−0.011, +0.002] | −0.004 [−0.009, +0.002] |
| D pieces attacked by a lesser piece beyond the first, beside their count | 0.3% | −0.002 [−0.016, +0.013]; mg +63 ± 51, eg +275 ± 136 | −0.005 [−0.019, +0.011] | −0.005 [−0.021, +0.011] |
| Rook mobility through pieces, beside its mobility | 64.7% | +0.010 [−0.006, +0.026] | +0.008 [−0.003, +0.019] | +0.007 [−0.003, +0.017] |
| An offset for each uneven piece configuration (25 with 400+ fit rows) | 2.2% | +0.009 [−0.100, +0.160] | +0.067 [−0.039, +0.197] | +0.018 [−0.094, +0.163] |

Part of A is the slope by phase and not the class: once the base's slope may vary with the phase (the row above; multipliers in the order pawn endings, pure opposite bishops, rook endings), the rook-ending multiplier is 1 on every base, the pawn-ending one falls to 1.28 on Rarog's and 1.19 on `sf_11`'s, and the pure opposite-bishop one stays (0.60 on Rarog's, 1.33 on `sf_11`'s).

A finer multiplier for each of 52 piece configurations gains −0.230 [−0.372, −0.065] on Rarog's, no better resolved than the three classes; on the donors it gains more (−0.855, −0.965), which is again their slope by phase.

**Dropped:** rook mobility counted through pieces, and an offset by piece configuration beside the evaluations' own imbalance. Neither gains on any base.

## Candidates

Every seed below is this record's fit on the evaluation named and is refitted on the evaluation in use when the candidate is read. Each is a playing change: it is registered with a frozen prediction and gated as `docs/rules/gating.md` requires.

### A. A scale by ending type

- **Form:** one multiplier on the final score for each of three classes, chosen by the non-pawn material alone: no pieces; one bishop each on opposite colours and nothing else; rooks only. It applies after the specialised endgames and scale factors, which keep precedence, and never to a mate, tablebase or recognised-draw score. Seeds on Rarog's evaluation as it is: 1.5, 0.7 and 1.13; beside a refitted slope by phase the rook class drops out and the seeds are 1.3 and 0.6. On the port the seeds are refitted against its own endgame slope (this record reads 1.2 for pawn endings and 1.3 for pure opposite bishops on the bridge), since `sf_11` already carries `51` for a pawn ending in its initiative term and its own opposite-bishop factor; Rarog's initiative term dropped the first. D.6.9's question in E.4.2, whole score or endgame half, is the same decision and is settled with it.
- **Mechanism:** the decision to trade into or out of a pawn ending, a rook ending or a pure opposite-bishop ending. Inside a class a multiplier changes no ordering; it changes the comparison between a leaf in the class and a leaf outside it. Active on 13% of validation rows.
- **Interactions:** the passed-pawn and unstoppable-pawn terms are scaled with the score, so B is fitted beside A, never before it. A capture that enters a class moves the static evaluation by the multiplier, which the kernel's static-evaluation margins and `LazyMargin` see (PROCESS, *Evaluation change under a fitted search*). Tablebases remove most of the effect at seven men or fewer, so the rated configuration with tables sees less than a pool without.
- **Invariants:** colour-flip symmetry; a recognised draw stays zero; the scaled score stays below the tablebase range; the evaluation cache's key covers the material that selects the class.
- **Falsifier:** E.4.2's Syzygy screen (RAR-E36's method) on the kernel's own game positions. The scaled evaluation must not raise the share of drawn pawn endings and drawn rook endings scored past the overclaim threshold by more than the won ones gain; if it does, the multiplier belongs only above a score threshold or not at all. Then the gate. Stop: a gate at H0 closes it until the evaluation changes again.

### B. The pawn race

- **Form:** in pawn endings only. For each side, find its passed pawn nearest to queening that cannot be stopped: its path is clear, and either the enemy king is outside its square counting the move, or its own king guards every square left on the path and the pawn cannot simply be taken. Count the plies that pawn needs (a pawn on its second rank steps twice; the side to move is one ply ahead). The term is the difference between the two sides' counts, in ranks, times one weight. It replaces `unstoppable_passer_eg`, which scores each pawn alone, ignores the opponent's runner and has no king escort. Seed: about 100 a rank alone, about 85 beside A1. `candidates.py`, `race`, is the reference definition.
- **Mechanism:** who queens first, read statically at a pawn-ending leaf instead of ten plies deeper. Active on 1.1% of rows.
- **Interactions:** A1 takes part of its information (the gain falls from −0.076 to −0.030 beside it). The search resolves a race two plies a rank, and tables resolve it outright at seven men or fewer. The side to move enters, so the evaluation cache must key on it.
- **Invariants:** colour-flip symmetry with the side to move flipped; zero when neither side has such a pawn; never against the KPK bitbase; bounded below a queen.
- **Falsifier:** zero games. On seven-man-or-fewer pawn endings from the kernel's games, the term's sign must agree with Syzygy's result more often than the present term's does, by a margin frozen at registration. No better: closed.

### C. A rook on the seventh in the endgame

- **Form:** an endgame-half bonus for a rook on its seventh rank while an enemy pawn stands on that rank or the enemy king is on its back rank. No middlegame weight. Seed: 30 to 40 on the endgame half.
- **Evidence:** small and the same on all three bases; the condition itself is not resolved beside a plain rook on the seventh (−0.006 [−0.013, +0.001]).
- **Interactions:** the rook's piece-square values on that rank, and the rook-behind-passer terms.
- **Falsifier:** re-screened on the port's dump. A coefficient within one standard error of zero there means the port's tables hold it: closed. By Phase H's rule it is a small gain and waits unless E.4.4.3's sweep clusters it.

### D. Several pieces attacked at once

- **Form:** take a side's pieces attacked by a lesser piece (a minor by a pawn, a rook by a pawn or minor, a queen by a pawn, minor or rook; the population `threat_weak_piece` already counts). Add a bonus to the attacker for each such piece beyond the first, since one move saves one piece. A variant splits it by the side to move.
- **Evidence:** none either way. Quiet rows hold it on 0.3% of positions and the read cannot see it.
- **The read it needs:** the kernel's own leaves. A probe build counts how often two or more pieces are attacked this way where the static evaluation decides a node (stand pat, the static-evaluation prunings), and the direction quantity is the static evaluation's error against a deeper search on those leaves, with and without the term.
- **Interactions:** quiescence and SEE resolve the captures that follow; the linear threat terms and the hanging term score the same pieces once each.
- **Falsifier:** activation under a threshold frozen at registration, or no reduction of that error: closed without a game.

- **Conditional lesson:** on this corpus a multiplier by ending type is the largest single piece of static information the three evaluations lack, and it is small beside the donor's gain (−0.17 against −1.19 for a whole second evaluation).
- **Retry trigger:** a candidate is reread when the evaluation it was fitted on is replaced (E.4.4.1), and not before.
- **Artifacts / commits:** `analysis/artifacts/endgame-scale-candidates-2026-11-26/` (ignored): `candidates.py` (`6DF0AD7A…`), `candidates.log` (`7CE5D241…`), `result.json` (`72908136…`).
