<#
.SYNOPSIS
    Build a Rarog test binary and copy it to the test-engines folder.

.DESCRIPTION
    Two modes:

    Normal (default): runs `cargo xtask build --arch pext --pgo` — produces a
    PGO-optimised pext binary.  Use for SPRT and gauntlet testing.  PGO changes
    hot-path timing enough to affect measured Elo comparisons; always use this
    for match testing.

    Tune (-Tune switch): runs `cargo build --release --features tune` — produces
    a non-PGO pext binary with search-parameter UCI options exposed.  Use ONLY
    for weather-factory SPSA runs.  PGO is skipped because (a) xtask does not
    support --features, and (b) SPSA accuracy does not depend on absolute NPS —
    both sides of each mini-match use the same binary.

    Output always goes to D:\chess\engines\test_engines\ (separate from released
    engines in D:\chess\engines\).

.PARAMETER Suffix
    Short label for the output file.
    Normal:  rarog-<Suffix>-pext-pgo.exe
    Tune:    rarog-<Suffix>-tune.exe

.PARAMETER Tune
    Build with --features tune instead of PGO.  Use for SPSA binaries only.

.PARAMETER TestEnginesDir
    Destination directory.  Default: D:\chess\engines\test_engines

.EXAMPLE
    # Normal SPRT binary
    ./tools/build_test.ps1 -Suffix phase1-lmr

.EXAMPLE
    # SPSA tuning binary (exposes UCI options)
    ./tools/build_test.ps1 -Suffix phase1-lmr-tune -Tune
#>
param(
    [Parameter(Mandatory)][string]$Suffix,
    [switch]$Tune,
    [string]$TestEnginesDir = "D:\chess\engines\test_engines"
)

$ErrorActionPreference = "Stop"

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
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    } else {
        Write-Host ""
        Write-Host "Building pext+PGO binary (suffix: $Suffix) ..."
        Write-Host ""

        cargo xtask build --arch pext --pgo
        if ($LASTEXITCODE -ne 0) { throw "xtask build failed (exit $LASTEXITCODE)" }

        $dist = Get-ChildItem "target/dist/rarog-*-pext-pgo.exe" |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1

        if (-not $dist) {
            throw "No pext-pgo binary found in target/dist/ — check xtask output above."
        }

        if (-not (Test-Path $TestEnginesDir)) {
            New-Item -ItemType Directory -Path $TestEnginesDir | Out-Null
        }

        $dest = Join-Path $TestEnginesDir "rarog-$Suffix-pext-pgo.exe"
        Copy-Item $dist.FullName $dest -Force
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    }
} finally {
    Pop-Location
}
