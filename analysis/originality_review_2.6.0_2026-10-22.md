# Originality review of Rarog 2.6.0 against its donors (2026-10-22)

Question: could Rarog 2.6.0 (tag `v2.6.0`, commit `0127e58`) be called a
copy of Stockfish or Reckless, and what must be disclosed for rating lists
and for tournaments with originality rules?

Evidence layer: source comparison, and a 2,000-position move-similarity
screen (run by the maintainer 2026-10-23, under *Move-similarity test*).
No games. The disclosure work this review calls for is PLAN E.3.0.

## Verdict

Rarog 2.6.0 is an independently written engine, not a clone. Its code is
its own, and its evaluation weights are its own fits. It does contain, at
native scale and unrefitted, two groups of donor constants and the
formulas around them:

1. **Stockfish's time-management formula** (`src/search/time.rs:136-164`):
   about 17 constants identical to current Stockfish's `timeman.cpp`
   (0.0029869, 0.00033554, 0.004905, 3.3744, 3.0608, 3.1441, 0.012112,
   3.22713, 0.46866, 0.19404, 6.873, 12.352, 0.88, 116.4, 1.3, 0.11,
   0.8097), with "SF:" comments.
2. **Reckless's move-ordering and history-update formulas**: 37 of the 50
   hard-coded search constants of magnitude 150 or more are Reckless
   values (against 8 of 50 found anywhere in Stockfish, the chance
   baseline): `src/search/movepick.rs:25-37` (the quiet and noisy scoring
   weights 1763, 1614, 1066, 1086, 1051, escape bonus 8854/8170/14051/20357,
   10723, 8875, 3446, 4494, 4558), `src/search/history.rs:24-391` (bonus
   and malus formulas with caps 1252, 949, 935, 1550, 654, 229, 2414,
   1379, history limit 15320), `src/search/correction.rs:17-18` (14605,
   16418) and `src/search/node.rs:630-631` (the TT-cutoff history bonus,
   190/81/96/73/1206). The adoption was a recorded design decision
   (`analysis/search_programme_2026-09-13.md`, B.2 "ADOPT").

Neither is hidden in the project record, but the public README says only
"thanks ... for the inspiration". That understates it. PROCESS's own
independence boundary says a constant crosses as a seed "converted through
the measured scale ratio, fitted locally and gated"; these two groups were
not refitted, so the README and the boundary are both out of step with the
shipped code.

What follows is disclosure, not plagiarism: Rarog is GPL-3.0-or-later,
open source, and these portions are legally reusable. What it needs is an
accurate acknowledgement (wording under *Recommended disclosure*), plus one
licence point about Reckless (AGPL-3.0).

## Method

Source trees: Rarog `v2.6.0` (`src/`, 51 files, 135,077 tokens); Stockfish
`sf_11` (the evaluation donor) and Stockfish master `effdc3fd`
(2026-10-17); Reckless master `ffe556d` (2026-10-22) and five monthly
snapshots back to `12af143` (2026-05-31), which covers Rarog's whole life
(first commit 2026-05-22). The scripts are
`tools/diag/originality_scan.py` (numeric runs, scaled tables, token runs),
`tools/diag/originality_constants.py` (distinctive constants) and
`tools/diag/originality_eval.py` (live evaluation literals). Rerun from the
repository on 2026-10-23, they reproduce this review's outputs byte for
byte. They measure:

- **Numeric runs.** Every numeric literal in order, per file; maximal
  common runs of five or more values between Rarog and each donor. For
  Stockfish, `S(mg, eg)` pairs also give separate mg and eg streams, so a
  table split into mg and eg arrays still matches. Trivial runs (only
  0/±1/2, unit steps, powers of two) are separated out.
- **Scaled tables.** Every Rarog integer array of six or more values tested
  against every donor window for `rarog ≈ k × donor` with every element
  within ±1 (k from 0.25 to 4, k ≠ 1): a table converted by a scale ratio
  but never refitted.
