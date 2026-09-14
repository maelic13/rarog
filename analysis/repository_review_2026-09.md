# Repository and document review — PLAN B.2.0.1

Revision `4e60e45` on `dev` (three B.2.0 tickets landed), 2026-09-14.
Research state of leaf B.2.0.1, class `R3`. **No source, document or
configuration changed by this review**; it defines the restructure, reformat
and clean-up of everything B.2.0 does not touch: the nine top-level
documents, `analysis/`, `docs/`, the tool READMEs and the repository root.
The architecture review (`analysis/architecture_review_2026-09.md`) covered
`src/`, `tests/`, `tools/` and the build configuration and stays the owner
of those. This document is the companion for the rest, written to the same
standard: every finding has a mechanical measurement, a classification and
an owner.

The standard applied: **one document, one purpose, stated in its first
paragraph; no content that another document owns; nothing a reader can no
longer act on; every tracked file reachable from an index or a citation.**
PLAN section 5 already assigns the purposes; the findings are where the
files have drifted from that table.

Method. `git ls-files` for the inventory; a script counting, for every
tracked Markdown file, how many other tracked files cite it by name; a scan
for backticked repository paths in tracked Markdown that exist neither in
the index nor on disk; section and row counts of the ledger; grep censuses
of repeated policy phrases across the top-level documents; the first
paragraph of every `analysis/` document read for its self-declared status.
Raw outputs are in ignored `tools/results/b20-20260914/` (`doc_census.txt`,
`ledger_shape.txt`), hashed in RAR-M52.

## 1. Verdict

The document set is complete, honest and checked mechanically where it
matters (the status board, the fingerprint, the workflow register). What it
is not is single-purpose. Each of the four working documents carries a
second document inside it:

- **PLAN carries HISTORY.** 532 of its 1,508 lines are the closed Phase A,
  a further 202 the closed B.0 and B.1, with eight "Original scope"
  paragraphs retained after completion. HISTORY already holds the same work
  as dated records.
- **HISTORY carries an archive and points at one that is gone.** Its
  540-line "Forward tracker" is the Phase 4–9 board that `docs/archive/`
  already holds verbatim, and its "Legacy tracker" section, which the
  resolution table sends readers to for the oldest numbering scheme, is a
  nine-line stub with no items in it. The 347 source comments that cite
  `Phase N`, `8.x`, `9.x` and `10.x` numbers resolve to nothing.
- **EXPERIMENTS carries essays and two formats.** 183 rows, but section 2
  holds 589 lines of prose around 20 rows, four evaluation gates are
  written as prose subsections while every other experiment is a row, and
  section 10 duplicates PROCESS's packet template.
- **AGENTS and PROCESS carry each other's material.** PROCESS holds a
  40-line policy narrative on adjudication, a design record on CPU
  compatibility, a stale pointer to a table PLAN no longer has, and
  "historical note, superseded" paragraphs; AGENTS states rules through the
  incidents that produced them.
- **README contradicts CI**: it tells a contributor to run
  `cargo test --workspace`, the one command PROCESS and CI forbid.

