<#
.SYNOPSIS
    Run a Rarog gate, fixed match, null pair, tune or gauntlet on Colosseum CLI,
    with every guard the fastchess path enforces.

.DESCRIPTION
    A thin wrapper. The conditions live in the committed run files under
    tools/colosseum/; the cap, the seed and the run directory belong to the
    experiment and stay on the command line. This script adds nothing to the
    measurement and decides nothing about it. What it does is refuse to start a
    run that could not be believed afterwards:

      - the host is idle (no engine, harness or build process; CPU under the
        ceiling), which AGENTS.md has required since RAR-M48 discarded a pool
        measured on a busy box, and which no script checked before B.2.6.1;
      - tools/bin/colosseum-cli.exe is the build tools/colosseum/colosseum.pin.json
        pins, by SHA-256;
      - each engine's sidecar describes that exact binary, records a bench
        verification, carries the right build flavour, and was not built from a
        dirty tree (-AllowDirtyTree, justified in the registration, waives the
        last);
      - both arms share a build flavour and a compiler;
      - -ExpectRevision, when given, matches both sidecars;
      - an SPSA surface names only options the tune binary advertises, as spins,
        at the engine's own defaults and ranges, still resolving at the horizon,
        and a Core* surface requires a tune build that advertises them;
      - the configuration Colosseum resolves is Rarog's policy, field by field:
        no adjudication, the registered clock and margin, Hash and Threads, the
        UHO book in random order, placement automatic with a core of headroom
        or an explicit list of game cores (one logical CPU per physical core,
        no SMT sibling, a game core left free), never CPU 0, and the
        registered concurrency;
      - the run directory is described by a manifest that hashes every input,
        and by the dry run that produced the configuration;
      - after the run, the CLI's own record shows no fault and no excess time
        loss, and the outputs are hashed into the manifest.

    fastchess, weather-factory, tools/sprt.ps1 and tools/spsa.ps1 remain
    installed and working as the backup and second opinion. They are the
    cross-check, not the main path (PROCESS.md, "Harness").

.PARAMETER Mode
    sprt      -> a registered gate. Needs -MaxPairs (the cap from RAR-M10).
    match     -> a fixed-length measurement with an interval. Decides nothing.
    calibrate -> the null pair, owed only on a runner, scheduler or topology
                 change (RAR-M03). Requires byte-identical binaries.
    spsa      -> a tune under PLAN rule 7c: one run to the registered -Ceiling,
                 read at checkpoints by tools/spsa_read.py and stopped when its
                 gradients are gone. Needs -ConfigGroup and -Ceiling.
    gauntlet  -> a rating gauntlet; participants are passed in -ExtraArgs.

.PARAMETER NameA
    Arm A's name, passed to the runner as its label (--a-label), so it names
    the side in the PGN, the run record and Colosseum's search statistics.
    -NameB is arm B's. The two must differ: with one executable on both sides
    the runner would otherwise merge the two sides' statistics under one name.

.PARAMETER ThreadsB
    Arm B's thread count, when it differs from arm A's (-Threads): a
    self-relative scaling match plays one engine at four threads against
    itself at one. A sprt or match only. Each game slot needs a physical core
    per thread of the larger side (--cores-per-game).

.PARAMETER BaseMsB
    Arm B's clock base, when it differs from arm A's (-BaseMs): a time-odds
    match plays one engine at twice the time against itself. -IncrementMsB is
    arm B's increment; each defaults to arm A's when not given. A sprt or
    match on a clock only.

.PARAMETER Bracket
    Which SPRT run file: default [0,3], removal [-1.75,0.25], repair [-5,5],
    wide [0,10]. Bounds never change after games are seen.

.PARAMETER ExpectRevision
    Refuse to start unless every engine sidecar records a git SHA with this
    prefix. A gate that measures a different revision than the one it registers
    is not evidence for that revision.

.PARAMETER ExpectBench
    One registered bench fingerprint per arm, in order. The sidecar records what
    the binary benched; this is where the registration says what it should have
    benched. AGENTS.md's one failure mode is a stale binary measured, and a
    fingerprint is the cheapest thing that catches it.

.PARAMETER Ceiling
    A tune's registered ceiling in iterations (PLAN rule 7c: 6,000 unless the
    registration argues another). The surface files must be the rule 7c
    conversion for it: tools/spsa_config_to_colosseum.py <group> --rule7c
    --ceiling <N>. The run is played a segment at a time with the runner's
    --stop-after-iteration, to iteration -FirstCheckpoint (2,000) and then every
    -CheckpointEvery (1,000), and at each checkpoint tools/spsa_read.py reads
    the last -Window (1,000) iterations and records the verdict:
      CONTINUE         the next segment runs;
      STOP_QUIET       the tune stops; theta is the checkpoint's rounded centres;
      STOP_CEILING     the ceiling is reached; theta likewise;
      STOP_RAIL        the tune stops for a new registration of the range;
      FAULT            the run recorded faults; it stops for the maintainer, who
                       may accept them once analysed (python tools/spsa_read.py
                       <dir> --accept-faults "<reason>") and run the command again;
      STOP_MAINTAINER  a file named STOP in the run directory stops the tune at
                       the next checkpoint, with that checkpoint's centres as theta.
    Running the same command again resumes an interrupted tune at its recorded
    seed; a stopped tune is never extended.

.PARAMETER AllowShortRun
    Lifts rule 7c's production shape (first checkpoint 2,000, every 1,000,
    window 1,000, 30 games an iteration on 15 slots) for an instrument smoke on
    a separately generated surface. Never for a registered tune.

.PARAMETER CategoricalTuneBuild
    A categorical A/B on ONE tune build: the same executable on both sides,
    differing only by UCI options (which exist only under `tune`). Waives the
    rule that a gate binary is a PGO build, because a same-binary comparison
    carries no build offset; `match` mode only, and both sidecars must still
    be bench-verified tune builds. The manifest records the waiver.

.PARAMETER AllowBusyHost
    Record the host as busy and run anyway. For a throwaway smoke only; say why
    in the registration. The manifest carries the waiver.

.PARAMETER DryRun
    Resolve and check everything, write the manifest and the dry-run JSON, and
    stop without playing a game.

.EXAMPLE
    ./tools/colosseum.ps1 -Mode sprt `
        -EngineA tools/test_engines/<candidate>.exe `
        -EngineB tools/test_engines/<baseline>.exe `
        -NameA candidate -NameB baseline `
        -MaxPairs 24000 -Seed 7 -Dir tools/results/<experiment>

