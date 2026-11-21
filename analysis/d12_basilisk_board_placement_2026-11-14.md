# D.12: Basilisk's board findings placed against Rarog's records (2026-11-14)

Research only; nothing was built, timed or changed in the engine. The source
is the import `analysis/basilisk_board_handoff_2026-11-14.md` (Basilisk
`21ce21c`); its section numbers are used below. Every figure from it is a
prior from Basilisk's engine and host. Rarog statements were read in source
at `basilisk-check` (engine source as released in 2.7.0) and in the records
cited. The leaves this produced are PLAN D.12.1.1, D.12.1.2, D.12.2.1 and
D.12.2.2. Before any of them runs, D.12.0 maps Rarog's own board
interactions and ranks Rarog's options from its own profile, these four
among them, and may keep, reshape or drop each (maintainer, 2026-11-14:
Basilisk's results are hints, not the best answer for Rarog).

## What the import changes in D.12's existing text

- **D.12's opening figure was mis-cited.** "Board work is 24% of the time
  (RAR-M48)": RAR-M48 is the 2.4.0 NPS baseline and holds no share. The
  latest profile is C.12's head (`analysis/c12_speed_review_2026-10-18.md`):
  generation 9.6%, SEE 7.1%, make/unmake 4.9%, about 22% together; corrected
  in PLAN.
- **Lead 2 is corrected.** It said Basilisk's B.7.3 copies Rarog's hinted
  make. Basilisk's frontier review found a hinted make no gain on its own
  replay, in a search that knows `gives_check` before only 43% of its
  makes; it was not built into Basilisk's engine. The −0.74% (make derives
  the checkers from the move) is another mechanism. A first correction here
  over-read this as superseding the hint; Basilisk's note of 2026-11-14
  corrected it, and PLAN D.12.2's lead 2 now holds the rule: keep the hint
  while Rarog's search already pays for the answer on nearly every make,
  and count that precondition (D.12.0) before deciding anything about
  make.
- **Lead 6 is superseded.** It said stored check information loses at
  Basilisk's mix (65.8 against 71.7 ns). The later review reads it as a wash:
  against classical Stockfish, cheaper queries save 17.0 ns a node and the
  dearer make costs 16.8.

## Placement, finding by finding

Have: Rarog already holds the result or the mechanism. Differs: Rarog
measured something that disagrees or qualifies. New: nothing in Rarog's
records covers it.