- **Token runs** (Rust against Reckless): shared runs of 20+ raw tokens,
  and 40+ tokens with identifiers normalised, comments and strings removed.
  Positive control: Reckless `12af143` against Reckless `ffe556d` shares
  96% of its tokens in raw runs (67,204 of 69,782), so the instrument
  detects copied code.
- **Distinctive constants.** Every non-round literal of magnitude 150+ in
  live (non-test) search and evaluation code, checked for presence in each
  donor. An engine that is not the donor for that part (Stockfish for
  search, Stockfish's non-evaluation files for evaluation) gives the chance
  rate.

## Results

### Code text

- Rarog against Reckless: 458 of 135,077 tokens (0.34%) lie in shared raw
  runs of 20+ tokens, all of them boilerplate (bitboard operators, colour
  enums, list punctuation). With identifiers normalised, the longest runs
  are declaration lists (struct fields, option tables). The control shows
  96% for a real copy. **No copied code text.**
- Rarog against Stockfish is Rust against C++, so text overlap is not
  meaningful; the risk is transcription, covered below.

### Evaluation (Stockfish 11 donor)

- **The weights are Rarog's own.** No live table matches a Stockfish 11
  table, exactly or scaled. Of 58 live evaluation literals of magnitude
  150+, 12 occur anywhere in Stockfish 11's evaluation files, and those are
  generic (4096, mate scores, 0xFF masks) or coincidental; 10 occur in
  Stockfish's non-evaluation files, the chance rate.
- The only verbatim Stockfish 11 tables are `ShelterStrength` and
  `UnblockedStorm` (28 values each) in `src/eval/king.rs:688-703`, inside
  `#[cfg(test)]`: a test fixture that checks the shelter code against the
  donor's own numbers. It is not compiled into the engine; the live tables
  (`src/eval/params.rs:193-194`) are refits at roughly half the scale.
- **The design is Stockfish 11's.** The term set (shelter and storm by
  file distance from the edge, king danger with safe checks, threats,
  passed-pawn terms, quadratic imbalance, initiative) and some endgame
  rules are Stockfish 11's. For example, the KRPKR rule in
  `src/eval/endgame/mod.rs:510-517` is Stockfish 11's
  `endgame.cpp:513-520` with its constants 10 and 24 − 2·d. This is
  feature-level derivation, common among classical engines. It is also the
  kind of evidence the Rybka/Fruit investigation weighed (see the rules
  section).

### Search (Reckless donor)

- The architecture follows Reckless: threat-indexed quiet history, a
  noisy history keyed by captured type, correction histories, staged move
  picking without killers, and the TT-cutoff and best-move history
  updates. The ported blocks keep Reckless's control flow, for example
  the best-move history update (`src/search/history.rs:257-337` against
  Reckless `search.rs:1093-1145`, including the `1024 + k·i` malus
  scaling) and the TT cutoff (`node.rs:617-631` against `search.rs:386-401`,
  down to `move_count < 4` and `halfmove_clock < 90`).
- The constants: 37 of 50 hard-coded literals of magnitude 150+ are
  Reckless values (by file: movepick 14, history 15, node 6,
  correction 2); 8 of 50 occur in Stockfish. Most entered Reckless in
  June 2026: of the 49 search literals of magnitude 150+ found in today's
  Reckless, 16 were already in the 2026-05-31 snapshot and all 49 in the
  2026-06-29 one.
- **The SPSA-tuned parameters are Rarog's own.** In `src/search/params.rs`,
  19 of 67 defaults of magnitude 150+ occur anywhere in Reckless and about
  12 anywhere in Stockfish, close to chance. The large, characteristic
  values (LMR 1932, 3330, 1748; null-move 4255; the singular-extension
  terms) are not in either donor.

### Time management (Stockfish donor)

- `src/search/time.rs:136-164` reproduces current Stockfish's
  `timeman.cpp` formula for both the sudden-death and the `movestogo`
  branch, with about 17 constants identical and "SF:" comments. The
  safety reserve and the effort and instability terms around it are
  Rarog's own. D.1.4's fit (RAR-R15) left this budget shape out of its
  coordinates on purpose: RAR-R14 kept "Rarog's Stockfish-shaped budget",
  and the falling-eval clamps 0.572 and 1.708 stay too. The constants are
  still in `d1`'s `src/search/time.rs` on 2026-10-23. (Corrected
  2026-10-23: the first version of this review said D.1.4 would refit
  them.)

