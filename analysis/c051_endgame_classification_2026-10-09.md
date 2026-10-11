# C.5.1 — endgame family classification, deciding instruments, and the opposite-bishop step (2026-10-09)

PLAN C.5.1 (`R2`): adopt the registered family order, confirm each family's
kind against the code, name each family's deciding instrument, and take the
first zero-game step, the opposite-bishop scale (RAR-E26). This packet holds
the derivation. The evaluation read throughout is `master`'s `cc13320`
(13,187,295 / EBF 2.546), unchanged on branch `c4`.

## 1. The order, adopted, and how old it is

The registered order is `tools/diag/endgame_ranking_v2.json`'s: KRPKR, KXK,
KRKN, KRKB, KRPKB, KBPKB, KRKP, KBPKN, KQKR, KPK, KPKP, KQKP, KBNK, KNNKP,
KNNK, then the measure-first families KPsK, KBPsK, KBPPKB, KQKRPs, and
KRPPKRP. It is a prioritiser (occurrence × defect), not an Elo estimate.
C.5.1 adopts it unchanged, for one reason and with one caveat.

- The reason: occurrence comes from game records and the tree, and those
  have not moved enough to reorder families whose priorities differ by a
  factor of two or more. Only the KRKN/KRKB/KRPKB/KBPKB block (0.0051 to
  0.0041) is close enough for a re-read to reorder it, and those four share
  owners (C.5.4, C.5.7) anyway.
- The caveat: its defect columns were read on 2.4.0-era binaries (the drawn
  census v1 and the reference results v1). C.2 refitted the whole surface
  and C.3 rebuilt king safety since then, so every overclaim and conversion
  figure in it is stale in magnitude. Each family cluster (C.5.3 to C.5.7)
  therefore re-reads its own families on the current head, on the
  instrument named below, as its first step, before it builds anything. That
  is the residual-step rule C.4, C.6 and C.7 already follow.

## 2. Each family's kind, checked against the code

The kinds are the ranking's three: **verdict**, which needs an exact or
near-exact answer (win or draw); **scale**, which must stop a drawn family
being scored as winning without suppressing the wins; and **conversion**,
where the score must steer a search to the mate or promotion. Code paths are
`src/eval/endgame/mod.rs` (`scale_endgame`, `specialized_endgame_scale` and
their recognisers) and `mop_up.rs` (`apply_mop_up`). Recognisers fire on
exact material, and the opposite-bishop rule fires on any material with one
bishop each on opposite colours.

| Family | Kind (ranking → confirmed) | What the code does today | Owner |
|---|---|---|---|
| KRPKR | scale → scale | `krpkr_scale`, the reference's case analysis | C.5.5 |
| KXK (KQK, KRK) | verdict → conversion | no recogniser; the general mop-up (edge push, king proximity) above 200 cp | C.5.3 |
| KRKN | scale → scale | nothing: the material score stands (drawn overclaim 100%, mean +346 cp) | C.5.4 |
| KRKB | scale → scale | nothing (overclaim 99.6%, +307 cp) | C.5.4 |
| KRPKB | scale → scale | `krpkb_scale`, rook-pawn fortresses only | C.5.4 |
| KBPKB | scale → scale, two sub-families | opposite bishops: the generic rule, `s/48` with `s = 40` (one pawn, one passer); same colour: nothing | C.5.7 (exact), C.5.2 (the generic rule) |
| KRKP | scale → scale | `krkp_drawish_scale`, a partial scale in the clear draw zone | C.5.5 |
| KBPKN | scale → scale | nothing | C.5.7 |
| KQKR | verdict → conversion | nothing by design (a win); the general mop-up | C.5.3 |
| KPK | scale → verdict | the KPK bitbase: drawn positions are forced to 0, wins fall through | C.5.5 (audit) |
| KPKP | scale → scale | nothing | C.5.5 (audit) |
| KQKP | verdict → verdict with a fortress scale | `kqkp_fortress_scale`, the rook- and bishop-pawn fortress on the seventh | C.5.7 |
| KBNK | verdict → conversion | the minor-mate mop-up to the bishop's corner | C.5.3 |
| KNNKP | scale → scale | nothing (the KNN-K draw needs a bare king) | C.5.7 |
| KNNK | scale → scale | `scale_endgame` returns 0; clean (overclaim 0 of 1,499) | C.5.7, `NO_CHANGE` expected |
| KPsK | — → verdict or scale by pawn count | the KPK bitbase covers one pawn only; nothing for two or more | C.5.6 |
| KBPsK | — → scale | `kbp_wrong_corner_draw` for one rook pawn; nothing for more | C.5.6 |
| KBPPKB | — → scale | opposite bishops: the generic rule never scales it (both pawns are passed, so `s = 48`); same colour: nothing | C.5.6, C.5.2 |
| KQKRPs | — → verdict | nothing | C.5.6 |
| KRPPKRP | — → scale | nothing | C.5.5 / C.5.8 (see §5) |

Two kinds change on the code read, and they change the instrument:

- KPK is a **verdict** family here, not a scale family: the bitbase makes the
  static score exact for draws. Its 4.6% "overclaim" at 60,000 nodes is
  therefore a search-side reading (the score of the line the search picks),
  and a scale change cannot move it. C.5.5's audit reads it as a verdict.
