# Reproducing the 2026-09-05 board comparison

**Historical recipe.** Its native-value SEE lines remain exact raw evidence but
are superseded for cross-engine SEE ranking by RAR-M29's normalized comparison:
`see_value_injection_2026-09-07.md`. The other five workload columns are not
superseded. Do not edit the embedded raw outputs below.

This is the retained RAR-M20 evidence, integrated into PLAN/GUIDE on
2026-09-05. The machine-readable sibling files are under
[artifacts/board-audit-20260905](artifacts/board-audit-20260905).
The complete original bundle and archived executables remain at
D:/chess/results/board-audit-20260905. This document embeds the full adapter
patch, runner, manifest and raw rounds; it does not require the old draft or
candidate branch. No expensive run is needed to validate the saved numbers.
Do not run the timing runner inside this historical
directory: it writes manifest.json and round files. Use a new output directory.

## Verify existing evidence without rebuilding or timing

Python 3.11 with python-chess (import chess version 1.11.2 was used):

```powershell
python -B D:\chess\results\board-audit-20260905\verify_bundle.py
```

This checks archived binary/patch hashes, every raw timing row, nine rounds,
work counts, aggregate medians, pairwise ordering, independent legal move
sets/exchange arithmetic, and the 26 draft PLAN/GUIDE leaf identifiers.
The live roadmap checker is intentionally not applied to fragment drafts.

## Sources and exact historical build configuration

Use the source revisions in manifest.json. Never switch or overwrite an active
collaborator's checkout to reproduce this run. Prepare separate checkouts when
needed, then point the same build commands at those paths. Full source hashes,
toolchains, flags and binary hashes are embedded below.

The historical working directories were D:\code\rarog, D:\code\basilisk and
D:\code\Reckless. Rarog board sources/benchmark remained unchanged through
a0aeb68, but a rebuild on a newer whole-engine revision must be labelled with
that actual revision, never silently called the original ca03a46 build.

Reckless was initially clean at 91b56c2. Its complete benchmark-only change is
reckless-board-adapter.patch (included below). Apply it only to a clean
reference checkout; the current D:\code\Reckless already has these changes.
No playing defaults or board algorithms were modified. The four changed files
are Cargo.toml, src/lib.rs, benches/board.rs and src/tools/board_bench.rs.
The last two are new files. The adapter is gated by board-bench.

```powershell
# In a fresh isolated Reckless checkout, once only:
git apply --check D:\chess\results\board-audit-20260905\reckless-board-adapter.patch
if ($LASTEXITCODE -ne 0) { throw "Adapter does not apply" }
git apply D:\chess\results\board-audit-20260905\reckless-board-adapter.patch
if ($LASTEXITCODE -ne 0) { throw "Adapter application failed" }
```

Exact build commands used (run each in its corresponding source directory;
adjust only the source/build path if reproducing in isolation):

```powershell
# Rarog: no Cargo features; native BMI2/PEXT; no PGO.
$boardSavedRustflags = $env:RUSTFLAGS
try {
    $env:RUSTFLAGS = '-C target-cpu=native --cfg rarog_pext'
    cargo bench --locked --bench board --no-run --target-dir "$env:TEMP\rarog-board-timing-20260905" -j 2
    if ($LASTEXITCODE -ne 0) { throw "Rarog board build failed" }
} finally {
    $env:RUSTFLAGS = $boardSavedRustflags
}

# Basilisk: Clang/MinGW, Release, native/PEXT, thin LTO; no PGO.
cmake -S D:\code\basilisk -B "$env:TEMP\basilisk-board-timing-20260905" -G Ninja -DCMAKE_BUILD_TYPE=Release -DUSE_PEXT=ON -DCOMP=clang
if ($LASTEXITCODE -ne 0) { throw "Basilisk configure failed" }
cmake --build "$env:TEMP\basilisk-board-timing-20260905" --target board_performance_test -j 2
if ($LASTEXITCODE -ne 0) { throw "Basilisk board build failed" }

# Reckless: native magic/AVX2, no Syzygy and no PGO.
$boardSavedRustflags = $env:RUSTFLAGS
try {
    $env:RUSTFLAGS = '-C target-cpu=native'
    cargo bench --locked --no-default-features --features board-bench --bench board --no-run --target-dir "$env:TEMP\reckless-board-timing-20260905" -j 2
    if ($LASTEXITCODE -ne 0) { throw "Reckless board build failed" }
} finally {
    $env:RUSTFLAGS = $boardSavedRustflags
}
```

Rust was rustc 1.97.1 / LLVM 22.1.6, x86_64-pc-windows-msvc. Both Rust release
profiles use fat LTO. Basilisk was Clang 22.1.8, x86_64-w64-windows-gnu,
-O3 -march=native -mbmi2 -flto=thin. Rarog feature fingerprint was [].
Do not run all-feature tests/clippy between a build and measuring its artifact.

Reckless's build script selects networks/v60-7f587dfb.nnue unless EVALFILE is
set, and embeds/builds its library. The adapter only initializes attack lookup
tables, not NNUE inference; it uses NullBoardObserver. --no-default-features
also avoids the Syzygy build script writing generated src/bindings.rs.
The build emitted existing rlib/cdylib output-collision warnings; the selected
board benchmark executable was unique and the build succeeded. This is not
a claim that the whole reference project is warning-clean.

The exact selected executables are in the manifest and are also copied to
binaries/. Hashes identify the actual measured files. Rebuild hashes can
depend on toolchain, source paths and build metadata; prove matching source,
features, preflight and known behavior, and record the new hash. Do not claim
bit-identical reproduction solely because a file has the same name.

## Timing protocol

After all three exact builds finish, resolve the actual benchmark paths and
update the bins mapping in a COPY of run_comparison.py in a fresh output
directory. Record source/features/toolchains there before interpreting results.
Do not select a random release exe or reuse a stale ordinary engine binary.

The runner pins itself to Windows affinity mask 4 and each child inherits it.
It runs three cyclic engine orders, each native harness performing 150 ms
warmup plus eleven 150 ms samples per workload. It rejects any observed
total-host-busy interval above 12%. There are six rows but SEE is native-value
diagnostic only. No binaries run concurrently. Actual observed host load is
recorded; this desktop run does not establish a sub-percent noise floor.