### Other

- Fathom (MIT, `vendor/fathom/`) does the Syzygy probing. It is a library
  almost every engine uses and an accepted one, but the README does not
  name it, and its MIT notice must travel with binaries.
- The `hybrid` Stockfish oracle and `tools/oracle-hce-ffi` are not part of
  the release build.

## What the venues require

Web research, 2026-10-22. Several primary pages could not be fetched, and
claims resting only on secondary sources are marked as such.

| Venue | Originality rule | What it means for Rarog |
| --- | --- | --- |
| CCRL | None: "We do not judge engine originality"; each tester chooses what to test (computerchess.org.uk). | Eligible today. Open source, GPL-compliant. |
| CEGT, SPCC (Pohl) | No published policy; selection by strength. CEGT once skipped the Ippolit family by practice. | Eligible today. |
| TCEC | No originality clause in the current rules; entry is by invitation, with a "fair play" catch-all. Sanctions so far (Houdini, Fire, December 2020) were for engines that "covertly contain code copied" from others; DeusX, a plain Leela clone, was excluded. The NNUE guideline (Season 20) asks that training data come from the engine's own search or evaluation, which does not apply to an HCE. | Eligible if the lineage is disclosed. The word that drew sanctions was *covertly*. |
| CCC (Chess.com) | No more than two finalists may share code. | Not a real constraint. |
| ICGA / WCCC, rule 2 (2019) | Original work, optionally including code or **data** from others with a legal right of use; all such sources **must be named** at registration; "undeclared derivatives" can be invalidated; the executable may be sent for similarity testing and the source to experts. | The strict venue. Entry needs the donor list below in the registration. Declared, the copied constants are a disclosure item, not a disqualification. Undeclared, they would be the kind of evidence the Rybka case rested on. |

How originality is judged in practice:

- **Move-similarity screen first.** Don Dailey's `sim` (2010) has every
  engine pick a move in about 8,000 quiet positions at about 100 ms and
  scores the share of identical moves. Chessprogramming.org treats more
  than 65% as relatively high similarity; Ed Schröder's SIMEX colours
  60-64% orange and 65%+ red. Unrelated engine pairs in published outputs
  sit in the mid-40s. A high score triggers an investigation; it does not
  prove copying.