.EXAMPLE
    ./tools/colosseum.ps1 -Mode spsa -Engine tools/test_engines/<tune build>.exe `
        -ConfigGroup <group> -Ceiling 6000 -Seed 7 -Dir tools/results/<experiment>
#>
param(
    [ValidateSet("sprt", "match", "calibrate", "spsa", "gauntlet")][string]$Mode = "sprt",
    [ValidateSet("default", "removal", "repair", "wide")][string]$Bracket = "default",
    [string]$EngineA = "",
    [string]$EngineB = "",
    [string]$Engine = "",
    [string]$NameA = "New",
    [string]$NameB = "Base",
    [Parameter(Mandatory)][string]$Dir,
    [int]$MaxPairs = 0,
    [int]$Games = 0,
    [string]$ConfigGroup = "",
    [int]$Ceiling = 0,
    [int]$FirstCheckpoint = 2000,
    [int]$CheckpointEvery = 1000,
    [int]$Window = 1000,
    [int]$GamesPerIteration = 30,
    [switch]$AllowShortRun,
    [int]$Seed = 0,
    [string[]]$OptionsA = @(),
    [string[]]$OptionsB = @(),
    [int]$Hash = 64,
    [int]$Threads = 1,
    [ValidateRange(1, 64)][int]$ThreadsB = 0,
    [ValidateRange(1, 3600000)][int]$BaseMsB = 0,
    [ValidateRange(0, 60000)][int]$IncrementMsB = 0,
    [int]$BaseMs = 3000,
    [int]$IncrementMs = 30,
    [int]$MarginMs = 20,
    # Fixed nodes per move instead of the clock, -Mode match only: a
    # deterministic diagnostic that removes speed and time management (the
    # equal-node companion of an equal-time read). Selects match-fixed-nodes.toml
    # and is checked against the resolved configuration like the clock is.
    [long]$Nodes = 0,
    [int]$Concurrency = 0,
    [string]$ExpectRevision = "",
    [long[]]$ExpectBench = @(),
    [switch]$AllowDirtyTree,
    [switch]$CategoricalTuneBuild,
    [switch]$AllowBusyHost,
    [switch]$DryRun,
    [double]$MaxHostBusyPercent = 15,
    [double]$TimeLossRateCeiling = 0.5,
    [string[]]$ExtraArgs = @(),
    [string]$RunFile = "",
    [string]$Book = "$PSScriptRoot\books\UHO_Lichess_4852_v1.epd",
    [string]$CliPath = "$PSScriptRoot\bin\colosseum-cli.exe",
    [string]$PinPath = "$PSScriptRoot\colosseum\colosseum.pin.json"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "harness_common.ps1")

# PowerShell's native-argument binding can deliver a comma-separated option list
# as one string. The leading comma is load-bearing: without it an EMPTY result
# unrolls to $null and [string[]]$null rebuilds a one-element array holding
# $null, which then reaches the advertisement guard as a phantom option.
$splitOpts = {
    param($items)
    ,@($items | ForEach-Object { $_ -split ',' } |
        ForEach-Object { $_.Trim().Trim('"') } |
        Where-Object { $_ })
}
$OptionsA = & $splitOpts $OptionsA
$OptionsB = & $splitOpts $OptionsB

# ─── Parameters the mode cannot honour ────────────────────────────────────
# An option this script ACCEPTS but the chosen mode IGNORES is the defect class
# that let `-Mode gainer -Games 5000` run to a different budget than the operator
# asked for. Refuse rather than reinterpret.
$modeIgnores = switch ($Mode) {
    "sprt"      { @{ Games = "-Games sizes a fixed match; an SPRT is capped by -MaxPairs"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Ceiling = "-Ceiling is a tune's ceiling"
                     FirstCheckpoint = "-FirstCheckpoint is a tune's"
                     CheckpointEvery = "-CheckpointEvery is a tune's"
                     Window = "-Window is a tune's read"
                     GamesPerIteration = "-GamesPerIteration is a tune's"
                     AllowShortRun = "-AllowShortRun is a tune smoke's" } }
    "match"     { @{ MaxPairs = "-MaxPairs caps an SPRT; a fixed match is sized by -Games"
                     Bracket = "-Bracket selects SPRT bounds; a fixed match has none"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Ceiling = "-Ceiling is a tune's ceiling"
                     FirstCheckpoint = "-FirstCheckpoint is a tune's"
                     CheckpointEvery = "-CheckpointEvery is a tune's"
                     Window = "-Window is a tune's read"
                     GamesPerIteration = "-GamesPerIteration is a tune's"
                     AllowShortRun = "-AllowShortRun is a tune smoke's" } }
    "calibrate" { @{ MaxPairs = "-MaxPairs caps an SPRT; a null pair is sized by -Games"
                     Bracket = "-Bracket selects SPRT bounds; a null pair has none"
                     ThreadsB = "-ThreadsB gives arm B its own thread count; a null pair's arms are identical"
                     BaseMsB = "-BaseMsB gives arm B its own clock; a null pair's arms are identical"
                     IncrementMsB = "-IncrementMsB gives arm B its own clock; a null pair's arms are identical"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Ceiling = "-Ceiling is a tune's ceiling"
                     FirstCheckpoint = "-FirstCheckpoint is a tune's"
                     CheckpointEvery = "-CheckpointEvery is a tune's"
                     Window = "-Window is a tune's read"
                     GamesPerIteration = "-GamesPerIteration is a tune's"
                     AllowShortRun = "-AllowShortRun is a tune smoke's" } }
    "spsa"      { @{ MaxPairs = "-MaxPairs caps an SPRT; a tune is sized by -Ceiling"
                     Games = "-Games sizes a fixed match; a tune is sized by -Ceiling"
                     Bracket = "-Bracket selects SPRT bounds; a tune has none"
                     EngineA = "a tune has one engine: pass -Engine"
                     EngineB = "a tune has one engine: pass -Engine"
                     ThreadsB = "-ThreadsB gives arm B its own thread count; a tune has one engine"
                     BaseMsB = "-BaseMsB gives arm B its own clock; a tune has one engine"
                     IncrementMsB = "-IncrementMsB gives arm B its own clock; a tune has one engine" } }
    "gauntlet"  { @{ MaxPairs = "-MaxPairs caps an SPRT"
                     Bracket = "-Bracket selects SPRT bounds"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Ceiling = "-Ceiling is a tune's ceiling"
                     FirstCheckpoint = "-FirstCheckpoint is a tune's"
                     CheckpointEvery = "-CheckpointEvery is a tune's"
                     Window = "-Window is a tune's read"
                     GamesPerIteration = "-GamesPerIteration is a tune's"
                     AllowShortRun = "-AllowShortRun is a tune smoke's"
                     EngineA = "gauntlet participants are passed in -ExtraArgs"
                     EngineB = "gauntlet participants are passed in -ExtraArgs"
                     ThreadsB = "-ThreadsB gives arm B its own thread count; gauntlet participants carry their own"
                     BaseMsB = "-BaseMsB gives arm B its own clock; a gauntlet has one clock"
                     IncrementMsB = "-IncrementMsB gives arm B its own clock; a gauntlet has one clock" } }
}
$ignored = @($modeIgnores.Keys | Where-Object { $PSBoundParameters.ContainsKey($_) })
if ($ignored.Count -gt 0) {
    $why = @($ignored | ForEach-Object { $modeIgnores[$_] } | Where-Object { $_ }) | Select-Object -First 1
    throw ("-Mode $Mode ignores: $($ignored -join ', '). $why. Remove the option or " +
           "change -Mode; this script will not accept a parameter it cannot honour.")
}

# ─── Mode-specific requirements ───────────────────────────────────────────
$twoArmModes = @("sprt", "match", "calibrate")
# Arm B plays at -ThreadsB and on -BaseMsB/-IncrementMsB when they are given,
# otherwise at arm A's count and on arm A's clock.
$threadsB = if ($ThreadsB -gt 0) { $ThreadsB } else { $Threads }
$baseMsB = if ($PSBoundParameters.ContainsKey('BaseMsB')) { $BaseMsB } else { $BaseMs }
$incrementMsB = if ($PSBoundParameters.ContainsKey('IncrementMsB')) { $IncrementMsB } else { $IncrementMs }
$clockOdds = $baseMsB -ne $BaseMs -or $incrementMsB -ne $IncrementMs
if ($clockOdds -and $Nodes -gt 0) { throw "-BaseMsB and -IncrementMsB set a clock; a fixed-node read (-Nodes) has none." }
if ($twoArmModes -contains $Mode) {
    if (-not $EngineA -or -not $EngineB) { throw "-Mode $Mode needs -EngineA and -EngineB." }
}
if ($Mode -eq "sprt" -and $MaxPairs -le 0) {
    throw "-MaxPairs is required for a gate: the finite cap belongs to the EXPERIMENTS.md registration."
}
if ($Mode -eq "match" -and $PSBoundParameters.ContainsKey('Games') -and ($Games -lt 2 -or ($Games % 2) -ne 0)) {
    throw "-Games must be a positive even number."
}
if ($Mode -eq "spsa") {
    if (-not $Engine) { throw "-Mode spsa needs -Engine (the tune build)." }
    if (-not $ConfigGroup) { throw "-ConfigGroup is required: name a registered surface in tools/spsa_configs." }
    if ($Ceiling -le 0) { throw "-Ceiling is required: the registered ceiling in iterations (PLAN rule 7c)." }
    # PLAN rule 7c's production shape; only an instrument smoke may lift it.
    $production = [ordered]@{ FirstCheckpoint = 2000; CheckpointEvery = 1000; Window = 1000; GamesPerIteration = 30 }
    if (-not $AllowShortRun) {
        foreach ($name in $production.Keys) {
            $value = Get-Variable -Name $name -ValueOnly
            if ($value -ne $production[$name]) {
                throw "PLAN rule 7c's production shape has -$name $($production[$name]), not $value; -AllowShortRun is for an instrument smoke only."
            }
        }
        if ($Concurrency -ne 0 -and $Concurrency -ne 15) {
            throw "PLAN rule 7c's production shape plays 15 slots, not -Concurrency $Concurrency; -AllowShortRun is for an instrument smoke only."
        }
    }
    if ($Ceiling -lt $FirstCheckpoint -or $FirstCheckpoint -lt $Window -or $CheckpointEvery -lt 1 -or $Window -lt 1 -or $GamesPerIteration -lt 2) {
        throw "A tune needs window <= first checkpoint <= ceiling, a positive spacing and at least one game pair an iteration."
    }
    # The checkpoints, then the ceiling where it is not on the grid.
    $checkpoints = [System.Collections.Generic.List[int]]::new()
    for ($k = $FirstCheckpoint; $k -lt $Ceiling; $k += $CheckpointEvery) { $checkpoints.Add($k) }
    $checkpoints.Add($Ceiling)
}
if ($Mode -eq "gauntlet" -and $ExtraArgs.Count -eq 0) {
    throw "-Mode gauntlet needs its participants, labels and ratings in -ExtraArgs."
}

# ─── Guard 1: an idle host ────────────────────────────────────────────────
Write-Host ""
Write-Host "Preflight:"
$hostState = Assert-HarnessHostIdle -MaxBusyPercent $MaxHostBusyPercent -Allow:$AllowBusyHost

# ─── Guard 2: the pinned runner ───────────────────────────────────────────
$cli = Assert-ColosseumCli -Path $CliPath -PinPath $PinPath

# ─── Guard 3: inputs exist ────────────────────────────────────────────────
if ($Nodes -gt 0 -and $Mode -ne "match") { throw "-Nodes is for -Mode match only: a fixed-node read is a diagnostic, never a gate or a tune." }
$runFileDefaults = @{
    sprt      = Join-Path $PSScriptRoot "colosseum\sprt-$Bracket.toml"
    match     = Join-Path $PSScriptRoot ("colosseum\" + $(if ($Nodes -gt 0) { "match-fixed-nodes.toml" } else { "match-fixed.toml" }))
    calibrate = Join-Path $PSScriptRoot "colosseum\calibrate-null.toml"
    spsa      = Join-Path $PSScriptRoot "spsa_configs\colosseum\$ConfigGroup.run.toml"
    gauntlet  = Join-Path $PSScriptRoot "colosseum\gauntlet.toml"
}
if (-not $RunFile) { $RunFile = $runFileDefaults[$Mode] }
if (-not (Test-Path -LiteralPath $RunFile)) { throw "Run file not found: $RunFile" }
$RunFile = (Resolve-Path -LiteralPath $RunFile).Path

$engines = @()
if ($twoArmModes -contains $Mode) {
    if ($NameA -eq $NameB) { throw "-NameA and -NameB are both '$NameA'; the arms need distinct names." }
    $engines = @(
        [pscustomobject]@{ Path = $EngineA; Label = $NameA; Options = $OptionsA }
        [pscustomobject]@{ Path = $EngineB; Label = $NameB; Options = $OptionsB }
    )
} elseif ($Mode -eq "spsa") {
    $engines = @([pscustomobject]@{ Path = $Engine; Label = "tune"; Options = @() })
}
foreach ($arm in $engines) {
    if (-not (Test-Path -LiteralPath $arm.Path)) { throw "Not found: $($arm.Path)" }
    $arm.Path = (Resolve-Path -LiteralPath $arm.Path).Path
}
if (-not (Test-Path -LiteralPath $Book)) { throw "Not found: $Book" }
$Book = (Resolve-Path -LiteralPath $Book).Path

# ─── Guard 4: sidecar provenance, per arm ─────────────────────────────────
if ($CategoricalTuneBuild) {
    if ($Mode -ne "match") { throw "-CategoricalTuneBuild is for -Mode match only: a categorical is a fixed-length read, never a gate." }
    if ($engines[0].Path -ne $engines[1].Path) {
        throw ("-CategoricalTuneBuild requires ONE binary on both sides (a same-binary A/B carries no build offset); " +
               "got $($engines[0].Path) and $($engines[1].Path).")
    }
    if (($OptionsA -join '|') -eq ($OptionsB -join '|')) { throw "-CategoricalTuneBuild needs the two sides to differ by UCI options." }
    Write-Host "  Categorical A/B on one tune build: the PGO-gate rule is waived and recorded." -ForegroundColor Yellow
}
$manifests = @{}
foreach ($arm in $engines) {
    $kind = if ($Mode -eq "spsa" -or $CategoricalTuneBuild) { "tune" } else { "gate" }
    $manifests[$arm.Label] = Assert-EngineProvenance -Path $arm.Path -Label $arm.Label -Kind $kind `
        -RequireManifest:($Mode -eq "spsa") -RequireBinaryHash:($Mode -eq "spsa") `
        -AllowDirtyTree:$AllowDirtyTree -ExpectRevision $ExpectRevision
}

# ─── Guard 4b: the registered bench fingerprint ───────────────────────────
if ($ExpectBench.Count -gt 0) {
    if ($ExpectBench.Count -ne $engines.Count) {
        throw "-ExpectBench takes one fingerprint per arm; $($engines.Count) arm(s), $($ExpectBench.Count) given."
    }
    for ($i = 0; $i -lt $engines.Count; $i++) {
        $manifest = $manifests[$engines[$i].Label]
        if (-not $manifest) {
            throw "-ExpectBench was given but $($engines[$i].Label) has no sidecar to read a fingerprint from."
        }
        if ([long]$manifest.bench_nodes -ne $ExpectBench[$i]) {
            throw ("WRONG FINGERPRINT - $($engines[$i].Label) benched $($manifest.bench_nodes), " +
                   "not the registered $($ExpectBench[$i]).`nA stale binary is this project's " +
                   "commonest wrong measurement; rebuild the arm before measuring it.")
        }
    }
    Write-Host "  Bench fingerprints match the registration: $($ExpectBench -join ', ')"
}