| § | Finding | Place | Against Rarog's records |
|---|---|---|---|
| 1 | Five boards under one contract; Rarog level with Basilisk at Basilisk's mix | Have | D.12.2's leads hold the same table. D.12.1 already plans Rarog's own call mix and the fast `gives_check_with`; Basilisk's mix is not Rarog's (RAR-M34 at the 2026-09 head: 0.99 SEE and 3.36 fast `gives_check` a node, against Basilisk's 1.42 and 1.44) |
| 1 | Make/unmake 15% ahead, from the hint | Have | Lead 2 (corrected above) |
| 1 | Generation 29% and perft 25% behind | Have | Lead 1; RAR-M44(c): a 12.7% generator gain read −0.55% pooled |
| 1 | After Basilisk's two changes, 47.6 against 54.6 ns a node | New, a prior | A non-PGO projection on Basilisk's mix; read only through D.12.2.1 and D.12.2.2 |
| 2 | The microbenchmark times game positions, uniform moves and SEE at 0 | Have, in part | RAR-M44 found the cross-engine columns timing different work (the 520-byte copy). The search's in-check rates and SEE threshold mix are not counted in Rarog: D.12.1.1's trace gives them |
| 2 | The search replay, with a verify mode and a stub control | **New** | Nothing in Rarog replays the search's board calls. The nearest are D.4.3's search-path perft (exhaustive, not the search's order), `tests/board_differential.rs` (random walks) and the `diag` counters (counts, no order). D.12.1.1 |
| 2 | The engine screen: alternating `bench`, one pinned CPU, 16 rounds, the per-round ratio, about ±0.3% in five minutes | **Differs** | C.12 ran the same shape (`analysis/artifacts/c12/quickscreen.py`, non-PGO, ABBA, about ±0.4%) and recorded where it fails: it put `081041c` at +0.1% where the pooled read gave +4.24%, and the pick-time prefetch at −0.04% where PGO gave +0.5%; a two-build PGO screen read +1.06% for a change the pooled read put at +0.02%. Basilisk's agreement is the screen against its own replay, in direction; its pooled read of the two kept kernels is still pending (frozen there at +2.0%, [+0.8, +3.2]). D.12.1.1 rebuilds the screen and calibrates it on C.12's pairs |
| 3 | Pre-tests that skip a lookup gain nothing or lose; the gains removed a whole path | Have | C.12's SEE legality superset pre-test +0.08% (SEE microbench unchanged), its SEE fast path −0.86% PGO, threat-set king legality −0.17% PGO; RAR-M44(c): two removed checks made the generator slower |
| 3 | Generator pins from aligned sliders: −0.5 ns, within noise | Have | RAR-M31 (withdrawn, no whole-search gain) and C.12's sniper pins (−0.66% PGO). Closed; no work |
| 3 | SEE attackers from aligned sliders: a loss | Have | RAR-M35 (carried attacker set, −1% to −3% on its benchmark) and C.12's attackers-once (−0.31% PGO) |
| 3 | `gives_check` pre-test and lookup-free forms: no gain | Have, in kind | Rarog's search already uses the fast `gives_check_with(CheckInfo)`; no Rarog form untried that Basilisk found to pay. No work |
| 3 | Make derives the checkers from the move: −0.74% | New, a negative prior | Not tried in Rarog; no work |
| 3, 4 | SEE pins read from aligned sliders: +1.56% non-PGO | **New, with a contract difference** | Rarog has no pin array to remove: `see_recapturer` tests each selected candidate with a full `attackers_to_color` on its king (`src/board/board.rs`). Basilisk's form excludes pinned pieces only; Rarog's test also refuses a recapture while the side is in check from a slider the previous capture uncovered. The form exact for Rarog is one condition (D.12.2.1). Its size is bounded by RAR-M43: the legality test added in 4.11b.5 cost 8.07% of the SEE microbenchmark, and SEE is 7.1% of the profile |
| 3, 4 | Unmake without discarded key work: +1.86% non-PGO | **Differs** | Rarog's unmake already restores the main key from its record and discards nothing. It recomputes the four derived keys (pawn, minor, two non-pawn) with one Zobrist load per piece change. The question is the trade, 32 more bytes a ply (24 → 56) against those loads, and the `UnmakeInfo <= 24` guard (RAR-M39). Triumviratus's TB-5 (a 64-byte copy at make, −0.61%) is a second prior. D.12.2.2 |
| 5 | 20.6% of SEE calls repeat a move at the node; 10.2% are settled by an earlier answer; 8.7% of capture generations repeat (ProbCut, then the picker) | New counts | Rarog asks repeatedly too: the picker's good-capture split, then the node's pruning threshold, and in quiescence `see_ge(0)` at scoring, then the threshold, then the bad-capture floor; ProbCut generates the noisy moves and the main picker generates captures again. Monotonicity is proved here: C.12's `seecheck` found `see_ge ≡ see ≥ t` on 172,081 moves × 161 thresholds. C.12's one-full-SEE-per-move form read −0.24%. D.12.1.2 |
| 6 | Check and pin information stored at make: a wash | Have | RAR-M34 (`NO_CHANGE` on structure); C.12's check information shared with quiet scoring −1.6%; TB-4 |
| 6 | Compressed slider tables: −0.98% | Have | C.12's 16-bit cells by PDEP −1.8%; F.5's record of TB-7 at −0.98% |
| 6 | Attack tables kept incrementally | Have, in kind | Rarog builds its `Threats` once a node; nothing incremental. No work |
| 6 | A byte mailbox read by vector permutes (AVX-512 VBMI) | Have, in part | Rarog's mailbox is already one byte a square; no AVX-512 tier before F.5. No work |
| 6 | Pseudo-legal generation with a late legality test | Have | RAR-M39: fast to full check calls 517:1; the legal generator stays |
| 6 | Side or stage as a template parameter | Have | RAR-M44(c), −0.55% pooled |
| 6 | Open in Basilisk: undo as one block, keys by table in make, next attacker by index, one enemy attack map a node | Have or listed | Undo as one block is D.12.2.2's question at its largest; keys by mask in make are `acd767e` (C.12); next attacker by index is TB-6, already in D.12.2; one enemy attack map is Rarog's `Threats` |
| 7 | Make writes nothing for a network; the ply's triple (move, mover, captured) is the delta | New, a prior | D.12.2 fixes the dirty-piece contract for F.1 (TB-10); this is one design it must weigh, with Triumviratus's (make records only the moved piece) |

## What is worth doing

The replay (D.12.1.1) is the import's real contribution: an exact check of a
board rewrite on the search's own operations, which D.12.3 needs whatever
D.12.2 decides, and a price in real search order. The two counts (D.12.1.2)
cost nothing once it exists. The two kernels (D.12.2.1, D.12.2.2) are
expected near or under the pooled read's resolution in Rarog, for the reasons
in their rows, so each stops at the replay unless it clears a floor
registered before it is timed.
