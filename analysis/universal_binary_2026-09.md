# Universal x86-64 binary — design record and deferral

**Status: OPTIONAL, not scheduled, may never be done.** Owner: PLAN `G.2`.
Written 2026-09-10 from the A.4 investigation. Nothing here is a
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
RAR-P22 made that run: `pext` over `base` **+6.65%** [+6.48%, +6.89%], so
chaining overstated the ladder by half a point.

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

## 8. The specialized assets carry no startup CPU guard

Moved from `PROCESS.md` on 2026-09-14.

There is deliberately no startup CPU guard inside specialized assets. When the
compiler is told that BMI2/AVX2/FMA are mandatory, ordinary feature-detection
macros fold those checks to true and the guard is removed. A working
in-process guard would require baseline-compiled CPUID code to execute before
specialized code, adding a separate dispatch boundary. The current design is
close to the specialized-binary model: users choose `x86-64`, `avx2`, `pext`
or `arm64`, the README states exact requirements, and release tooling
disassembles each asset to enforce the promise. If a single universal binary
becomes a product goal, G.2 may add a Stockfish-style baseline dispatcher.

## 9. Leads from Basilisk's built universal binary (2026-10-29)

Basilisk (C++, clang, lld) built the fat binary of section 5 (a) and shipped
it after two pooled NPS reads on this 5950X. Its record is
`D:/code/basilisk/analysis/universal_binary_2026-10.md` (sections 12 and 13
describe the form it built) and its BAS-P20 and BAS-P21. What follows are leads for G.2, not
evidence for Rarog: a different language, compiler and linker. Each needs
checking here before it is relied on.

- **The hot path is free only once function placement is controlled.**
  - BAS-P20: the universal binary read **−0.58%** [−0.82%, −0.33%] against
    the `pext` asset and failed its no-regression gate.
  - The `pext` tier's hot code was byte-identical in both: `negamax` was
    the same 4,713 instructions, and 8 adjacent inversions separated the
    function orders of 560 functions.
  - The cause was placement. lld's call-graph sort put a shared 16-byte
    runtime stub (`operator delete`) inside the tier build's hot cluster.
    In the universal link the three tiers share that stub and it landed
    elsewhere, so every hot function after it moved 16 bytes, from offset 0
    to 48 of a cache line.
  - BAS-P21 aligned every tier function to 64 bytes
    (`-falign-functions=64`, +44 KB). Then:
    - the relink alone (universal against its own tier build): −0.13%
      [−0.51%, +0.26%];
    - the alignment alone: −0.16% [−0.56%, +0.24%];
    - the gate: −0.29% [−0.73%, +0.16%] at six cycles, PASS.
  - The Rust analogue to try is LLVM's `-C llvm-args=-align-all-functions=6`
    (log2 of the alignment) or nightly `-Z min-function-alignment=64`.
    Neither has been checked on this toolchain.
- **A pool of builds does not randomise link layout.**
  - Builds of one source differ only in their PGO profile. Basilisk's four
    builds all shared one placement, so the pool's interval did not cover
    layout.
  - A change that relinks code needs a layout control, such as an alignment
    or padding sweep, not more builds.
  - Read it as separate contrasts, each between two pools:
    - **hot:** universal against the per-tier asset, the gate;
    - **tier:** the tier build against the asset (Basilisk +0.04%);
    - **place:** universal against its own tier build;
    - **align:** the alignment on its own.
  - Basilisk's `nps_read` reads three pools with named contrasts in one
    pass, about 9 minutes at step 1 against 30 to 65 for separate reads
    (`875ec6e`). Rarog's `tools/nps_read.py` reads two arms only.
- **The diagnosis that found it** compared the two images directly, in
  three steps:
  - each hot function's instructions, modulo addresses;
  - the function order;
  - each hot function's start offset modulo 64.

  It is cheap, and it separates codegen from placement before any pooled
  read.
