<#
.SYNOPSIS
    Run an SPRT self-play match between two Rarog binaries using fastchess.

.DESCRIPTION
    Starts a fastchess match with the built-in SPRT stopping rule.  The match
    runs until the test accepts H0 (no meaningful improvement) or H1
    (improvement).  Real-time output is printed to the console.

    Tooling:
      - fastchess (NOT cutechess-cli): faster, no Qt dependency, built-in SPRT.
        Download a release from https://github.com/Disservin/fastchess/releases
        and place it at $FastchessPath (default tools\bin\fastchess.exe),
        or pass -FastchessPath. The cutechess GUI is still handy for *viewing*
        the resulting PGNs, but is not used to run matches.

    Conditions (chosen to match the Little Blitzer deployment exactly):
      - st=0.1  -> fixed 100 ms/move (Little Blitzer = "100ms per move").
                   Fixed movetime, so engine time-management is NOT exercised
                   (correct: it is not used in deployment either).
      - Hash 64 MB, Threads 1, SuperGM_4mvs.pgn opening book (random order),
        each opening played from both colours (-games 2 -repeat).
      - model=normalized (nElo) — fastchess default, more time-control-robust
        than logistic Elo.

    IMPORTANT — concurrency and fixed movetime:
      With fixed movetime, oversubscribing cores adds timing noise (each engine
      gets 100 ms wall but fewer nodes). Keep -Concurrency <= PHYSICAL cores - 1.
      This machine has 16 physical cores, so the default is 15. Do NOT raise it
      to the 30 logical processors as in the old cutechess tournament script —
      that was fine for tc-based play but is noisy for fixed movetime.

    CALIBRATION CHECK — run this FIRST, before testing any feature:
        ./tools/sprt.ps1 `
            -EngineA "tools\test_engines\rarog-v2.1.0-windows-pext-pgo-codex-work.exe" `
            -EngineB "tools\test_engines\rarog-v2.0.2-windows-pext-pgo.exe" `
            -NameA "CW" -NameB "2.0.2"
        Expected: H0 accepted (these two are behavior-identical). If the harness
        returns H1 here, something is wrong — fix it before trusting any result.

.PARAMETER EngineA
    Path to the new/candidate engine (usually in tools\test_engines).

.PARAMETER EngineB
    Path to the baseline engine (the current integration head, or a released
    reference copied into tools\test_engines).

.PARAMETER NameA / NameB
    Display names. Defaults: "New" / "Base".

.PARAMETER Mode
    "gainer"       -> H0: elo<=0,  H1: elo>=Elo1  (default; test a real gain).
    "simplify"     -> H0: elo<=-5, H1: elo>=0     (non-regression / cleanup).
    The explicit -Elo0/-Elo1 parameters override the mode if supplied.

.PARAMETER Elo1
    Upper SPRT bound for "gainer" mode. Default 5 (nElo). Use 3 for small,
    incremental features (e.g. a single tuned search constant) to demand a
    cleaner signal.

.PARAMETER Hash
    Hash MB per engine. Default 64 (matches deployment).

.PARAMETER Concurrency
    Parallel games. Default 15 (physical cores - 1 on this machine).

.PARAMETER TimeMargin
    fastchess timeout margin in milliseconds. Default 20. This prevents small
    Windows scheduler / process IO jitter at st=0.1 from being counted as a
    time forfeit. It does not change the engine's fixed 100 ms/move budget.

.PARAMETER Book
    Opening book PGN. Default tools\books\SuperGM_4mvs.pgn.

.PARAMETER FastchessPath
    Path to fastchess.exe. Default tools\bin\fastchess.exe (or found on PATH).

.EXAMPLE
    ./tools/sprt.ps1 `
        -EngineA "tools\test_engines\rarog-feat-probcut-pext-pgo.exe" `
        -EngineB "tools\test_engines\rarog-head-pext-pgo.exe" `
        -NameA "ProbCut" -NameB "Head" -Elo1 3
