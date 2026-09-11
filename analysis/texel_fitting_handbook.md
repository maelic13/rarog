# Texel fitting in Rarog — the handbook

This is the reference for fitting Rarog's hand-crafted evaluation. It exists so
that the next person (or agent) to run a fit does not have to re-derive the
pipeline, re-measure the corpus, or rediscover the traps. `PROCESS.md` holds the
policy in ten numbered rules; this holds the operation, the resources, the
settings and the reasons.

Read this **before** touching a corpus or a fit. It is long on purpose and it
is not loaded into context by default.

---

## 1. What this is, and when to use it

Texel fitting here means: minimise a logistic loss over labelled positions, with
the label being the **game result** and the prediction being a sigmoid of the
static evaluation. It fits **1,218 `EvalParams` scalars** in one schedule.

**It costs zero games.** That is the whole reason it is the primary instrument
in this project: a real SPRT gate is an overnight run (`AGENTS.md`), so anything
that can be decided offline should be.

### Texel or SPSA?

| | Texel | SPSA |
|---|---|---|
| cost | zero games, hours of CPU | tens of thousands of games |
| scale | all 1,218 slots at once | a handful of knobs |
| objective | loss on labelled positions | playing strength directly |
| when | whole-surface refit, after structure changes | small, high-curvature knob sets |

**Stockfish is not the precedent people assume.** `D:\code\stockfish\src\tune.h`
is a harness that exposes parameters as UCI options for **fishtest**; there is
no logistic, sigmoid or Texel fitting anywhere in their tree. They tuned HCE by
playing games, on a distributed cluster. Rarog has one box. Do not conclude
"Stockfish used SPSA, so we should" — the two projects have different budgets by
three or four orders of magnitude.

`PLAN.md` rule 4 makes SPSA conditional: run it "only when activation,
interaction and curvature justify the cost", established first with a zero-game
sweep. A flat or monotone surface is evidence *against* spending it.

**Loss is not Elo.** A lower frozen-test loss accepts nothing. Every fit ends
with `strength_verdict = "not run"` and must be gated. RAR-S64 is the standing
reminder: a mechanism with a clean bench signal measured exactly zero in games.

---

## 2. Resources

### The position store — READ ONLY

```
A:\Chess\Beast\data\txt\positions.txt
```

**7,121,976,716 bytes, ~124.8M positions**, one FEN per line. Never write to it,
never move it, never "clean" it. Everything else is derived from it.

Measured profile (826,608 lines sampled from 12 sequential chunks):

| phase bucket | share | absolute |
|---|---|---|
| opening | 36.8% | ~45.1M |
| early_mid | 21.4% | ~26.2M |
| middlegame | 20.6% | ~25.2M |
| endgame | 17.4% | ~21.4M |
| deep_endgame | 3.9% | ~4.8M |

Exact 4-field duplicate rate: **0.02%**. Random seeks on this drive are very
slow; always read it sequentially.

### Start books

Books live in `tools/texel/data/` and are **gitignored** — they are not in the
repository. Their `*.manifest.json` (with SHA-256) is the evidence, so a book
can be rebuilt and proved identical.

| book | positions | composition | note |
|---|---|---|---|
| `beast_seed.epd` | 750,000 | 20/20/20/20/20 | legacy; produced `hce-v2` |
| `phase_book_v1.epd` | 1,000,000 | 50/10/10/10/20 | current, seed `0x5EED2`, SHA-256 `31E9B655…` |

Both are four-field EPD, no in-check positions, no terminal positions, 50%
white to move. See §6 for why the composition changed.

`tools/books/UHO_Lichess_4852_v1.epd` is a **different thing** — the SPRT/SPSA
gate book. Never use it for datagen: UHO openings are deliberately unbalanced,
which is right for a gate and wrong for label generation.

### Tablebases

```
D:\chess\tablebases\syzygy3456
```

3-4-5-6 man, 510 WDL + 510 DTZ files. Used for label correction (§5) and for the
endgame truth instrument. WDL 2 = clean win, 1 = cursed win, 0 = draw. DTZ is
distance to a zeroing move, **not** distance to mate — do not compare plies-to-
mate against DTZ.

### Corpora

Published corpora are directories under `tools/texel/data/` holding
`train.csv`, `validation.csv`, `test.csv` and `manifest.json`. Rows are
`FEN;target` with the target in `{0, 0.5, 1}`, white perspective.

| corpus | rows (train) | starts | profile | book |
|---|---|---|---|---|
| `hce-v2` / `hce-v2-tb` | 2,300,000 | 600,000 | `datagen-v1` | `beast_seed.epd` |
| `hce-v3` / `hce-v3-tb` | 3,500,000 | 602,619 | `datagen-v2` | `phase_book_v1.epd` |

