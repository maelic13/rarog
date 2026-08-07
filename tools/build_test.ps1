<#
.SYNOPSIS
    Build a Rarog test binary and copy it to the test-engines folder.

.DESCRIPTION
    Three modes:

    Normal (default): runs `cargo xtask build --arch pext --pgo` — produces a
    PGO-optimised pext binary.  Use for SPRT and gauntlet testing.  PGO changes
    hot-path timing enough to affect measured Elo comparisons; always use this
    for match testing.

    Native (-Native switch): runs `cargo xtask build --arch pext --native --pgo` —
    produces a PGO-optimised binary built with `-C target-cpu=native` for the
    exact host CPU (e.g. znver3 on a 5950X), instead of the portable
    x86-64-v3 baseline.  Use for local/own-match testing and deployment on the
    machine that built it; do NOT distribute this binary, since it is not
    guaranteed to run on other CPUs.

    Tune (-Tune switch): runs `cargo build --release --features tune` — produces
    a non-PGO pext binary with search-parameter UCI options exposed.  Use ONLY
    for Colosseum CLI SPSA runs. PGO is skipped because (a) xtask does not
    support --features, and (b) SPSA accuracy does not depend on absolute NPS —
    both sides of each mini-match use the same binary.

    Output always goes to tools\test_engines\ (repo-local and separate from
    released engines).

.PARAMETER Suffix
    Short label for the output file.
    Normal:  rarog-<Suffix>-pext-pgo.exe
    Native:  rarog-<Suffix>-native-pgo.exe
    Tune:    rarog-<Suffix>-tune.exe

.PARAMETER Native
    Build with `--arch pext --native --pgo` instead of `--arch pext --pgo`.
    Same PEXT code path; only the codegen baseline changes.  Local-only.

.PARAMETER Tune
    Build with --features tune instead of PGO.  Use for SPSA binaries only.

.PARAMETER TestEnginesDir
    Destination directory.  Default: tools\test_engines

.EXAMPLE
    # Normal SPRT binary
    ./tools/build_test.ps1 -Suffix phase1-lmr

.EXAMPLE
    # Native (-march=native-equivalent) binary for local-only testing
    ./tools/build_test.ps1 -Suffix phase292-native -Native

.EXAMPLE
    # SPSA tuning binary (exposes UCI options)
    ./tools/build_test.ps1 -Suffix phase1-lmr -Tune
#>
param(
    [Parameter(Mandatory)][string]$Suffix,
    [switch]$Tune,
    [switch]$Native,
    [string]$TestEnginesDir = "$PSScriptRoot\test_engines"
)

if ($Tune -and $Native) {
    throw "-Tune and -Native are mutually exclusive."
}

$ErrorActionPreference = "Stop"

