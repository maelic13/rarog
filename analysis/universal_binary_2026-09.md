# Universal x86-64 binary — design record and deferral

**Status: OPTIONAL, not scheduled, may never be done.** Owner: PLAN `G.2`.
Written 2026-09-10 from the A.4 investigation, at `1eb3f23`. Nothing here is a
commitment to build it; it exists so the work does not have to be redone if the
triggers in section 6 ever fire.

The per-tier assets (`base`, `avx2`, `pext`, `arm64`) remain the shipped
product. The improvements that came out of this investigation and *are*
scheduled live in PLAN `A.4`.

---

## 1. The question

One Windows and one Linux x86-64 executable that selects its code path at
startup, so a user never chooses between `base`, `avx2` and `pext`. Two things
motivated it: a user who picks too conservatively loses real strength, and a
user who picks `pext` on the wrong AMD part gets our *slowest* engine.

## 2. What the tier ladder is worth (RAR-P20, measured)

Four independent PGO builds per tier, idle 5950X, pooled and interleaved:

| Comparison | Delta | 95% CI |
|---|---|---|
| null, `base` vs `base` | −0.15% | [−1.26%, +0.66%] |
| `avx2` over `base` | **+4.59%** | [+3.79%, +5.83%] |
| `pext` over `avx2` | **+2.45%** | [+2.32%, +2.62%] |

At RAR-M20's ~2 Elo per 1% NPS constant that is roughly 9 Elo and 5 Elo. The
ladder is real; the tiers are not decoration.

**Do not chain those two runs.** The same four `avx2` binaries read 3,087,416
n/s in RUN2 and 3,132,586 n/s in RUN3 minutes later — +1.46% for identical
bytes, from thermal and boost settling. Within-run interleaving absorbs it, so
each row above stands, but the compounded `base`→`pext` figure of about +7.2%
is an **estimate**. It needs its own direct run before anyone quotes it.

## 3. What was proven on this machine

A two-crate link prototype, MSVC, `x86_64-pc-windows-msvc`.

**Rust symbol isolation works.** Two `staticlib`s from one source with distinct
`-C metadata` and `-C target-feature`, each exporting one `extern "C"` entry,
linked by a baseline dispatcher: no duplicate-symbol errors, both entries ran
and returned different values (nothing folded), and disassembly found exactly
**one `pextq` in the whole image**, inside the pext tier. The registered
falsifier — duplicate Rust `std`/allocator symbols failing the link — did not
fire. The linker shared one copy of `std` and the final binary was 168 KB from
two 12.4 MB staticlibs.

**C symbols do NOT isolate, and they fail silently.** `-C metadata` does not
touch C symbol names. Giving both tiers a C function with the same name and a
tier-dependent `#define` produced this:

| Link order | base tier resolves to | pext tier resolves to |
|---|---|---|
| base first | base's C | base's C — **wrong** |
| pext first | pext's C | pext's C |

Both tiers use whichever copy the linker sees first. No error, no warning. This
matters because `build.rs` compiles Fathom with a tier-dependent
`TB_NO_HW_POP_COUNT`: if the v3 copy wins, the baseline tier ships `popcntq` and
`#UD`-crashes on a pre-Nehalem CPU inside Syzygy probing — exactly the defect
that shipped 15 illegal `popcntq` in the 2.3.0 and 2.3.1 baseline assets.

**The fix is better than Stockfish's, and it was validated.** Rather than
allowing multiple definitions and ordering the safe copy first, compile Fathom
into *none* of the tiers: leave `tb_*` undefined in each staticlib and link one
baseline copy at the final link. Undefined symbols in a staticlib resolve fine
at that stage. Rebuilt that way, the prototype linked and ran, and flipping the
single copy's `#define` moved **both** tiers together — one copy, controlled in
one place, with no ordering to get wrong. It also avoids giving one process
three independent `tb_init` states.

Nothing is lost by it: `grep` confirms no `linker-plugin-lto` anywhere, so
`lto = "fat"` is Rust-only and has never crossed into the C object.

## 4. How Stockfish does it

Read from `/d/code/stockfish/src/universal/` and `src/Makefile`, not from
memory.

- **Isolation by macro-renamed namespace.** The engine lives in `namespace
  Stockfish`; each tier compiles with `-DStockfish=Stockfish_x86_64_avx2`.
  Eight x86 tiers. Rust has no equivalent — you cannot rename a crate's module
  path from a flag — which is why `-C metadata` is the mechanism here.
- **They hit the same C collision and surrender to it.** The Makefile passes
  `-Wl,--allow-multiple-definition` and comments: *"Baseline build must come
  first in the final link (Windows uses the first copy of inline duplicates)."*
  Same first-wins semantics measured above, handled by ordering.