# ─── Guard 5: both arms share a flavour and a compiler ────────────────────
if ($twoArmModes -contains $Mode) {
    Assert-EngineArmEquality -Manifests $manifests -LabelA $NameA -LabelB $NameB
}

# ─── Guard 6: binary identity matches the question being asked ────────────
if ($twoArmModes -contains $Mode) {
    $shaA = Get-HarnessSha256 $engines[0].Path
    $shaB = Get-HarnessSha256 $engines[1].Path
    if ($Mode -eq "calibrate" -and $shaA -ne $shaB) {
        throw "A null pair requires byte-identical engine binaries (SHA-256 differs)."
    }
    if ($Mode -ne "calibrate" -and $shaA -eq $shaB) {
        # One binary on both sides is legitimate when the arms differ by UCI
        # options: it removes the ~0.36% per-build PGO offset. Refuse only when
        # binary AND options match, because then the test is a null in disguise.
        if (($OptionsA -join '|') -eq ($OptionsB -join '|') -and $threadsB -eq $Threads -and -not $clockOdds) {
            throw ("Identical binaries AND options require -Mode calibrate; an SPRT " +
                   "centred on zero is not a valid null calibration.")
        }
        Write-Host "  NOTE: one binary on both sides, differing only by UCI options, threads or clock." -ForegroundColor Yellow
    }
}

# ─── Guard 7: every requested option is advertised ────────────────────────
$advertised = @{}
for ($i = 0; $i -lt $engines.Count; $i++) {
    $arm = $engines[$i]
    $details = @(Get-EngineUciOptions -Path $arm.Path -Detailed)
    $advertised[$arm.Label] = $details
    Assert-AdvertisedOptions -Advertised $details -Wanted $arm.Options -Label $arm.Label
    $armThreads = if ($i -eq 1) { $threadsB } else { $Threads }
    Assert-AdvertisedOptions -Advertised $details -Wanted @("Hash=$Hash", "Threads=$armThreads") -Label $arm.Label
}