# --- 9.7 provenance manifest -------------------------------------------------
# Every test binary gets a sidecar JSON next to it: git SHA + dirty flag,
# branch, rustc, and a bench fingerprint VERIFIED by running the binary just
# built (which doubles as a smoke test — a broken build fails here, not in an
# SPRT). This is Rarog-owned build provenance; Colosseum independently hashes
# each ordinary executable and does not require or inspect this sidecar.
#
# LOCAL-ONLY BY DESIGN (user decision 2026-07-20): manifests exist for
# development provenance. tools/test_engines/ and tools/results/ are
# gitignored, and the release workflow (build.yml) has NO manifest step —
# nothing here can ever appear on the GitHub release page.
function Write-EngineManifest {
    param(
        [Parameter(Mandatory)][string]$BinaryPath,
        [Parameter(Mandatory)][string]$Suffix,
        [Parameter(Mandatory)][string]$Flavor
    )

    $sha    = (git rev-parse HEAD).Trim()
    $branch = (git rev-parse --abbrev-ref HEAD).Trim()
    $dirty  = [bool](git status --porcelain)
    $rustc  = (rustc -V).Trim()

    Write-Host "Verifying bench fingerprint of $([IO.Path]::GetFileName($BinaryPath)) ..."
    $benchOut  = "bench" | & $BinaryPath 2>&1 | Out-String
    $benchLine = ($benchOut -split "`n" | Where-Object { $_ -match "Nodes searched" }) -join ""
    if ($benchLine -notmatch "([0-9][0-9,]*)\s*$") {
        throw "Could not parse a bench node count from the built binary — refusing to write a manifest for an unverified engine."
    }
    $nodes = [int64]($Matches[1] -replace ",", "")
    if ($nodes -le 0) { throw "Bench reported $nodes nodes — broken binary." }

    $manifest = [ordered]@{
        engine        = [IO.Path]::GetFileName($BinaryPath)
        suffix        = $Suffix
        flavor        = $Flavor
        git_sha       = $sha
        git_branch    = $branch
        git_dirty     = $dirty
        rustc         = $rustc
        bench_nodes   = $nodes
        pgo_workload  = if ($Flavor -like "*-pgo") { "bench 13 (xtask default)" } else { $null }
        built_utc     = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    }

    $manifestPath = [IO.Path]::ChangeExtension($BinaryPath, ".json")
    $manifest | ConvertTo-Json | Out-File -FilePath $manifestPath -Encoding utf8
    Write-Host "Manifest: $manifestPath  (bench $nodes$(if ($dirty) { ', DIRTY WORKING TREE' }))"
    if ($dirty) {
        Write-Host "WARNING: built from a DIRTY working tree — this binary is not reproducible from git_sha alone." -ForegroundColor Yellow
    }
}


$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    if ($Tune) {
        Write-Host ""
        Write-Host "Building pext tune binary (--features tune, no PGO) — suffix: $Suffix"
        Write-Host "NOTE: Use this binary only for SPSA, never for SPRT."
        Write-Host ""

        # pext RUSTFLAGS matching xtask's pext arch (rarog_pext cfg + BMI2 target features).
        $env:RUSTFLAGS = "--cfg rarog_pext -C target-cpu=x86-64-v3 -C target-feature=+bmi2"
        cargo build --release --features tune
        if ($LASTEXITCODE -ne 0) { throw "cargo build --features tune failed (exit $LASTEXITCODE)" }
        $env:RUSTFLAGS = $null

        $src = Join-Path $repoRoot "target\release\rarog.exe"
        if (-not (Test-Path $src)) { throw "Binary not found at: $src" }

        if (-not (Test-Path $TestEnginesDir)) {
            New-Item -ItemType Directory -Path $TestEnginesDir | Out-Null
        }

        $dest = Join-Path $TestEnginesDir "rarog-$Suffix-tune.exe"
        Copy-Item $src $dest -Force
        Write-EngineManifest -BinaryPath $dest -Suffix $Suffix -Flavor "tune"
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    } else {
        # 2.3.0: `--native` is now ORTHOGONAL to `--arch`. Both flavours build
        # the PEXT code path; -Native only swaps the portable x86-64-v3 baseline
        # for `target-cpu=native`. Gate binaries deliberately stay portable, so
        # what we SPRT matches the shipped pext asset (PLAN S3).
        $arch = "pext"
        $label = if ($Native) { "pext+native" } else { "pext" }
        Write-Host ""
        Write-Host "Building $label+PGO binary (suffix: $Suffix) ..."
        Write-Host ""

        if ($Native) {
            cargo xtask build --arch $arch --native --pgo
        } else {
            cargo xtask build --arch $arch --pgo
        }
        if ($LASTEXITCODE -ne 0) { throw "xtask build failed (exit $LASTEXITCODE)" }

        $dist = Get-ChildItem "target/dist/rarog-*-$arch-pgo.exe" |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1

        if (-not $dist) {
            throw "No $arch-pgo binary found in target/dist/ — check xtask output above."
        }

        if (-not (Test-Path $TestEnginesDir)) {
            New-Item -ItemType Directory -Path $TestEnginesDir | Out-Null
        }

        $dest = Join-Path $TestEnginesDir "rarog-$Suffix-$arch-pgo.exe"
        Copy-Item $dist.FullName $dest -Force
        Write-EngineManifest -BinaryPath $dest -Suffix $Suffix -Flavor "$arch-pgo"
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    }
} finally {
    Pop-Location
}