- **Then source and feature comparison.** In the Rybka/Fruit case
  (Wegner, Watkins) the evidence was the overlap of evaluation features,
  their exact conditions and their numbers, measured against control
  engines. A common technique (for example "how history works in every
  modern engine", from the 2026 thread about Reckless) is treated as an
  idea. A reproduced set of tuned values is treated as copied work.
- **Disclosure decides.** Every sanction found was for undeclared
  copying. Declared derivatives (ShashChess at WCCC 2024, as reported)
  have been admitted.

## Recommended disclosure

Disclose; no admission of plagiarism is involved. Plagiarism is undeclared
copying, and the remedy is to declare. The README's Acknowledgements could
read (user-facing, short):

> Rarog is written from scratch in Rust, but it owes a great deal to other
> engines. Its evaluation follows the design of Stockfish 11's classical
> evaluation, with weights fitted by Rarog's own tuning. Its search, move
> ordering and history heuristics follow Reckless, and some move-ordering
> and history constants come from Reckless unchanged. Its time-management
> formula and constants come from Stockfish. Syzygy probing uses Fathom.
> Many thanks to the Stockfish, Reckless and Fathom authors.

For an ICGA-style registration, list the same three sources with the
files concerned (`src/search/time.rs`, `src/search/movepick.rs`,
`src/search/history.rs`, `src/search/correction.rs`, the TT-cutoff block
in `src/search/node.rs`, and the Stockfish 11-derived evaluation design).

## Licence

- Stockfish is GPL-3.0; Rarog is GPL-3.0-or-later. Transcribed Stockfish
  formulas are compatible. GPL practice is to credit the origin of
  derived code, and the acknowledgement above does that.
- Reckless is **AGPL-3.0**. Independently written code and ideas are not
  bound by it. The blocks that follow Reckless's control flow and
  constants closely (best-move history update, TT-cutoff bonus, quiet and
  noisy move scoring) are where a derivative-work argument could be made.
  GPL-3.0 and AGPL-3.0 may be combined (section 13 of each), but the
  derived parts would keep AGPL terms. Not legal advice. The options, from
  cheapest:
  1. Credit Reckless explicitly, and ask its authors whether they
     consider these blocks derivative.
  2. Relicense Rarog as AGPL-3.0-or-later, which removes the question,
     since the maintainer owns Rarog's code.
  3. Refit the copied constants and rewrite the blocks in Rarog's own
     form. Refitting alone removes the numeric fingerprint, not the code
     shape.

## Removing the numeric fingerprint (optional, engineering)

The copied constants are the part that would look worst in a
Watkins-style comparison, and refitting them is ordinary tuning work:

- **Time management:** no current step refits the budget shape. D.1.4
  fitted thirteen clock coordinates around it and left the budget alone
  (RAR-R15). Disclosure covers the shape. Refitting it would be a new
  clock leaf with its own registration.
- **Move ordering and history:** the 37 Reckless constants could join a
  later SPSA surface. This needs its own registration and gate under
  PLAN's rules and is not proposed here as a strength change.

## Move-similarity test

`tools/diag/sim_test.py` runs the screen against the donors and against
unrelated engines (smoke test: 20 positions, same binary twice 100%,
Rarog against Stockfish 11 55%; a missing binary is refused). The
prediction, frozen 2026-10-22 before exposure: Rarog 2.6.0 against
Stockfish 11 and against Reckless both stay below 60%, within the range
of the unrelated pairs in the same matrix. Rarog against Stockfish 11 is
the pair most likely to be raised, because they share the evaluation
design. The command, as handed over (about five minutes on an idle host,
one core per engine):

```bash
python tools/diag/sim_test.py --epd D:/chess/books/UHO_Lichess_4852_v1.epd --positions 2000 --movetime 100 --out analysis/artifacts/sim-2.6.0-2026-10-22 --engine "rarog=D:/chess/engines/rarog/rarog-v2.6.0-windows-pext.exe" --engine "sf11=D:/chess/engines/stockfish/stockfish-11-x64-bmi2.exe" --engine "sf10=D:/chess/engines/stockfish/stockfish-10-x64-bmi2.exe" --engine "reckless=D:/chess/engines/reckless-windows-avx2.exe" --engine "ethereal=D:/chess/engines/ethereal/Ethereal12.75-x64-pext-avx2.exe" --engine "fire=D:/chess/engines/Fire_8.3_x64_bmi2.exe" --engine "komodo13=D:/chess/engines/komodo/komodo-13.3-64bit-bmi2.exe" --engine "houdini6=D:/chess/engines/houdini/Houdini_6.03_Pro_pext_x64.exe" --engine "critter=D:/chess/engines/Critter_1.6a_64bit.exe" --engine "fruit=D:/chess/engines/fruit_21.exe"
```

The UHO positions are early middlegames, so every pair reads somewhat
higher than on `sim`'s quiet set. Read Rarog's pairs against the
matrix's own unrelated pairs (Ethereal–Komodo, Fruit–Critter), not
against the fixed 60% and 65% lines. SF10–SF11 is the related-pair
control.

### Result (2026-10-23)

Run by the maintainer. The artifacts are in
`analysis/artifacts/sim-2.6.0-2026-10-22/`: ten move files of 2,000
well-formed moves each, plus `matrix.txt`, which matches the table
below. Same-move percentage over 2,000 positions at 100 ms:

|  | rarog | sf11 | sf10 | reckless | ethereal | fire | komodo13 | houdini6 | critter | fruit |
|---|---|---|---|---|---|---|---|---|---|---|
| rarog | 100.0 | 56.4 | 57.7 | 54.2 | 57.0 | 57.6 | 55.6 | 56.5 | 57.2 | 50.6 |
| sf11 | 56.4 | 100.0 | 67.0 | 58.9 | 61.0 | 63.4 | 61.1 | 64.2 | 57.9 | 48.2 |
| sf10 | 57.7 | 67.0 | 100.0 | 57.8 | 62.1 | 64.0 | 61.6 | 64.2 | 58.5 | 48.4 |
| reckless | 54.2 | 58.9 | 57.8 | 100.0 | 58.2 | 56.2 | 54.9 | 56.0 | 52.1 | 45.7 |
| ethereal | 57.0 | 61.0 | 62.1 | 58.2 | 100.0 | 60.2 | 58.0 | 60.8 | 56.4 | 49.1 |
| fire | 57.6 | 63.4 | 64.0 | 56.2 | 60.2 | 100.0 | 63.6 | 68.8 | 60.1 | 51.2 |
| komodo13 | 55.6 | 61.1 | 61.6 | 54.9 | 58.0 | 63.6 | 100.0 | 62.9 | 58.2 | 51.0 |
| houdini6 | 56.5 | 64.2 | 64.2 | 56.0 | 60.8 | 68.8 | 62.9 | 100.0 | 60.2 | 49.5 |
| critter | 57.2 | 57.9 | 58.5 | 52.1 | 56.4 | 60.1 | 58.2 | 60.2 | 100.0 | 57.2 |
| fruit | 50.6 | 48.2 | 48.4 | 45.7 | 49.1 | 51.2 | 51.0 | 49.5 | 57.2 | 100.0 |

Binaries hashed on 2026-10-23 (SHA-256 prefix):
- Rarog 2.6.0 `02df47c54176e937`: `rarog/rarog-v2.6.0-windows-pext-native-pgo.exe`. The file in the handed-over command is no longer in the engine folder; this is the 2.6.0 binary there now.
- Reckless `b74ead5648cfa7a7`
- Stockfish 11 `271b62dce0226789`

Reading:

- **The instrument separates known relatives.** Both known-related pairs
  are above the 65% line: SF10–SF11 at 67.0, and Houdini 6–Fire 8.3 at
  68.8 (Fire is the engine TCEC disqualified alongside Houdini). No other
  pair in the matrix reaches 65%.
- **Rarog's row is flat.** Its pairs run from 50.6 (Fruit) to 57.7
  (SF10), with a mean of 55.9 over the nine. Against its donors it reads
  Stockfish 11 56.4, Stockfish 10 57.7 and Reckless 54.2. Reckless is
  Rarog's second-lowest pair, and Stockfish 11 is below Rarog's pairs
  with Fire (57.6), Critter (57.2) and Ethereal (57.0). The standard
  error of one pair is about 1.1 points (binomial at 2,000), so
  differences of a point or two within the row are noise.
- **No donor pair stands out against the unrelated ones.** For example,
  Ethereal–Komodo reads 58.0 and Critter–Ethereal 56.4. Rarog plays like
  an unrelated engine of its strength band against both donors.

Calibration: the prediction held in sign and range. Both donor pairs are
under 60% and inside the unrelated range. Its side remark was wrong in
magnitude. The shared evaluation design does not lift Rarog–Stockfish 11
above Rarog's other pairs, so the evaluation features and weights do not
show up as move agreement at 100 ms. The screen measures behaviour only.
It says nothing about the copied constants, which stay a disclosure item
(E.3.0).

## Sources

computerchess.org.uk (home and CCRL 40/15 *About*);
wiki.chessdom.org/Rules, /TCEC_Season_20_Rules, /Engines_and_authors;
chessprogramming.org/ICGA_Tournament_Rules, /Engine_Similarity,
/Rybka_Controversy, /Quantifying_Evaluation_Features;
chess.com news on CCC 7 rules and Fat Fritz 2;
stockfishchess.org/blog/2022/chessbase-stockfish-agreement;
talkchess.com/viewtopic.php?t=86133 (the 2026 thread about Reckless);
github.com/codedeliveryservice/Reckless; sp-cc.de.