The runner validates return status, preflight PASS, six rows and exact work
vector 128/10/128/10/197281/4597. It retains each raw output and calculates the
median of three round medians plus their range. It parses the board harness
format, not search bench diagnostic counters (those still require the
repository's standard counter parser).

The native harness reports median and MAD, not all eleven raw sample values.
This evidence has all emitted data and the nine round outputs; do not claim
per-sample observations that were not emitted. A stronger future instrument
should retain those individual samples too.

## Focused probes

audit_probes.rs records the two SEE examples, Unicode panic and fullmove
overflow. Archived debug/release outputs are probe-debug.txt and
probe-release.txt; original executables and the differential bridge are in
audit-binaries/. The Rust same-square capture oracle uses Rarog move
generation. verify_bundle.py independently repeats the score arithmetic with
python-chess legal moves.

audit_bridge.rs and audit_differential.py preserve the earlier random legal
move/FEN/hinted-make comparison recipe and seeds. The Python differential
currently names the original temporary bridge exe; point a COPY at the
archived bridge or a freshly compiled one when reusing the test. The historical
bridge/probes were diagnostic binaries, not the timed benchmark.

To compile a new diagnostic probe without editing Rarog source, build the
library in a dedicated target directory, then pass the resulting unique rlib
to rustc via --extern rarog=... and -L dependency=...; include --edition=2024.
Debug and release must be distinct outputs. For the archived optimized
test/unwind rlib librarog-f0ccc9b297cc5f61.rlib, the working command included
-C opt-level=3 -C lto=fat. The other release rlib used an incompatible panic
strategy for catch_unwind. Do not pick an rlib blindly: inspect the build
profile and use an unwind-compatible library for a panic-catching probe.
Recompiling diagnostic probes is not needed to verify the archived findings.

The original wider audit also checked 107,648 magic occupancy/ray cases and
selected repository suites. Their totals are reported in BOARD_AUDIT.md;
this bundle does not claim to contain a standalone reconstruction of every
original shell invocation. PEXT exhaustive validation remains future work.

## Historical snapshot and adopted ownership

The original external bundle retains snapshots/PLAN.a0aeb68.md and
snapshots/GUIDE.a0aeb68.md for historical comparison. Integration preserved
their existing statuses and v2 endgame order. Current owners are PLAN 4.11b,
5.2.1/5.2.5/5.3.4 and 6.4.3. Follow current GUIDE rather than the old drafts.
The external verify_bundle.py checks the historical bundle and draft IDs;
it does not certify current roadmap synchronization. For that use:

```powershell
python -B tools/diag/check_guide.py --next 10
```

The following sections embed the complete adapter patch, runner, manifest and
all nine raw outputs so the essential measurement evidence is also in one
portable document. Keep sibling probe/validator files with the bundle.


## Embedded evidence

### reckless-board-adapter.patch (base64; byte-exact)

The original patch has meaningful whitespace-only context lines. It is stored
as base64 here and in `artifacts/board-audit-20260905/reckless-board-adapter.patch.b64`
so Git text normalization cannot alter its bytes. Decode and verify before
applying to an isolated Reckless checkout:

```powershell
@'
import base64, hashlib, json, pathlib, tempfile
root = pathlib.Path("analysis/artifacts/board-audit-20260905")
raw = base64.b64decode((root / "reckless-board-adapter.patch.b64").read_text())
manifest = json.loads((root / "manifest.json").read_text())
assert hashlib.sha256(raw).hexdigest() == manifest["adapter_patch_sha256"]
output = pathlib.Path(tempfile.gettempdir()) / "rarog-reckless-board-adapter.patch"
output.write_bytes(raw)
print(output)
'@ | python -
```

```text
ZGlmZiAtLWdpdCBhL0NhcmdvLnRvbWwgYi9DYXJnby50b21sDQppbmRleCAzYWQyOTcyLi44Njk3
NmM3IDEwMDY0NA0KLS0tIGEvQ2FyZ28udG9tbA0KKysrIGIvQ2FyZ28udG9tbA0KQEAgLTEyLDYg
KzEyLDggQEAgY3JhdGUtdHlwZSA9IFsiY2R5bGliIiwgInJsaWIiXQ0KIGRlZmF1bHQgPSBbInN5
enlneSJdDQogc3l6eWd5ID0gW10NCiBzcHNhID0gW10NCisjIFN0YW5kYWxvbmUgYm9hcmQgdGhy
b3VnaHB1dCBpbnN0cnVtZW50OyBkb2VzIG5vdCBjaGFuZ2UgZW5naW5lIGRlZmF1bHRzLg0KK2Jv
YXJkLWJlbmNoID0gW10NCiANCiBbcHJvZmlsZS5kZXZdDQogb3B0LWxldmVsID0gMw0KQEAgLTM0
LDMgKzM2LDggQEAgbGliYyA9ICIwLjIuMTc1Ig0KIHdlYi10aW1lID0gIjEiDQogd2FzbS1iaW5k
Z2VuID0gIjAuMiINCiBqcy1zeXMgPSAiMC4zIg0KKw0KK1tbYmVuY2hdXQ0KK25hbWUgPSAiYm9h
cmQiDQoraGFybmVzcyA9IGZhbHNlDQorcmVxdWlyZWQtZmVhdHVyZXMgPSBbImJvYXJkLWJlbmNo
Il0NCmRpZmYgLS1naXQgYS9zcmMvbGliLnJzIGIvc3JjL2xpYi5ycw0KaW5kZXggMDliYjkwYy4u
MWIzOWJhNSAxMDA2NDQNCi0tLSBhL3NyYy9saWIucnMNCisrKyBiL3NyYy9saWIucnMNCkBAIC0y
NCw2ICsyNCwxMCBAQCBtb2QgdHlwZXM7DQogDQogbW9kIHRvb2xzOw0KIA0KKyNbY2ZnKGZlYXR1
cmUgPSAiYm9hcmQtYmVuY2giKV0NCisjW3BhdGggPSAidG9vbHMvYm9hcmRfYmVuY2gucnMiXQ0K
K3B1YiBtb2QgYm9hcmRfYmVuY2g7DQorDQogI1tjZmcobm90KHRhcmdldF9hcmNoID0gIndhc20z
MiIpKV0NCiBtb2QgdWNpOw0KIA0KZGlmZiAtLWdpdCBhL2JlbmNoZXMvYm9hcmQucnMgYi9iZW5j
aGVzL2JvYXJkLnJzDQpuZXcgZmlsZSBtb2RlIDEwMDY0NA0KLS0tIC9kZXYvbnVsbA0KKysrIGIv
YmVuY2hlcy9ib2FyZC5ycw0KQEAgLTAsMCArMSwzIEBADQorZm4gbWFpbigpIHsNCisgICAgcmVj
a2xlc3M6OmJvYXJkX2JlbmNoOjpydW4oKTsNCit9DQpkaWZmIC0tZ2l0IGEvc3JjL3Rvb2xzL2Jv
YXJkX2JlbmNoLnJzIGIvc3JjL3Rvb2xzL2JvYXJkX2JlbmNoLnJzDQpuZXcgZmlsZSBtb2RlIDEw
MDY0NA0KLS0tIC9kZXYvbnVsbA0KKysrIGIvc3JjL3Rvb2xzL2JvYXJkX2JlbmNoLnJzDQpAQCAt
MCwwICsxLDI5OSBAQA0KKy8vISBGcm96ZW4gY3Jvc3MtZW5naW5lLWJvYXJkLXYxIGFkYXB0ZXIg
Zm9yIHRoZSAyMDI2LTA5LTA1IFJhcm9nIGF1ZGl0Lg0KKy8vISBTRUUgaXMgbmF0aXZlLXZhbHVl
IGRpYWdub3N0aWMgb25seTsgZml2ZSBvdGhlciBjb2x1bW5zIGZvbGxvdyB0aGUgY29udHJhY3Qu
DQorLy8hIE5vcm1hbCBib2FyZCBib29ra2VlcGluZyByZW1haW5zIGVuYWJsZWQuIE5OVUUgYXJp
dGhtZXRpYyBpcyBkaXNjb25uZWN0ZWQuDQorDQordXNlIHN0ZDo6aGludDo6YmxhY2tfYm94Ow0K
K3VzZSBzdGQ6OnRpbWU6OntEdXJhdGlvbiwgSW5zdGFudH07DQorDQordXNlIGNyYXRlOjpib2Fy
ZDo6e0JvYXJkLCBOdWxsQm9hcmRPYnNlcnZlcn07DQordXNlIGNyYXRlOjp0eXBlczo6TW92ZUxp
c3Q7DQorDQorY29uc3QgV0FSTVVQOiBEdXJhdGlvbiA9IER1cmF0aW9uOjpmcm9tX21pbGxpcygx
NTApOw0KK2NvbnN0IFNBTVBMRVM6IHVzaXplID0gMTE7DQorY29uc3QgU0FNUExFX1RJTUU6IER1
cmF0aW9uID0gRHVyYXRpb246OmZyb21fbWlsbGlzKDE1MCk7DQorDQorY29uc3QgQkVOQ0hNQVJL
X0ZFTlM6ICZbKCZzdHIsICZzdHIpXSA9ICZbDQorICAgICgic3RhcnRwb3MiLCAicm5icWtibnIv
cHBwcHBwcHAvOC84LzgvOC9QUFBQUFBQUC9STkJRS0JOUiB3IEtRa3EgLSAwIDEiKSwNCisgICAg
KCJraXdpcGV0ZSIsICJyM2syci9wMXBwcXBiMS9ibjJwbnAxLzNQTjMvMXAyUDMvMk4yUTFwL1BQ
UEJCUFBQL1IzSzJSIHcgS1FrcSAtIDAgMSIpLA0KKyAgICAoIm1pZGdhbWUiLCAicm5icTFrMXIv
cHBwcDFwcHAvNHBuMi84LzFiMVBQMy8yTjJOMi9QUFAyUFBQL1IxQlFLQjFSIHcgS1EgLSAyIDUi
KSwNCisgICAgKCJlbmRnYW1lIiwgIjgvMnA1LzNwNC9LUDVyLzgvOC84LzdrIHcgLSAtIDAgMSIp
LA0KKyAgICAoImluLWNoZWNrIiwgInJuYnFrYjFyL3BwcHAxcHBwLzVuMi80cDJRLzJCMVAzLzgv
UFBQUDFQUFAvUk5CMUsxTlIgYiBLUWtxIC0gMyAzIiksDQorXTsNCisNCitjb25zdCBFWFBFQ1RF
RF9PUFM6IFt1NjQ7IDZdID0gWzEyOCwgMTAsIDEyOCwgMTAsIDE5N18yODEsIDRfNTk3XTsNCisN
CitzdHJ1Y3QgQmVuY2hSZXN1bHQgew0KKyAgICBsYWJlbDogJidzdGF0aWMgc3RyLA0KKyAgICB1
bml0OiAmJ3N0YXRpYyBzdHIsDQorICAgIHNhbXBsZXM6IFZlYzxmNjQ+LA0KKyAgICBvcHNfcGVy
X2l0ZXI6IHU2NCwNCisgICAgaXRlcmF0aW9uczogdTY0LA0KK30NCisNCitpbXBsIEJlbmNoUmVz
dWx0IHsNCisgICAgZm4gbWVkaWFuKCZzZWxmKSAtPiBmNjQgew0KKyAgICAgICAgbGV0IG11dCBz
b3J0ZWQgPSBzZWxmLnNhbXBsZXMuY2xvbmUoKTsNCisgICAgICAgIHNvcnRlZC5zb3J0X2J5KGY2
NDo6dG90YWxfY21wKTsNCisgICAgICAgIHNvcnRlZFtzb3J0ZWQubGVuKCkgLyAyXQ0KKyAgICB9
DQorDQorICAgIGZuIG1hZCgmc2VsZikgLT4gZjY0IHsNCisgICAgICAgIGxldCBtZWQgPSBzZWxm
Lm1lZGlhbigpOw0KKyAgICAgICAgbGV0IG11dCBkZXY6IFZlYzxmNjQ+ID0gc2VsZi5zYW1wbGVz
Lml0ZXIoKS5tYXAofHN8IChzIC0gbWVkKS5hYnMoKSkuY29sbGVjdCgpOw0KKyAgICAgICAgZGV2
LnNvcnRfYnkoZjY0Ojp0b3RhbF9jbXApOw0KKyAgICAgICAgZGV2W2Rldi5sZW4oKSAvIDJdDQor
ICAgIH0NCisNCisgICAgZm4gc3ByZWFkX3BjdCgmc2VsZikgLT4gZjY0IHsNCisgICAgICAgIDEw
MC4wICogc2VsZi5tYWQoKSAvIHNlbGYubWVkaWFuKCkNCisgICAgfQ0KK30NCisNCitwdWIgZm4g
cnVuKCkgew0KKyAgICBjcmF0ZTo6bG9va3VwOjppbml0aWFsaXplKCk7DQorICAgIGxldCBib2Fy
ZHM6IFZlYzxCb2FyZD4gPSBCRU5DSE1BUktfRkVOUy5pdGVyKCkubWFwKHwoXywgZmVuKXwgQm9h
cmQ6OmZyb21fZmVuKGZlbikudW53cmFwKCkpLmNvbGxlY3QoKTsNCisNCisgICAgbGV0IG11dCBj
YXB0dXJlX2JvYXJkcyA9IGJvYXJkcy5jbG9uZSgpOw0KKyAgICBsZXQgbXV0IG11dGFibGVfYm9h
cmRzID0gYm9hcmRzLmNsb25lKCk7DQorICAgIGxldCBtdXQgc2VlX2JvYXJkcyA9IGJvYXJkcy5j
bG9uZSgpOw0KKyAgICBsZXQgbXV0IHNpbXVsYXRpb25fYm9hcmRzID0gYm9hcmRzLmNsb25lKCk7
DQorICAgIGxldCBtdXQgcGVyZnRfYm9hcmQgPSBCb2FyZDo6c3RhcnRpbmdfcG9zaXRpb24oKTsN
CisNCisgICAgew0KKyAgICAgICAgbGV0IG1lYXN1cmVkID0gWw0KKyAgICAgICAgICAgIGxlZ2Fs
X21vdmVnZW4oJmJvYXJkcyksDQorICAgICAgICAgICAgY2FwdHVyZV9nZW4oJm11dCBjYXB0dXJl
X2JvYXJkcyksDQorICAgICAgICAgICAgbWFrZV91bm1ha2UoJm11dCBtdXRhYmxlX2JvYXJkcyks
DQorICAgICAgICAgICAgc2VlX2NhcHR1cmVzKCZtdXQgc2VlX2JvYXJkcyksDQorICAgICAgICAg
ICAgcGVyZnQoJm11dCBwZXJmdF9ib2FyZCwgNCksDQorICAgICAgICAgICAgZ2FtZV9zaW11bGF0
aW9uKCZtdXQgc2ltdWxhdGlvbl9ib2FyZHMpLA0KKyAgICAgICAgXTsNCisgICAgICAgIGxldCBt
dXQgb2sgPSB0cnVlOw0KKyAgICAgICAgZm9yIChpLCAoJmdvdCwgJndhbnQpKSBpbiBtZWFzdXJl
ZC5pdGVyKCkuemlwKEVYUEVDVEVEX09QUy5pdGVyKCkpLmVudW1lcmF0ZSgpIHsNCisgICAgICAg
ICAgICBpZiBnb3QgIT0gd2FudCB7DQorICAgICAgICAgICAgICAgIGVwcmludGxuISgid29yayBt
aXNtYXRjaCBmb3Igd29ya2xvYWQge2l9OiBleHBlY3RlZCB7d2FudH0sIHJlY2VpdmVkIHtnb3R9
Iik7DQorICAgICAgICAgICAgICAgIG9rID0gZmFsc2U7DQorICAgICAgICAgICAgfQ0KKyAgICAg
ICAgfQ0KKyAgICAgICAgYXNzZXJ0IShvaywgInByZWZsaWdodCBmYWlsZWQ6IHdvcmsgcXVhbnRh
IGRvIG5vdCBtYXRjaCB0aGUgY29udHJhY3QiKTsNCisgICAgfQ0KKw0KKyAgICBwcmVmbGlnaHRf
c2VtYW50aWNzKCZib2FyZHMpOw0KKyAgICBpZiBzdGQ6OmVudjo6YXJncygpLmFueSh8YXJnfCBh
cmcgPT0gIi0tcHJlZmxpZ2h0LW9ubHkiKSB7DQorICAgICAgICBwcmludGxuISgicHJlZmxpZ2h0
OiBQQVNTIik7DQorICAgICAgICByZXR1cm47DQorICAgIH0NCisNCisgICAgbGV0IHJlc3VsdHMg
PSBbDQorICAgICAgICBtZWFzdXJlKCJsZWdhbCBtb3ZlcyIsICJtb3ZlcyIsIEVYUEVDVEVEX09Q
U1swXSwgfHwgbGVnYWxfbW92ZWdlbigmYm9hcmRzKSksDQorICAgICAgICBtZWFzdXJlKCJsZWdh
bCBjYXB0dXJlcyIsICJtb3ZlcyIsIEVYUEVDVEVEX09QU1sxXSwgfHwgY2FwdHVyZV9nZW4oJm11
dCBjYXB0dXJlX2JvYXJkcykpLA0KKyAgICAgICAgbWVhc3VyZSgibWFrZS91bm1ha2UiLCAibW92
ZXMiLCBFWFBFQ1RFRF9PUFNbMl0sIHx8IG1ha2VfdW5tYWtlKCZtdXQgbXV0YWJsZV9ib2FyZHMp
KSwNCisgICAgICAgIG1lYXN1cmUoInRocmVzaG9sZCBTRUUiLCAiY2FwdHVyZXMiLCBFWFBFQ1RF
RF9PUFNbM10sIHx8IHNlZV9jYXB0dXJlcygmbXV0IHNlZV9ib2FyZHMpKSwNCisgICAgICAgIG1l
YXN1cmUoInBlcmZ0KDQpIHN0YXJ0cG9zIiwgIm5vZGVzIiwgRVhQRUNURURfT1BTWzRdLCB8fCBw
ZXJmdCgmbXV0IHBlcmZ0X2JvYXJkLCA0KSksDQorICAgICAgICBtZWFzdXJlKCJ0d28tcGx5IHNp
bXVsYXRpb24iLCAibW92ZXMiLCBFWFBFQ1RFRF9PUFNbNV0sIHx8IGdhbWVfc2ltdWxhdGlvbigm
bXV0IHNpbXVsYXRpb25fYm9hcmRzKSksDQorICAgIF07DQorDQorICAgIHByaW50bG4hKCk7DQor
ICAgIHByaW50bG4hKCJSZWNrbGVzcyBib2FyZCBiZW5jaG1hcmsiKTsNCisgICAgcHJpbnRsbiEo
InByb2ZpbGU6IGNyb3NzLWVuZ2luZS1ib2FyZC12MSIpOw0KKyAgICBwcmludGxuISgiU0VFOiBu
YXRpdmUgdmFsdWVzIDEwOS80MDMvNDM1LzY3OS8xMjQyLzA7IE5PVCBjb250cmFjdC1jb21wYXJh
YmxlIik7DQorICAgIHByaW50bG4hKCJvYnNlcnZlcjogTnVsbEJvYXJkT2JzZXJ2ZXI7IG5hdGl2
ZSB0aHJlYXQvcGluL2tleSBzdGF0ZSByZXRhaW5lZCIpOw0KKyAgICBwcmludGxuISgicG9zaXRp
b25zOiB7fSIsIEJFTkNITUFSS19GRU5TLmxlbigpKTsNCisgICAgcHJpbnRsbiEoDQorICAgICAg
ICAic2FtcGxlczoge1NBTVBMRVN9IHgge30gbXMgYWZ0ZXIgYSB7fSBtcyB3YXJtLXVwIChtZWRp
YW4gKy8tIE1BRCkiLA0KKyAgICAgICAgU0FNUExFX1RJTUUuYXNfbWlsbGlzKCksDQorICAgICAg
ICBXQVJNVVAuYXNfbWlsbGlzKCkNCisgICAgKTsNCisgICAgcHJpbnRsbiEoInByZWZsaWdodDog
UEFTUyIpOw0KKyAgICBwcmludGxuISgpOw0KKyAgICBwcmludGxuISgNCisgICAgICAgICJ7Ojwy
Mn0gezo+MTV9IHs6PjE1fSB7Oj4xMH0gezo+MTJ9IHs6PjEyfSIsDQorICAgICAgICAid29ya2xv
YWQiLCAiZXN0aW1hdGUgb3BzL3MiLCAiTUFEIG9wcy9zIiwgIk1BRCAlIiwgIm9wcy9pdGVyIiwg
InRvdGFsIGl0ZXJzIg0KKyAgICApOw0KKw0KKyAgICBmb3IgcmVzdWx0IGluICZyZXN1bHRzIHsN
CisgICAgICAgIHByaW50bG4hKA0KKyAgICAgICAgICAgICJ7OjwyMn0gezo+MTUuMH0gezo+MTUu
MH0gezo+OS4yfSUgezo+MTJ9IHs6PjEyfSB7fSIsDQorICAgICAgICAgICAgcmVzdWx0LmxhYmVs
LA0KKyAgICAgICAgICAgIHJlc3VsdC5tZWRpYW4oKSwNCisgICAgICAgICAgICByZXN1bHQubWFk
KCksDQorICAgICAgICAgICAgcmVzdWx0LnNwcmVhZF9wY3QoKSwNCisgICAgICAgICAgICByZXN1
bHQub3BzX3Blcl9pdGVyLA0KKyAgICAgICAgICAgIHJlc3VsdC5pdGVyYXRpb25zLA0KKyAgICAg
ICAgICAgIHJlc3VsdC51bml0DQorICAgICAgICApOw0KKyAgICB9DQorfQ0KKw0KK2ZuIGNhbGli
cmF0ZV9iYXRjaDxGPih3b3JrbG9hZDogJm11dCBGKSAtPiB1NjQNCit3aGVyZQ0KKyAgICBGOiBG
bk11dCgpIC0+IHU2NCwNCit7DQorICAgIGNvbnN0IFRBUkdFVDogRHVyYXRpb24gPSBEdXJhdGlv
bjo6ZnJvbV9taWxsaXMoMSk7DQorICAgIGNvbnN0IE1BWF9CQVRDSDogdTY0ID0gMV8wMDBfMDAw
Ow0KKyAgICBjb25zdCBQUk9CRVM6IHUzMiA9IDMyOw0KKw0KKyAgICBsZXQgc3RhcnQgPSBJbnN0
YW50Ojpub3coKTsNCisgICAgZm9yIF8gaW4gMC4uUFJPQkVTIHsNCisgICAgICAgIGJsYWNrX2Jv
eCh3b3JrbG9hZCgpKTsNCisgICAgfQ0KKyAgICBsZXQgcGVyX2l0ZXIgPSBzdGFydC5lbGFwc2Vk
KCkgLyBQUk9CRVM7DQorICAgIGlmIHBlcl9pdGVyLmlzX3plcm8oKSB7DQorICAgICAgICByZXR1
cm4gTUFYX0JBVENIOw0KKyAgICB9DQorICAgIGxldCBiYXRjaCA9IFRBUkdFVC5hc19uYW5vcygp
IC8gcGVyX2l0ZXIuYXNfbmFub3MoKTsNCisgICAgdTY0Ojp0cnlfZnJvbShiYXRjaCkudW53cmFw
X29yKE1BWF9CQVRDSCkuY2xhbXAoMSwgTUFYX0JBVENIKQ0KK30NCisNCitmbiBtZWFzdXJlPEY+
KGxhYmVsOiAmJ3N0YXRpYyBzdHIsIHVuaXQ6ICYnc3RhdGljIHN0ciwgZXhwZWN0ZWQ6IHU2NCwg
bXV0IHdvcmtsb2FkOiBGKSAtPiBCZW5jaFJlc3VsdA0KK3doZXJlDQorICAgIEY6IEZuTXV0KCkg
LT4gdTY0LA0KK3sNCisgICAgbGV0IHdhcm11cF9zdGFydCA9IEluc3RhbnQ6Om5vdygpOw0KKyAg
ICB3aGlsZSB3YXJtdXBfc3RhcnQuZWxhcHNlZCgpIDwgV0FSTVVQIHsNCisgICAgICAgIGJsYWNr
X2JveCh3b3JrbG9hZCgpKTsNCisgICAgfQ0KKw0KKyAgICBsZXQgYmF0Y2ggPSBjYWxpYnJhdGVf
YmF0Y2goJm11dCB3b3JrbG9hZCk7DQorDQorICAgIGxldCBtdXQgc2FtcGxlcyA9IFZlYzo6d2l0
aF9jYXBhY2l0eShTQU1QTEVTKTsNCisgICAgbGV0IG11dCBpdGVyYXRpb25zID0gMHU2NDsNCisg
ICAgZm9yIF8gaW4gMC4uU0FNUExFUyB7DQorICAgICAgICBsZXQgc3RhcnQgPSBJbnN0YW50Ojpu
b3coKTsNCisgICAgICAgIGxldCBtdXQgb3BzID0gMHU2NDsNCisgICAgICAgIGxldCBtdXQgaXRl
cnMgPSAwdTY0Ow0KKyAgICAgICAgd2hpbGUgc3RhcnQuZWxhcHNlZCgpIDwgU0FNUExFX1RJTUUg
ew0KKyAgICAgICAgICAgIGZvciBfIGluIDAuLmJhdGNoIHsNCisgICAgICAgICAgICAgICAgb3Bz
ICs9IGJsYWNrX2JveCh3b3JrbG9hZCgpKTsNCisgICAgICAgICAgICB9DQorICAgICAgICAgICAg
aXRlcnMgKz0gYmF0Y2g7DQorICAgICAgICB9DQorICAgICAgICBsZXQgZWxhcHNlZCA9IHN0YXJ0
LmVsYXBzZWQoKS5hc19zZWNzX2Y2NCgpOw0KKyAgICAgICAgYXNzZXJ0X2VxIShvcHMsIGV4cGVj
dGVkICogaXRlcnMsICJ7bGFiZWx9IGRyaWZ0ZWQgb2ZmIGl0cyBmcm96ZW4gd29yayBxdWFudHVt
IG1pZC1tZWFzdXJlbWVudCIpOw0KKyAgICAgICAgc2FtcGxlcy5wdXNoKG9wcyBhcyBmNjQgLyBl
bGFwc2VkKTsNCisgICAgICAgIGl0ZXJhdGlvbnMgKz0gaXRlcnM7DQorICAgIH0NCisNCisgICAg
QmVuY2hSZXN1bHQgeyBsYWJlbCwgdW5pdCwgc2FtcGxlcywgb3BzX3Blcl9pdGVyOiBleHBlY3Rl
ZCwgaXRlcmF0aW9ucyB9DQorfQ0KKw0KK2ZuIGxlZ2FsX21vdmVnZW4oYm9hcmRzOiAmW0JvYXJk
XSkgLT4gdTY0IHsNCisgICAgbGV0IG11dCB0b3RhbCA9IDB1NjQ7DQorICAgIGZvciBib2FyZCBp
biBib2FyZHMgew0KKyAgICAgICAgbGV0IG1vdmVzID0gZ2VuZXJhdGVfbGVnYWxfbW92ZWxpc3Qo
YmxhY2tfYm94KGJvYXJkKSk7DQorICAgICAgICB0b3RhbCArPSBtb3Zlcy5sZW4oKSBhcyB1NjQ7
DQorICAgICAgICBibGFja19ib3goJm1vdmVzKTsNCisgICAgfQ0KKyAgICB0b3RhbA0KK30NCisN
CitmbiBjYXB0dXJlX2dlbihib2FyZHM6ICZtdXQgW0JvYXJkXSkgLT4gdTY0IHsNCisgICAgbGV0
IG11dCB0b3RhbCA9IDB1NjQ7DQorICAgIGZvciBib2FyZCBpbiBib2FyZHMgew0KKyAgICAgICAg
bGV0IG1vdmVzID0gZ2VuZXJhdGVfY2FwdHVyZXMoYmxhY2tfYm94KGJvYXJkKSk7DQorICAgICAg
ICB0b3RhbCArPSBtb3Zlcy5sZW4oKSBhcyB1NjQ7DQorICAgICAgICBibGFja19ib3goJm1vdmVz
KTsNCisgICAgfQ0KKyAgICB0b3RhbA0KK30NCisNCitmbiBtYWtlX3VubWFrZShib2FyZHM6ICZt
dXQgW0JvYXJkXSkgLT4gdTY0IHsNCisgICAgbGV0IG11dCBvcHMgPSAwdTY0Ow0KKyAgICBmb3Ig
Ym9hcmQgaW4gYm9hcmRzIHsNCisgICAgICAgIGxldCBtb3ZlczogTW92ZUxpc3QgPSBnZW5lcmF0
ZV9sZWdhbF9tb3ZlbGlzdChib2FyZCk7DQorICAgICAgICBmb3IgZW50cnkgaW4gbW92ZXMuaXRl
cigpIHsNCisgICAgICAgICAgICBsZXQgbXYgPSBlbnRyeS5tdjsNCisgICAgICAgICAgICBib2Fy
ZC5tYWtlX21vdmUobXYsICZtdXQgTnVsbEJvYXJkT2JzZXJ2ZXIpOw0KKyAgICAgICAgICAgIGJs
YWNrX2JveCgmYm9hcmQpOw0KKyAgICAgICAgICAgIGJvYXJkLnVuZG9fbW92ZShtdik7DQorICAg
ICAgICAgICAgb3BzICs9IDE7DQorICAgICAgICB9DQorICAgIH0NCisgICAgb3BzDQorfQ0KKw0K
K2ZuIHNlZV9jYXB0dXJlcyhib2FyZHM6ICZtdXQgW0JvYXJkXSkgLT4gdTY0IHsNCisgICAgbGV0
IG11dCBvcHMgPSAwdTY0Ow0KKyAgICBmb3IgYm9hcmQgaW4gYm9hcmRzIHsNCisgICAgICAgIGxl
dCBjYXB0dXJlcyA9IGdlbmVyYXRlX2NhcHR1cmVzKGJvYXJkKTsNCisgICAgICAgIGZvciBlbnRy
eSBpbiBjYXB0dXJlcy5pdGVyKCkgew0KKyAgICAgICAgICAgIGxldCBtdiA9IGVudHJ5Lm12Ow0K
KyAgICAgICAgICAgIGJsYWNrX2JveChib2FyZC5zZWUobXYsIDApKTsNCisgICAgICAgICAgICBv
cHMgKz0gMTsNCisgICAgICAgIH0NCisgICAgfQ0KKyAgICBvcHMNCit9DQorDQorZm4gZ2FtZV9z
aW11bGF0aW9uKGJvYXJkczogJm11dCBbQm9hcmRdKSAtPiB1NjQgew0KKyAgICBsZXQgbXV0IG9w
cyA9IDB1NjQ7DQorICAgIGZvciBib2FyZCBpbiBib2FyZHMgew0KKyAgICAgICAgbGV0IG1vdmVz
OiBNb3ZlTGlzdCA9IGdlbmVyYXRlX2xlZ2FsX21vdmVsaXN0KGJvYXJkKTsNCisgICAgICAgIGZv
ciBlbnRyeSBpbiBtb3Zlcy5pdGVyKCkgew0KKyAgICAgICAgICAgIGxldCBtdiA9IGVudHJ5Lm12
Ow0KKyAgICAgICAgICAgIGJvYXJkLm1ha2VfbW92ZShtdiwgJm11dCBOdWxsQm9hcmRPYnNlcnZl
cik7DQorICAgICAgICAgICAgbGV0IHJlcGxpZXMgPSBnZW5lcmF0ZV9sZWdhbF9tb3ZlbGlzdChi
b2FyZCk7DQorICAgICAgICAgICAgb3BzICs9IHJlcGxpZXMubGVuKCkgYXMgdTY0Ow0KKyAgICAg
ICAgICAgIGJsYWNrX2JveCgmcmVwbGllcyk7DQorICAgICAgICAgICAgYm9hcmQudW5kb19tb3Zl
KG12KTsNCisgICAgICAgIH0NCisgICAgfQ0KKyAgICBvcHMNCit9DQorDQorZm4gZ2VuZXJhdGVf
bGVnYWxfbW92ZWxpc3QoYm9hcmQ6ICZCb2FyZCkgLT4gTW92ZUxpc3Qgew0KKyAgICBib2FyZC5n
ZW5lcmF0ZV9hbGxfbW92ZXMoKQ0KK30NCisNCisvLyBUaGUgZnJvemVuIHYxIGNvcnB1cyBoYXMg
bm8gcXVpZXQgcHJvbW90aW9ucy4gUHJlZmxpZ2h0IHByb3ZlcyB0aGF0IHRoZQ0KKy8vIG5hdGl2
ZSBub2lzeSBsaXN0IGlzIGV4YWN0bHkgaXRzIGxlZ2FsIGNhcHR1cmUgbGlzdCwgc28gbm8gdGlt
ZWQgZmlsdGVyaW5nDQorLy8gb3IgY29udGFpbmVyIGNvcHlpbmcgaXMgYWRkZWQgdG8gdGhlIG5h
dGl2ZSBBUEkuDQorZm4gZ2VuZXJhdGVfY2FwdHVyZXMoYm9hcmQ6ICZtdXQgQm9hcmQpIC0+IE1v
dmVMaXN0IHsNCisgICAgbGV0IG11dCBtb3ZlcyA9IE1vdmVMaXN0OjpuZXcoKTsNCisgICAgYm9h
cmQuYXBwZW5kX25vaXN5X21vdmVzKCZtdXQgbW92ZXMpOw0KKyAgICBtb3Zlcw0KK30NCisNCitm
biBwZXJmdChib2FyZDogJm11dCBCb2FyZCwgZGVwdGg6IHUzMikgLT4gdTY0IHsNCisgICAgaWYg
ZGVwdGggPT0gMCB7DQorICAgICAgICByZXR1cm4gMTsNCisgICAgfQ0KKyAgICBsZXQgbW92ZXMg
PSBib2FyZC5nZW5lcmF0ZV9hbGxfbW92ZXMoKTsNCisgICAgaWYgZGVwdGggPT0gMSB7DQorICAg
ICAgICByZXR1cm4gbW92ZXMubGVuKCkgYXMgdTY0Ow0KKyAgICB9DQorICAgIGxldCBtdXQgbm9k
ZXMgPSAwOw0KKyAgICBmb3IgZW50cnkgaW4gbW92ZXMuaXRlcigpIHsNCisgICAgICAgIGJvYXJk
Lm1ha2VfbW92ZShlbnRyeS5tdiwgJm11dCBOdWxsQm9hcmRPYnNlcnZlcik7DQorICAgICAgICBu
b2RlcyArPSBwZXJmdChib2FyZCwgZGVwdGggLSAxKTsNCisgICAgICAgIGJvYXJkLnVuZG9fbW92
ZShlbnRyeS5tdik7DQorICAgIH0NCisgICAgbm9kZXMNCit9DQorDQorZm4gcHJlZmxpZ2h0X3Nl
bWFudGljcyhib2FyZHM6ICZbQm9hcmRdKSB7DQorICAgIGZvciAoaW5kZXgsIG9yaWdpbmFsKSBp
biBib2FyZHMuaXRlcigpLmVudW1lcmF0ZSgpIHsNCisgICAgICAgIGxldCBtdXQgYm9hcmQgPSBv
cmlnaW5hbC5jbG9uZSgpOw0KKyAgICAgICAgbGV0IGJlZm9yZSA9IGJvYXJkLnRvX2ZlbigpOw0K
KyAgICAgICAgbGV0IGhhc2ggPSBib2FyZC5oYXNoKCk7DQorICAgICAgICBsZXQgbGVnYWwgPSBi
b2FyZC5nZW5lcmF0ZV9hbGxfbW92ZXMoKTsNCisgICAgICAgIGxldCBjYXB0dXJlcyA9IGdlbmVy
YXRlX2NhcHR1cmVzKCZtdXQgYm9hcmQpOw0KKyAgICAgICAgYXNzZXJ0IShjYXB0dXJlcy5pdGVy
KCkuYWxsKHxlbnRyeXwgZW50cnkubXYuaXNfY2FwdHVyZSgpKSk7DQorICAgICAgICBsZXQgbXV0
IG5vaXN5OiBWZWM8Xz4gPSBjYXB0dXJlcy5pdGVyKCkubWFwKHxlbnRyeXwgZW50cnkubXYudG9f
dWNpKCZib2FyZCkpLmNvbGxlY3QoKTsNCisgICAgICAgIGxldCBtdXQgZmlsdGVyZWQ6IFZlYzxf
PiA9DQorICAgICAgICAgICAgbGVnYWwuaXRlcigpLmZpbHRlcih8ZW50cnl8IGVudHJ5Lm12Lmlz
X2NhcHR1cmUoKSkubWFwKHxlbnRyeXwgZW50cnkubXYudG9fdWNpKCZib2FyZCkpLmNvbGxlY3Qo
KTsNCisgICAgICAgIG5vaXN5LnNvcnQoKTsNCisgICAgICAgIGZpbHRlcmVkLnNvcnQoKTsNCisg
ICAgICAgIGFzc2VydF9lcSEobm9pc3ksIGZpbHRlcmVkKTsNCisgICAgICAgIGxldCBtdXQgbmFt
ZXM6IFZlYzxfPiA9IGxlZ2FsLml0ZXIoKS5tYXAofGVudHJ5fCBlbnRyeS5tdi50b191Y2koJmJv
YXJkKSkuY29sbGVjdCgpOw0KKyAgICAgICAgbmFtZXMuc29ydCgpOw0KKyAgICAgICAgcHJpbnRs
biEoImxlZ2FsW3tpbmRleH1dOiB7fSIsIG5hbWVzLmpvaW4oIiwiKSk7DQorICAgICAgICBmb3Ig
ZW50cnkgaW4gbGVnYWwuaXRlcigpIHsNCisgICAgICAgICAgICBhc3NlcnQhKGJvYXJkLmlzX2xl
Z2FsKGVudHJ5Lm12KSk7DQorICAgICAgICAgICAgYm9hcmQubWFrZV9tb3ZlKGVudHJ5Lm12LCAm
bXV0IE51bGxCb2FyZE9ic2VydmVyKTsNCisgICAgICAgICAgICBib2FyZC51bmRvX21vdmUoZW50
cnkubXYpOw0KKyAgICAgICAgICAgIGFzc2VydF9lcSEoYm9hcmQudG9fZmVuKCksIGJlZm9yZSk7
DQorICAgICAgICAgICAgYXNzZXJ0X2VxIShib2FyZC5oYXNoKCksIGhhc2gpOw0KKyAgICAgICAg
fQ0KKyAgICB9DQorfQ0K
```

### run_comparison.py

```python
import ctypes, datetime, hashlib, json, pathlib, re, statistics, subprocess, time
OUT=pathlib.Path(__file__).resolve().parent
bins={
"rarog":pathlib.Path(r"C:\Users\macur\AppData\Local\Temp\rarog-board-timing-20260905\release\deps\board-12ca175dd86ea15e.exe"),
"basilisk":pathlib.Path(r"C:\Users\macur\AppData\Local\Temp\basilisk-board-timing-20260905\board_performance_test.exe"),
"reckless":pathlib.Path(r"C:\Users\macur\AppData\Local\Temp\reckless-board-timing-20260905\release\deps\board-8ae3e21b46b77bc3.exe")}
k=ctypes.WinDLL("kernel32",use_last_error=True)
k.GetCurrentProcess.restype=ctypes.c_void_p
k.SetProcessAffinityMask.argtypes=[ctypes.c_void_p,ctypes.c_size_t]
assert k.SetProcessAffinityMask(k.GetCurrentProcess(),4),ctypes.get_last_error()
def times():
 vals=[ctypes.c_ulonglong() for _ in range(3)]
 assert k.GetSystemTimes(*(ctypes.byref(v) for v in vals))
 return [v.value for v in vals]
def busy(a,b):
 d=[y-x for x,y in zip(a,b)]
 return 100*(d[1]+d[2]-d[0])/(d[1]+d[2])
pattern=re.compile(r"^(legal moves|legal captures|make/unmake|threshold SEE|perft\(4\) startpos|two-ply simulation)\s+(\d+)\s+(\d+)\s+([\d.]+)%\s+(\d+)\s+(\d+)")
expected=[128,10,128,10,197281,4597]
manifest={"date":datetime.datetime.now(datetime.timezone.utc).isoformat(),"cpu":"AMD Ryzen 9 5950X","affinity_mask":4,"logical_processor":2,"sampling":"150ms warmup + 11x150ms per workload; 3 cyclic rounds; median of printed round medians","reject_host_busy_above_percent":12,"pgo":False,"binaries":{n:{"path":str(p),"sha256":hashlib.sha256(p.read_bytes()).hexdigest()}for n,p in bins.items()},"runs":[]}
(OUT/"manifest.json").write_text(json.dumps(manifest,indent=2))
for r in range(3):
 order=list(bins)[r:]+list(bins)[:r]
 for n in order:
  a=times();start=time.monotonic()
  p=subprocess.run([str(bins[n])],capture_output=True,text=True,timeout=60,creationflags=0x08000000)
  load=busy(a,times());elapsed=time.monotonic()-start
  (OUT/f"round{r+1}-{n}.txt").write_text(p.stdout+p.stderr)
  assert p.returncode==0,(n,p.returncode,p.stderr)
  assert "preflight: PASS" in p.stdout
  rows=[]
  for line in p.stdout.splitlines():
   m=pattern.match(line)
   if m: rows.append({"workload":m[1],"ops_per_sec":int(m[2]),"mad":int(m[3]),"mad_percent":float(m[4]),"ops_per_iter":int(m[5]),"iterations":int(m[6])})
  assert len(rows)==6,(n,rows,p.stdout)
  assert [x["ops_per_iter"]for x in rows]==expected
  run={"round":r+1,"engine":n,"elapsed":elapsed,"host_busy_percent":load,"rows":rows}
  manifest["runs"].append(run);(OUT/"manifest.json").write_text(json.dumps(manifest,indent=2))
  print(f"round {r+1} {n}: host busy {load:.2f}%; "+", ".join(f'{x["workload"]}={x["ops_per_sec"]/1e6:.3f}M'for x in rows),flush=True)
  assert load<=12,("contaminated run",n,load)
summary={}
for n in bins:
 summary[n]={}
 for i in range(6):
  rows=[r["rows"][i]for r in manifest["runs"]if r["engine"]==n]
  vals=[v["ops_per_sec"]for v in rows];med=statistics.median(vals)
  summary[n][rows[0]["workload"]]={"median_ops_per_sec":med,"round_medians":vals,"round_range_percent":100*(max(vals)-min(vals))/med,"max_within_run_mad_percent":max(x["mad_percent"]for x in rows)}
manifest["summary"]=summary
(OUT/"manifest.json").write_text(json.dumps(manifest,indent=2))
print(json.dumps(summary,indent=2),flush=True)
```

### manifest.json

```json
{
  "date": "2026-09-05T11:08:29.183433+00:00",
  "cpu": "AMD Ryzen 9 5950X",
  "affinity_mask": 4,
  "logical_processor": 2,
  "sampling": "150ms warmup + 11x150ms per workload; 3 cyclic rounds; median of printed round medians",
  "reject_host_busy_above_percent": 12,
  "pgo": false,
  "binaries": {
    "rarog": {
      "path": "C:\\Users\\macur\\AppData\\Local\\Temp\\rarog-board-timing-20260905\\release\\deps\\board-12ca175dd86ea15e.exe",
      "sha256": "40f8fa53874bd0e8155b846ce6eba30995c158bf6edf093016f8f367f031c636",
      "archived_path": "D:\\chess\\results\\board-audit-20260905\\binaries\\rarog-board.exe"
    },
    "basilisk": {
      "path": "C:\\Users\\macur\\AppData\\Local\\Temp\\basilisk-board-timing-20260905\\board_performance_test.exe",
      "sha256": "7eeaff0cf52dc304eee5e9d68c2982662023b8050c3d8394f578d8b3386c81b8",
      "archived_path": "D:\\chess\\results\\board-audit-20260905\\binaries\\basilisk-board.exe"
    },
    "reckless": {
      "path": "C:\\Users\\macur\\AppData\\Local\\Temp\\reckless-board-timing-20260905\\release\\deps\\board-8ae3e21b46b77bc3.exe",
      "sha256": "449897a137b43ff9154484d906f57fdb9a482a197151ee3a85b244f9b748d70a",
      "archived_path": "D:\\chess\\results\\board-audit-20260905\\binaries\\reckless-board.exe"
    }
  },
  "runs": [
    {
      "round": 1,
      "engine": "rarog",
      "elapsed": 10.905999999959022,
      "host_busy_percent": 6.858218656147743,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 446835610,
          "mad": 398857,
          "mad_percent": 0.09,
          "ops_per_iter": 128,
          "iterations": 5786924
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 98203955,
          "mad": 342960,
          "mad_percent": 0.35,
          "ops_per_iter": 10,
          "iterations": 15898542
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 42880416,
          "mad": 571762,
          "mad_percent": 1.33,
          "ops_per_iter": 128,
          "iterations": 526750
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 46362836,
          "mad": 360229,
          "mad_percent": 0.78,
          "ops_per_iter": 10,
          "iterations": 7443904
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 270582440,
          "mad": 5964154,
          "mad_percent": 2.2,
          "ops_per_iter": 197281,
          "iterations": 2221
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 334454431,
          "mad": 24208463,
          "mad_percent": 7.24,
          "ops_per_iter": 4597,
          "iterations": 119280
        }
      ]
    },
    {
      "round": 1,
      "engine": "basilisk",
      "elapsed": 10.875,
      "host_busy_percent": 6.535771920387305,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 641375549,
          "mad": 383916,
          "mad_percent": 0.06,
          "ops_per_iter": 128,
          "iterations": 8299776
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 120044251,
          "mad": 140070,
          "mad_percent": 0.12,
          "ops_per_iter": 10,
          "iterations": 18867628
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 56420047,
          "mad": 289578,
          "mad_percent": 0.51,
          "ops_per_iter": 128,
          "iterations": 699660
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 58638662,
          "mad": 107069,
          "mad_percent": 0.18,
          "ops_per_iter": 10,
          "iterations": 9604425
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 382726065,
          "mad": 7836417,
          "mad_percent": 2.05,
          "ops_per_iter": 197281,
          "iterations": 3029
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 506161729,
          "mad": 14986168,
          "mad_percent": 2.96,
          "ops_per_iter": 4597,
          "iterations": 180612
        }
      ]
    },
    {
      "round": 1,
      "engine": "reckless",
      "elapsed": 10.890999999945052,
      "host_busy_percent": 6.724045185583647,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 315357805,
          "mad": 460036,
          "mad_percent": 0.15,
          "ops_per_iter": 128,
          "iterations": 4072187
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 53646306,
          "mad": 89794,
          "mad_percent": 0.17,
          "ops_per_iter": 10,
          "iterations": 8640060
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 23494488,
          "mad": 33145,
          "mad_percent": 0.14,
          "ops_per_iter": 128,
          "iterations": 300900
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 34603389,
          "mad": 1495736,
          "mad_percent": 4.32,
          "ops_per_iter": 10,
          "iterations": 5619930
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 177943984,
          "mad": 1366565,
          "mad_percent": 0.77,
          "ops_per_iter": 197281,
          "iterations": 1436
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 237570427,
          "mad": 6853828,
          "mad_percent": 2.88,
          "ops_per_iter": 4597,
          "iterations": 83480
        }
      ]
    },
    {
      "round": 2,
      "engine": "basilisk",
      "elapsed": 10.875,
      "host_busy_percent": 7.046483482002779,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 642645770,
          "mad": 290149,
          "mad_percent": 0.05,
          "ops_per_iter": 128,
          "iterations": 8310024
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 120137542,
          "mad": 215488,
          "mad_percent": 0.18,
          "ops_per_iter": 10,
          "iterations": 19519024
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 53741047,
          "mad": 2411718,
          "mad_percent": 4.49,
          "ops_per_iter": 128,
          "iterations": 680016
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 58983916,
          "mad": 233613,
          "mad_percent": 0.4,
          "ops_per_iter": 10,
          "iterations": 9483340
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 374068307,
          "mad": 13529803,
          "mad_percent": 3.62,
          "ops_per_iter": 197281,
          "iterations": 3052
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 513536852,
          "mad": 9544855,
          "mad_percent": 1.86,
          "ops_per_iter": 4597,
          "iterations": 171054
        }
      ]
    },
    {
      "round": 2,
      "engine": "reckless",
      "elapsed": 10.89000000001397,
      "host_busy_percent": 9.16913148695886,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 341709244,
          "mad": 168434,
          "mad_percent": 0.05,
          "ops_per_iter": 128,
          "iterations": 4418304
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 61704779,
          "mad": 163130,
          "mad_percent": 0.26,
          "ops_per_iter": 10,
          "iterations": 9467848
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 18168284,
          "mad": 1646452,
          "mad_percent": 9.06,
          "ops_per_iter": 128,
          "iterations": 254951
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 40518759,
          "mad": 366683,
          "mad_percent": 0.9,
          "ops_per_iter": 10,
          "iterations": 6478130
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 173339067,
          "mad": 1258203,
          "mad_percent": 0.73,
          "ops_per_iter": 197281,
          "iterations": 1417
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 246625657,
          "mad": 11206429,
          "mad_percent": 4.54,
          "ops_per_iter": 4597,
          "iterations": 84436
        }
      ]
    },
    {
      "round": 2,
      "engine": "rarog",
      "elapsed": 10.891000000061467,
      "host_busy_percent": 6.251686606098768,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 450020873,
          "mad": 287778,
          "mad_percent": 0.06,
          "ops_per_iter": 128,
          "iterations": 5820224
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 97706249,
          "mad": 333571,
          "mad_percent": 0.34,
          "ops_per_iter": 10,
          "iterations": 15852672
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 42520810,
          "mad": 683248,
          "mad_percent": 1.61,
          "ops_per_iter": 128,
          "iterations": 534232
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 47290470,
          "mad": 462700,
          "mad_percent": 0.98,
          "ops_per_iter": 10,
          "iterations": 7635342
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 279844251,
          "mad": 2115395,
          "mad_percent": 0.76,
          "ops_per_iter": 197281,
          "iterations": 2257
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 351808501,
          "mad": 5651799,
          "mad_percent": 1.61,
          "ops_per_iter": 4597,
          "iterations": 121028
        }
      ]
    },
    {
      "round": 3,
      "engine": "reckless",
      "elapsed": 10.905999999959022,
      "host_busy_percent": 7.177996422182469,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 339844427,
          "mad": 400528,
          "mad_percent": 0.12,
          "ops_per_iter": 128,
          "iterations": 4338981
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 61597216,
          "mad": 60838,
          "mad_percent": 0.1,
          "ops_per_iter": 10,
          "iterations": 9961608
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 23737926,
          "mad": 319704,
          "mad_percent": 1.35,
          "ops_per_iter": 128,
          "iterations": 295740
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 39722097,
          "mad": 358141,
          "mad_percent": 0.9,
          "ops_per_iter": 10,
          "iterations": 6308880
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 180350887,
          "mad": 329152,
          "mad_percent": 0.18,
          "ops_per_iter": 197281,
          "iterations": 1474
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 256257625,
          "mad": 1691080,
          "mad_percent": 0.66,
          "ops_per_iter": 4597,
          "iterations": 89490
        }
      ]
    },
    {
      "round": 3,
      "engine": "rarog",
      "elapsed": 10.875,
      "host_busy_percent": 6.586395902596819,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 447131123,
          "mad": 664847,
          "mad_percent": 0.15,
          "ops_per_iter": 128,
          "iterations": 5783440
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 100675265,
          "mad": 193008,
          "mad_percent": 0.19,
          "ops_per_iter": 10,
          "iterations": 16348068
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 41952617,
          "mad": 1051183,
          "mad_percent": 2.51,
          "ops_per_iter": 128,
          "iterations": 523912
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 46676180,
          "mad": 1509425,
          "mad_percent": 3.23,
          "ops_per_iter": 10,
          "iterations": 7492404
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 273741073,
          "mad": 7636994,
          "mad_percent": 2.79,
          "ops_per_iter": 197281,
          "iterations": 2215
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 362488899,
          "mad": 1407429,
          "mad_percent": 0.39,
          "ops_per_iter": 4597,
          "iterations": 123222
        }
      ]
    },
    {
      "round": 3,
      "engine": "basilisk",
      "elapsed": 10.875,
      "host_busy_percent": 6.650500696034847,
      "rows": [
        {
          "workload": "legal moves",
          "ops_per_sec": 646286043,
          "mad": 357443,
          "mad_percent": 0.06,
          "ops_per_iter": 128,
          "iterations": 8344485
        },
        {
          "workload": "legal captures",
          "ops_per_sec": 120405617,
          "mad": 287756,
          "mad_percent": 0.24,
          "ops_per_iter": 10,
          "iterations": 19519024
        },
        {
          "workload": "make/unmake",
          "ops_per_sec": 55031107,
          "mad": 1825084,
          "mad_percent": 3.32,
          "ops_per_iter": 128,
          "iterations": 687573
        },
        {
          "workload": "threshold SEE",
          "ops_per_sec": 58814379,
          "mad": 95255,
          "mad_percent": 0.16,
          "ops_per_iter": 10,
          "iterations": 9562950
        },
        {
          "workload": "perft(4) startpos",
          "ops_per_sec": 388373995,
          "mad": 5265052,
          "mad_percent": 1.36,
          "ops_per_iter": 197281,
          "iterations": 3159
        },
        {
          "workload": "two-ply simulation",
          "ops_per_sec": 520436862,
          "mad": 3367547,
          "mad_percent": 0.65,
          "ops_per_iter": 4597,
          "iterations": 183512
        }
      ]
    }
  ],
  "summary": {
    "rarog": {
      "legal moves": {
        "median_ops_per_sec": 447131123,
        "round_medians": [
          446835610,
          450020873,
          447131123
        ],
        "round_range_percent": 0.7123778319497567,
        "max_within_run_mad_percent": 0.15
      },
      "legal captures": {
        "median_ops_per_sec": 98203955,
        "round_medians": [
          98203955,
          97706249,
          100675265
        ],
        "round_range_percent": 3.0233161179710124,
        "max_within_run_mad_percent": 0.35
      },
      "make/unmake": {
        "median_ops_per_sec": 42520810,
        "round_medians": [
          42880416,
          42520810,
          41952617
        ],
        "round_range_percent": 2.1819880665490614,
        "max_within_run_mad_percent": 2.51
      },
      "threshold SEE": {
        "median_ops_per_sec": 46676180,
        "round_medians": [
          46362836,
          47290470,
          46676180
        ],
        "round_range_percent": 1.9873820008406857,
        "max_within_run_mad_percent": 3.23
      },
      "perft(4) startpos": {
        "median_ops_per_sec": 273741073,
        "round_medians": [
          270582440,
          279844251,
          273741073
        ],
        "round_range_percent": 3.3834202878279798,
        "max_within_run_mad_percent": 2.79
      },
      "two-ply simulation": {
        "median_ops_per_sec": 351808501,
        "round_medians": [
          334454431,
          351808501,
          362488899
        ],
        "round_range_percent": 7.968672706973615,
        "max_within_run_mad_percent": 7.24
      }
    },
    "basilisk": {
      "legal moves": {
        "median_ops_per_sec": 642645770,
        "round_medians": [
          641375549,
          642645770,
          646286043
        ],
        "round_range_percent": 0.7641058619276371,
        "max_within_run_mad_percent": 0.06
      },
      "legal captures": {
        "median_ops_per_sec": 120137542,
        "round_medians": [
          120044251,
          120137542,
          120405617
        ],
        "round_range_percent": 0.300793568758049,
        "max_within_run_mad_percent": 0.24
      },
      "make/unmake": {
        "median_ops_per_sec": 55031107,
        "round_medians": [
          56420047,
          53741047,
          55031107
        ],
        "round_range_percent": 4.868155750528515,
        "max_within_run_mad_percent": 4.49
      },
      "threshold SEE": {
        "median_ops_per_sec": 58814379,
        "round_medians": [
          58638662,
          58983916,
          58814379
        ],
        "round_range_percent": 0.5870231155547864,
        "max_within_run_mad_percent": 0.4
      },
      "perft(4) startpos": {
        "median_ops_per_sec": 382726065,
        "round_medians": [
          382726065,
          374068307,
          388373995
        ],
        "round_range_percent": 3.7378400135877863,
        "max_within_run_mad_percent": 3.62
      },
      "two-ply simulation": {
        "median_ops_per_sec": 513536852,
        "round_medians": [
          506161729,
          513536852,
          520436862
        ],
        "round_range_percent": 2.7797679844016336,
        "max_within_run_mad_percent": 2.96
      }
    },
    "reckless": {
      "legal moves": {
        "median_ops_per_sec": 339844427,
        "round_medians": [
          315357805,
          341709244,
          339844427
        ],
        "round_range_percent": 7.753971201652219,
        "max_within_run_mad_percent": 0.15
      },
      "legal captures": {
        "median_ops_per_sec": 61597216,
        "round_medians": [
          53646306,
          61704779,
          61597216
        ],
        "round_range_percent": 13.082527950613873,
        "max_within_run_mad_percent": 0.26
      },
      "make/unmake": {
        "median_ops_per_sec": 23494488,
        "round_medians": [
          23494488,
          18168284,
          23737926
        ],
        "round_range_percent": 23.706164611886837,
        "max_within_run_mad_percent": 9.06
      },
      "threshold SEE": {
        "median_ops_per_sec": 39722097,
        "round_medians": [
          34603389,
          40518759,
          39722097
        ],
        "round_range_percent": 14.891887505334877,
        "max_within_run_mad_percent": 4.32
      },
      "perft(4) startpos": {
        "median_ops_per_sec": 177943984,
        "round_medians": [
          177943984,
          173339067,
          180350887
        ],
        "round_range_percent": 3.9404647700818027,
        "max_within_run_mad_percent": 0.77
      },
      "two-ply simulation": {
        "median_ops_per_sec": 246625657,
        "round_medians": [
          237570427,
          246625657,
          256257625
        ],
        "round_range_percent": 7.577150823363037,
        "max_within_run_mad_percent": 4.54
      }
    }
  },
  "adapter_patch_sha256": "ffd86bf66efc1f86f6b3c722644816281d87dd79cbef036bebb92f637f206bea",
  "source_revisions": {
    "rarog": "60bd1f1106a8cb3c3f29651ab5652b36b1ccae11",
    "basilisk": "d73476614701863e61871de62f12568b52191d79",
    "Reckless": "91b56c29861f0a5713204bdeffd6c45e9eb9f649",
    "stockfish": "1dc0912d86dafb99e96d679a6ac76cbdf1553459",
    "rarog_at_measurement": "ca03a46db74197bc32c8cf3441359de421fcddd5"
  },
  "source_revision_note": "Rarog advanced to 60bd1f1 while documentation was being prepared. Board, bench and Cargo.toml are unchanged from measured ca03a46; ranking and crash reporting changed in parallel.",
  "toolchains": {
    "rust": "rustc 1.97.1 (8bab26f4f), LLVM 22.1.6, x86_64-pc-windows-msvc",
    "basilisk": "Clang 22.1.8, x86_64-w64-windows-gnu, thin LTO"
  },
  "builds": {
    "rarog": "cargo bench --locked --bench board --no-run -j 2; RUSTFLAGS=-C target-cpu=native --cfg rarog_pext; default features (none), release fat LTO",
    "basilisk": "cmake Release, COMP=clang, USE_PEXT=ON; -O3 -march=native -mbmi2 -flto=thin; board_performance_test",
    "reckless": "cargo bench --locked --no-default-features --features board-bench --bench board --no-run -j 2; RUSTFLAGS=-C target-cpu=native; release fat LTO; native magic sliders, AVX2 setwise attacks; NullBoardObserver"
  },
  "source_hashes": {
    "rarog": {
      "src/board.rs": "9994fd2a31aff9eeb80dfaced344ddf64f540abf013f9393ce97f700f800f262",
      "src/board/board.rs": "138b6e507aa60b1644a5be1b8d95168217d5d75cd21ea613aaaf9fb9a61f02c7",
      "src/board/movegen.rs": "7277ce0f5f2b9d64b079665e3b28cb2bd0eac8f4f53ecc98e6387a940af7bd60",
      "src/board/moves.rs": "51d7f1a1b3c26b0fc5d161b8b6ccb00507a7799d9170e4c644ae0e38aca05c29",
      "src/board/attacks.rs": "29c73b28cdf28ed50669e789fd75ff4d74351da99e32e9b6dc441647bedb9e7d",
      "benches/board.rs": "3c864f5610514585513af65d0837cb89fd3178e9cdfa85991a718edbbc738e96"
    },
    "reckless": {
      "src/board.rs": "d0b4cac8ea23311f6e9b52c10a2e258ffc87e0f72e904c109423db91c5d76f88",
      "src/board/makemove.rs": "75f91d7f248fee1ca8187af793fd887bd039c87a658908e0c7d1371628e4164c",
      "src/board/see.rs": "db6d412c8181443d075edd723a2e82db1e2af6490e70a348c7958be9535eb4e3",
      "src/board/movegen.rs": "410c64c710415056c2576940c27a9aebc81f6ae9f87f2d9dc2de2cf2958b3ef5",
      "src/tools/board_bench.rs": "066f9e264f83e993e8938b0ce408e584c1c5cf69d4017706acc708856896f79e",
      "benches/board.rs": "089d4bd9dbe9b0b9b8b9160f9a199ad43fca3cfc195f2f3abecd6cd67b91ba80"
    },
    "basilisk": {
      "src/board.h": "6ba1dfbf8aada3d43acb2100c399181abdb16b520a608a2dc000754fe18591bd",
      "src/board.cpp": "3336ddee1da0666cdf04384957612d955ccdd9c9af3faa481d71e62a20995ac9",
      "tests/board_performance.cpp": "a44c98aae12b6f98b79f0e75d450268b1aa7d557da9b3ec1b60ebdaca4295761"
    }
  },
  "adapter_patch_sha256_lf_normalized": "7a8ebd53dad6bbf27c04858fa0bb7e8fcedb87e96996f6fecb6534f7d0b43574",
  "adapter_patch_hash_note": "Original field hashed LF-normalized text. On-disk patch uses CRLF. Both digests retained; adapter content unchanged. Reconciled by verify_bundle.py preparation."
}
```

### round1-rarog.txt

```text

Rarog board benchmark
profile: cross-engine-board-v1
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  446835610          398857      0.09%          128      5786924 moves
legal captures                98203955          342960      0.35%           10     15898542 moves
make/unmake                   42880416          571762      1.33%          128       526750 moves
threshold SEE                 46362836          360229      0.78%           10      7443904 captures
perft(4) startpos            270582440         5964154      2.20%       197281         2221 nodes
two-ply simulation           334454431        24208463      7.24%         4597       119280 moves
```

### round1-basilisk.txt

```text

Basilisk board benchmark
profile: cross-engine-board-v1
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  641375549          383916      0.06%          128      8299776 moves
legal captures               120044251          140070      0.12%           10     18867628 moves
make/unmake                   56420047          289578      0.51%          128       699660 moves
threshold SEE                 58638662          107069      0.18%           10      9604425 captures
perft(4) startpos            382726065         7836417      2.05%       197281         3029 nodes
two-ply simulation           506161729        14986168      2.96%         4597       180612 moves
```

### round1-reckless.txt

```text
legal[0]: a2a3,a2a4,b1a3,b1c3,b2b3,b2b4,c2c3,c2c4,d2d3,d2d4,e2e3,e2e4,f2f3,f2f4,g1f3,g1h3,g2g3,g2g4,h2h3,h2h4
legal[1]: a1b1,a1c1,a1d1,a2a3,a2a4,b2b3,c3a4,c3b1,c3b5,c3d1,d2c1,d2e3,d2f4,d2g5,d2h6,d5d6,d5e6,e1c1,e1d1,e1f1,e1g1,e2a6,e2b5,e2c4,e2d1,e2d3,e2f1,e5c4,e5c6,e5d3,e5d7,e5f7,e5g4,e5g6,f3d3,f3e3,f3f4,f3f5,f3f6,f3g3,f3g4,f3h3,f3h5,g2g3,g2g4,g2h3,h1f1,h1g1
legal[2]: a1b1,a2a3,a2a4,b2b3,c1d2,c1e3,c1f4,c1g5,c1h6,d1d2,d1d3,d1e2,d4d5,e1d2,e1e2,e4e5,f1a6,f1b5,f1c4,f1d3,f1e2,f3d2,f3e5,f3g1,f3g5,f3h4,g2g3,g2g4,h1g1,h2h3,h2h4
legal[3]: a5a4,a5a6,a5b4
legal[4]: a7a5,a7a6,b7b5,b7b6,b8a6,b8c6,c7c5,c7c6,d7d5,d7d6,d8e7,e8e7,f6d5,f6e4,f6g4,f6g8,f6h5,f8a3,f8b4,f8c5,f8d6,f8e7,g7g5,g7g6,h7h6,h8g8

Reckless board benchmark
profile: cross-engine-board-v1
SEE: native values 109/403/435/679/1242/0; NOT contract-comparable
observer: NullBoardObserver; native threat/pin/key state retained
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  315357805          460036      0.15%          128      4072187 moves
legal captures                53646306           89794      0.17%           10      8640060 moves
make/unmake                   23494488           33145      0.14%          128       300900 moves
threshold SEE                 34603389         1495736      4.32%           10      5619930 captures
perft(4) startpos            177943984         1366565      0.77%       197281         1436 nodes
two-ply simulation           237570427         6853828      2.88%         4597        83480 moves
```

### round2-rarog.txt

```text

Rarog board benchmark
profile: cross-engine-board-v1
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  450020873          287778      0.06%          128      5820224 moves
legal captures                97706249          333571      0.34%           10     15852672 moves
make/unmake                   42520810          683248      1.61%          128       534232 moves
threshold SEE                 47290470          462700      0.98%           10      7635342 captures
perft(4) startpos            279844251         2115395      0.76%       197281         2257 nodes
two-ply simulation           351808501         5651799      1.61%         4597       121028 moves
```

### round2-basilisk.txt

```text

Basilisk board benchmark
profile: cross-engine-board-v1
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  642645770          290149      0.05%          128      8310024 moves
legal captures               120137542          215488      0.18%           10     19519024 moves
make/unmake                   53741047         2411718      4.49%          128       680016 moves
threshold SEE                 58983916          233613      0.40%           10      9483340 captures
perft(4) startpos            374068307        13529803      3.62%       197281         3052 nodes
two-ply simulation           513536852         9544855      1.86%         4597       171054 moves
```

### round2-reckless.txt

```text
legal[0]: a2a3,a2a4,b1a3,b1c3,b2b3,b2b4,c2c3,c2c4,d2d3,d2d4,e2e3,e2e4,f2f3,f2f4,g1f3,g1h3,g2g3,g2g4,h2h3,h2h4
legal[1]: a1b1,a1c1,a1d1,a2a3,a2a4,b2b3,c3a4,c3b1,c3b5,c3d1,d2c1,d2e3,d2f4,d2g5,d2h6,d5d6,d5e6,e1c1,e1d1,e1f1,e1g1,e2a6,e2b5,e2c4,e2d1,e2d3,e2f1,e5c4,e5c6,e5d3,e5d7,e5f7,e5g4,e5g6,f3d3,f3e3,f3f4,f3f5,f3f6,f3g3,f3g4,f3h3,f3h5,g2g3,g2g4,g2h3,h1f1,h1g1
legal[2]: a1b1,a2a3,a2a4,b2b3,c1d2,c1e3,c1f4,c1g5,c1h6,d1d2,d1d3,d1e2,d4d5,e1d2,e1e2,e4e5,f1a6,f1b5,f1c4,f1d3,f1e2,f3d2,f3e5,f3g1,f3g5,f3h4,g2g3,g2g4,h1g1,h2h3,h2h4
legal[3]: a5a4,a5a6,a5b4
legal[4]: a7a5,a7a6,b7b5,b7b6,b8a6,b8c6,c7c5,c7c6,d7d5,d7d6,d8e7,e8e7,f6d5,f6e4,f6g4,f6g8,f6h5,f8a3,f8b4,f8c5,f8d6,f8e7,g7g5,g7g6,h7h6,h8g8

Reckless board benchmark
profile: cross-engine-board-v1
SEE: native values 109/403/435/679/1242/0; NOT contract-comparable
observer: NullBoardObserver; native threat/pin/key state retained
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  341709244          168434      0.05%          128      4418304 moves
legal captures                61704779          163130      0.26%           10      9467848 moves
make/unmake                   18168284         1646452      9.06%          128       254951 moves
threshold SEE                 40518759          366683      0.90%           10      6478130 captures
perft(4) startpos            173339067         1258203      0.73%       197281         1417 nodes
two-ply simulation           246625657        11206429      4.54%         4597        84436 moves
```

### round3-rarog.txt

```text

Rarog board benchmark
profile: cross-engine-board-v1
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  447131123          664847      0.15%          128      5783440 moves
legal captures               100675265          193008      0.19%           10     16348068 moves
make/unmake                   41952617         1051183      2.51%          128       523912 moves
threshold SEE                 46676180         1509425      3.23%           10      7492404 captures
perft(4) startpos            273741073         7636994      2.79%       197281         2215 nodes
two-ply simulation           362488899         1407429      0.39%         4597       123222 moves
```

### round3-basilisk.txt

```text

Basilisk board benchmark
profile: cross-engine-board-v1
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  646286043          357443      0.06%          128      8344485 moves
legal captures               120405617          287756      0.24%           10     19519024 moves
make/unmake                   55031107         1825084      3.32%          128       687573 moves
threshold SEE                 58814379           95255      0.16%           10      9562950 captures
perft(4) startpos            388373995         5265052      1.36%       197281         3159 nodes
two-ply simulation           520436862         3367547      0.65%         4597       183512 moves
```

### round3-reckless.txt

```text
legal[0]: a2a3,a2a4,b1a3,b1c3,b2b3,b2b4,c2c3,c2c4,d2d3,d2d4,e2e3,e2e4,f2f3,f2f4,g1f3,g1h3,g2g3,g2g4,h2h3,h2h4
legal[1]: a1b1,a1c1,a1d1,a2a3,a2a4,b2b3,c3a4,c3b1,c3b5,c3d1,d2c1,d2e3,d2f4,d2g5,d2h6,d5d6,d5e6,e1c1,e1d1,e1f1,e1g1,e2a6,e2b5,e2c4,e2d1,e2d3,e2f1,e5c4,e5c6,e5d3,e5d7,e5f7,e5g4,e5g6,f3d3,f3e3,f3f4,f3f5,f3f6,f3g3,f3g4,f3h3,f3h5,g2g3,g2g4,g2h3,h1f1,h1g1
legal[2]: a1b1,a2a3,a2a4,b2b3,c1d2,c1e3,c1f4,c1g5,c1h6,d1d2,d1d3,d1e2,d4d5,e1d2,e1e2,e4e5,f1a6,f1b5,f1c4,f1d3,f1e2,f3d2,f3e5,f3g1,f3g5,f3h4,g2g3,g2g4,h1g1,h2h3,h2h4
legal[3]: a5a4,a5a6,a5b4
legal[4]: a7a5,a7a6,b7b5,b7b6,b8a6,b8c6,c7c5,c7c6,d7d5,d7d6,d8e7,e8e7,f6d5,f6e4,f6g4,f6g8,f6h5,f8a3,f8b4,f8c5,f8d6,f8e7,g7g5,g7g6,h7h6,h8g8

Reckless board benchmark
profile: cross-engine-board-v1
SEE: native values 109/403/435/679/1242/0; NOT contract-comparable
observer: NullBoardObserver; native threat/pin/key state retained
positions: 5
samples: 11 x 150 ms after a 150 ms warm-up (median +/- MAD)
preflight: PASS

workload                estimate ops/s       MAD ops/s      MAD %     ops/iter  total iters
legal moves                  339844427          400528      0.12%          128      4338981 moves
legal captures                61597216           60838      0.10%           10      9961608 moves
make/unmake                   23737926          319704      1.35%          128       295740 moves
threshold SEE                 39722097          358141      0.90%           10      6308880 captures
perft(4) startpos            180350887          329152      0.18%       197281         1474 nodes
two-ply simulation           256257625         1691080      0.66%         4597        89490 moves
```
