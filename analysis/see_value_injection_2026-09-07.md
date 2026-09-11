# Neutral SEE values and normalized comparison — RAR-M29 / 4.11b.6

Entry `2c59911`; engine/test commit `46f1af2`. No SEE value was tuned and no
search caller changed. This step adds a board-owned `SeeValues` value object,
keeps production fixed at **100/320/330/500/900/20000**, and lets diagnostics
call the same full and threshold kernels with an explicit vector. There is no
trait object, function pointer, UCI option or runtime production setting.

This source inspection corrects the record: RAR-M19, PLAN, RAR-M27 and RAR-M28
described Rarog's SEE king sentinel as 32,000/`MATE_SCORE`. The board-local
function was actually **20,000** before this step. Kings are never legal SEE
victims, so the correction changes no legal exchange result, but the interface
and evidence now record the executable source rather than the evaluator's
separate `piece_value()` vector.

## Production identity and independent contracts

Existing `Board::see`, `see_ge` and `see_ge_quiet_aware` pass the constant
`PRODUCTION_SEE_VALUES` to the shared implementation. New explicit-value calls
take a small copyable value object. Production and explicit-production calls
agree on all **41** independent legal-tree fixtures. The normalized
100/300/300/500/900/20000 vector also agrees with independently scored expected
values on all 41 fixtures, including both colors, king legality, evolving pins,
en passant, underpromotions and a promoted recapturer captured again. Seven
Python oracle tests independently reproduce both scales; their king sentinel
is now corrected to 20,000.

Complete suites pass **270 debug / 271 release**, zero failures or ignores in
26 result groups per profile. The release-only process-garbage test explains
the one-test difference. `cargo fmt --check` and all-feature/all-target Clippy
pass. Rebuilt after those checks with `cargo build --release
--no-default-features`, the production `bench 13` remains exactly
**7,601,220 nodes / EBF 2.474**. That proves behavior identity for the immediate
4.11b.5 development baseline; its playing-strength qualification still belongs
to 4.11b.17.

## Live benchmark wire and answer contract

`cross-engine-board-v1` now defaults to the normalized vector. Before timing,
each adapter prints a canonical sorted set of its ten move/verdict pairs. The
coordinator refuses a different vector, a missing/duplicate answer, a differing
answer set, changed work quantum, nonzero exit, failed preflight, or a timed run
above 12% whole-host busy. Rarog, Basilisk and Reckless produced the same set:

```text
1:d5e6=1,1:e2a6=1,1:e5d7=0,1:e5f7=0,1:e5g6=0,
1:f3f6=0,1:f3h3=0,1:g2h3=1,4:f6e4=1,4:f6h5=1
```

This proves that the timed ten calls return the same booleans; it does not
claim that all three kernels agree on arbitrary chess positions. Rarog's
41-fixture oracle supplies the broader independent correctness check for its
injected interface.

The Rarog adapter also accepts a diagnostic-only `--see-values` argument.
Through the actual benchmark executable, the normal vector makes the defended
`Rxd5 cxd5` probe false at threshold zero; changing only rook from 500 to the
deliberately absurd **1** makes it true. The preflight outputs are retained
locally under `analysis/artifacts/see-normalized-20260907/` (Git-ignored).
The ten corpus verdicts happen not to change under that absurd vector, which is
why the dedicated independently known probe is necessary. This is a live-wire
test, not a proposed value candidate.

Basilisk already used the normalized vector and needed only a verdict reporter;
that temporary source patch was reverted after its binary was built. Reckless's
existing benchmark-only adapter now calls a `board-bench`-gated value-injection
method; its normal native engine remains 109/403/435/679/1242/0. The complete
patch against clean Reckless `91b56c2` is archived. Neither peer adapter changes
playing defaults, and neither is a reference correctness oracle.

## Normalized timing result

Ryzen 9 5950X, affinity mask 4, native optimized PEXT Rarog/Basilisk and native
Reckless, no PGO. Three cyclic rounds, each with 150 ms warm-up and eleven
150 ms samples. Every timed run stayed below the registered 12% host-busy cap.

| Engine | Median million captures/s | Three round medians | Range / median | Max within-run MAD |
|---|---:|---|---:|---:|
| Rarog | **44.923** | 45.167 / 44.923 / 39.687 | **12.20%** | 2.52% |
| Basilisk | **58.335** | 58.335 / 58.737 / 56.880 | 3.18% | 2.25% |
| Reckless | **40.823** | 40.823 / 39.529 / 41.834 | 5.65% | 0.90% |

Basilisk is 29.86% faster than Rarog by the median-of-round-medians; Rarog is
10.04% faster than Reckless. Treat the magnitudes as directional. Rarog's third
round was unusually slow despite 6.14% host busy, producing a 12.20% span; it
beat Reckless in two of three rounds and lost the third. The run passes its
prospective host-load rule, but it does not resolve a small speed difference.

The older 46.676/58.814/39.722 native-value row remains valid historical timing
but is **SUPERSEDED for cross-engine SEE ranking**. It mixed value vectors and
preceded Rarog's 4.11b.5 kernel repair, so comparing old and new rates cannot
attribute injection overhead or algorithm speed. The current result overturns
the earlier inability to rank this column: under the normalized ten-call
contract, Basilisk leads, then Rarog, then Reckless, with the Rarog/Reckless
gap qualified by the observed scatter. This microbenchmark is neither NPS nor
Elo; 4.11b.7 must measure SEE's actual HCE search share and caller frequencies.

## Reproduction and limitations

Machine-readable results, all nine raw runs, preflights, build logs, hashes,
compiler/source identities, complete peer patches and the engine verification
logs are under `analysis/artifacts/see-normalized-20260907/`. Verify without
timing:

```powershell
python tools/diag/verify_normalized_see.py analysis/artifacts/see-normalized-20260907
```

The exact build commands live in `source-manifest.json`. The measurement runner
is `tools/diag/normalized_see_compare.py`; use a fresh output directory. The
external binary paths in the run manifest are convenience paths, while their
hashes plus source heads, complete patches, compiler identities and build
commands are the reproducible recipe. The original full bundle also remains at
`D:/chess/results/see-normalized-20260907`.

Production fitting remains exclusively at 4.15.3–4.15.4. Later work must not
replace `PRODUCTION_SEE_VALUES` merely because normalized peer timing uses a
different vector. Any such change affects pruning, ordering, reductions and
history learning and requires its registered playing gate.
