# The hand-crafted evaluation's donor: the evidence, 2026-11-22

- **For:** the research sub-step that opens Rarog's evaluation programme
  in Phase E (PLAN E.4.0): which classical Stockfish evaluation is the donor,
  and whether Rarog's evaluation is rebuilt in its form.
- **Kind:** a record of evidence, most of it Basilisk's, copied here so that
  it does not depend on Basilisk's repository. Nothing here was measured on
  Rarog unless it says so. Read from Basilisk's `docs/PLAN.md` (Phase C,
  *Why classical Stockfish* and its *Correction*), `docs/experiments/BAS-O09.md`
  and `docs/experiments/BAS-E58.md` at Basilisk `d75d459` (2026-11-22).

## The three candidates

| Evaluation | Stockfish source | Date | What it is |
|---|---|---|---|
| Stockfish 11 | release `sf_11` | 2020-01 | the design Rarog's evaluation follows today (PLAN's originality row) |
| `9587eeeb` | master, tag `SF_classical` | 2020-08 | the last master before NNUE: its search and evaluation are the last pair tuned together as a classical engine |
| Stockfish 16 | release `sf_16` | 2023-06 | the last classical evaluation Stockfish shipped; `af110e02` removed it on 2023-07-11, its message calling it "probably the best of its class" |

What moved between them, as Basilisk read the sources:
- From `9587eeeb` to `sf_16`: the pawn, material and PSQT values, and new
  terms (`DoubledEarly`, `KingOnFile`, a two-phase imbalance). `sf_16`'s lazy
  cut reads the search's root score, and it damps specialised endings by
  the halfmove clock.
- From `sf_16` to the removal's parent `6a8767a0`, the evaluation files
  differ only in the network's name.
- Stockfish 16 with `Use NNUE=false` evaluates with that evaluation alone;
  it is the strongest hand-crafted engine in Rarog's frontier field
  (RAR-M91: 3,600.8, Stockfish 11 3,549.3, both as whole engines).
- Basilisk's caveat: from Stockfish 12 on, the classical evaluation was one
  part of an NNUE engine, and some of its later tuning was done with NNUE
  on, for the lopsided positions it still evaluated. Not all of it need
  help a classical engine.
- Stockfish 11's evaluation is older than `9587eeeb`'s, so Basilisk never
  measured it as a candidate.

## BAS-O09: each evaluation in both searches (games)

- **Question:** `9587eeeb`'s or `sf_16`'s classical evaluation as the donor?
- **Design:** a search's margins are tuned to its own evaluation, so each
  evaluation was played in both hosts. In `sf_16`'s search, its own
  evaluation against `9587eeeb`'s ported in; in `9587eeeb`'s search, its own
  against `sf_16`'s ported back. Same sign in both hosts decides; the
  frozen rule kept `sf_16` unless the comparison said otherwise.
- **Ports:** each hybrid's raw evaluation equals its origin's on about
  122,000 positions with no mismatch. One clause: `sf_16`'s lazy cut reads
  the root score, so its port receives it from `9587eeeb`'s root loop.
  Built from `sf_16` (`68e1e9b`) and `SF_classical` (`9587eeeb`), clang
  23.1.5, no PGO.
- **Conditions:** Colosseum, 3,000 games a host, `3+0.03`, one thread, Hash
  64, UHO in random order, no adjudication, no tablebases, no faults or time
  losses.
- **Results:**
  - in `sf_16`'s search, `9587eeeb`'s evaluation against `sf_16`'s:
    +3.7 [−3.5, +10.9] Elo (784-1464-752);
  - in `9587eeeb`'s search, `sf_16`'s evaluation against `9587eeeb`'s:
    −1.6 [−9.4, +6.1] (826-1334-840);
  - pooled, `sf_16`'s against `9587eeeb`'s: −2.7 [−8.0, +2.6]; the hosts'
    binding to their own evaluation −1.0 [−6.3, +4.3].
- **Reading:** the two evaluations are equal in play within about 8 Elo in
  either search, and neither search favours its own. By the rule `sf_16`
  stayed the donor and `9587eeeb` became the second seed column.
- **A host effect:** `sf_16`'s evaluation in `9587eeeb`'s search drew seven
  elementary mates by the fifty-move rule (KR-K 4, KBN-K 2, KRB-K 1), about
  0.8 Elo: its halfmove damping took those scores under `9587eeeb`'s static
  cut.
- **Calibration Basilisk recorded:** it expected a host binding of about 12
  Elo and found none measurable; the two evaluations differ by 1.7% in
  scale, which a search's margins do not see.

## BAS-E58: the static read (positions, not games)

- 254,486 positions from 25,950 games, read on the 243,108 rows without
  Basilisk's known-win overrides, each evaluator with its own K.
- Cross-entropy against results: `9587eeeb`'s evaluation leads `sf_16`'s by
  12.3e-4 nats [9.5, 15.1]; Basilisk's trails `9587eeeb`'s by 0.68% and
  `sf_16`'s by 0.45%.
- By phase the donors and Basilisk are equal in the middlegame; the donor
  leads by 1.5% to 3.4% at both ends.
- Donor families that add outcome information to Basilisk's whole score:
  space and mobility on the full board, king safety in the middlegame,
  "winnable" in endings, pawn structure throughout. Passers, threats, rooks
  and queens add nothing.
- A bounded refit of Basilisk's existing terms closes 56% of the static gap
  to the donor.
- Fit loss ranks and explains; it does not accept a change (both engines
  have lost games on better held-out loss).

## What Rarog has measured itself

- Rarog's evaluation is more accurate than Basilisk's against Stockfish 19
  on 10,000 game positions (RAR-O10: skill +0.0123 [+0.0016, +0.0235]).
  Rarog's evaluation has not been read against `sf_16`'s or `9587eeeb`'s
  directly.
- Rarog's oracle tags (`oracle/hybrid` and its two siblings) carry a
  vendored classical Stockfish tree, and E.4's king-danger lead uses a
  `9587eeeb` patch (`tools/diag/patches/sf9587_plain_maps.patch`).
- The D.6 review on the compact kernel found Rarog's evaluation off its
  donor's shape in several places, most of them endgame knowledge: the
  endgame scale's target (D.6.9), the pawnless default factor (D.6.10,
  which 15% of the kernel's games reach, `analysis/artifacts/d610-review-2026-11-22/`),
  and D.6.21 to D.6.26.

## Open for E.4.0

- Stockfish 11 against `sf_16` (and `9587eeeb`) as Rarog's donor: no read
  exists. Basilisk's games say `sf_16` and `9587eeeb` are equal in play;
  its static read favours `9587eeeb` slightly; Stockfish 11 is older than
  both.
- Whether Rarog's evaluation is rebuilt whole in the chosen donor's form,
  as D.7 rebuilt the search, or repaired family by family.
