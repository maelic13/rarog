<#
.SYNOPSIS
    Build a pext-PGO Rarog binary and copy it to D:\chess\engines\ for testing.

.DESCRIPTION
    Runs `cargo xtask build --arch pext --pgo`, which profiles with bench and
    then builds an optimized binary.  The result is copied to D:\chess\engines\
    with a human-readable name so it can be referenced by sprt.ps1 and spsa.py.

    Always use this script (not a plain `cargo build --release`) when building
    binaries for SPRT or gauntlet testing.  PGO changes hot-path timing enough
    to affect measured NPS and therefore Elo comparisons.

.PARAMETER Suffix
    Short label for the output file, e.g. "feat-probcut" or "phase1-tuned".
    Output: D:\chess\engines\rarog-<Suffix>-pext-pgo.exe

.PARAMETER EnginesDir
    Directory where the binary is copied.  Default: D:\chess\engines

.EXAMPLE
    ./tools/build_test.ps1 -Suffix feat-probcut
    # Builds and copies to D:\chess\engines\rarog-feat-probcut-pext-pgo.exe

.EXAMPLE
    ./tools/build_test.ps1 -Suffix head
    # Quick way to refresh the reference "head" binary after a merge.
#>
param(
    [Parameter(Mandatory)][string]$Suffix,
    [string]$EnginesDir = "D:\chess\engines"
)

$ErrorActionPreference = "Stop"

# Must run from repo root (where Cargo.toml lives).
$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    Write-Host ""
    Write-Host "Building pext+PGO binary (suffix: $Suffix) ..."
    Write-Host ""

    cargo xtask build --arch pext --pgo
    if ($LASTEXITCODE -ne 0) { throw "xtask build failed (exit $LASTEXITCODE)" }

    # xtask drops the binary in target/dist/ with the version embedded.
    $dist = Get-ChildItem "target/dist/rarog-*-pext-pgo.exe" |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1

    if (-not $dist) {
        throw "No pext-pgo binary found in target/dist/ — check xtask output above."
    }

    if (-not (Test-Path $EnginesDir)) {
        New-Item -ItemType Directory -Path $EnginesDir | Out-Null
    }

    $dest = Join-Path $EnginesDir "rarog-$Suffix-pext-pgo.exe"
    Copy-Item $dist.FullName $dest -Force
    Write-Host ""
    Write-Host "Done: $dest"
    Write-Host ""
} finally {
    Pop-Location
}