# ─── Guard 8: the tune surface belongs to this binary and this ceiling ────
$surfaceFiles = @()
if ($Mode -eq "spsa") {
    $configs = Join-Path $PSScriptRoot "spsa_configs"
    $surfacePath = Join-Path $configs "config_$ConfigGroup.json"
    $fixedPath = Join-Path $configs "fixed_$ConfigGroup.json"
    if (-not (Test-Path $surfacePath)) { throw "Config not found: $surfacePath" }
    $surface = Get-Content $surfacePath -Raw | ConvertFrom-Json
    $fixed = if (Test-Path $fixedPath) { Get-Content $fixedPath -Raw | ConvertFrom-Json } else { $null }

    $names = @($surface.PSObject.Properties.Name)
    if ($fixed) { $names += @($fixed.PSObject.Properties.Name) }
    Assert-CoreSurfaceArm -Names $names -Advertised $advertised["tune"] -Flavor $manifests["tune"].flavor -ConfigGroup $ConfigGroup

    $tuned = @(Assert-TuneSurface -Advertised $advertised["tune"] -Surface $surface `
        -Iterations $Ceiling -Label (Split-Path $engines[0].Path -Leaf) -Fixed $fixed)
    Write-Host "  Tune surface verified: $($tuned.Count) coordinates at the ceiling N = $Ceiling"

    # The generated files are the registration in another notation. The files
    # this run reads, beside its run file, must be the rule 7c conversion of the
    # JSON for this ceiling, or the run would tune a surface nobody registered.
    if ((Split-Path $RunFile -Leaf) -ne "$ConfigGroup.run.toml") {
        throw "A tune's run file is the generated $ConfigGroup.run.toml, not $(Split-Path $RunFile -Leaf)."
    }
    $surfaceDir = Split-Path $RunFile -Parent
    python (Join-Path $PSScriptRoot "spsa_config_to_colosseum.py") $ConfigGroup --rule7c --ceiling $Ceiling --out $surfaceDir --check
    if ($LASTEXITCODE -ne 0) {
        throw ("The Colosseum surface for '$ConfigGroup' in $surfaceDir is not the rule 7c conversion of " +
               "config_$ConfigGroup.json at the ceiling N = $Ceiling. Regenerate it with " +
               "tools/spsa_config_to_colosseum.py $ConfigGroup --rule7c --ceiling $Ceiling and commit it before tuning.")
    }
    $surfaceFiles = @(
        $surfacePath
        $(if (Test-Path $fixedPath) { $fixedPath })
        (Join-Path $surfaceDir "$ConfigGroup.tune.toml")
    ) | Where-Object { $_ }
}

# ─── The command ──────────────────────────────────────────────────────────
$Dir = [System.IO.Path]::GetFullPath($Dir, (Get-Location).Path)
# An existing run directory resumes only under the configuration it was started
# with, the seed included, and a dry run does not check that; so the recorded
# seed is carried over rather than drawn afresh.
$existingConfig = Join-Path $Dir "resolved-config.json"
if (Test-Path -LiteralPath $existingConfig) {
    $existingSeed = [int64](Get-Content -LiteralPath $existingConfig -Raw | ConvertFrom-Json).master_seed
    if ($Seed -eq 0) {
        if ($existingSeed -lt 1 -or $existingSeed -gt [int]::MaxValue) {
            throw "$Dir records seed $existingSeed, which -Seed cannot carry; resume it with colosseum-cli directly."
        }
        $Seed = [int]$existingSeed
        Write-Host "  Resuming $Dir at its recorded seed $Seed" -ForegroundColor Yellow
    } elseif ($Seed -ne $existingSeed) {
        throw ("$Dir holds a run at seed $existingSeed; -Seed $Seed would be refused on resume. " +
               "Omit -Seed to resume that run, or choose a new -Dir.")
    }
}
$Seed = New-HarnessSeed -Requested $Seed

$expectedConcurrency = if ($Concurrency -gt 0) { $Concurrency } elseif ($Mode -eq "spsa") { 15 } else { 14 }
$commandArgs = @('--run-file', $RunFile)
if ($Mode -eq "gauntlet") { $commandArgs += @('tournament', 'run') }

switch ($Mode) {
    "sprt"      { $commandArgs += @($engines[0].Path, $engines[1].Path, '--max-pairs', "$MaxPairs") }
    "match"     { $commandArgs += @($engines[0].Path, $engines[1].Path)
                  if ($Games -gt 0) { $commandArgs += @('--games', "$Games") } }
    "calibrate" { $commandArgs += @($engines[0].Path, $engines[1].Path)
                  if ($Games -gt 0) { $commandArgs += @('--games', "$Games") } }
    "spsa"      { $commandArgs += @($engines[0].Path, '--iterations', "$Ceiling", '--games-per-iteration', "$GamesPerIteration") }
}
# Without labels the runner names each side after its executable, and a
# same-binary A/B becomes `name (A)` and `name (B)`; the labels give every
# report and the PGN the arms' registered names instead.
if ($twoArmModes -contains $Mode) { $commandArgs += @('--a-label', $NameA, '--b-label', $NameB) }
# A side-specific option list on the command line REPLACES the run file's list
# for that side (Colosseum's command-line-over-run-file rule applies to the
# whole key), so a bare `-OptionsA CoreX=1` would play arm A without Hash and
# Threads. The shared options are therefore restated ahead of the arm's own,
# unless the arm sets them itself; the policy check reads the result.
# Every side states its own list, always: the run file's lists carry the run
# file's Hash and Threads, so -Hash alone would otherwise never reach the
# engines (a time-odds leg at -Hash 256 resolved to the run file's 64).
function Get-SideOptions { param([string[]]$Own, [int]$SideThreads)
    $shared = @("Hash=$Hash", "Threads=$SideThreads")
    $names = @($Own | ForEach-Object { ($_ -split '=', 2)[0].Trim().ToLowerInvariant() })
    @($shared | Where-Object { $names -notcontains ($_ -split '=', 2)[0].ToLowerInvariant() }) + @($Own)
}
if ($Mode -ne "spsa" -and $Mode -ne "gauntlet") {
    foreach ($option in (Get-SideOptions $OptionsA $Threads)) { $commandArgs += @('--a-option', $option) }
    foreach ($option in (Get-SideOptions $OptionsB $threadsB)) { $commandArgs += @('--b-option', $option) }
}
if ($Concurrency -gt 0) { $commandArgs += @('--concurrency', "$Concurrency") }
if ($Nodes -gt 0) { $commandArgs += @('--a-nodes', "$Nodes", '--b-nodes', "$Nodes") }
if ($clockOdds) { $commandArgs += @('--b-base-ms', "$baseMsB", '--b-increment-ms', "$incrementMsB") }
$commandArgs += @('--seed', "$Seed", '--dir', $Dir)
if ($ExtraArgs.Count -gt 0) { $commandArgs += $ExtraArgs }

# ─── Guard 9: the configuration Colosseum resolves is Rarog's policy ──────
# Read from the CLI's own dry run, not from the run file: what matters is what
# the runner decided, after inheritance, overrides and topology.
$resultsDir = Join-Path $PSScriptRoot "results"
New-Item -ItemType Directory -Force -Path $resultsDir | Out-Null
$stamp = Get-Date -Format "yyyyMMdd_HHmmss"
$label = Split-Path $Dir -Leaf
$dryPath = Join-Path $resultsDir "colosseum_${Mode}_${label}_${stamp}.dry-run.json"
$manifestPath = Join-Path $resultsDir "colosseum_${Mode}_${label}_${stamp}.manifest.txt"
$logPath = Join-Path $resultsDir "colosseum_${Mode}_${label}_${stamp}.log"

# stdout is the JSON document and nothing else; the runner's notes go to stderr
# (a resume says "resuming the stored SPSA horizon" there), so the two are kept
# apart or the document does not parse on exactly the run that matters most.
$dryNotesPath = [System.IO.Path]::ChangeExtension($dryPath, '.notes.txt')
& $cli.Path @commandArgs --dry-run --json 2> $dryNotesPath > $dryPath
$dryNotes = "$(if (Test-Path -LiteralPath $dryNotesPath) { Get-Content -LiteralPath $dryNotesPath -Raw })"
if ($dryNotes.Trim()) { Write-Host "  Runner: $($dryNotes.Trim())" -ForegroundColor Yellow }
if ($LASTEXITCODE -ne 0) {
    Write-Host (Get-Content -LiteralPath $dryPath -Raw)
    # The runner's own reason travels in the refusal, so a guard the CLI
    # enforces (one time control per side, say) is named like the wrapper's.
    throw "colosseum-cli refused the dry run (exit $LASTEXITCODE): $($dryNotes.Trim()); nothing was played."
}
$dry = Get-Content -LiteralPath $dryPath -Raw | ConvertFrom-Json
$resolved = $dry.resolved_configuration

$violations = [System.Collections.Generic.List[string]]::new()
function Add-Violation { param([string]$Text) $violations.Add($Text) }

# The dry run names the subcommand; a gauntlet is `tournament run`.
if ($dry.command -ne $(if ($Mode -eq 'gauntlet') { 'tournament' } else { $Mode })) {
    Add-Violation "resolved command is '$($dry.command)', expected '$Mode'"
}
foreach ($rule in @('draw', 'resign', 'max_moves')) {
    if ($null -ne $resolved.adjudication.$rule) {
        Add-Violation "adjudication.$rule is set; Rarog plays games out (RAR-M15, RAR-M17)"
    }
}
$controls = @()
# A two-arm command resolves one clock per arm, a tune one clock, and a
# tournament one clock for the field.
foreach ($field in @('engine_a_time_control', 'engine_b_time_control', 'engine_time_control', 'time_control')) {
    if ($resolved.PSObject.Properties.Name -contains $field) { $controls += ,@($field, $resolved.$field) }
}
if ($controls.Count -eq 0) { Add-Violation "the resolved configuration states no time control" }
foreach ($pair in $controls) {
    $name = $pair[0]; $control = $pair[1]
    $kind = @($control.control.PSObject.Properties.Name)[0]
    if ($Nodes -gt 0) {
        # A fixed-node read must resolve to fixed nodes on every side; a clock
        # here would measure speed under a label that says it does not.
        if ($kind -ne 'Nodes') {
            Add-Violation "$name is a '$kind' control, expected fixed nodes (-Nodes $Nodes)"
        } elseif ([long]$control.control.Nodes.nodes -ne $Nodes) {
            Add-Violation "$name is $($control.control.Nodes.nodes) nodes a move, expected $Nodes"
        }
    } else {
        $expectedBase = if ($name -eq 'engine_b_time_control') { $baseMsB } else { $BaseMs }
        $expectedIncrement = if ($name -eq 'engine_b_time_control') { $incrementMsB } else { $IncrementMs }
        if ($kind -ne 'Increment') {
            Add-Violation "$name is a '$kind' control, expected the clock ${expectedBase}+${expectedIncrement} ms (pass -Nodes for a fixed-node read)"
        } else {
            if ([int]$control.control.Increment.base_ms -ne $expectedBase) {
                Add-Violation "$name base is $($control.control.Increment.base_ms) ms, expected $expectedBase"
            }
            if ([int]$control.control.Increment.inc_ms -ne $expectedIncrement) {
                Add-Violation "$name increment is $($control.control.Increment.inc_ms) ms, expected $expectedIncrement"
            }
        }
    }
    if ([int]$control.margin_ms -ne $MarginMs) {
        Add-Violation "$name margin is $($control.margin_ms) ms, expected $MarginMs"
    }
}
$optionSets = @()
foreach ($field in @('engine_a', 'engine_b', 'engine')) {
    if ($resolved.PSObject.Properties.Name -contains $field) { $optionSets += ,@($field, $resolved.$field.options) }
}
foreach ($pair in $optionSets) {
    $name = $pair[0]; $options = $pair[1]
    if ([int]$options.Hash.value -ne $Hash) { Add-Violation "$name Hash is $($options.Hash.value), expected $Hash" }
    $expectedThreads = if ($name -eq 'engine_b') { $threadsB } else { $Threads }
    if ([int]$options.Threads.value -ne $expectedThreads) { Add-Violation "$name Threads is $($options.Threads.value), expected $expectedThreads" }
}
# Tablebases: a side that probes must probe the same tables as the other, and
# the tables must be there, or the two sides play different games without a
# word in the record. The manifest names the path with its file counts.
$syzygyBySide = @{}
foreach ($pair in $optionSets) {
    $value = $pair[1].PSObject.Properties | Where-Object { $_.Name -eq 'SyzygyPath' } |
        ForEach-Object { "$($_.Value.value)" }
    $syzygyBySide[$pair[0]] = "$value"
}
$syzygyPaths = @($syzygyBySide.Values | Sort-Object -Unique)
$syzygyPath = ""
$syzygyRecord = "none"
if ($syzygyPaths.Count -gt 1) {
    $sides = ($syzygyBySide.GetEnumerator() | Sort-Object Name | ForEach-Object { "$($_.Name)='$($_.Value)'" }) -join ', '
    Add-Violation "SyzygyPath differs between the sides ($sides)"
} elseif ($syzygyPaths.Count -eq 1 -and $syzygyPaths[0]) {
    $syzygyPath = $syzygyPaths[0]
    $wdlFiles = 0; $dtzFiles = 0
    foreach ($folder in ($syzygyPath -split ';' | ForEach-Object { $_.Trim() } | Where-Object { $_ })) {
        if (Test-Path -LiteralPath $folder -PathType Container) {
            $wdlFiles += @(Get-ChildItem -LiteralPath $folder -Filter '*.rtbw' -File).Count
            $dtzFiles += @(Get-ChildItem -LiteralPath $folder -Filter '*.rtbz' -File).Count
        }
    }
    if ($wdlFiles -eq 0) { Add-Violation "SyzygyPath '$syzygyPath' holds no .rtbw tables" }
    $syzygyRecord = "$syzygyPath ($wdlFiles WDL, $dtzFiles DTZ files)"
}
if ($Mode -eq 'gauntlet') {
    # Participants carry their options in the plan. Hash is Rarog's policy;
    # a thread option is engine-specific (`Threads`, `Max CPUs`), so each
    # participant's is listed in the manifest rather than judged here.
    $participants = @($resolved.plan.participants)
    if ($participants.Count -lt 2) { Add-Violation "the gauntlet plan names $($participants.Count) participant(s)" }
    foreach ($entry in $participants) {
        $launch = $entry.participant.launch
        $hashValue = $launch.options.PSObject.Properties | Where-Object { $_.Name -eq 'Hash' } | ForEach-Object { $_.Value.value }
        if ("$hashValue" -ne "$Hash") {
            Add-Violation "participant '$($launch.label)' Hash is '$hashValue', expected $Hash (pass --option Hash=$Hash)"
        }
    }
}
if ($resolved.openings.path -ne $Book) {
    Add-Violation "book is '$($resolved.openings.path)', expected '$Book'"
}
if ("$($resolved.openings.order)" -ne "Random") {
    Add-Violation "opening order is '$($resolved.openings.order)', expected Random"
}
if ($resolved.openings.wrap) { Add-Violation "openings wrap; a run must not replay its own book" }
if ([int]$resolved.execution.concurrency -ne $expectedConcurrency) {
    Add-Violation "concurrency is $($resolved.execution.concurrency), expected $expectedConcurrency"
}
# A multi-thread engine needs a physical core per thread. The runner gives a
# game one core unless told otherwise, so a four-thread match resolved to two
# four-thread engines on one core, and nothing refused it (RAR-M76 passed
# `--cores-per-game 4` by hand).
$coresPerGame = [int]$resolved.execution.allocation.cores_per_game
$maxThreads = [Math]::Max($Threads, $threadsB)
if ($maxThreads -gt 1 -and $coresPerGame -lt $maxThreads) {
    Add-Violation ("cores_per_game is $coresPerGame for Threads $maxThreads, so the engines' threads share " +
        "fewer physical cores than they run; pass -ExtraArgs @('--cores-per-game', '$maxThreads')")
}
# Every run keeps each slot's engine processes between games. A command's
# default has changed between runner versions, so the resolved value is read.
$processes = $resolved.PSObject.Properties['engine_processes']
if (-not $processes -or "$($processes.Value)" -ne 'per-slot') {
    Add-Violation "engine processes are '$(if ($processes) { $processes.Value } else { 'unstated' })', expected per-slot"
}
if ([int]$resolved.master_seed -ne $Seed) {
    Add-Violation "resolved seed is $($resolved.master_seed), expected $Seed"
}if ($Mode -eq "sprt") {
    if ("$($resolved.design.parameters.model)" -ne "normalized") {
        Add-Violation "SPRT model is '$($resolved.design.parameters.model)', expected normalized"
    }
    if ([int]$resolved.design.max_pairs -ne $MaxPairs) {
        Add-Violation "cap is $($resolved.design.max_pairs) pairs, expected $MaxPairs"
    }
}
if ($Mode -eq "spsa") {
    if ([int]$resolved.settings.iterations -ne $Ceiling) {
        Add-Violation "tune horizon is $($resolved.settings.iterations) iterations, expected the ceiling $Ceiling"
    }
    if ([int]$resolved.settings.games_per_iteration -ne $GamesPerIteration) {
        Add-Violation "games per iteration is $($resolved.settings.games_per_iteration), expected $GamesPerIteration"
    }
    # PLAN rule 7c's gain ratio, which the rule 7c run file sets.
    if ([Math]::Abs([double]$resolved.r_end - 0.00155) -gt 1e-12) {
        Add-Violation "r_end is $($resolved.r_end), expected PLAN rule 7c's 0.00155"
    }
    # Theta is a checkpoint's rounded centres, never a window average.
    if ($null -ne $resolved.final_window_percent) {
        Add-Violation "the estimator is a final-window average; rule 7c's theta is a checkpoint's centres"
    }
    # A tune is one run: the blocks seeded from a finished run are retired.
    if ($resolved.tune.seeded_from) {
        Add-Violation "the resolved tune is seeded from $($resolved.tune.seeded_from.run_directory); a tune is one run (PLAN rule 7c)"
    }
}

# Placement: every slot must sit on a game core. Windows services most device
# interrupts on CPU 0, so an engine pinned there inherits stalls no other core
# sees; the fastchess list has excluded it since Get-HarnessGameCpus existed.
$gameCpus = @((Get-HarnessGameCpus).Cpu)
$slotCpus = @()
foreach ($slot in $resolved.execution.slots) {
    foreach ($side in @('engine_a', 'engine_b')) {
        if ($slot.PSObject.Properties.Name -contains $side) {
            foreach ($cpu in $slot.$side.allocation.cpus) { $slotCpus += [int]$cpu.number }
        }
    }
    if ($slot.PSObject.Properties.Name -contains 'engine') {
        foreach ($cpu in $slot.engine.allocation.cpus) { $slotCpus += [int]$cpu.number }
    }
}
$slotCpus = @($slotCpus | Sort-Object -Unique)
$physicalCpus = @($slotCpus | Where-Object { $_ % 2 -eq 0 })
$offGameCores = @($physicalCpus | Where-Object { $gameCpus -notcontains $_ })
if ($slotCpus.Count -gt 0 -and $offGameCores.Count -gt 0) {
    Add-Violation ("slots use physical cores outside the game-core set (" +
        ($offGameCores -join ',') + "); CPU 0 is reserved for the operating system")
}
# Placement is automatic with a core of headroom (RAR-M48: unpinned games
# carry a hidden per-run offset), or an explicit list of game cores: one
# logical CPU per physical core and no SMT sibling, so a multi-thread engine
# measures with each thread on its own core (Windows packs four threads on two
# cores' siblings inside an auto slot, RAR-M81's sampler), and a game core
# left free as headroom.
$placement = $resolved.execution.placement_policy
switch ("$($placement.mode)") {
    'auto' {
        if ([int]$placement.headroom_physical_cores -lt 1) {
            Add-Violation "placement leaves no physical core of headroom"
        }
    }
    'explicit' {
        $listed = @($placement.cpus | ForEach-Object { [int]$_.number } | Sort-Object -Unique)
        $notGame = @($listed | Where-Object { $gameCpus -notcontains $_ })
        if ($notGame.Count -gt 0) {
            Add-Violation ("explicit placement names logical CPUs outside the game-core set (" +
                ($notGame -join ',') + "): one logical CPU per physical core, never CPU 0, no SMT sibling")
        }
        $freeGameCores = @($gameCpus | Where-Object { $slotCpus -notcontains $_ })
        if ($freeGameCores.Count -lt 1) {
            Add-Violation "explicit placement leaves no game core free as headroom"
        }
    }
    default {
        Add-Violation "placement is '$($placement.mode)', expected auto or an explicit game-core list (RAR-M48: unpinned games carry a hidden per-run offset)"
    }
}

if ($violations.Count -gt 0) {
    Write-Host ""
    foreach ($violation in $violations) { Write-Host "  POLICY: $violation" -ForegroundColor Red }
    throw ("The resolved configuration is not Rarog's policy ($($violations.Count) field(s): $($violations -join '; ')). " +
           "Fix the run file or the command line; do not run a measurement under conditions the " +
           "ledger cannot reproduce. Dry run: $dryPath")
}
Write-Host "  Resolved configuration matches policy in every checked field."

# ─── The per-run manifest ─────────────────────────────────────────────────
$repoSha = (git rev-parse HEAD 2>$null)
if (-not $repoSha) { $repoSha = "n/a" } else { $repoSha = $repoSha.Trim() }
$repoDirty = [bool](git status --porcelain)

$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add("mode:             $Mode$(if ($Mode -eq 'sprt') { " ($Bracket bracket)" })")
$lines.Add("run_file:         $RunFile")
$lines.Add("run_file_sha256:  $(Get-HarnessSha256 $RunFile)")
$lines.Add("run_directory:    $Dir")
foreach ($arm in $engines) {
    $manifest = $manifests[$arm.Label]
    $lines.Add("engine_$($arm.Label):  $($arm.Path)")
    $lines.Add("  sha256:         $(Get-HarnessSha256 $arm.Path)")
    $lines.Add("  git_sha:        $(if ($manifest) { $manifest.git_sha } else { 'unknown (no sidecar)' })")
    $lines.Add("  git_dirty:      $(if ($manifest) { $manifest.git_dirty } else { 'unknown' })")
    $lines.Add("  flavor:         $(if ($manifest) { $manifest.flavor } else { 'unknown' })")
    $lines.Add("  rustc:          $(if ($manifest) { $manifest.rustc } else { 'unknown' })")
    $lines.Add("  bench_nodes:    $(if ($manifest) { $manifest.bench_nodes } else { 'unknown' })")
    $lines.Add("  options:        $(if ($arm.Options) { $arm.Options -join ' ' } else { '(none)' })")
}
foreach ($surfaceFile in $surfaceFiles) {
    $lines.Add("surface:          $surfaceFile")
    $lines.Add("  sha256:         $(Get-HarnessSha256 $surfaceFile)")
}
$lines.Add("expect_revision:  $(if ($ExpectRevision) { $ExpectRevision } else { '(not constrained)' })")
$lines.Add("expect_bench:     $(if ($ExpectBench.Count -gt 0) { $ExpectBench -join ', ' } else { '(not constrained)' })")
$lines.Add("repo_revision:    $repoSha")
$lines.Add("repo_dirty:       $repoDirty")
$lines.Add("runner:           $($cli.Path)")
$lines.Add("runner_version:   $($cli.Version)")
$lines.Add("runner_sha256:    $($cli.Sha256)")
$lines.Add("runner_revision:  $($cli.Pin.revision)")
$lines.Add("syzygy:           $syzygyRecord")
$lines.Add("book:             $Book")
$lines.Add("book_sha256:      $(Get-HarnessSha256 $Book)")
$lines.Add("opening_order:    $($resolved.openings.order)")
$lines.Add("opening_seed:     $Seed")
$clockText = if ($Nodes -gt 0) { "$Nodes nodes a move (fixed nodes; no clock, no time management)" }
             elseif ($clockOdds) { "A ${BaseMs}ms + ${IncrementMs}ms, B ${baseMsB}ms + ${incrementMsB}ms" }
             else { "${BaseMs}ms + ${IncrementMs}ms" }
$lines.Add("time_control:     $clockText; margin ${MarginMs}ms")
$lines.Add("adjudication:     none (games play to a rules result)")
$lines.Add("hash_mb:          $Hash")
$lines.Add("threads:          $(if ($threadsB -ne $Threads) { "A $Threads, B $threadsB" } else { $Threads })")
$lines.Add("concurrency:      $($resolved.execution.concurrency)")
$placementText = if ("$($placement.mode)" -eq 'explicit') {
    "explicit (" + (@($placement.cpus | ForEach-Object { [int]$_.number }) -join ',') + "), one logical CPU per physical core"
} else {
    "$($placement.mode), headroom $($placement.headroom_physical_cores) physical core(s)"
}
$lines.Add("placement:        $placementText")
$lines.Add("slot_cpus:        $($slotCpus -join ',')")
if ($Mode -eq "sprt") {
    $design = $resolved.design.parameters
    $lines.Add("test_design:      SPRT elo0=$($design.elo0) elo1=$($design.elo1) alpha=$($design.alpha) beta=$($design.beta) model=$($design.model)")
    $lines.Add("game_budget:      $([int]$resolved.design.max_pairs * 2) games ($($resolved.design.max_pairs) colour-reversed pairs)")
}
if ($Mode -eq "spsa") {
    $lines.Add("tune_surface:     $ConfigGroup, $($resolved.tune.parameters.Count) coordinates")
    $lines.Add("tune_ceiling:     $($resolved.settings.iterations) iterations x $($resolved.settings.games_per_iteration) games = $($resolved.total_games) games at most")
    $lines.Add("tune_r_end:       $($resolved.r_end)")
    $lines.Add("tune_checkpoints: $($checkpoints -join ', ') (window $Window; tools/spsa_read.py, PLAN rule 7c)$(if ($AllowShortRun) { '; SHORT RUN, an instrument smoke' })")
    $lines.Add((Format-ColosseumRailWarnings -Warnings @($dry.rail_warnings)))
}
$lines.Add("host_busy_percent: $(if ($null -eq $hostState.BusyPercent) { 'unreadable' } else { '{0:N1}' -f $hostState.BusyPercent })")
$lines.Add("categorical:      $(if ($CategoricalTuneBuild) { 'one tune build on both sides, options A vs B (-CategoricalTuneBuild)' } else { 'no' })")
$lines.Add("host_idle_waived: $($hostState.Waived)$(if ($hostState.Reasons.Count -gt 0) { " (" + ($hostState.Reasons -join '; ') + ")" })")
$lines.Add("dry_run_json:     $dryPath")
$lines.Add("dry_run_sha256:   $(Get-HarnessSha256 $dryPath)")
$lines.Add("config_sha256:    $($dry.config_sha256)")
$lines.Add("command:          $($cli.Path) $($commandArgs -join ' ')")
$lines.Add("started_utc:      $((Get-Date).ToUniversalTime().ToString('u'))")
$lines | Set-Content -LiteralPath $manifestPath -Encoding utf8

Write-Host ""
Write-Host "======================================================="
Write-Host "  Colosseum $Mode$(if ($Mode -eq 'sprt') { " [$Bracket]" }): $(($engines | ForEach-Object { $_.Label }) -join ' vs ')"
if ($Mode -eq "sprt") {
    $design = $resolved.design.parameters
    Write-Host "  H0: nElo<=$($design.elo0)   H1: nElo>=$($design.elo1)   alpha=$($design.alpha)  beta=$($design.beta)"
    Write-Host "  Cap: $([int]$resolved.design.max_pairs * 2) games; no boundary at the cap is unresolved, not an acceptance"
}
Write-Host "  TC: $(if ($Nodes -gt 0) { "$Nodes nodes/move (fixed; speed and time management removed)" } elseif ($clockOdds) { "A ${BaseMs}+${IncrementMs}ms, B ${baseMsB}+${incrementMsB}ms" } else { "${BaseMs}+${IncrementMs}ms" })  Margin: ${MarginMs}ms  Hash: ${Hash}  Threads: $(if ($threadsB -ne $Threads) { "A $Threads, B $threadsB" } else { $Threads })  Conc: $($resolved.execution.concurrency)"
Write-Host "  Adjudication: none    Book: $(Split-Path $Book -Leaf) (random, seed $Seed)"
Write-Host "  Runner: $($cli.Version), sha256 $($cli.Sha256.Substring(0,8))..., pinned at $($cli.Pin.revision.Substring(0,7))"
Write-Host "  Manifest: $manifestPath"
Write-Host "  Run dir:  $Dir"
Write-Host "======================================================="
Write-Host ""

if ($DryRun) {
    # The manifest must say this invocation played nothing, or a later reader
    # takes a rehearsal for a run that was interrupted before its report.
    Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value "dry_run_only:     true (no game was played)"
    Write-Host "Dry run only; no game was played. Resolved configuration: $dryPath"
    return
}

# ─── A tune: one run, a segment to each checkpoint, read there (rule 7c) ──
if ($Mode -eq "spsa") {
    $reader = Join-Path $PSScriptRoot "spsa_read.py"
    $readsPath = Join-Path $Dir "spsa-read.jsonl"
    $acceptHint = ("After the analysis the maintainer may accept the recorded faults: " +
                   "python tools/spsa_read.py `"$Dir`" --accept-faults `"<reason>`"; then run this command again.")
    # The runner's own count of completed iterations, safe while it is paused.
    function Get-TuneCompleted {
        if (-not (Test-Path -LiteralPath (Join-Path $Dir "checkpoint.json"))) { return 0 }
        $text = & $cli.Path spsa history $Dir --json 2>$null
        if ($LASTEXITCODE -ne 0) { throw "colosseum-cli spsa history exited $LASTEXITCODE on $Dir." }
        [int](($text -join "`n") | ConvertFrom-Json).report.completed_iterations
    }
    function Test-CheckpointRead([int]$At) {
        if (-not (Test-Path -LiteralPath $readsPath)) { return $false }
        [bool](@(Get-Content -LiteralPath $readsPath | Where-Object { $_.Trim() } |
            ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { [int]$_.at -eq $At }).Count)
    }
    function Read-TuneCheckpoint([int]$At, [switch]$WriteTheta) {
        $readArgs = @($reader, $Dir, '--at', "$At", '--window', "$Window", '--first', "$FirstCheckpoint",
            '--every', "$CheckpointEvery", '--cli', $cli.Path, '--record', '--json')
        if ($WriteTheta) { $readArgs += '--write-theta' }
        $text = & python @readArgs
        if ($LASTEXITCODE -ne 0) { throw "tools/spsa_read.py failed at iteration $At (exit $LASTEXITCODE)." }
        ($text -join "`n") | ConvertFrom-Json
    }

    # A stopped tune is never extended. A FAULT read stops it for the
    # maintainer; once its faults are accepted the reader gives that checkpoint
    # its real verdict, so the run goes on from that read.
    if (Test-Path -LiteralPath $readsPath) {
        $last = Get-Content -LiteralPath $readsPath | Where-Object { $_.Trim() } | Select-Object -Last 1 | ConvertFrom-Json
        $faultAccepted = "$($last.verdict)" -eq "FAULT" -and (Test-Path -LiteralPath (Join-Path $Dir "faults-accepted.json"))
        if (-not $faultAccepted) {
            if ("$($last.verdict)" -eq "FAULT") { throw "STOPPED - $Dir read FAULT at iteration $($last.at). $acceptHint" }
            if ("$($last.verdict)" -ne "CONTINUE" -or (Test-Path -LiteralPath (Join-Path $Dir "theta-$($last.at).txt"))) {
                throw "STOPPED - $Dir ended at iteration $($last.at) with $($last.verdict); a stopped tune is never extended."
            }
        }
    }

    $verdict = $null
    $read = $null
    foreach ($checkpoint in $checkpoints) {
        $completed = Get-TuneCompleted
        if ($completed -gt $checkpoint) {
            if (-not (Test-CheckpointRead $checkpoint)) { throw "The run is past the checkpoint $checkpoint, which was never read." }
            continue
        }
        if ($completed -lt $checkpoint) {
            $segmentLog = $logPath -replace '\.log$', "_to$checkpoint.log"
            # Ctrl+C stops this script with the runner; the manifest must still
            # say the invocation ended without a read.
            $segmentReturned = $false
            try {
                & $cli.Path @commandArgs --stop-after-iteration "$checkpoint" 2>&1 | Tee-Object -FilePath $segmentLog
                $segmentExit = $LASTEXITCODE
                $segmentReturned = $true
            } finally {
                if (-not $segmentReturned) {
                    Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value @(
                        "interrupted_utc:  $((Get-Date).ToUniversalTime().ToString('u'))"
                        "interrupted:      the wrapper was stopped before iteration $checkpoint; the run directory holds its durable iterations. Re-run the same command to resume."
                    )
                }
            }
            # 6 is the runner's clean pause at --stop-after-iteration, 0 its completed horizon.
            $expectedExit = if ($checkpoint -eq $Ceiling) { 0 } else { 6 }
            if ($segmentExit -ne $expectedExit) {
                Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value "segment_to_$($checkpoint): exit $segmentExit, expected $expectedExit"
                throw "colosseum-cli exited $segmentExit before iteration $checkpoint (expected $expectedExit); see $segmentLog and $Dir."
            }
            $completed = Get-TuneCompleted
            if ($completed -ne $checkpoint) { throw "The run paused at iteration $completed, not at the checkpoint $checkpoint." }
        }
        $read = Read-TuneCheckpoint -At $checkpoint
        $verdict = "$($read.verdict)"
        if ($verdict -eq "CONTINUE" -and (Test-Path -LiteralPath (Join-Path $Dir "STOP"))) {
            $read = Read-TuneCheckpoint -At $checkpoint -WriteTheta
            $verdict = "STOP_MAINTAINER"
        }
        Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value ("checkpoint {0,5}:  {1}; max |z| {2} (line {3}); excess {4} SD; gain {5} (null SD {6}); forecast {7} +/- {8}" -f
            $checkpoint, $verdict, $read.max_abs_z, $read.familywise_line, $read.excess_sd, $read.gain.elo, $read.gain.null_sd, $read.forecast.elo, $read.forecast.sd)
        Write-Host "Checkpoint ${checkpoint}: $verdict (max |z| $($read.max_abs_z) against $($read.familywise_line); excess $($read.excess_sd) SD)"
        if ($verdict -ne "CONTINUE") { break }
    }

    $status = Get-ColosseumRunStatus -CliPath $cli.Path -Dir $Dir
    $completion = [System.Collections.Generic.List[string]]::new()
    $completion.Add("completed_utc:    $((Get-Date).ToUniversalTime().ToString('u'))")
    $completion.Add("verdict:          $verdict at iteration $($read.at)")
    if ($verdict -in @("STOP_QUIET", "STOP_CEILING", "STOP_MAINTAINER")) {
        $completion.Add("theta:            $(Join-Path $Dir "theta-$($read.at).txt")")
    }
    $completion.Add("read:             $(Join-Path $Dir 'spsa-read.txt')")
    $resultPath = Join-Path $Dir "result.json"
    if (Test-Path -LiteralPath $resultPath) {
        $completion.Add((Format-ColosseumClampWarnings -Result (Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json)))
    }
    foreach ($artifact in @('spsa-read.jsonl', 'result.json', 'run-record.json', 'games.pgn', 'resolved-config.json')) {
        $path = Join-Path $Dir $artifact
        if (Test-Path -LiteralPath $path) { $completion.Add("$($artifact)_sha256: $(Get-HarnessSha256 $path)") }
    }
    $completion | ForEach-Object { Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value $_ }
    if ($status.durable.journal.refused) {
        throw "The run's journal does not match its checkpoint: $($status.durable.journal.refused). See $Dir."
    }

    Write-Host ""
    switch ($verdict) {
        { $_ -in "STOP_QUIET", "STOP_CEILING", "STOP_MAINTAINER" } {
            Write-Host "Tune stopped ($verdict) at iteration $($read.at). Theta: $(Join-Path $Dir "theta-$($read.at).txt")"
            Write-Host "  Read:     $(Join-Path $Dir 'spsa-read.txt')"
            Write-Host "  Manifest: $manifestPath"
            Write-Host "  A tune is judged by its gate, not by its read."
            return
        }
        "STOP_RAIL" { throw "RAIL - $($read.rails -join ', ') at a bound with z pointing outward past the familywise line at iteration $($read.at): register the range again; no theta." }
        "FAULT" {
            $named = @($read.faults.PSObject.Properties | Where-Object { $_.Value } | ForEach-Object { "$($_.Name) $($_.Value)" })
            throw "FAULT - the run recorded faults by iteration $($read.at): $($named -join ', '). The maintainer decides. $acceptHint"
        }
        default { throw "Unexpected verdict '$verdict' at a registered checkpoint." }
    }
}