- **Symbol isolation leaks through LTO.**
  - After ThinLTO, every function one module calls in another is promoted
    to a global (407 in Basilisk's Windows `pext` tier). About 48 runtime
    template functions also stay global COMDATs.
  - Each would resolve first-wins across tiers, silently: the C-symbol trap
    of section 3, from another route.
  - Basilisk renamed every symbol each tier defines, in its post-LTO
    objects, to `__basilisk_<tier>_<name>`, and the script fails if
    anything else stays global. Only type identities and identical
    section-named data stay shared.
  - Lead for Rust: list each tier staticlib's defined external symbols
    (`llvm-nm --defined-only --extern-only`) and intersect them across
    tiers. Any name defined in two tiers with tier-dependent code is the
    same hazard, whatever `-C metadata` does to mangling. Generic
    instantiations of `std` code are the suspects.
- **Renaming after PGO removes this record's first trap.**
  - Basilisk trains and links each tier as its ordinary PGO build, under
    unchanged names in its own build directory. It renames only the kept
    post-LTO objects (`-Wl,--Xlink=-lldsavetemps` on Windows,
    `-plugin-opt=save-temps` with gold on Linux).
  - So no profile is ever matched against renamed symbols. That removes
    section 7's `-C metadata` trap instead of guarding against it.
  - The tier build then read +0.04% [−0.12%, +0.19%] against the asset: the
    tier object is the measured binary's code.
  - Whether rustc's fat-LTO output can be kept per tier in this way is the
    first thing for G.2 to establish.
- **The ladder agrees with ours.**
  - Basilisk, hand-crafted evaluation, no intrinsics:
    - `avx2` over `portable` +5.04%;
    - `pext` over `avx2` +2.82%;
    - `pext` over `portable` +8.49%, read directly.
  - Rarog: +4.59% and +2.45% (RAR-P20), +6.65% direct (RAR-P22).
  - For an engine of this kind, the ladder is worth 7 to 8.5% on Zen 3,
    before NNUE.
- **Dispatcher details Basilisk settled:**
  - `pext` only where it is fast: not AMD family 15h or 17h (Excavator,
    Zen 1 and 2), read from CPUID leaf 1, since MSVC has no
    `__builtin_cpu_is`.
  - An environment override (`BASILISK_ISA`) forces a tier, for a
    forced-tier bench identity test and for measuring every tier on one
    host. An unsupported forced tier exits 1 with an `info string`.
  - The start-up banner names the tier, and `id name` does not change,
    because rating lists key on it.
  - Intel SDE selection runs on CI cover Intel models only. The Zen rule
    gets a truth-table unit test instead.
- **The ISA floor matches ours.**
  - LLVM emits `tzcnt` on the baseline, and it runs as `bsf` before BMI1.
    Rarog already allows it (`TZCNT_IS_BASELINE_SAFE` in `xtask`).
  - Basilisk's portable floor is SSE3, because MSYS2's static libc++ is
    built for `nocona`. Rust's precompiled `std` targets plain x86-64, so
    Rarog's floor need not move.
  - Basilisk checks the image by symbol once every tier function carries its
    tier's name, which works the same with lld and GNU ld. That answers
    section 7's "`verify-isa` cannot express this region is baseline".
- **Size and shape.**
  - Size: 3.5 MB on Windows (2.2 MB per tier asset), 5.1 MB static on Linux.
  - When the first read failed, the release went back to per-tier assets
    while the build stayed in the tree (`d601807`).
  - A CPU refused by an asset is told which per-tier asset it should run
    (`a575ab8`).

None of this fires section 6's triggers. It lowers the expected cost and
risk of the design. It also names the first two checks if G.2 starts:
whether Rust's post-LTO tier objects can be kept and renamed, and whether
function alignment makes the relink placement-neutral here.

## 10. Sources

RAR-P20 (tier value, and the frozen decision rule it resolved); RAR-P19 (the
`cc` macOS `-fprofile-use` gap found alongside); RAR-M20 (~2 Elo per 1% NPS);
`/d/code/stockfish/src/universal/`, `src/Makefile`; `build.rs`;
`xtask/src/main.rs` (`rustflags`, `tier_features`, `build_with_pgo`);
`src/main.rs` (why the old startup CPU guard could never fire).
