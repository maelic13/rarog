<#
.SYNOPSIS
    Run an SPRT self-play match between two Rarog binaries using cutechess-cli.

.DESCRIPTION
    Starts a cutechess-cli match with the built-in SPRT stopping rule.  The
    match runs until the test accepts H0 (no meaningful improvement) or H1
    (improvement ≥ Elo1).  Real-time output is printed to the console.

    Use pext-PGO builds (see tools/build_test.ps1) for both engines.

    CALIBRATION CHECK (run this first, before any feature testing):
        ./tools/sprt.ps1 `
            -EngineA "D:\chess\engines\rarog-v2.1.0-windows-pext-pgo-codex-work.exe" `
            -EngineB "D:\chess\engines\rarog-v2.0.2-windows-pext-pgo.exe" `
            -NameA "CW" -NameB "2.0.2"
        Expected result: H0 accepted (the two engines are behavior-identical).
        If the harness returns H1 here, something is wrong — investigate before
        trusting any further SPRT results.

.PARAMETER EngineA
    Path to the new/candidate engine (the one you are testing).

.PARAMETER EngineB
    Path to the baseline engine (the current integration head).

.PARAMETER NameA
    Display name for engine A in cutechess output.  Default: "New"

.PARAMETER NameB
    Display name for engine B.  Default: "Base"

.PARAMETER Elo0
    Lower bound: H0 is "A is at most Elo0 better than B".  Default: 0.
    For a small incremental feature use 0; for a large expected gain use -3.

.PARAMETER Elo1
    Upper bound: H1 is "A is at least Elo1 better than B".  Default: 5.
    Tighten to 3 for small, incremental features (e.g. one search constant).

.PARAMETER Alpha
    False-positive rate.  Default: 0.05

.PARAMETER Beta
    False-negative rate.  Default: 0.05

.PARAMETER Hash
    Hash table size in MB per engine.  Default: 64 (matches tournament).

.PARAMETER Concurrency
    Parallel games.  Default: logical CPU count − 1, minimum 1.
    Set lower if the machine is also running other work.

.PARAMETER Book
    Path to the opening book PGN.
    Default: D:\chess\books\SuperGM_4mvs.pgn (matches existing tournament).

.EXAMPLE
    # Feature test: new ProbCut port vs current integration head
    ./tools/sprt.ps1 `
        -EngineA "D:\chess\engines\rarog-feat-probcut-pext-pgo.exe" `
        -EngineB "D:\chess\engines\rarog-head-pext-pgo.exe" `
        -NameA "ProbCut" -NameB "Head" -Elo1 3
#>
param(
    [Parameter(Mandatory)][string]$EngineA,
    [Parameter(Mandatory)][string]$EngineB,
    [string]$NameA = "New",
    [string]$NameB = "Base",
    [double]$Elo0 = 0,
    [double]$Elo1 = 5,
    [double]$Alpha = 0.05,
    [double]$Beta  = 0.05,
    [int]$Hash = 64,
    [int]$Concurrency = [Math]::Max(1, $env:NUMBER_OF_PROCESSORS - 1),
    [string]$Book = "D:\chess\books\SuperGM_4mvs.pgn"
)

$ErrorActionPreference = "Stop"

$cutechess = "D:\chess\cutechess-cli\cutechess-cli.exe"
if (-not (Test-Path $cutechess)) {
    throw "cutechess-cli not found at: $cutechess"
}
if (-not (Test-Path $EngineA)) {
    throw "EngineA not found: $EngineA"
}
if (-not (Test-Path $EngineB)) {
    throw "EngineB not found: $EngineB"
}
if (-not (Test-Path $Book)) {
    throw "Opening book not found: $Book"
}

$resultsDir = Join-Path $PSScriptRoot "results"
New-Item -ItemType Directory -Force -Path $resultsDir | Out-Null

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$pgnOut    = Join-Path $resultsDir "sprt_${NameA}_vs_${NameB}_${timestamp}.pgn"

$EngineA = Resolve-Path $EngineA
$EngineB = Resolve-Path $EngineB

Write-Host ""
Write-Host "======================================================="
Write-Host "  SPRT: $NameA  vs  $NameB"
Write-Host "  H0: A <= +${Elo0}   H1: A >= +${Elo1}   alpha=$Alpha  beta=$Beta"
Write-Host "  TC: 100 ms/move   Hash: ${Hash} MB   Conc: $Concurrency"
Write-Host "  Book: $(Split-Path $Book -Leaf)"
Write-Host "  PGN:  $pgnOut"
Write-Host "======================================================="
Write-Host ""

# -each st=0.1 → 100 ms per move (matches existing tournament settings)
# -rounds 50000 → effectively unlimited; SPRT stops it early
# -games 2 -repeat → each opening played once per colour
& $cutechess `
    -engine "name=$NameA" "cmd=$EngineA" proto=uci "option.Hash=$Hash" option.Threads=1 `
    -engine "name=$NameB" "cmd=$EngineB" proto=uci "option.Hash=$Hash" option.Threads=1 `
    -each st=0.1 `
    -openings "file=$Book" format=pgn order=random `
    -rounds 50000 -games 2 -repeat `
    -concurrency $Concurrency `
    -sprt "elo0=$Elo0" "elo1=$Elo1" "alpha=$Alpha" "beta=$Beta" `
    -pgnout "$pgnOut" `
    -draw movenumber=40 movecount=10 score=5 `
    -resign movecount=3 score=600 `
    -ratinginterval 200

Write-Host ""
Write-Host "Match finished. PGN saved to: $pgnOut"
