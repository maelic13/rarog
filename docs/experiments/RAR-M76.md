# RAR-M76 — The 2.6.0 release reads: the C head against the Rarog 2.5.0 release at 3+0.03, 10+0.1 and four threads

Indexed under *5. Evaluation and data experiments* in [`docs/EXPERIMENTS.md`](../EXPERIMENTS.md).

- **Date / owner:** registered 2026-10-22, before any game. The maintainer's run, through `analysis/artifacts/rel260/run.ps1`, at the maintainer's request (2026-10-22: prepare the reads for an interim 2.6.0 release from the closed evaluation programme, ahead of PLAN's 3.0.0 at E.3).
- **Kind:** the release rule's reads. PLAN §4 gave 2.4.0 its rule (at least +40 Elo at STC over the previous release with the lower bound above +25, and positive lower bounds at LTC and four threads) and 2.5.0 adopted it against 2.4.0 (RAR-M64, RAR-M65, RAR-M66). These are the same three reads for 2.6.0 against 2.5.0, played head to head; the pool readings (RAR-M73, RAR-M74) compared the two only through two separate runs.
- **Arms:** A, the C head, `tools/test_engines/rarog-c11head-pext-pgo.exe` (`9750D3DE…`, `Rarog 2.6.0-dev+c11head`, 13,987,223 / EBF 2.565, rustc 1.99.1; recipe `analysis/arm_patches/a0146a3-rar-m73-c11head.patch`). B, the published 2.5.0 release asset, `D:\chess\engines\rarog\rarog-v2.5.0-windows-pext-native-pgo.exe` (`AF529F9E…`, `id name Rarog 2.5.0`, `bench 13` 11,171,726 / EBF 2.512 checked 2026-10-22; no build sidecar, so the wrapper notes that compiler equality is not checkable: immaterial at this effect size, as for RAR-M65).
- **Reads** (Colosseum match mode, pinned runner 0.3.0, UHO_Lichess_4852_v1 in random order, no adjudication, margin 20 ms; each driver in `analysis/artifacts/rel260/`):
  1. **STC:** `match-fixed.toml`, `3+0.03`, Hash 64, one thread, concurrency 14, **2,000 games**, seed 45 (`stc_match.ps1`); dry run at policy, `dry-run.json` **`B9174C4598800F0A458D0CCB6887AF3C1FDE1BFA31F99ACF0D9C40762CF47C3F`**.
  2. **LTC:** `match-fixed-ltc.toml`, `10+0.1`, otherwise as read 1, **1,000 games**, seed 46 (`ltc_match.ps1`, RAR-M65's conditions); `dry-run.json` **`9E6F49033D6D394186532549F5E4404B787E9160A7B196728B5FF6EAEF9B4B59`**.
  3. **4T:** `match-fixed.toml` at `3+0.03` with four threads each, Hash 512, three games at a time, each slot owning four physical cores (CPUs 2–9, 16–23, 24–31 in the dry run; RAR-M66's four-thread conditions), **1,000 games**, seed 47 (`mt4_match.ps1`); `dry-run.json` **`8ADFA0C3C66B40833CE0F6092D0DB09ADF7D42CB338C64A64E67ECCF99E378F6`**. Its first dry run was refused by the wrapper's policy check (the run file's per-engine `Hash=64`, `Threads=1` overrode the four-thread settings); the driver now sets each side's options and the plan matches policy in every checked field.
  Each `dry-run.json` was stable over two dry runs; each run's manifest must repeat its hash. The wrapper's `-SelfTest` proved the idle wait, the live stream and the stop on a failing read.
- **Reading:** Colosseum's report for each match (Elo with its interval, nElo, W-D-L, pentanomial, faults and time losses), trusted as reported; in-game speed from Colosseum CLI 0.4.0's `stats`.
- **Rule (2.4.0's, as 2.5.0 adopted it):** read 1 at least +40 Elo with its 95% lower bound above +25; reads 2 and 3 with their lower bounds above 0. Both sides are Rarog, so any engine fault or time loss above the harness's ceiling voids a read, which is repeated, not read.
- **PRE-REGISTERED PREDICTION (frozen 2026-10-22, before any game):**
  - Read 1, STC: **+150**, 80% band **[+115, +185]**. Basis: RAR-M73 against RAR-M64 put the C head +142 above the B.9 head (2.5.0's engine) through the pool; head to head against the B.9 head's lineage, B.10's 2.5.0-against-2.4.0 read (+272) sat close to its pool difference (+286).
  - Read 2, LTC: **+135**, band **[+95, +175]**: B.10 found the longer control narrowing the gap by about 12, not resolved.
  - Read 3, 4T: **+95**, band **[+55, +135]**: RAR-M74 against RAR-M66 (+94), the C head's thread scaling having fallen (RAR-M74).
  - The rule passes on all three: probability 0.97.
  - Most likely failure mode: the 4T read under the band, if the scaling loss weighs more head to head than against the pool.
- **Stop rule:** each read once, no early stop, no extension; a voided read is repeated.
- **Use:** if the rule passes, an interim 2.6.0 release is the maintainer's decision, cut by PROCESS's *Release* procedure from `master` after the work branch lands; GUIDE's *Next release* row and PLAN §4 record the decision. If it fails, no 2.6.0, and the failing read is reported with what it says.
- **Result:** not played.
- **Artifacts:** `analysis/artifacts/rel260/` (drivers, wrapper, dry-run logs); results `tools/results/rel260-{stc,ltc,4t}/`.