# Ctrl+C stops this script along with the runner, so nothing after the run
# executes; the manifest must still say the invocation ended without a report.
$runReturned = $false
try {
    & $cli.Path @commandArgs 2>&1 | Tee-Object -FilePath $logPath
    $runExit = $LASTEXITCODE
    $runReturned = $true
} finally {
    if (-not $runReturned) {
        Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value @(
            "interrupted_utc:  $((Get-Date).ToUniversalTime().ToString('u'))"
            "interrupted:      the wrapper was stopped before the runner reported; the run directory holds its durable games. Re-run the same command to resume."
        )
    }
}

# ─── After the run: the runner's own record, not the console ──────────────
$recordPath = Join-Path $Dir "run-record.json"
if (-not (Test-Path -LiteralPath $recordPath)) {
    Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value @(
        "completed_utc:    $((Get-Date).ToUniversalTime().ToString('u'))"
        "exit_code:        $runExit"
        "run_record:       MISSING"
    )
    throw "colosseum-cli exited $runExit and wrote no run-record.json in $Dir."
}
$record = Get-Content -LiteralPath $recordPath -Raw | ConvertFrom-Json
$faultField = @($record.progress.fields | Where-Object { $_.label -eq 'faults' } | Select-Object -First 1)
$faults = if ($faultField) { $faultField.value } else { "(not reported)" }
$scored = [int]$record.official_sample.scored_games

