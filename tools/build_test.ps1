<#
.SYNOPSIS
    Build a Rarog test binary and copy it to the test-engines folder.

.DESCRIPTION
    Three modes:

    Normal (default): runs `cargo xtask build --arch pext --pgo` — produces a
    PGO-optimised pext binary.  Use for SPRT and gauntlet testing.  PGO changes
    hot-path timing enough to affect measured Elo comparisons; always use this
    for match testing.

    Native (-Native switch): runs `cargo xtask build --arch native --pgo` —
    produces a PGO-optimised binary built with `-C target-cpu=native` for the
    exact host CPU (e.g. znver3 on a 5950X), instead of the portable
    x86-64-v3 baseline.  Use for local/own-match testing and deployment on the
    machine that built it; do NOT distribute this binary, since it is not
    guaranteed to run on other CPUs.

    Tune (-Tune switch): runs `cargo build --release --features tune` — produces
    a non-PGO pext binary with search-parameter UCI options exposed.  Use ONLY
    for weather-factory SPSA runs.  PGO is skipped because (a) xtask does not
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
    Build with `--arch native --pgo` instead of `--arch pext --pgo`.  Local-only.

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
        $arch = if ($Native) { "native" } else { "pext" }
        Write-Host ""
        Write-Host "Building $arch+PGO binary (suffix: $Suffix) ..."
        Write-Host ""

        cargo xtask build --arch $arch --pgo
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
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    }
} finally {
    Pop-Location
}