**Never edit a published corpus in place** — `hce-v2` is what the accepted head
was fitted on and has to stay reproducible. A new corpus gets a new name, and
the `-tb` suffix is the tablebase-relabelled variant of the same rows.

The two differ by more than size, which matters when reading a fit result:

| | hce-v2 | hce-v3 |
|---|---|---|
| adjudicated games | 312,918 (52.2%) | 40 (0.007%) |
| natural mates | 6,428 | 367,664 |
| mean plies | 66.4 | 91.0 |
| draw share of rows | 45.7% | 35.3% |
| ≤6-man rows | 10.1% | 17.4% |
| relabelled by Syzygy | 1.325% | 3.230% |

`hce-v2` resigned most of its games out, so the evaluator largely learned from
outcomes that were **asserted rather than played**. A gate comparing a fit on
`hce-v3-tb` against the `hce-v2-tb` head therefore conflates row count, phase
mix and label provenance. That is a legitimate cluster to gate, but say so in
the registration — do not let a good result be attributed afterwards to
whichever cause seems most appealing.

---

## 3. The tools

Everything below lives in the repository and is the supported path. Do not
hand-roll replacements; `AGENTS.md` records that hand-rolled parsers are a
recurring source of wrong results here.

| tool | what it does |
|---|---|
| `tools/texel/build_book.py` | builds a phase-weighted start book from the store |
| `tools/diag/book_yield.py` | measures rows/game **by start-position phase** |
| `tools/datagen.ps1` | self-play generation, writes PGN + provenance manifest |
| `tools/texel/extract.py` | PGN → labelled CSV splits; also the sizing preflight |
| `tools/texel/extract_parallel.py` | the same, parallel; what the fit driver calls |
| `tools/texel/relabel_tb.py` | rewrites ≤6-man labels to Syzygy verdicts |
| `tools/texel/fit_complete.ps1` | the whole fit, one command, fully audited |
| `tools/texel/bake_params.py` | writes a fitted vector into `src/eval.rs` |
| `tools/texel/confirm_hce_fit.ps1` | re-verification of a completed fit |
| `tools/texel/sample_fens.py` | ad-hoc FEN sampling |
| `tools/texel/import_beast.py` | imports externally-evaluated positions (legacy) |
| `tools/texel/test_datagen.py` | tests for the datagen path |

The tuner binary itself is `tools/texel-tuner`, built to
`target/release/rarog-texel.exe`. Its options:

```
--write-defaults   --audit-coverage   --feature-support   --buckets
--verify           --weights          --initial           --from-cp
--tune <group>     --tune-kingsafety  --fix-k             --lr  --l2
--epochs           --max-positions
--test             --test-baseline    --test-marker       --compare-frozen
--report-endgames
```

You should almost never invoke it directly. `fit_complete.ps1` drives it in the
correct order with the correct guards.

---

## 4. The parameter surface

`EvalParams::FLAT_SIZE` is **1,218**, partitioned exactly once:

| class | slots | instrument |
|---|---|---|
| identifiable linear | 1,194 | sparse traced Adam |
| nonlinear king-danger selectors | 12 | dedicated king-safety pass |
| material/PST gauge anchors | 10 | **held fixed** (gauge freedom) |
| invariant king material | 2 | **held fixed** |

The 10 anchors are `pst_mg` and `pst_eg` at piece offset 0 for each of 5 pieces
(`pst_gauge_anchors()` in the tuner). They are pinned because a PST and its
material term are not separately identifiable — without the anchors the fit
wanders along a flat direction and produces a vector that is numerically
different and behaviourally identical.

The partition must sum to 1,218 exactly. `--audit-coverage` checks it.

---

## 5. The pipeline, end to end

Five stages. Each produces an artifact that the next one hash-verifies.

```
store ──build_book──▶ book ──datagen──▶ PGN ──extract──▶ corpus
                                                            │
                                              relabel_tb ───┤
                                                            ▼
                                                    fit_complete ──▶ vector ──▶ SPRT
```

### 5.1 Build the book

```bash
python tools/texel/build_book.py --count 1000000 --out tools/texel/data/phase_book_v1.epd
```

One sequential pass over the store (~3.5 min), reservoir sampling per bucket.
Default composition `50,10,10,10,20` (`--weights`). It refuses to overwrite an
existing book — books are never rewritten, because a corpus manifest cites the
book's SHA-256.

Three properties are load-bearing:

- It imports `PHASE_BUCKETS` and `PHASE_W` from `extract.py`, so the book's
  phase definition **cannot drift** from the extractor's.
- It reproduces the measured `beast_seed.epd` contract: no in-check, no
  terminal, 50% white to move. (Rebuilding `beast_seed.epd` through it rejects
  0 positions — that is the cross-check.)
- **It shuffles the output.** `datagen.ps1` hands out contiguous segments from
  `-Start`; a book grouped by bucket would give each segment a single phase.

### 5.2 Size the run

Never guess the game count. Measure it, twice over:

```bash
python tools/diag/book_yield.py <pilot.pgn>
python tools/texel/extract.py <pilot.pgn> --preflight-games 3000 --target-train 3500000
```

`book_yield.py` gives the matrix of rows/game **conditioned on the start
position's phase** — that is what tells you whether the *book* is right.
`extract.py --preflight-games` gives the recommended game count for a target —
that tells you how long to run. You need both; see §6 for why one alone misleads.

A nonzero exit from the preflight inside `book_yield.py` is **expected** and is
the finding, not a failure: it refuses to recommend a count when some phase had
zero yield, which is exactly what every non-opening start bucket does.

### 5.3 Generate

```bash
pwsh -File tools\datagen.ps1 -Suffix <engine-suffix> -Rounds <N> -Start 1 -Nodes 8000 -Book tools\texel\data\phase_book_v1.epd -BookFormat epd
```

- The engine is `tools/test_engines/rarog-<Suffix>-pext-pgo.exe` with a JSON
  manifest beside it. **Verify its bench fingerprint before generating** — the
  fit will be attributed to whatever binary actually played.
- `-Nodes 8000` is the standing budget. Fixed nodes, not time: results must not
  depend on machine load.
- Concurrency is automatic and **oversubscribes** (all 32 logical processors);
  datagen is node-limited, so oversubscription costs nothing in correctness.
- The profile is **`datagen-v2` — no adjudication** (default since 2026-09-01).
  `-Adjudicate` opts back into `datagen-v1`; `-SyzygyPath` selects `datagen-v3`
  (tablebase game adjudication), which is **untested** and changes the result of
  every position sampled from the game.
- Segments never wrap the book; the harness throws instead. Track which opening
  ranges you have consumed.
- Output PGN and a `.manifest.json` recording book path, SHA-256, seed, and
  opening range. That manifest is the provenance the fit driver verifies.

Throughput reference: **~1,714 games/min** from `phase_book_v1.epd` at 8,000
nodes, concurrency 30. Output is ~2.5 KB/game.

### 5.4 Extract

`fit_complete.ps1` does this for you if the corpus directory does not exist. To
do it standalone:

```bash
python tools/texel/extract_parallel.py <pgn...> --out-dir tools/texel/data/<name> --target-train 3500000 --jobs 14
```

What the extractor does, and the settings that matter:

| setting | value | why |
|---|---|---|
| phase buckets | 5, by **material** (`opening` = 20–24) | matches `src/eval.rs` |
| per-bucket quota | `target_train / 5`, equal | every phase equally represented |
| `--max-per-phase-per-game` | 8 | limits within-game correlation |
| `--max-per-game` | 16 | global safety cap; the phase cap is the primary control |
| `--skip-start` | 2 | drops the book position and its successor |
| `--skip-end` | 6 | drops the decided tail |
| quiet filter | on | no positions with a tactical resolution pending |
| splits | 90/5/5 by **hash of the game start** | a game's rows never straddle splits |

The hash-of-start split is what prevents leakage: the same start replayed would
otherwise put correlated rows on both sides.

### 5.5 Relabel against tablebases

**This is a standard step, not an experiment.** RAR-E08 measured it at
**+6.73 ± 3.82 Elo**.

```bash
python tools\texel\relabel_tb.py --source <corpus> --syzygy D:\chess\tablebases\syzygy3456 --out <corpus>-tb
```

Every position of ≤6 men gets its Syzygy verdict instead of the game result.
**Cursed wins count as draws** (`{2:"1", 1:"0.5", 0:"0.5", -1:"0.5", -2:"0"}`) —
under the fifty-move rule a cursed win is a draw, and labelling it a win teaches
the evaluator something false. Positions above 6 men are untouched.

On `hce-v2` this changed 30,480 train labels (1.325%); on `hce-v3`, 113,046
(3.230%). Both had 0 probe failures.

The tool writes **two** manifests: `relabel-manifest.json` (its own report) and
`manifest.json` (the corpus manifest, with the label string retargeted, the
three output hashes replaced and a `derived_from` provenance block). The second
is what makes the output fittable — it was hand-built for RAR-E08, which is a
transcription step on the critical path of a multi-hour fit, and is now
emitted automatically.