# The status view is the runner's machine interface: structured faults, and
# the checkpoint's aggregates, which for a match hold the pentanomial its run
# record leaves at zero. The recount of games.pgn must agree with them.
$status = $null
$statusFailure = $null
try { $status = Get-ColosseumRunStatus -CliPath $cli.Path -Dir $Dir } catch { $statusFailure = $_.Exception.Message }
$pentanomial = if ($status -and $status.durable.checkpoint.pentanomial) {
    "$($status.durable.checkpoint.pentanomial -join ', ')"
} else { "$($record.official_sample.pentanomial -join ', ')" }
$recountFailure = $null
if ($Mode -in @('match', 'sprt', 'calibrate') -and (Test-Path -LiteralPath (Join-Path $Dir "games.pgn"))) {
    $recountJson = & python (Join-Path $PSScriptRoot "diag\colosseum_recount.py") --json $Dir 2>&1
    $recountExit = $LASTEXITCODE
    try {
        $recount = @(($recountJson -join "`n") | ConvertFrom-Json)[0]
        if ($recountExit -ne 0) {
            $recountFailure = ("The recount of games.pgn ($($recount.pentanomial -join ', '), " +
                               "$($recount.official_games) games) disagrees with the runner's " +
                               "$($recount.recorded_source) ($($recount.recorded_pentanomial -join ', ')).")
        } elseif (-not $recount.comparable) {
            $recountFailure = "The runner recorded no pentanomial to check the recount of games.pgn against."
        } else {
            $pentanomial += " (recount of games.pgn agrees)"
        }
    } catch {
        $recountFailure = "colosseum_recount.py failed (exit $recountExit): $recountJson"
    }
    if ($recountFailure) { $pentanomial += " (RECOUNT FAILED)" }
}

