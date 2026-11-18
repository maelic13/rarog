# RAR-S127 — The compact kernel's trees in won endings with a mate in view: the donor's line shape, not a port defect — READ 2026-11-18, zero games

Indexed under *3. Search and selectivity* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** 2026-11-18, the agent's, at the maintainer's request
  ("diagnose mates first") after RAR-S126 and before D.7.6's registration.
- **Kind:** a zero-game diagnosis of D.7.4.1's open item, position 202's
  class (`analysis/d741_fold_2026-11-18.md`).
- **Disposition:** no port defect found. The trees grow because lines run
  to whatever ply cap exists once a mate is in view; ordinary lines are the
  donor's length; the one containment, Rarog's line bound, reshapes the whole
  kernel (bench −36%). Nothing is changed before D.7.6; the item stays in
  D.7.8 with this evidence.
- **The question:** why the compact kernel, before and after the fold, grows
  trees one to two orders of magnitude past its donor's in won KBP-against-K
  endings, so that an iteration does not finish (202 at depth 14: stopped at
  590 s after about 1.5 billion nodes).
- **Candidates named before any test** (PLAN D.7.8, 2026-11-18): the
  placement of Rarog's decisive band (`MAX_PLY` 128, decisive from 31,744;
  the donor's 240 and 31,519), and the donor's downgrade of tablebase scores,
  which Rarog's table lacks.

## Method

Positions: 202 (`6B1/8/5k2/8/4pK2/4P3/8/8 b - - 4 77`) and one from its
line (`B7/5k2/8/8/5K2/4P3/8/8 b - - 6 83`, "KBPK"). One thread, Hash 64, a
fresh process a run, 20 s cap. Diagnostic builds of the compact kernel at
`2820eaa`, never committed, each switch read from the environment and off
by default (bench 7,543,485 reproduced with every switch off):

- `RAROG_DIAG_LINE`: a non-root node's depth held to `2·root_depth − ply`,
  the classic kernel's line bound (`node.rs`, `bound_line_depth`);
- `RAROG_DIAG_NODOWN`: `score_from_tt` reads a stored mate exactly, without
  the fifty-move downgrade;
- `RAROG_DIAG_NO_EXT2`, `_NO_LMR_DEEP`, `_NO_RESEARCH_UP`, `_NO_HS_UP`: the
  singular extension capped at one ply, LMR's reduced depth capped at the
  new depth, the deeper re-search and hindsight's extra ply removed;
- `RAROG_DIAG_PVSEL`: seldepth counted as the donor counts it, the deepest
  PV ply;
- a second build with `MAX_PLY` 240 in the search, the table and the mop-up
  mirror (bench 7,551,778), which also puts Rarog's decisive band at 31,520.

Patches, drivers and logs: `analysis/artifacts/d741-mates-2026-11-18/`
(`pdiagmates.py`, `pdiagmates2.py`, `arms.sh`, `arms*.log`,
`seldepth_cmp.py`, `seldepth_cmp.log`).

## Results

**The trigger.** On KBPK the compact root's first depth-16 search finds a
real mate (mate −44) and fails low on it; the next windows near the mate
get back the downgraded ordinary value, the window swings between the two
sixteen times, and the last search, fully open, runs lines to the ply cap
(seldepth 128): 8.1 million nodes, 77 million at depth 17 unfinished. The
classic kernel finds mate −37 at depth 15 and keeps it (seldepth 34 to 48 to
depth 22). The twin finds mate −19 at depth 20 in 570,796 nodes.

| Arm | KBPK | 202 |
|---|---|---|
| As built | depth 16, 8.1M; depth 17 unfinished at 77M | depth 13; depth 14 unfinished |
| Line bound | depth 22 in 485k | depth 20 in 767k |
| No downgrade | depth 19 in 4.1M, seldepth 128 | mate −26 at depth 19 in 32M, seldepth 128 |
| Both | depth 22 in 508k | depth 20 in 734k |
| Singular extension ≤ 1 | depth 22 in 5.3M | depth 20 in 902k |
| LMR not deeper | depth 22 in 500k | depth 20 in 866k |
| No deeper re-search | depth 22 in 732k | depth 20 in 63.5M |
| No hindsight ply | depth 16 in 3.1M; 17 unfinished | depth 20 in 810k |
| `MAX_PLY` 240 | mate −15 at depth 22 in 76M, seldepth to 218 | depth 15 unfinished at 62M, seldepth 188 |
| `MAX_PLY` 240, line bound | depth 22 in 546k | depth 20 in 767k |

- **Both named candidates are refuted.** With the cap and band at the
  donor's (240, 31,520) the trees still explode and the lines run to 188 to
  224 plies; no tablebase is in play. Removing the downgrade does not stop
  the explosion either; it only removes the swinging window.
- **No single deepener is the cause.** Each ablation changes the whole tree:
  three rescue KBPK, and one makes 202 worse (63.5M). Only the line bound
  contains both positions under both caps.
- **Ordinary lines are the donor's length.** On the first 100 parity
  positions at depth 14, counted in the donor's unit: PV seldepth mean 23.08
  compact against 22.96 twin, median 23 against 24, maximum 42 against 40,
  17 against 17 past twice the depth; compact deeper at 41, equal at 14,
  shallower at 45.
- **The donor has the class too:** the twin needs 34.9 million nodes (25 s)
  on 202 at depth 18 before it reaches mate −20, from 1.2 million at depth
  17.
- **The line bound is not a narrow repair:** `bench 13` falls from 7,543,485
  to 4,804,671 (−36%, EBF 2.390), so it binds across ordinary positions and
  would change the kernel that D.7.6 fits.
- **Games:** RAR-S126 shows no cost: the band appears in 301 of the compact
  arm's games, 300 won, and it converts winning scores better than the
  classic arm (98.1% against 96.2%).

## Reading

The compact kernel's line growth is its donor's shape: without a line bound,
a won ending with a mate in view drives lines to the ply cap and the tree
explodes, in the donor too, in different positions because the trees
differ. Rarog's fifty-move downgrade adds a swinging window, not the
explosion. Rarog's line bound contains it but is a shape change across the
kernel, which D.7's rule tries again only on the adopted kernel, with its own
gate (D.7.9), and which D.7.8 already lists ("the line bound and the
extension budget stay only if the chains return"). The chains have returned
in won endings; in games they cost nothing visible unfitted. Retry trigger: a
fitted kernel whose games show a conversion loss in won endings, or the
D.7.9 retry of Rarog's additions.

**Evidence layer:** node counts and line lengths on two positions and 100
parity positions; not Elo.

## Correction, 2026-11-18 (RAR-S128's activation read)

The reading above took the class to be confined to won endings and costless
in games, and so changed nothing before the fit. RAR-S128's activation read
found it far more fragile: a perturbation of two steps of almost any seed,
up or down, made bench 16, a won pawn ending, run from 43 thousand nodes to
about a billion, in 106 of 482 readings. The diagnosis above stands (the
donor's line shape, no port defect); its use does not. A bound of four
times the iteration's depth stops every case tried and binds no tree of the
bench (four times failed the KBNK anchor), and the maintainer decided to
carry a bound before the fit (`f7c89fb`). Rarog's own bound of twice the
depth stays D.7.9's question.