#>
param(
    [Parameter(Mandatory)][string]$EngineA,
    [Parameter(Mandatory)][string]$EngineB,
    [string]$NameA = "New",
    [string]$NameB = "Base",
    [ValidateSet("gainer", "simplify")][string]$Mode = "gainer",
    [Nullable[int]]$Elo0 = $null,
    [Nullable[int]]$Elo1 = $null,
    [double]$Alpha = 0.05,
    [double]$Beta  = 0.05,
    [int]$Hash = 64,
    [int]$Concurrency = 15,
    [int]$TimeMargin = 20,
    [string]$Book = "$PSScriptRoot\books\SuperGM_4mvs.pgn",
    [string]$FastchessPath = "$PSScriptRoot\bin\fastchess.exe"
)

$ErrorActionPreference = "Stop"

# Resolve SPRT bounds from mode unless explicitly overridden.
if ($null -eq $Elo0) { $Elo0 = if ($Mode -eq "simplify") { -5 } else { 0 } }
if ($null -eq $Elo1) { $Elo1 = if ($Mode -eq "simplify") {  0 } else { 5 } }

# Locate fastchess.
$fastchess = $FastchessPath
if (-not (Test-Path $fastchess)) {
    $onPath = Get-Command fastchess -ErrorAction SilentlyContinue
    if ($onPath) { $fastchess = $onPath.Source }
    else {
        throw "fastchess not found at '$FastchessPath' or on PATH. Download from " +
              "https://github.com/Disservin/fastchess/releases and place it there."
    }
}
foreach ($p in @($EngineA, $EngineB, $Book)) {
    if (-not (Test-Path $p)) { throw "Not found: $p" }
}

$EngineA = (Resolve-Path $EngineA).Path
$EngineB = (Resolve-Path $EngineB).Path

$resultsDir = Join-Path $PSScriptRoot "results"
New-Item -ItemType Directory -Force -Path $resultsDir | Out-Null
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$pgnOut    = Join-Path $resultsDir "sprt_${NameA}_vs_${NameB}_${timestamp}.pgn"

Write-Host ""
Write-Host "======================================================="
Write-Host "  SPRT ($Mode): $NameA  vs  $NameB"
Write-Host "  H0: elo<=$Elo0   H1: elo>=$Elo1   alpha=$Alpha  beta=$Beta  (nElo)"
Write-Host "  TC: st=0.1 (100 ms/move)   Margin: ${TimeMargin} ms   Hash: ${Hash} MB   Conc: $Concurrency"
Write-Host "  Book: $(Split-Path $Book -Leaf)"
Write-Host "  Runner: $fastchess"
Write-Host "  PGN:  $pgnOut"
Write-Host "======================================================="
Write-Host ""

# NOTE: flag names follow the fastchess man page (man.md). If your installed
# fastchess build rejects a flag, run `fastchess --help` and adjust here.
& $fastchess `
    -engine "cmd=$EngineA" "name=$NameA" "option.Hash=$Hash" "option.Threads=1" `
    -engine "cmd=$EngineB" "name=$NameB" "option.Hash=$Hash" "option.Threads=1" `
    -each st=0.1 "timemargin=$TimeMargin" `
    -openings "file=$Book" format=pgn order=random `
    -rounds 50000 -games 2 -repeat `
    -concurrency $Concurrency `
    -sprt "elo0=$Elo0" "elo1=$Elo1" "alpha=$Alpha" "beta=$Beta" model=normalized `
    -draw movenumber=40 movecount=8 score=10 `
    -resign movecount=3 score=600 twosided=true `
    -pgnout "file=$pgnOut" `
    -output format=fastchess   # console ticker format (not the PGN path)

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Error "fastchess exited with code $LASTEXITCODE — no games were played."
} else {
    Write-Host ""
    Write-Host "Match finished. PGN saved to: $pgnOut"
}