$completion = [System.Collections.Generic.List[string]]::new()
$completion.Add("completed_utc:    $((Get-Date).ToUniversalTime().ToString('u'))")
$exit = Resolve-ColosseumExit -Mode $Mode -ExitCode $runExit
$completion.Add("exit_code:        $runExit ($(if ($exit.Verdict) { $exit.Verdict } else { $exit.Kind }))")
$completion.Add("run_status:       $($record.status)")
$completion.Add("scored_games:     $scored")
$completion.Add("completed_pairs:  $($record.official_sample.completed_pairs) ($([int]$record.official_sample.completed_pairs * 2) paired games)")
$completion.Add("pentanomial:      $pentanomial")
$completion.Add("faults:           $faults")
if ($Mode -eq 'spsa') {
    $resultPath = Join-Path $Dir "result.json"
    $tuneResult = if (Test-Path -LiteralPath $resultPath) { Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json }
    $completion.Add((Format-ColosseumClampWarnings -Result $tuneResult))
} else {
    foreach ($line in (Format-ColosseumResults -Record $record)) { $completion.Add($line) }
}
# The runner writes its end-of-run results to results.txt rather than the
# console; the manifest quotes the same figures from the run record above.
$resultsText = Join-Path $Dir "results.txt"
if (Test-Path -LiteralPath $resultsText) { $completion.Add("results_file:     $resultsText") }
foreach ($artifact in @('result.json', 'results.txt', 'run-record.json', 'games.pgn', 'resolved-config.json')) {
    $path = Join-Path $Dir $artifact
    if (Test-Path -LiteralPath $path) { $completion.Add("$($artifact)_sha256: $(Get-HarnessSha256 $path)") }
}
$completion | ForEach-Object { Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value $_ }