- KXK, KQKR and KBNK are **conversion** families: the code has no verdict
  to get wrong (they are won and scored as won) and their measured defect is
  conversion at a budget (KQKR 23/13/3% failures at 60k/200k/600k).

## 3. The deciding instrument per kind

One instrument decides each family. The others run as vetoes.

| Kind | Deciding instrument | Vetoes |
|---|---|---|
| scale | `endgame_drawn.py`: drawn-cohort overclaim at +100 cp, a real search at 60,000 nodes, 400 generated positions | `endgame_truth.py` win preservation and DTZ progress on the same family's wins; `tests/endgames.rs` (no won position scored as drawn); `endgame_floors.py` |
| verdict | `endgame_truth.py`: theory agreement on Syzygy-labelled positions | `tests/endgames.rs`; the drawn cohort where a draw verdict is involved |
| conversion | `endgame_budget_bracket.py` over `endgame_truth.py`: conversion at 60k/200k/600k nodes, the same cohort | the A.4 game-level audit (`conversion_audit.py`) on the gate's games; floors |
| generic scale (opposite bishops, pawn count, rule 50) | held-out outcome loss on `hce-v4-tb` with the rest fixed (`ocb_scale_screen.py` for the opposite-bishop rule) | the static or search drawn read **and** win preservation on the Syzygy families the rule reaches |

All of this ranks candidates. Strength is C.5.8's (the endgame-start cohort
SPRT and the STC SPRT for the cluster).

A family whose deciding read needs more than six men has no instrument
today: `endgame_truth.parse_family` refuses seven men. §5 is about the one
family where that matters.

## 4. The opposite-bishop step (RAR-E26)

Recorded in full in [`docs/experiments/RAR-E26.md`](../docs/experiments/RAR-E26.md).
In short:

- **The rule today:** `s = min(32 + 4·pawns + 4·passers, 48)` on `/48`, over
  both sides, whenever each side has one bishop on opposite colours. Because
  the pawn term pushes `s` up, a pure bishop ending with four or more pawns is
  not scaled at all (70% of pure rows; the head's mean `s` on them is 47.35).
- **Held-out outcome loss (the frozen read):** the rule refitted inside its
  own form on the 72,904 pure training rows gives `s = 2·pawns + 10·passers`
  and gains **+21.0 ± 1.3%** of the pure cohort's held-out squared error
  (4,097 rows), +0.24% of the whole held-out file. The strong-side-passer
  form adds +0.1 ± 0.4 over it (not selected). With other pieces the refit
  gains +0.075 ± 0.025%, under the registered +0.5% bar, so that cohort
  keeps the head's constants. Rows were 67% draws. The head gives the
  stronger side a mean expected score of 0.735 against an actual 0.649, and
  the candidate gives 0.653.
- **The veto fired (static, Syzygy-labelled generated positions):** in
  KBP-KB the candidate cuts the draws scored above +100 cp from 509 of 704 to
  104, but the wins scored under +100 cp rise from 12 of 253 to 75. KBP-KBP
  shows the same trade (draws 176 → 78 of 752, wins 44 → 78 of 203). In
  KBPP-KB the draws barely move (248 → 220 of 258) and the wins hold (0 → 10
  of 754).
- **What that means:** a factor that sees only pawn and passer counts cannot
  separate won from drawn KBP-KB. It trades one error for the other, and the
  outcome loss favours the trade because in games those endings are draws
  (98.8% of the one-pawn rows). The generated cohort is 26% wins, so it
  weights the other side of the trade far more heavily than games do. The
  default gates play without tablebases (`sprt-default.toml` sets no
  `SyzygyPath`), so static scores at six men or fewer reach the rated games.
- **Disposition by the frozen rule:** the candidate goes to the maintainer
  before C.5.2 builds it. C.5.2 is held on that decision.

## 5. KRPPKRP: the truth exists on this machine

The hold *KRPPKRP 7-man truth gap* resumes when "independent truth becomes
available". It is: `D:\chess\tablebases\syzygy7\KRPPvKRP.rtbw` (23.3 GB) and
`.rtbz` (9.4 GB) have been on this machine since 2026-09-04, and python-chess
1.11.2 probes them (WDL and DTZ on three hand-set positions, beside a 6-man
control through the same opener). No tracked document recorded this, so the
ranking still calls the family unverifiable. What remains is tooling: the
truth, drawn and book tools refuse seven men (`parse_family`), and a 7-man
family needs its own generator constraint, since random KRPP-KRP placements
are mostly not game-like. The family is 5.4% of games, the fourth most
common in the set. The hold's resume condition has fired. Its owner (C.5.5,
or C.5.8's exclusion) now has a measurable family rather than an exclusion
to record.

## 6. What C.5.1's cut supports for C.5.2

- **Opposite bishops:** a candidate exists. It is large at the static layer
  and vetoed on win preservation at six men or fewer, so it is held for the
  maintainer (§4).
- **The rest of C.5.2's list** (pawn-count scaling for the stronger side, the
  rule-50 term in the scale, the complexity term): RAR-E21 measured them as
  directions at seven men or more (+0.09%, +0.03%, +0.01%), and their signal
  is at six men or fewer and low phase, which is the family clusters'
  territory. C.5.1's cut supports none of them for C.5.2 now.