- **Per-tier static-initializer sections.** All eight tiers' C++ global
  constructors would otherwise run at startup, so each tier's init pointers go
  into their own section, walked via `__start_/__stop_` (ELF) or
  `getsectiondata` (Mach-O). **Rarog needs none of this** — it has zero eager
  static init; `ATTACKS` and every other static is `LazyLock`/`OnceLock`, so
  only the selected tier ever builds its tables.
- **PGO per tier**, with LTO assembly rewritten by `rewrite_asm_sections.awk`,
  and Intel SDE via `RUN_PREFIX` to train tiers the host cannot execute.
- **The NNUE net is embedded once** and shared across all eight tiers.
- Their toolchain is GCC/clang, never MSVC.
- **Exactly one model-based check exists in their entire source**:
  `__builtin_cpu_is("amd") && (bdver4 || znver1 || znver2)`. Everything else —
  all eight tiers, seventeen feature bits — is pure CPUID feature testing.
- **Why eight tiers:** 196 of their 249 ISA-specific lines are in `nnue/`.
  The ladder exists for int8/int16 inference. We have no NNUE.

## 5. Options considered, and their runtime cost

Size method: `.text` is 616/593/584 KB for base/avx2/pext, and the prototype
puts `std` + runtime at ~165 KB, so engine code is ~450 KB per tier.

| | (a) Fat binary | (b) Multiversioned kernels | (c) Launcher + siblings | (d) Embedded, cached to disk |
|---|---|---|---|---|
| Single file | yes | yes | **no** | yes |
| Startup | CPUID (~100 cycles) + ~3x relocations; against a measured 20–24 ms process launch, unmeasurable | none | +1 process spawn | writes ~2 MB on first run — **violates no-runtime-writes** |
| Size | ~1.9–2.0 MB vs 800 KB (~2.4x) | unchanged | sum of tiers | ~2 MB |
| Resident memory | unchanged (demand paging); ~+200–400 KB from relocated `.rdata` | unchanged | unchanged | unchanged |
| Hot path | **zero**, if the tier staticlib holds the whole engine including the UCI loop and only a thin dispatcher sits outside | **loses global v3 codegen everywhere else — measured at 4.59%** | zero | zero |
| New failure modes | C first-wins; per-region ISA check needed; PGO metadata trap | adds `#[target_feature]` unsafe call sites against the frozen unsafe floor | archive must stay intact | cache poisoning, AV, sandboxes |

**(a) is the design**, decided by RAR-P20 under a rule frozen before the run:
(b) forfeits 4.59%, and (c) and (d) each fail a fixed criterion outright.

## 6. Why it is deferred, and what would revive it

Deferred because it is worth approximately **zero Elo to a correctly-chosen
asset**, while the engine is ~250 Elo behind on search and ~330 on evaluation.
It also adds two permanent *silent* failure modes to a project whose one
recurring defect is checks that did not check what they appeared to. And most
of the concrete user harm is addressable without it — see PLAN `A.4.2`.

It becomes worth doing if any of these fire:

1. **NNUE lands (Phase F)** and a wider tier ladder starts paying in inference
   throughput. This is the strongest trigger: it is why Stockfish has eight.
2. **Evidence that asset choice is costing us rated strength** — e.g. a CCRL
   listing measured on a conservative build.
3. **The advisories in A.4.2 prove insufficient**, because unattended testers
   never read startup output.

## 7. Traps for whoever picks this up

- **PGO matches profile data to functions by mangled symbol, and `-C metadata`
  is part of that symbol.** It is the same mechanism that keeps the tier copies
  apart. A tier staticlib built with different metadata than its training binary
  silently receives **no profile at all** — no error, ~5–10% slower, ships fine.
  Train per tier with identical metadata, then prove liveness with the
  absurd-value method: a deliberately mismatched build must measurably lose NPS.
  *This was reasoned from the mangling mechanism, not measured. Verify it first.*
- **An instrumented universal binary only trains the tier the host selects.**
  The others get no profile. Release builds must train each tier separately.
- **`--native` and universal are incoherent** — one compiles for this CPU, the
  other selects at runtime. Refuse the pair rather than accepting it silently.
- **`verify-isa` cannot express "this region is baseline"** when scanning a
  linked fat binary. Run it per tier on the staticlib inputs before linking,
  plus a whole-image union check after.
- **A tier this host cannot execute needs an emulator to PGO-train**, exactly
  Stockfish's `RUN_PREFIX` arrangement. Another reason not to grow the ladder
  before NNUE gives it a reason.

## 8. Sources

RAR-P20 (tier value, and the frozen decision rule it resolved); RAR-P19 (the
`cc` macOS `-fprofile-use` gap found alongside); RAR-M20 (~2 Elo per 1% NPS);
`/d/code/stockfish/src/universal/`, `src/Makefile`; `build.rs`;
`xtask/src/main.rs` (`rustflags`, `tier_features`, `build_with_pgo`);
`src/main.rs` (why the old startup CPU guard could never fire).