Around them: 44 backticked repository paths in tracked Markdown point at
files that do not exist (five of them in PLAN's current text), seven of the
nine tracked logo files are used by nothing while the storage policy says
unused variants are ignored, two analyses are cited by no tracked file, and
`analysis/` has 71 files with no index that says which are live contracts,
which are current deliverables and which are superseded records.

None of this is a defect in the work; it is the residue of a project that
kept everything and never had a leaf whose job was to put each thing in one
place. B.2.0.1 is that leaf. Its tickets are in §5. It runs after B.2.0
closes, because B.2.0's T7, T9 and T11 edit PROCESS, AGENTS, CI and the
tools directory and the two must not overlap.

## 2. Measurements

### 2.1 Inventory

| Item | Value |
|---|---|
| Tracked files | 273: `tools/` 99, `analysis/` 71, `src/` 37, `tests/` 27, `logo/` 9, `vendor/` 6, root documents 9, `docs/archive/` 2, `xtask` 2, `benches/` 2, `.github/` 2, configuration 5 |
| Top-level Markdown | README 178 lines, AGENTS 447, CLAUDE 8, GUIDE 249, PLAN 1,508 (106 KB), PROCESS 448, EXPERIMENTS 1,674 (427 KB), HISTORY 928 (64 KB), CHANGELOG 912 (45 KB) |
| `analysis/` | 71 Markdown files, 1,539 KB in total across the tree; 2 cited by no other tracked file; 5 carry a "historical" or "superseded" banner; 3 are dated 2026-07-13 audits of revision `ff21dc1` |
| `docs/archive/` | the Phase-4 PLAN (4,401 lines) and GUIDE (351 lines), verbatim |
| Ledger | 183 rows in eight series (M 30, S 82, E 16, O 3, R 13, P 26, C 4, X 9); section 2: 20 rows, 589 prose lines; section 5: 18 rows plus four prose registrations (RAR-E06, E08, E12, E13) |
| Dangling paths | 44 distinct backticked repository paths that exist nowhere: PLAN 5, EXPERIMENTS 9, analysis 21, `docs/archive` 8, `tools/texel/README.md` 1 |
| Logo | 9 tracked PNG files; 1 referenced (`README.md`) |
| Policy phrase spread | "adjudicat": PLAN 15, PROCESS 13, EXPERIMENTS 51, HISTORY 5, AGENTS 1; "fingerprint": AGENTS 14, GUIDE 11, PLAN 29, EXPERIMENTS 56, HISTORY 12; "workspace": PROCESS 4, EXPERIMENTS 5, README 1, AGENTS 1, GUIDE 1, PLAN 1 |
| Checker | `check_guide.py` accepted two-level IDs only; extended this session to three levels so `B.2.0.1` can exist on the board |

### 2.2 Purpose declared against content found

| File | Declared purpose (PLAN §5, first paragraph) | Found beside it |
|---|---|---|
| `PLAN.md` | roadmap: objective, rules, phases, protocols | 734 lines of closed-leaf narrative and "Original scope" text; the number map (§6) that PLAN §5 assigns to HISTORY; the target module layout (right place) |
| `GUIDE.md` | operator contract, model mapping, prompts, status board, checkpoint | a five-paragraph "Next and held work" that restates the checkpoint table and PLAN; otherwise clean |
| `EXPERIMENTS.md` | frozen predictions, results, calibration, retry triggers, recipes | three narrative sections in §2 (deleted-branch audit, test-engine store clearance, Phase-4 registration); four prose registrations; a template PROCESS also holds |
| `PROCESS.md` | research packet, recurring procedures | adjudication policy history (40 lines); the independence boundary as prose with a pointer to "PLAN §4", which is now the release rules; a CPU-compatibility design note citing a retired `8.1`; "historical note, superseded" paragraphs; a datagen-profile history |
| `HISTORY.md` | completed work, retired numbering, number map | a 540-line second copy of the Phase 4–9 tracker; a legacy-tracker section with no tracker in it |
| `AGENTS.md` | operating rules | incident narratives as the body of rules (`Gating` 42 lines, `Verification` 49, `Measurement` 42); a comment rule that repeats PLAN's B.2.0 scope |
| `CHANGELOG.md` | user-facing release record | fine; older entries carry "Evaluated and rejected" and "Verified" subsections, which is ledger material but stable and released |
| `README.md` | user-facing product page | a test command CI forbids; otherwise fine |
| `analysis/README.md` | storage boundary | claims unused logo variants are ignored; they are tracked |
| `tools/texel/README.md` | fitting toolchain | a "roadmap note" mapping to `4.7`–`4.10`, all retired, and a path `tools/texel/tuner.cpp` that does not exist |

### 2.3 `analysis/` by class

| Class | Count | Examples | Treatment |
|---|---|---|---|
| Live contracts (PLAN §2 standing-contracts table) | 11 | `see_contract_2026-09-06`, `history_contracts_2026-09-08`, `draw_policy_2026-09-08`, `phase4_counter_spec`, `endgame_measurement_layers`, `texel_fitting_handbook` | keep, list as contracts in the index |
| Current-roadmap deliverables and design records | 9 | `consolidation_2026-09-10`, `search_programme_2026-09-13`, `architecture_review_2026-09`, `feature_inventory_2026-09-09`, `time_forfeit_2026-09-09`, `universal_binary_2026-09`, `ablation_design`, `ablation_results`, this file | keep |
| Phase-4 leaf records still cited by rows or PLAN | about 36 | `see_repair_2026-09-06`, `relocation_2026-09-07`, `endgame_refresh_2026-09-09`, `node_budget_2026-09-04` | keep as evidence, indexed as records |
| Superseded or historical, self-declared or dated before the current head | 13 | `hce_analysis` (2026-07-13), `infra_analysis` (2026-07-13), `search_analysis` (2026-07-13), `engine-choice-audit-2026-09-07`, `board_comparison_2026-09-09`, `board_perft_compare` (2026-07-22), `speed_profile_8_12c` (2026-07-23), `smp_analysis` (2026-07-22), `phase4_10_obligations`, `phase4_6_audit`, `manta_tooling_audit_2026-08-25`, `basilisk_audit_2026-08-30`, `answer_harness_calibration` | move to `analysis/archive/` with banners intact; citations updated mechanically |
| Uncited by any tracked file | 2 | `answer_harness_calibration`, `engine-choice-audit-2026-09-07` | both fall in the superseded class above |

The counts are from the census; the class of each file is assigned in the
index U7 produces, and a file may move class there if its citations say so.

## 3. Findings

**D1 — PLAN is the roadmap and a second HISTORY.** Evidence: 734 lines of
closed work, eight retained "Original scope" paragraphs, five current-text
paths that do not exist (`analysis/eval_programme_2026-xx.md` placeholder,
`analysis/search_programme_2026-xx.md`, `src/search.rs`,
`tests/tt_provenance.rs`, `tools/texel/reference/basilisk_tuner.cpp`), the
number map PLAN §5 assigns to HISTORY. Treatment (U2): a closed leaf keeps
one line — ID, one-clause result, date, ledger row — and its narrative lives
in HISTORY's dated records, which already exist for every Phase A leaf and
for B.0/B.1; the placeholders become the real names or "to be named by the
investigation"; §6 moves to HISTORY. Expected size: at most 950 lines.

**D2 — HISTORY duplicates the archive and has lost the legacy tracker.**
Evidence: the "Forward tracker" (540 lines, 52 checkbox items) is a second
form of `docs/archive/GUIDE-phase4-2026-09-09.md` (59 items, verbatim); the
"Legacy tracker (retired numbering, frozen)" section is nine lines and
holds no items, while the resolution table in the same file says the
oldest scheme is resolved "in the tracker section of this file". Treatment
(U1): recover the pre-2026-08-21 GUIDE that held the legacy tracker from Git
history into `docs/archive/GUIDE-legacy-2026-08-21.md` (the split-out
commit is findable with `git log -S "Legacy tracker"`); delete HISTORY's
forward tracker in favour of the archive; point the resolution table at the
two archives; keep the 23-line "what it established" summary and the dated
records. Expected size: at most 450 lines.

**D3 — EXPERIMENTS mixes a ledger with essays and two row formats.**
Evidence: §2 has 589 prose lines around 20 rows; RAR-E06, E08, E12 and E13
are prose subsections with their own headings while the other 179
experiments are rows; §10 is a template that PROCESS also carries as the
packet; nine backticked paths in rows point at retired module names.
Treatment (U3): one format — every experiment is one row, a registration
longer than a row lives as a packet under `analysis/` and the row cites it;
the three essays become dated HISTORY records; one template, in PROCESS;
the numbering note is rewritten after U1 so it points at the two archives;
historical rows keep their retired paths, and the note says so once.
Expected: rows unchanged at 183; prose lines in the row sections near zero.

**D4 — AGENTS states rules through stories.** Evidence: the `Gating`,
`Verification` and `Measurement` sections are 133 lines for about 20 rules;
the `Changes` comment rule repeats PLAN's B.2.0 scope sentence. Treatment
(U4): each rule in the imperative in at most three sentences, plus at most
one measured reason with its ledger ID; the incident that produced the rule
stays in the ledger row it cites or in HISTORY. This is the maintainer's
operating contract, so U4 is drafted and shown before it is committed.
Expected size: at most 280 lines with every rule kept.

**D5 — PROCESS holds policy, history and design beside its procedures.**
Evidence: the adjudication section (40 lines) argues the policy and records
its history; "Historical note, superseded" paragraphs; "CPU compatibility
design" is a design record that cites a retired `8.1`; "PLAN §4 holds the
full table" points at the release rules; the datagen-profile history.
Treatment (U5): the adjudication rule in five lines citing RAR-M16 and
RAR-M17; superseded notes deleted; the CPU design note folded into
`analysis/universal_binary_2026-09.md`, which already owns that decision;
the independence boundary stated once, here, with PLAN rule 1 pointing to
it; `harness_common.ps1`'s 25-line adjudication comment reduced to one line
citing PROCESS. Expected size: at most 350 lines.

**D6 — README's test command is the forbidden one.** `cargo test
--workspace --all-targets` unifies `texel` into the engine; PROCESS and CI
say `-p rarog`. Treatment (U6): after B.2.0's T7 takes the tuner out of the
workspace the command becomes correct again and the warnings elsewhere go;
until then README says what CI runs. One line either way.

**D7 — `analysis/` has no index and carries 13 superseded records beside
live contracts.** Evidence: §2.3. Treatment (U7): `analysis/README.md`
grows a table of every document with its class, owner leaf and status
(live contract, deliverable, record, archived), superseded documents move to
`analysis/archive/` with their banners, and the citation census script is
run before and after so no row or PLAN line points at a moved file without
being updated. The storage-boundary text it already holds stays.

**D8 — Repository root and assets disagree with the storage policy.**
Evidence: nine tracked logo files, one used, and `analysis/README.md`
stating that unused variants are ignored; `hybrid/` (the oracle package
directory PLAN A.2.2 documents) is absent from the storage list;
`uci_specification.txt` at the root, which B.2.0's T9 moves to `docs/`.
Treatment (U8): untrack the seven unused logo files with `git rm --cached`
and an ignore rule, keeping `rarog_detailed.png` and the light/dark pair
for a future README; add `hybrid/` to the storage list; verify every
statement in `analysis/README.md` against `.gitignore` and the index.

**D9 — GUIDE's prose section is longer than its job.** "Next and held work"
restates the checkpoint table and PLAN across five paragraphs. Treatment
(U9): the holds table plus one paragraph naming the next leaf and its
handoff; everything else already lives in the table above it or in PLAN.

**D10 — Nothing checks a current document for dead paths.** Evidence: 44
dangling paths, five in PLAN's current text, found only by this review.
Treatment (U2, tooling): `check_guide.py` gains a check that every
backticked repository path in GUIDE, PLAN, PROCESS and AGENTS exists,
tracked or on disk; the ledger and `analysis/` are exempt because their
historical paths are evidence.

**Keep, unchanged:** `CHANGELOG.md` (released text is not rewritten; future
entries use only Added/Changed/Fixed/Removed), `CLAUDE.md`, `docs/archive/`
(grows by the recovered legacy GUIDE), `LICENSE`, `vendor/fathom/`, the
`.gitkeep` placeholders, the frozen JSON fixtures under `tools/diag/`,
`tools/spsa_configs/README.md` (current and single-purpose).

## 4. What B.2.0 already owns and this leaf must not repeat

B.2.0's tickets T7 (tuner out of the workspace; PROCESS, AGENTS and CI text
that exists only because of the unification), T9 (`tools/README.md` index,
`uci_specification.txt` to `docs/`, tool prose hygiene) and T11 (comments
in `Cargo.toml`, `build.rs`, `.cargo/`, `rust-toolchain.toml`, `.github/`)
overlap this leaf's files. B.2.0.1 therefore starts when B.2.0 is `CLOSED`,
takes the tree as B.2.0 leaves it, and U4–U6 are written against that
state. `tools/texel/README.md`'s roadmap note and dead path are the one
tooling item T9's prose pass may or may not reach; U7 checks and fixes it
if it is still there.

## 5. Handoff — B.2.0.1 tickets

State `READY_FOR_IMPLEMENTATION`, class `I2` (maintainer decision: the
leaf restructures the operating documents and needs judgement about what a
sentence owns, not only where it goes). Documents-only: no engine input
changes, so no bench; every ticket is its own commit; `check_guide.py`
passes after each; the citation census is re-run after any move.

| # | Ticket | Findings | Files | Check |
|---|---|---|---|---|
| U1 | HISTORY: recover the legacy tracker into `docs/archive/`, delete the duplicated Phase 4–9 tracker, fix the resolution table | D2 | `HISTORY.md`, `docs/archive/GUIDE-legacy-2026-08-21.md` | every scheme in the resolution table resolves to a file that exists |
| U2 | PLAN: closed leaves to one line each with their HISTORY record confirmed present; §6 to HISTORY; placeholders named; dead paths fixed; `check_guide.py` dead-path check | D1, D10 | `PLAN.md`, `HISTORY.md`, `GUIDE.md`, `tools/diag/check_guide.py` | checker green with the new check; PLAN ≤ 950 lines |
| U3 | EXPERIMENTS: one row format, essays to HISTORY, one template (PROCESS), numbering note rewritten | D3 | `EXPERIMENTS.md`, `HISTORY.md`, `PROCESS.md`, `analysis/` packets for the four gates | 183 rows before and after; every RAR ID still resolves to exactly one row |
| U4 | AGENTS: rule-first rewrite, stories to their ledger rows, no duplication of PROCESS or PLAN; drafted, shown, then committed | D4 | `AGENTS.md`, `CLAUDE.md` unchanged | every rule of the current file present in the new one (a checklist in the commit message); ≤ 280 lines |
| U5 | PROCESS: procedures only; adjudication to five lines; CPU design to the universal-binary record; independence boundary owned here; `harness_common.ps1` comment to one line | D5 | `PROCESS.md`, `analysis/universal_binary_2026-09.md`, `tools/harness_common.ps1`, `PLAN.md` rule 1 pointer | no "superseded" or "historical" paragraph remains; ≤ 350 lines |
| U6 | README test command | D6 | `README.md` | matches `ci.yml` |
| U7 | `analysis/` index with class and status; 13 superseded records to `analysis/archive/`; citations updated; `tools/texel/README.md` note and path | D7 | `analysis/README.md`, `analysis/archive/`, every citing file, `tools/texel/README.md` | census: zero uncited live documents; zero dangling paths introduced by the moves |
| U8 | Logo untracking per the storage policy; `hybrid/` in the storage list; `.gitignore` agrees with `analysis/README.md` | D8 | `logo/`, `.gitignore`, `analysis/README.md` | `git ls-files logo` lists the three kept files; README renders |
| U9 | GUIDE prose trimmed to the holds table and one paragraph | D9 | `GUIDE.md`, `PLAN.md` (same commit) | checker green; GUIDE ≤ 220 lines |

Forbidden: deleting any ledger row, any dated HISTORY record or any
`analysis/` file (archive, never delete; the evidence rule); changing any
number, date, ID or verdict while moving it; rewriting a released
CHANGELOG entry; editing `src/`, `tests/`, `benches/` or `xtask` (B.2.0's
and the clusters'); renaming a tool or fixture a row cites.

Done criteria: the checker passes with the dead-path check; every tracked
Markdown file is cited by at least one other tracked file or listed in
`analysis/README.md`; zero dangling paths in GUIDE, PLAN, PROCESS and
AGENTS; each of the seven working documents states its purpose in its first
paragraph and PLAN §5 agrees; sizes recorded in the closing ledger row
against the prediction below.

Prospective prediction, frozen before implementation: PLAN 1,508 → ≤ 950
lines; HISTORY 928 → ≤ 450 plus a 300–400 line recovered archive;
EXPERIMENTS 1,674 → ≤ 1,450 with 183 rows unchanged; AGENTS 447 → ≤ 280;
PROCESS 448 → ≤ 350; GUIDE 249 → ≤ 220; `analysis/` live set ≤ 58 files
with 13 archived; dangling paths in the four current documents 44 → 0;
tracked logo files 9 → 3. Most likely failure: the legacy tracker cannot be
recovered from history in one piece, in which case the resolution table
says so and B.2.0's comment hygiene, which deletes most of the citing
comments, is the remedy.

## 6. Handed to owners

| Owner | Item |
|---|---|
| B.2.0 (in progress) | T7 workspace text, T9 tools index and spec-file move, T11 configuration comments — this leaf takes the result |
| E.1 | re-run the census recipe (RAR-M52) on the release head; decide whether the ledger splits by phase once B and C have added their rows |
| E.3 | CHANGELOG entry style for 3.0.0 / 2.5.0: Added/Changed/Fixed/Removed only |