Do **not** confuse this with `datagen-v3`. This corrects labels post hoc,
leaving the games as played. `datagen-v3` adjudicates the game itself and
therefore changes the label of every position in it, including positions far
above 6 men. That has never been tested.

### 5.6 Fit

From a **clean worktree** — the driver refuses tracked changes:

```bash
pwsh -NoProfile -File tools\texel\fit_complete.ps1
```

The schedule, and why it is shaped this way:

1. **Fit K once** on validation (`--buckets`), then **pin it for every stage**
   (`--fix-k`). A K that moves between stages makes the losses incomparable.
2. **Nonlinear king safety**, 40 epochs, 200k positions. First, because the
   king-danger selectors gate large parts of the linear surface.
3. **Complete linear**, 200 epochs, sparse Adam, `lr=0.3`, L2 `1e-7`.
4. **Nonlinear king safety again** — the linear surface moved under it.
5. **Linear polish**, 60 epochs, same hyperparameters.

The L2 term is **to the stage prior**, not to zero: `grad/n + 2λ(w − base_w)`.
Pulling toward zero would be a statement that zero is the right value for every
term, which it is not.

The **frozen test is opened exactly once**, at the end, and a marker file
records it. If the marker exists the driver refuses to run. This is the one
defence against selecting on the test set, and it is worth more than the extra
information a second look would give.

Artifacts land in `tools/results/hce-fit-<timestamp>/`: every log, every
intermediate vector, `settings.json`, `summary.json`, the source patch, the
candidate binary, and hashes of all of it. `src/eval.rs` is restored
byte-for-byte and the release binary rebuilt, verified against the accepted
fingerprint — including in the `finally` block if the run dies partway.

---

## 6. Why the book composition is what it is

This is the finding that is easiest to lose, so it is written down twice (see
also `analysis/texel_corpus_book_shape_2026-09-02.md`).

The extractor's phase is **material, not ply**. A game started below phase 20
can never produce an `opening` row. Measured yield per game, by the phase of the
game's own start position:

| start bucket | opening rows/game | all buckets |
|---|---|---|
| opening | **3.4392** | **13.31** |
| early_mid | 0.0008 | 12.39 |
| middlegame | 0.0000 | 11.65 |
| endgame | 0.0000 | 9.85 |
| deep_endgame | 0.0000 | 9.10 |

Two consequences:

1. **Only opening starts feed the opening bucket.**
2. **Opening starts are also the most productive overall**, because one game
   traverses every phase on its way down.

So a phase-*balanced* book spends four fifths of its starts on positions that
cannot contribute to the bucket that binds. With `beast_seed.epd` the preflight
asked for **1,113,504 games** — more than the 750,000 openings in the book, i.e.
unreachable at any schedule. The preflight was not wrong; it sizes *games* and
had no way to say the *book* was the problem. **This is why you run
`book_yield.py` too.**

The yield-maximising composition is 68/10/0/0/22. The adopted default is the
hedged **50/10/10/10/20**, giving up 26% of the theoretical yield, because the
corner would make every middlegame and endgame row a *reached* position
correlated with the opening play that led there. Keeping 10% direct starts in
each holds those buckets independently sampled.

Result: 3.5M rows at **602,619 games** instead of 1,113,504 for 3.0M.

### Two levers deliberately not pulled

- **`--skip-start`.** Rarog drops 2 plies, Basilisk drops 0. Those plies are
  opening-phase by construction; the cost is 3.7796 → 3.4392 rows/game on
  opening starts, **9% of the scarcest bucket**.
- **`--max-per-game`.** Rarog caps at 16, Basilisk at 0. Raising it to 40 moves
  the mixed-book opening rate 0.6735 → 0.9458, but it buys rows with
  **within-game correlation** rather than with games.

Both are held so a new corpus differs from `hce-v2` in the book alone. Change
one variable at a time.

---

## 7. Contract gates, and how to extend them

`fit_complete.ps1` verifies the corpus before fitting it. These gates have
caught real errors and must not be loosened. They are, however, **pinned to the
`hce-v2` contract**, and a new corpus will be refused until each is extended
**by naming the new contract** — never by widening the test:

| gate | current requirement | note |
|---|---|---|
| label whitelist | two named strings | extended once, for the TB relabel |
| corpus contract | named `(profile, starts)` pairs | `datagen-v1`/600,000 and `datagen-v2`/602,619 |
| `recorded_games` | `== independent_starts` | proves no start was replayed |
| `rows.train` | `== TargetTrain` (default 2,300,000) | pass `-TargetTrain` |
| book format | must be `epd` | auditable |
| book SHA-256, seed, openings | must agree across all inputs | one book contract |
| split sizes | frozen 5% / 5% | |
| `train/validation/test` SHA-256 | must match the manifest | |
| `parse_errors`, `paired_replays_discarded` | must be 0 | |

The label whitelist is the model to copy. It is a **named list**, each entry
written down with what it means:

```powershell
$acceptedLabels = @(
    "white-perspective self-play WDL",
    "white-perspective self-play WDL, <=6-man Syzygy corrected"
)
```

An unrecognised label still throws. That is the point: it correctly refused
RAR-E08's arm B until arm B was named. Do the same for the adjudication profile
and the start count rather than deleting the checks.

### The fingerprint guard, and what it cannot prove

The driver asserts the baseline `bench 13` fingerprint — currently **6,901,489 /
EBF 2.458** — before and after. A fingerprint identifies the **search**, so a
change confined to positions the bench suite never reaches is invisible to it.
The 4.9a.4 mate drive moves KBN-K conversion from 19.4% to 96.9% and leaves the
fingerprint byte-identical. So the guard will happily pass a tree carrying an
unaccepted eval change: **also check `git rev-parse HEAD`** against the commit
the fit is supposed to start from. The run manifest records it for that purpose.

---

## 8. Traps

Each of these cost real work in this project.

1. **`bench` reads from stdin, not argv.** `rarog.exe bench 13` prints the
   banner, does nothing, and **exits 0**. Use
   `printf 'bench 13\n' | rarog.exe`.
2. **A stale binary.** `cargo test`, `cargo clippy` and `cargo bench` all
   rebuild `target/release/rarog.exe` with *their* features. `--all-features`
   enables `texel`, which bypasses the eval and pawn caches and must never be
   measured. Rebuild immediately before measuring.
3. **Loss is not Elo.** Register an SPRT. `[0,3]` nElo is the default bracket;
   a removal or simplification wants `[-1.75, 0.25]`.
4. **The preflight sizes games, not books.** §6.
5. **Random seeks on the A: drive** are pathologically slow. Read sequentially.
6. **Books are gitignored.** The manifest SHA is the evidence. A bare path is
   not a citation; record the SHA and the command that rebuilds it.
7. **Never tune on the frozen test.** The marker file exists to stop you.
8. **Cursed wins are draws.** Anything else teaches a falsehood.
9. **`datagen-v3` is untested.** It is not the same thing as the TB relabel.
10. **Check exit status directly**, not through a pipe: `cmd > out 2>&1; echo $?`.
    `cmd | tail` reports `tail`'s status, which is always 0.

---

## 9. Reference numbers

| quantity | value |
|---|---|
| parameter registry | 1,218 (1,194 + 12 + 10 + 2) |
| accepted bench fingerprint | 6,901,489 / EBF 2.458 |
| store | ~124.8M positions, 36.8% opening-bucket |
| `phase_book_v1.epd` | 1,000,000 positions, 50/10/10/10/20 |
| datagen budget | 8,000 nodes/move, `datagen-v2` |
| datagen throughput | ~1,714 games/min at concurrency 30 |
| binding yield, new book | 1.6932 rows/game (measured at full scale) |
| 3.5M rows | 602,619 games, 5 h 10 m at 1,941 games/min |
| `hce-v3` published | 3,888,888 rows (3,500,000 / 194,444 / 194,444) |
| linear fit | 200 epochs, Adam, `lr=0.3`, L2 `1e-7` to stage prior |
| nonlinear fit | 40 epochs, 200,000 positions |
| polish | 60 epochs |
| splits | 90 / 5 / 5 by hash of game start |

### Accepted results

| id | change | result |
|---|---|---|
| RAR-E06 | complete HCE refit | **+22.04 ± 7.51 Elo** |
| RAR-E08 | ≤6-man Syzygy label correction | **+6.73 ± 3.82 Elo** |

---

## 10. Related documents

- `PROCESS.md` § *Texel convergence procedure* — the ten policy rules
- `analysis/texel_corpus_book_shape_2026-09-02.md` — the book-shape derivation
- `analysis/hce_maturity_2026-08-25.md` — the 1,218-slot partition
- `analysis/hce_residuals_2026-09-01.md` — what the fitted surface still misses
- `EXPERIMENTS.md` — RAR-E06 through RAR-E11 registrations and evidence
- `AGENTS.md` — measurement and gating rules that override anything here
