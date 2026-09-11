<#
.SYNOPSIS
    Run an SPRT self-play match between two Rarog binaries using fastchess.

.DESCRIPTION
    Starts a fastchess match with the built-in SPRT stopping rule.  The match
    runs until the test accepts H0, accepts H1, or exhausts the registered game
    budget. Real-time output is printed to the console. A budget-exhausted test
    has not accepted H1 and therefore cannot promote the candidate.

    Tooling:
      - fastchess (NOT cutechess-cli): faster, no Qt dependency, built-in SPRT.
        Download a release from https://github.com/Disservin/fastchess/releases
        and place it at $FastchessPath (default tools\bin\fastchess.exe),
        or pass -FastchessPath. The cutechess GUI is still handy for *viewing*
        the resulting PGNs, but is not used to run matches.

    Conditions (unified with SPSA as of 2026-06-17 — see PLAN.md guiding
    principle #7 and the "Test-TC methodology" note):
      - tc=3+0.03 -> 3 s + 30 ms/move increment, CLOCK-based (default $TC).
                   This is the same TC the SPSA tuner uses, so there is no
                   tune->confirm transfer gap, and it exercises the real
                   time-management code (active under a clock, unlike fixed
                   movetime). 1% increment = the Stockfish convention; reaches
                   ~depth 16, generalizes across time controls.
      - Pass -MoveTime 0.1 for the optional fixed 100 ms/move sanity gauntlet
        (the old Little Blitzer condition) at a phase boundary — NOT the
        per-feature gate any more.
      - LTC confirmation runs at tc=10+0.1 (pass -TC "10+0.1") at phase
        boundaries and for TC-suspect features.
      - Pass -Nodes N for a fixed-NODES diagnostic (10.0b) — it removes speed
        AND time management, so it answers "is the remaining gap pure search
        quality?" and nothing else. Never a strength gate.
      - Hash 64 MB, Threads 1, UHO_Lichess_4852_v1.epd opening book (random
        order). Adopted 2026-07-17: the Stockfish/OpenBench-standard
        "Unbalanced Human Openings" set — 2,632,036 positions, 3–4 moves deep,
        curated to a ~+0.5-pawn White edge (the 4852 = the 0.48–0.52 eval
        band). Played from both colours per pair, so the imbalance is
        symmetric: unbiased but decisive. Cuts the draw rate (~56% → ~35–45%
        at our level), so SPRTs resolve in substantially fewer games; kills
        opening reuse forever (SuperGM's 2,668 lines were exhausted by any
        run > 5,336 games, correlating 23% of pairs in 7.2b). Draw rates and
        logistic-Elo magnitudes are NOT comparable to pre-UHO runs; verdicts
        are (each SPRT is self-contained). Legacy PGN books still work via
        -Book (format auto-detected from the extension),
        each opening played from both colours (-games 2 -repeat).
      - AFFINITY + CALIBRATION: the 2026-07-21 null result (+9.34 +/- 8.20)
        was evidence worth investigating, not proof of a persistent +9 Elo
        bias. The old [-3,+3] null SPRT was invalid as a calibration because
        true 0 lies midway between its hypotheses and has no preferred result.
        Real implementation hazards were present: fastchess before 1.7.0 did
        not correctly apply Windows process affinity, and 1.8.0 auto-topology
        guesses SMT siblings from alternating logical CPU IDs. This harness
        requires >=1.7.0, discovers physical cores through Windows, supplies
        an explicit CPU list, and uses fixed-size confidence-interval
        equivalence for null calibration.
      - model=normalized (nElo) — fastchess default, more time-control-robust
        than logistic Elo.

    IMPORTANT — concurrency:
      In a self-play game only the side to move computes, so ~16 concurrent
      games already saturate 16 physical cores. Oversubscribing (e.g. the 30
      logical processors) halves NPS and changes the depth reached, distorting
      results. The default detects physical cores and leaves two free; it does
      not derive concurrency or affinity from logical-CPU numbering.

    CALIBRATION CHECK — run this FIRST, before testing any feature:
        ./tools/sprt.ps1 `
            -EngineA "tools\test_engines\rarog-null.exe" `
            -EngineB "tools\test_engines\rarog-null.exe" `
            -NameA "NullA" -NameB "NullB" -Mode calibrate
        Calibration is a fixed 30,000-game identical-binary match. PASS
        requires the entire 95% normalized-Elo (nElo) interval inside
        [-5,+5] and zero anomalies.

.PARAMETER EngineA
    Path to the new/candidate engine (usually in tools\test_engines).

.PARAMETER EngineB
    Path to the baseline engine (the current integration head, or a released
    reference copied into tools\test_engines).

.PARAMETER NameA / NameB
    Display names. Defaults: "New" / "Base".

.PARAMETER Mode
    "gainer"       -> H0: elo<=3,  H1: elo>=10 (default; demand a material gain).
                      NOTE these are the HARNESS defaults, not the project's.
                      AGENTS.md makes [0,3] the default bracket: wide bounds
                      anchored high REJECT a small true gain, so pass
                      -Elo0 0 -Elo1 3 for an ordinary candidate.
    "simplify"     -> H0: elo<=-5, H1: elo>=0     (non-regression / cleanup).
    "calibrate"    -> fixed-size identical-binary null match; no SPRT. Asserts
                      the two arms are symmetric.
    "fixed"        -> fixed-size match of -Games games between DIFFERENT arms;
                      no SPRT and no early stop. This is the ablation and
                      observation mode: it reports an Elo with an interval and
                      never decides anything. Use it for matched ablation
                      (see PROCESS.md), never as a strength gate.
    The explicit -Elo0/-Elo1 parameters override the mode if supplied.

.PARAMETER Elo0 / Elo1
    SPRT hypotheses for "gainer" mode. Defaults 3 and 10 nElo: the project is
    intentionally parking marginal changes while larger pre-NNUE work remains.
    Override prospectively for a broad/risky bundle, never after looking at its
    games.

.PARAMETER MaxGames
    Maximum games for an SPRT mode. Default 16000 and must be positive/even.
    Reaching it without H1 means park/revert, not acceptance from the point
    estimate. Fixed/calibration modes continue to use -Games.

    Why 16000 and not a rounder 12000: the LLR drift of this project's own gates
    fits `drift/game ~ 8.3e-6 * (Elo1-Elo0) * (true_nElo - midpoint)`, calibrated
    within 1% on three runs (RAR-M10). Under the default [3,10] that puts a
    candidate sitting exactly ON H1 at about 14,500 games to the boundary, so a
    12,000 cap would park a share of the very changes the bounds are meant to
    accept. 16000 leaves headroom for that case plus variance. Raise it
    PROSPECTIVELY for tighter bounds; never after looking at a run's games.

.PARAMETER Hash
    Hash MB per engine. Default 64 (matches deployment).

.PARAMETER Concurrency
    Parallel games. Default 0 auto-detects physical cores and leaves two free.

.PARAMETER Games
    Fixed game count for -Mode calibrate. Default 30000. If the final interval
    is still too wide, the result is explicitly inconclusive.

.PARAMETER CalibrationTolerance
    Calibration equivalence tolerance. Default 5 nElo. PASS requires the full
    reported 95% confidence interval inside [-tolerance,+tolerance].

.PARAMETER Seed
    Opening randomization seed. Default 0 generates and records a seed.

.PARAMETER TC
    Clock time control "base+inc" in seconds. Default "3+0.03" (the unified
    SPSA/SPRT TC). Use "10+0.1" for the LTC phase-gate. Ignored if -MoveTime
    is supplied.

.PARAMETER MoveTime
    Fixed seconds-per-move. Default 0 (use clock TC instead). Set 0.1 for the
    optional fixed 100 ms/move Little Blitzer sanity gauntlet; this disables
    the clock and time-management is not exercised.

.PARAMETER Nodes
    Fixed NODES-per-move (fastchess `nodes=N`). Default 0 (use clock TC).
    Mutually exclusive with -MoveTime.

    Added for 10.0(b): it removes BOTH speed and time management from the
    comparison, which is the only way to ask "is the gap pure search quality?"
    of an engine that matches us on NPS. Use it for that diagnostic and for
    cross-engine search-accuracy questions — never as a strength gate, because
    a node-limited match cannot see time management at all, and TM is the one
    thing 9.7.5 identified as a live ~16 Elo lever.

    ⚠ Equal nodes is NOT equal work across different engines: a node means
    whatever each engine counts, and Rarog counts interior + qsearch nodes.
    Treat the absolute Elo as a comparison against the CLOCK result for the
    same pair, not as a rating.

.PARAMETER TimeMargin
    fastchess timeout margin in milliseconds. Default 20. This prevents small
    Windows scheduler / process IO jitter from being counted as a time
    forfeit. It does not change the engine's own time budget.

.PARAMETER AllowDirtyTree
    Permit an engine built from uncommitted changes. Refused by default (4.10.9):
    such a binary cannot be reproduced from git, so a ledger row citing it is a
    promise that someone is still storing the evidence. Use only for a
    deliberate throwaway screen, and say why in the registration.

.PARAMETER ExpectRevision
    Refuse to start unless BOTH engines' manifests record a git SHA with this
    prefix. A gate that measures a different revision than the one it registers
    is not evidence for that revision.

.PARAMETER Adjudicate
    Opt IN to fastchess draw and resign adjudication under the named
    strength-v2 profile. Off by default since 2026-09-01: RAR-M16 priced
    playing games out at about 10% wall time, against RAR-M15's finding that
    adjudication destroys 52.7% of all endgames before they are reached.

    Use it only when wall time genuinely binds AND the change provably cannot
    touch conversion or defensive holding. A result produced with this flag is
    not comparable with one produced without it.

.PARAMETER NoAdjudication
    Omit fastchess draw and resign adjudication. This is now the DEFAULT; the
    switch is retained because it still describes exactly what happens, so
    every recipe recorded in EXPERIMENTS.md reproduces verbatim. Passing both
    -Adjudicate and -NoAdjudication is refused.

.PARAMETER Book
    Opening book, PGN or EPD (format auto-detected from the extension).
    Default tools\books\UHO_Lichess_4852_v1.epd. Balanced-book fallback:
    tools\books\IM_4mvs.pgn.

.PARAMETER FastchessPath
    Path to fastchess.exe. Default tools\bin\fastchess.exe (or found on PATH).

.EXAMPLE
    ./tools/sprt.ps1 `
        -EngineA "tools\test_engines\rarog-feat-probcut-pext-pgo.exe" `
        -EngineB "tools\test_engines\rarog-head-pext-pgo.exe" `
        -NameA "ProbCut" -NameB "Head" -Elo0 3 -Elo1 10 -MaxGames 16000
#>
param(
    [Parameter(Mandatory)][string]$EngineA,
    [Parameter(Mandatory)][string]$EngineB,
    [string]$NameA = "New",
    [string]$NameB = "Base",
    [ValidateSet("gainer", "simplify", "calibrate", "fixed")][string]$Mode = "gainer",
    [Nullable[int]]$Elo0 = $null,
    [Nullable[int]]$Elo1 = $null,
    [double]$Alpha = 0.05,
    [double]$Beta  = 0.05,
    [int]$Hash = 64,
    [int]$Concurrency = 0,
    # 8.13: engine Threads for BOTH sides. Concurrency and the affinity list
    # scale with it automatically; a multi-thread gate must be null-pair
    # calibrated at the same Threads value before it is trusted.
    [int]$Threads = 1,
    # 8.13 tie-breaker: per-engine Threads override (defaults to $Threads).
    # Enables the asymmetric 4T-vs-1T self-play delta. The core budget reserves
    # max(ThreadsA,ThreadsB) cores per game slot, so neither side oversubscribes.
    # A calibration (null) must stay symmetric — ThreadsA must equal ThreadsB.
    [Nullable[int]]$ThreadsA = $null,
    [Nullable[int]]$ThreadsB = $null,
    [int]$Games = 30000,
    [int]$MaxGames = 16000,
    [double]$CalibrationTolerance = 5,
    [int]$Seed = 0,
    [string[]]$OptionsA = @(),
    [string[]]$OptionsB = @(),
    [string]$TC = "3+0.03",
    [double]$MoveTime = 0,
    [int]$Nodes = 0,
    [int]$TimeMargin = 20,
    [switch]$Adjudicate,
    [switch]$NoAdjudication,
    [switch]$AllowDirtyTree,
    [string]$ExpectRevision = "",
    [string]$Book = "$PSScriptRoot\books\UHO_Lichess_4852_v1.epd",
    [string]$FastchessPath = "$PSScriptRoot\bin\fastchess.exe"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "harness_common.ps1")

# PowerShell's native-argument binding can deliver a comma-separated option
# list as one string. Normalize it before equality checks, manifests or UCI
# validation so all three describe what fastchess will actually receive.
# The leading comma is load-bearing: without it PowerShell unrolls an EMPTY
# result to $null, and `[string[]]$null` then rebuilds a ONE-element array
# holding $null. That fake element made the advertisement guard below throw on
# every options-free gate, and made $optArgsA emit a bare `option.` to
# fastchess. Return the array itself, always.
$splitOpts = {
    param($items)
    ,@($items | ForEach-Object { $_ -split ',' } |
        ForEach-Object { $_.Trim().Trim('"') } |
        Where-Object { $_ })
}
$OptionsA = & $splitOpts $OptionsA
$OptionsB = & $splitOpts $OptionsB

$strengthProfile = Get-StrengthTestProfile
$resignArgs = @(Get-StrengthTestResignArgs)
# ADJUDICATION IS OFF BY DEFAULT since 2026-09-01 (RAR-M16, maintainer
# decision). Playing games out costs about 10% wall time -- 97.5 games/min
# adjudicated against RAR-E06's 88.4 -- and adjudication destroys 52.7% of all
# endgames before they are reached (RAR-M15). It is not unfair, being symmetric
# between arms, but it is lossy, and the loss scales with how badly the engine
# converts: for an engine that converts KRP-KR at 52%, the adjudicated verdict
# and the played-out verdict disagree far more often than for one that converts
# it at 99%. Revisit the default if conversion ever gets that good.
#
# `-NoAdjudication` is kept and still means exactly what it says, so every
# recipe recorded in EXPERIMENTS.md (RAR-E06's included) reproduces verbatim.
if ($Adjudicate -and $NoAdjudication) {
    throw "-Adjudicate and -NoAdjudication are contradictory; pass at most one."
}
if (-not $Adjudicate) {
    $adjudicationArgs = @()
    $adjudicationLabel = "none (games play to a rules result)"
} else {
    $adjudicationArgs = @(
        '-draw'
        "movenumber=$($strengthProfile.DrawMoveNumber)"
        "movecount=$($strengthProfile.DrawMoveCount)"
        "score=$($strengthProfile.DrawScore)"
    ) + $resignArgs
    $adjudicationLabel = "$($strengthProfile.Name); resign=$($strengthProfile.ResignScore)/$($strengthProfile.ResignMoveCount)$(if ($strengthProfile.ResignTwoSided) { ' two-sided' } else { ' one-sided' }); draw=$($strengthProfile.DrawScore)/$($strengthProfile.DrawMoveCount) from move $($strengthProfile.DrawMoveNumber)"
}

# Per-engine Threads resolve to $Threads unless overridden. The game slot must
# hold the larger of the two, so the core arithmetic uses max(ThreadsA,ThreadsB).
if ($null -eq $ThreadsA) { $ThreadsA = $Threads }
if ($null -eq $ThreadsB) { $ThreadsB = $Threads }
if ($ThreadsA -lt 1 -or $ThreadsB -lt 1) { throw "-ThreadsA/-ThreadsB must be >= 1." }
$maxThreads = [Math]::Max($ThreadsA, $ThreadsB)

$concurrencyInfo = Resolve-HarnessConcurrency -Requested $Concurrency -ThreadsPerGame $maxThreads
$Concurrency = $concurrencyInfo.Concurrency
$AffinityCpus = Get-HarnessAffinityCpuList -Concurrency $Concurrency -ThreadsPerGame $maxThreads
$Seed = New-HarnessSeed -Requested $Seed

# AFFINITY vs THREADS (2026-07-24): fastchess 1.8.0 `-use-affinity` binds each
# GAME to a single core regardless of the engine Threads option — verified by
# direct core sampling: Threads=4 concurrency=3 pinned only 3 cores (4 engine
# threads crammed onto 1), starving every multi-thread search and corrupting the
# 8.13(d) SmpVariant comparison (VarA read -100 purely from starvation). At
# Threads=1 the one-core-per-game rule is exactly right and the explicit list
# still removes the Zen-3 CCX placement bias, so it stays. At Threads>1 the OS
# scheduler spreads the pool across cores far better than fastchess's broken
# pinning (sampled ~9 vs 3 busy cores), so `-use-affinity` is dropped and the
# Threads>1 null MUST be recalibrated to confirm the unpinned pool stays centred.
if ($maxThreads -gt 1) {
    $affinityArgs = @()
    Write-Host "AFFINITY: -use-affinity DROPPED for Threads>1 (fastchess 1.8.0 pins 1 core/game, which starves multi-thread engines). OS-scheduled across all physical cores. Null-calibrate at this Threads before trusting a verdict." -ForegroundColor Yellow
} else {
    $affinityArgs = @('-use-affinity', $AffinityCpus)
}

# 4.2a.4: an option this script ACCEPTS but the chosen mode cannot HONOR is
# the same defect class as a dead `--rset` -- the run completes, reports a
# plausible number, and measured something other than what was asked for.
# Basilisk hit exactly this: `-Games` was accepted in a mode that ignored it.
# Here, `-Games` is read only by calibrate/fixed while gainer/simplify size
# themselves from `-MaxGames`, so `-Mode gainer -Games 5000` silently ran to
# the default 16,000 instead. Refuse rather than reinterpret.
$modeIgnores = if ($Mode -eq "calibrate" -or $Mode -eq "fixed") {
    @{ Elo0 = "-MaxGames/-Elo0/-Elo1/-Alpha/-Beta describe an SPRT; this mode runs a fixed-size match with no stop rule"
       Elo1 = $null; Alpha = $null; Beta = $null; MaxGames = $null }
} else {
    @{ Games = "-Games sizes calibrate/fixed matches; an SPRT is sized by -MaxGames"
       CalibrationTolerance = "-CalibrationTolerance is the calibrate-mode null bound; an SPRT is bounded by -Elo0/-Elo1" }
}
$ignored = @($modeIgnores.Keys | Where-Object { $PSBoundParameters.ContainsKey($_) })
if ($ignored.Count -gt 0) {
    $why = @($ignored | ForEach-Object { $modeIgnores[$_] } | Where-Object { $_ }) | Select-Object -First 1
    throw ("-Mode $Mode ignores: $($ignored -join ', '). $why. " +
           "Remove the option or change -Mode; this script will not accept a " +
           "parameter it cannot honor.")
}

if ($Mode -eq "calibrate" -or $Mode -eq "fixed") {
    if ($Games -lt 2 -or ($Games % 2) -ne 0) { throw "-Games must be a positive even number." }
    if ($CalibrationTolerance -le 0) { throw "-CalibrationTolerance must be positive." }
    if ($ThreadsA -ne $ThreadsB) { throw "Calibration must be symmetric: -ThreadsA ($ThreadsA) must equal -ThreadsB ($ThreadsB)." }
}
if ($Mode -ne "calibrate" -and $Mode -ne "fixed" -and
    ($MaxGames -lt 2 -or ($MaxGames % 2) -ne 0)) {
    throw "-MaxGames must be a positive even number."
}

# Resolve SPRT bounds from mode unless explicitly overridden.
if ($null -eq $Elo0) { $Elo0 = if ($Mode -eq "simplify") { -5 } else { 3 } }
if ($null -eq $Elo1) { $Elo1 = if ($Mode -eq "simplify") {  0 } else { 10 } }
if ($Mode -ne "calibrate" -and $Mode -ne "fixed" -and $Elo0 -ge $Elo1) {
    throw "SPRT requires -Elo0 lower than -Elo1."
}

# Resolve the search limit: clock (default) unless a fixed movetime or a fixed
# node count is given. All three are mutually exclusive; fastchess would accept
# two limits at once and silently apply whichever it parses last, so refuse.
if ($MoveTime -gt 0 -and $Nodes -gt 0) {
    throw "-MoveTime and -Nodes are mutually exclusive: pick one search limit."
}
if ($Nodes -gt 0) {
    $tcArg   = "nodes=$Nodes"
    $tcLabel = "nodes=$Nodes (fixed nodes/move; NO time management)"
    Write-Host "NOTE: fixed-nodes match - speed and time management are both removed." -ForegroundColor Yellow
    Write-Host "      Diagnostic only (10.0b). Not a strength gate: TM is invisible here." -ForegroundColor Yellow
} elseif ($MoveTime -gt 0) {
    $tcArg   = "st=$MoveTime"
    $tcLabel = "st=$MoveTime (fixed ${MoveTime}s/move)"
} else {
    $tcArg   = "tc=$TC"
    $tcLabel = "tc=$TC (clock)"
}

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
$Book    = (Resolve-Path $Book).Path

$optionDetailsA = @(Get-EngineUciOptions -Path $EngineA -Detailed)
$optionDetailsB = @(Get-EngineUciOptions -Path $EngineB -Detailed)
$optionsAdvertisedA = @($optionDetailsA.Name)
$optionsAdvertisedB = @($optionDetailsB.Name)
$normalizeOption = { param($value) ($value -replace '\s+', ' ').Trim().ToLowerInvariant() }
function Assert-RequestedOptions {
    param([object[]]$Advertised, [string[]]$Wanted, [string]$Label)
    if (-not $Wanted -or @($Wanted).Count -eq 0) { return }
    $have = @($Advertised.Name | ForEach-Object { & $normalizeOption $_ })
    $missing = @($Wanted | Where-Object { $_ } |
        ForEach-Object { ($_ -split '=', 2)[0] } |
        Where-Object { $have -notcontains (& $normalizeOption $_) })
    if ($missing.Count -gt 0) {
        throw ("$Label does not advertise: $($missing -join ', '). Rebuild it before measuring; " +
               "fastchess would otherwise play the match at default values.")
    }
}
Assert-RequestedOptions -Advertised $optionDetailsA -Wanted $OptionsA -Label $NameA
Assert-RequestedOptions -Advertised $optionDetailsB -Wanted $OptionsB -Label $NameB

$shaA = Get-HarnessSha256 $EngineA
$shaB = Get-HarnessSha256 $EngineB
if ($Mode -eq "calibrate" -and $shaA -ne $shaB) {
    throw "Calibration requires byte-identical engine binaries (SHA-256 differs)."
}
if ($Mode -ne "calibrate" -and $shaA -eq $shaB) {
    # 8.13(d): identical binaries ARE legitimate when the two sides differ by
    # UCI options (the SmpVariant arms) OR by Threads (the 4T-vs-1T tie-break) —
    # running one binary is strictly better than two, since it removes the
    # ~0.36% per-build PGO offset from the measurement. Refuse only when the
    # binary, the options AND the thread counts all match, because then the
    # "test" really is a null dressed as an SPRT.
    $sameOptions = (($OptionsA -join '|') -eq ($OptionsB -join '|'))
    if ($sameOptions -and $ThreadsA -eq $ThreadsB) {
        throw "Identical binaries, options AND Threads require -Mode calibrate; an SPRT centered around 0 is not a valid null calibration."
    }
    Write-Host "NOTE: same binary on both sides, differing only by UCI options -" -ForegroundColor Yellow
    Write-Host "      A: $($OptionsA -join ', ')   B: $($OptionsB -join ', ')" -ForegroundColor Yellow
}
$fcInfo = Assert-AffinityFastchess -Path $fastchess

# Book format auto-detected from the extension (.epd -> format=epd, else pgn),
# so -Book can point at either the UHO EPD or a legacy PGN book.
$bookFormat = if ([System.IO.Path]::GetExtension($Book) -ieq ".epd") { "epd" } else { "pgn" }

$resultsDir = Join-Path $PSScriptRoot "results"
New-Item -ItemType Directory -Force -Path $resultsDir | Out-Null
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$pgnOut    = Join-Path $resultsDir "sprt_${NameA}_vs_${NameB}_${timestamp}.pgn"
$logOut    = Join-Path $resultsDir "sprt_${NameA}_vs_${NameB}_${timestamp}.log"
$manifestPath = [System.IO.Path]::ChangeExtension($pgnOut, ".manifest.txt")

# 9.7: copy both engines' provenance manifests (written by build_test.ps1 next
# to each binary) into the result dir, so the result is permanently
# self-describing: which SHA vs which SHA, both bench fingerprints, dirty
# flags. Warn-not-fail on absence — pre-9.7 binaries have no manifest.
# Local-only: tools/results/ is gitignored; nothing here reaches a release.
$engineManifests = @{}
foreach ($pair in @(@($EngineA, $NameA), @($EngineB, $NameB))) {
    $manifest = [System.IO.Path]::ChangeExtension($pair[0], ".json")
    if (Test-Path $manifest) {
        $manifestData = Get-Content $manifest -Raw | ConvertFrom-Json
        $engineManifests[$pair[1]] = $manifestData
        if ($manifestData.engine -and $manifestData.engine -ne (Split-Path $pair[0] -Leaf)) {
            throw "Manifest for $($pair[1]) names '$($manifestData.engine)', not the selected binary."
        }
        if ($manifestData.binary_sha256) {
            $actualHash = Get-HarnessSha256 $pair[0]
            if ($actualHash -ne $manifestData.binary_sha256) {
                throw ("PROVENANCE MISMATCH - sidecar does not describe the selected binary.`n" +
                       "  Engine:  $($pair[1])`n  Actual:  $actualHash`n" +
                       "  Sidecar: $($manifestData.binary_sha256)`nRebuild with tools/build_test.ps1.")
            }
        } else {
            Write-Warning "Legacy manifest for $($pair[1]) is not bound to its binary SHA-256."
        }
        if ($manifestData.verification -and $manifestData.verification -ne "bench") {
            throw "Manifest for $($pair[1]) records '$($manifestData.verification)', not bench verification."
        }
        if ($manifestData.flavor -like "*-tune") {
            throw "Manifest for $($pair[1]) is a tune build; rebuild a PGO gate binary."
        }
        # 4.10.9: a dirty tree is a REFUSAL, not a warning. The rule this
        # protects is AGENTS.md's evidence rule -- a ledger row must reproduce
        # its artifact without the branch it came from -- and a binary built
        # from uncommitted changes cannot, by construction. A warning here is
        # read once and forgotten; by the time the row is questioned the tree
        # is long gone. -AllowDirtyTree exists for a deliberate throwaway
        # screen and must be justified in the registration.
        if ($manifestData.git_dirty -and -not $AllowDirtyTree) {
            throw ("DIRTY TREE - $($pair[1]) was built from uncommitted changes at " +
                   "$($manifestData.git_sha), so this result cannot be reproduced " +
                   "from git alone.`nCommit the change and rebuild with " +
                   "tools/build_test.ps1, or pass -AllowDirtyTree and say why in " +
                   "the EXPERIMENTS.md registration.")
        }
        if ($manifestData.git_dirty) {
            Write-Warning ("$($pair[1]) was built from a DIRTY tree and -AllowDirtyTree " +
                           "was passed. This result is not reproducible from git.")
        }
        if ($manifestData.git_sha -and $ExpectRevision -and
            $manifestData.git_sha -notlike "$ExpectRevision*") {
            throw ("WRONG REVISION - $($pair[1]) was built at $($manifestData.git_sha), " +
                   "not the expected $ExpectRevision.`nA gate that measures a " +
                   "different revision than the one it registers is not evidence " +
                   "for that revision.")
        }
        Copy-Item $manifest (Join-Path $resultsDir "sprt_${NameA}_vs_${NameB}_${timestamp}.$($pair[1]).manifest.json") -Force
    } else {
        Write-Host "NOTE: no manifest next to $(Split-Path $pair[0] -Leaf) (pre-9.7 build) — result will lack provenance for $($pair[1])." -ForegroundColor Yellow
    }
}

# 8.10a COMPILER-EQUALITY GUARD (2026-07-22) - the toolchain-pin analogue for
# BINARIES. A rustc change between building engine A and engine B folds the
# compiler delta into the measured Elo, and no null pair can see it: a null
# runs ONE binary against itself, so both sides always share a compiler.
#
# This is not hypothetical. The 9.1 bump (1.97.0 -> 1.97.1) landed 2026-07-19
# 21:19, AFTER p82a-nocheckext was built. Every gate before that split reads
# -5.68..+30.75; the three run after it, all candidate-1.97.1 vs
# baseline-1.97.0, read -8.68 / -8.22 / -7.37. Tight clustering across three
# unrelated subsystems is the signature of a per-binary constant, not of three
# independently bad ideas. HARD-FAIL so it can never recur silently.
$compilers = @{}
foreach ($pair in @(@($EngineA, $NameA), @($EngineB, $NameB))) {
    if ($engineManifests.ContainsKey($pair[1])) {
        $compilers[$pair[1]] = $engineManifests[$pair[1]].rustc
    } else {
        Write-Warning ("No manifest for $($pair[1]) - compiler equality NOT checkable. " +
            "Rebuild it with tools/build_test.ps1 before trusting a small verdict.")
    }
}

if ($engineManifests.Count -eq 2) {
    $flavorA = $engineManifests[$NameA].flavor
    $flavorB = $engineManifests[$NameB].flavor
    if ($flavorA -and $flavorB -and $flavorA -ne $flavorB) {
        throw ("BUILD FLAVOR MISMATCH - both sides must use the same target/PGO contract.`n" +
               "  $NameA : $flavorA`n  $NameB : $flavorB")
    }
    if ($flavorA -and $flavorB) { Write-Host "  Build flavor equality OK: $flavorA" }
}
if ($compilers.Count -eq 2) {
    $cA = $compilers[$NameA]; $cB = $compilers[$NameB]
    if ($cA -ne $cB) {
        throw ("COMPILER MISMATCH - this match would measure the compiler, not the change.`n" +
               "  $NameA : $cA`n  $NameB : $cB`n" +
               "Rebuild BOTH engines with the pinned toolchain (rust-toolchain.toml) " +
               "via tools/build_test.ps1, then re-run.")
    }
    Write-Host "  Compiler equality OK: $cA"
}

$repoSha = (git rev-parse HEAD 2>$null)
if (-not $repoSha) { $repoSha = "n/a" } else { $repoSha = $repoSha.Trim() }
@(
    "mode:            $Mode"
    "engineA:         $NameA = $EngineA"
    "engineA_sha256:  $shaA"
    "engineB:         $NameB = $EngineB"
    "engineB_sha256:  $shaB"
    "repo_revision:   $repoSha"
    "test_design:     $(if ($Mode -eq 'calibrate') { "fixed ${Games}-game null; tolerance +/-${CalibrationTolerance} nElo" } elseif ($Mode -eq 'fixed') { "fixed ${Games}-game match; no stop rule" } else { "SPRT elo0=$Elo0 elo1=$Elo1 alpha=$Alpha beta=$Beta model=normalized" })"
    "game_budget:     $(if ($Mode -eq 'calibrate' -or $Mode -eq 'fixed') { $Games } else { $MaxGames })"
    "time_control:    $tcLabel; timemargin=${TimeMargin}ms"
    "adjudication:    $adjudicationLabel"
    "termination:     $(if ($Adjudicate) { 'ADJUDICATED - do NOT pool with natural-termination runs; different sampling processes with different draw rates bias a pooled estimate by the mixing ratio' } else { 'natural (games played out)' })"
    "hash_mb:         $Hash"
    "threads:         $(if ($ThreadsA -eq $ThreadsB) { $ThreadsA } else { "$NameA=$ThreadsA $NameB=$ThreadsB" })"
    "concurrency:     $Concurrency"
    "physical_cores:  $($concurrencyInfo.PhysicalCores)"
    "affinity_cpus:   $(if ($maxThreads -gt 1) { "(dropped: Threads>1, fastchess 1.8.0 1-core/game starves multi-thread; OS-scheduled)" } else { $AffinityCpus })"
    "book:            $Book"
    "book_sha256:     $(Get-HarnessSha256 $Book)"
    "opening_order:   random"
    "opening_seed:    $Seed"
    "optionsA:        $(if ($OptionsA) { $OptionsA -join ' ' } else { '(none)' })"
    "optionsB:        $(if ($OptionsB) { $OptionsB -join ' ' } else { '(none)' })"
    "advertised_A:    $($optionsAdvertisedA -join ', ')"
    "advertised_B:    $($optionsAdvertisedB -join ', ')"
    "fastchess:       $($fcInfo.Text)"
    "fastchess_sha256: $(Get-HarnessSha256 $fastchess)"
    "started_utc:     $((Get-Date).ToUniversalTime().ToString('u'))"
) | Set-Content -Path $manifestPath -Encoding utf8

Write-Host ""
Write-Host "======================================================="
Write-Host "  SPRT ($Mode): $NameA  vs  $NameB"
if ($Mode -eq "calibrate") {
    Write-Host "  Fixed null calibration: $Games games; 95% nElo CI must fit inside +/-$CalibrationTolerance"
} elseif ($Mode -eq "fixed") {
    # No SPRT is run in this mode -- `$sprtArgs` is empty -- so printing H0/H1
    # and a park/revert cap described a test that was not happening. An
    # observation that looks like a gate invites being read as one.
    Write-Host "  Fixed-size observation: $Games games, no SPRT and no early stop"
    Write-Host "  Reports an Elo with an interval; it decides nothing"
} else {
    Write-Host "  H0: elo<=$Elo0   H1: elo>=$Elo1   alpha=$Alpha  beta=$Beta  (nElo)"
    Write-Host "  Budget: $MaxGames games; no H1 at the cap means park/revert"
}
Write-Host "  TC: $tcLabel   Margin: ${TimeMargin} ms   Hash: ${Hash} MB   Conc: $Concurrency"
Write-Host "  Adjudication: $adjudicationLabel"
Write-Host "  CPUs: $AffinityCpus"
Write-Host "  Book: $(Split-Path $Book -Leaf)"
Write-Host "  Runner: $($fcInfo.Text)"
Write-Host "  Manifest: $manifestPath"
Write-Host "  PGN:  $pgnOut"
Write-Host "  Log:  $logOut  (full output; console shows report blocks only)"
Write-Host "======================================================="
Write-Host ""

# Per-engine UCI options (8.10a): "Name=Value" pairs become option.Name=Value
# so ONE binary can be A/B-tested on a knob without a rebuild. Empty by
# default, so the emitted fastchess command is byte-identical to before -
# no null-pair re-calibration is required for the default path.
$optArgsA = @($OptionsA | ForEach-Object { "option.$_" })
$optArgsB = @($OptionsB | ForEach-Object { "option.$_" })

$rounds = if ($Mode -eq "calibrate" -or $Mode -eq "fixed") {
    [int]($Games / 2)
} else {
    [int]($MaxGames / 2)
}
$sprtArgs = if ($Mode -eq "calibrate" -or $Mode -eq "fixed") {
    @()
} else {
    @('-sprt', "elo0=$Elo0", "elo1=$Elo1", "alpha=$Alpha", "beta=$Beta", 'model=normalized')
}

# Console-noise filter (2026-07-16): the per-game 'Started game …' / normal
# 'Finished game … {Draw/wins by adjudication}' / 'Score of …' lines bury the
# periodic Elo/LLR report blocks and make it impossible to scroll back through
# how the result progressed. So: TEE the FULL stream to $logOut (nothing lost —
# grep/scroll it for detail), and on the CONSOLE keep everything EXCEPT that
# per-game noise. Keep-by-default is deliberate — report blocks, errors, and
# any time-loss / disconnect / illegal 'Finished game' lines (the SPRT canaries)
# all still print. `-ratinginterval 20` reports state every 20 games.
$dropNoise = {
    param($l)
    $l = "$l"
    if ($l -match '^\s*Started game \d+ of') { return $true }
    if ($l -match '^\s*Score of .+ vs .+:\s*\d+ - \d+ - \d+') { return $true }
    # Drop only NORMAL game finishes; keep anomalies (time loss / disconnect /
    # illegal / crash / forfeit) so a poisoned SPRT is still visible on console.
    if (($l -match '^\s*Finished game \d') -and
        ($l -notmatch '(?i)(on time|timeout|disconnect|illegal|crash|forfeit|stall)')) { return $true }
    return $false
}

# NOTE: flag names follow the fastchess man page (man.md). If your installed
# fastchess build rejects a flag, run `fastchess --help` and adjust here.
& $fastchess `
    -engine "cmd=$EngineA" "name=$NameA" "option.Hash=$Hash" "option.Threads=$ThreadsA" @optArgsA `
    -engine "cmd=$EngineB" "name=$NameB" "option.Hash=$Hash" "option.Threads=$ThreadsB" @optArgsB `
    -each $tcArg "timemargin=$TimeMargin" `
    -openings "file=$Book" "format=$bookFormat" order=random `
    -rounds $rounds -games 2 -repeat `
    -concurrency $Concurrency `
    @affinityArgs `
    -srand $Seed `
    -ratinginterval 20 `
    @sprtArgs `
    @adjudicationArgs `
    -pgnout "file=$pgnOut" `
    -output format=fastchess 2>&1 |    # console ticker format (not the PGN path)
    Tee-Object -FilePath $logOut |
    Where-Object { -not (& $dropNoise $_) }
# $LASTEXITCODE reflects fastchess (Tee-Object/Where-Object are cmdlets and do
# not touch it), so the exit check below stays valid.

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Error "fastchess exited with code $LASTEXITCODE — no games were played."
} else {
    Assert-NoAffinityFailure -LogPath $logOut
    Assert-NoMatchAnomaly -LogPath $logOut
    Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value @(
        "completed_utc:   $((Get-Date).ToUniversalTime().ToString('u'))"
        "pgn_sha256:      $(Get-HarnessSha256 $pgnOut)"
        "log_sha256:      $(Get-HarnessSha256 $logOut)"
    )
    Write-Host ""
    Write-Host "Match finished. PGN: $pgnOut"
    Write-Host "Full console log (all per-game lines): $logOut"

    if ($Mode -ne "calibrate" -and $Mode -ne "fixed") {
        Write-Host "Only an H1 boundary in the log promotes the candidate; a game-budget stop is unresolved and must be parked/reverted."
    }

    if ($Mode -eq "calibrate") {
        $eloLine = Select-String -LiteralPath $logOut `
            -Pattern '\bnElo:\s*(?<estimate>[+-]?\d+(?:\.\d+)?)\s*\+/-\s*(?<error>\d+(?:\.\d+)?)' |
            Select-Object -Last 1
        if (-not $eloLine) { throw "Could not parse the final Elo confidence interval from '$logOut'." }

        $estimate = [double]$eloLine.Matches[0].Groups['estimate'].Value
        $error = [double]$eloLine.Matches[0].Groups['error'].Value
        $lower = $estimate - $error
        $upper = $estimate + $error
        $passes = $lower -ge -$CalibrationTolerance -and $upper -le $CalibrationTolerance
        Write-Host ""
        Write-Host ("Calibration 95% nElo CI: [{0:F2}, {1:F2}]; required inside [-{2:F2}, +{2:F2}]" -f $lower, $upper, $CalibrationTolerance)
        if ($passes) {
            Write-Host "CALIBRATION PASS" -ForegroundColor Green
        } else {
            throw "CALIBRATION INCONCLUSIVE/FAIL: the confidence interval does not establish the requested bias bound. Increase -Games only after resolving anomalies."
        }
    }
}