switch ($exit.Kind) {
    "cancelled" {
        throw ("The run was cancelled at a committed boundary with $scored games scored. Resume it with the " +
               "same command and -Dir $Dir; the wrapper carries its seed ($Seed) over.")
    }
    "invalid" {
        throw "The runner invalidated the run under its fault policy (exit $runExit); the result is void. See $Dir."
    }
    "outcome" { }
    default {
        throw "colosseum-cli exited $runExit ($($exit.Kind)). Status '$($record.status)'; see $logPath and $Dir."
    }
}

if ($statusFailure) { throw $statusFailure }
if ($status.durable.journal.refused) {
    throw "The run's journal does not match its checkpoint: $($status.durable.journal.refused). See $Dir."
}
if ($recountFailure) { throw "$recountFailure See $Dir." }
Assert-ColosseumRunFaults -Status $status -ScoredGames $scored `
    -TimeLossRateCeiling $TimeLossRateCeiling -Dir $Dir | Out-Null

Write-Host ""
Write-Host "Run finished: status '$($record.status)', outcome '$($exit.Verdict)', $scored scored games."
Write-Host "  Manifest: $manifestPath"
Write-Host "  Log:      $logPath"
if (Test-Path -LiteralPath $resultsText) { Write-Host "  Results:  $resultsText" }
Write-Host "  Evidence: $Dir"
if ($Mode -eq "sprt") {
    Write-Host "  Only an H1 boundary promotes the candidate; a cap stop is unresolved and must be parked or reverted."
}
if ($Mode -eq "match") {
    Write-Host "  A fixed match reports an interval and decides nothing."
}
