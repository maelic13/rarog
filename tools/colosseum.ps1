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
        and a Core* surface requires a b2core tune build;
      - the configuration Colosseum resolves is Rarog's policy, field by field:
        no adjudication, the registered clock and margin, Hash and Threads, the
        UHO book in random order, automatic placement with a core of headroom,
        never CPU 0, and the registered concurrency;
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
    spsa      -> a tune. Needs -ConfigGroup, -Iterations and -TotalGames.
    gauntlet  -> a rating gauntlet; participants are passed in -ExtraArgs.

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
    ./tools/colosseum.ps1 -Mode spsa -Engine tools/test_engines/rarog-b23core-tune.exe `
        -ConfigGroup b23core -Iterations 5000 -TotalGames 150000 `
        -Seed 7 -Dir tools/results/<experiment>
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
    [int]$TotalGames = 0,
    [string]$ConfigGroup = "",
    [int]$Iterations = 0,
    [int]$Seed = 0,
    [string[]]$OptionsA = @(),
    [string[]]$OptionsB = @(),
    [int]$Hash = 64,
    [int]$Threads = 1,
    [int]$BaseMs = 3000,
    [int]$IncrementMs = 30,
    [int]$MarginMs = 20,
    [int]$Concurrency = 0,
    [string]$ExpectRevision = "",
    [long[]]$ExpectBench = @(),
    [switch]$AllowDirtyTree,
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
                     TotalGames = "-TotalGames is a tune budget"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Iterations = "-Iterations is a tune horizon" } }
    "match"     { @{ MaxPairs = "-MaxPairs caps an SPRT; a fixed match is sized by -Games"
                     Bracket = "-Bracket selects SPRT bounds; a fixed match has none"
                     TotalGames = "-TotalGames is a tune budget"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Iterations = "-Iterations is a tune horizon" } }
    "calibrate" { @{ MaxPairs = "-MaxPairs caps an SPRT; a null pair is sized by -Games"
                     Bracket = "-Bracket selects SPRT bounds; a null pair has none"
                     TotalGames = "-TotalGames is a tune budget"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Iterations = "-Iterations is a tune horizon" } }
    "spsa"      { @{ MaxPairs = "-MaxPairs caps an SPRT; a tune is sized by -TotalGames"
                     Games = "-Games sizes a fixed match; a tune is sized by -TotalGames"
                     Bracket = "-Bracket selects SPRT bounds; a tune has none"
                     EngineA = "a tune has one engine: pass -Engine"
                     EngineB = "a tune has one engine: pass -Engine" } }
    "gauntlet"  { @{ MaxPairs = "-MaxPairs caps an SPRT"
                     Bracket = "-Bracket selects SPRT bounds"
                     TotalGames = "-TotalGames is a tune budget"
                     ConfigGroup = "-ConfigGroup names a tune surface"
                     Iterations = "-Iterations is a tune horizon"
                     EngineA = "gauntlet participants are passed in -ExtraArgs"
                     EngineB = "gauntlet participants are passed in -ExtraArgs" } }
}
$ignored = @($modeIgnores.Keys | Where-Object { $PSBoundParameters.ContainsKey($_) })
if ($ignored.Count -gt 0) {
    $why = @($ignored | ForEach-Object { $modeIgnores[$_] } | Where-Object { $_ }) | Select-Object -First 1
    throw ("-Mode $Mode ignores: $($ignored -join ', '). $why. Remove the option or " +
           "change -Mode; this script will not accept a parameter it cannot honour.")
}

# ─── Mode-specific requirements ───────────────────────────────────────────
$twoArmModes = @("sprt", "match", "calibrate")
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
    if ($Iterations -le 0) { throw "-Iterations is required: the registered horizon sizes c_end." }
    if ($TotalGames -le 0) { throw "-TotalGames is required: a tune budget is stated in games." }
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
$runFileDefaults = @{
    sprt      = Join-Path $PSScriptRoot "colosseum\sprt-$Bracket.toml"
    match     = Join-Path $PSScriptRoot "colosseum\match-fixed.toml"
    calibrate = Join-Path $PSScriptRoot "colosseum\calibrate-null.toml"
    spsa      = Join-Path $PSScriptRoot "spsa_configs\colosseum\$ConfigGroup.run.toml"
    gauntlet  = Join-Path $PSScriptRoot "colosseum\gauntlet.toml"
}
if (-not $RunFile) { $RunFile = $runFileDefaults[$Mode] }
if (-not (Test-Path -LiteralPath $RunFile)) { throw "Run file not found: $RunFile" }
$RunFile = (Resolve-Path -LiteralPath $RunFile).Path

$engines = @()
if ($twoArmModes -contains $Mode) {
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
$manifests = @{}
foreach ($arm in $engines) {
    $kind = if ($Mode -eq "spsa") { "tune" } else { "gate" }
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
        if (($OptionsA -join '|') -eq ($OptionsB -join '|')) {
            throw ("Identical binaries AND options require -Mode calibrate; an SPRT " +
                   "centred on zero is not a valid null calibration.")
        }
        Write-Host "  NOTE: one binary on both sides, differing only by UCI options." -ForegroundColor Yellow
    }
}

# ─── Guard 7: every requested option is advertised ────────────────────────
$advertised = @{}
foreach ($arm in $engines) {
    $details = @(Get-EngineUciOptions -Path $arm.Path -Detailed)
    $advertised[$arm.Label] = $details
    Assert-AdvertisedOptions -Advertised $details -Wanted $arm.Options -Label $arm.Label
    Assert-AdvertisedOptions -Advertised $details -Wanted @("Hash=$Hash", "Threads=$Threads") -Label $arm.Label
}

# ─── Guard 8: the tune surface belongs to this binary and this horizon ────
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
    Assert-CoreSurfaceArm -Names $names -Flavor $manifests["tune"].flavor -ConfigGroup $ConfigGroup

    $tuned = @(Assert-TuneSurface -Advertised $advertised["tune"] -Surface $surface `
        -Iterations $Iterations -Label (Split-Path $engines[0].Path -Leaf) -Fixed $fixed)
    Write-Host "  Tune surface verified: $($tuned.Count) coordinates at N = $Iterations"

    # The generated tune file is the registration in another notation. If it has
    # drifted from the JSON, the run would tune a surface nobody registered.
    python (Join-Path $PSScriptRoot "spsa_config_to_colosseum.py") $ConfigGroup --iterations $Iterations --check
    if ($LASTEXITCODE -ne 0) {
        throw ("The generated Colosseum surface for '$ConfigGroup' at N = $Iterations does not match " +
               "config_$ConfigGroup.json. Regenerate it with tools/spsa_config_to_colosseum.py " +
               "and commit the result before tuning.")
    }
    $surfaceFiles = @(
        $surfacePath
        $(if (Test-Path $fixedPath) { $fixedPath })
        (Join-Path $configs "colosseum\$ConfigGroup.tune.toml")
    ) | Where-Object { $_ }
}

# ─── The command ──────────────────────────────────────────────────────────
$Dir = [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $Dir))
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
    "spsa"      { $commandArgs += @($engines[0].Path, '--total-games', "$TotalGames") }
}
foreach ($option in $OptionsA) { if ($Mode -ne "spsa" -and $Mode -ne "gauntlet") { $commandArgs += @('--a-option', $option) } }
foreach ($option in $OptionsB) { if ($Mode -ne "spsa" -and $Mode -ne "gauntlet") { $commandArgs += @('--b-option', $option) } }
if ($Concurrency -gt 0) { $commandArgs += @('--concurrency', "$Concurrency") }
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

& $cli.Path @commandArgs --dry-run --json > $dryPath 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host (Get-Content -LiteralPath $dryPath -Raw)
    throw "colosseum-cli refused the dry run (exit $LASTEXITCODE); nothing was played."
}
$dry = Get-Content -LiteralPath $dryPath -Raw | ConvertFrom-Json
$resolved = $dry.resolved_configuration

$violations = [System.Collections.Generic.List[string]]::new()
function Add-Violation { param([string]$Text) $violations.Add($Text) }

if ($dry.command -ne $(if ($Mode -eq 'gauntlet') { 'tournament run' } else { $Mode })) {
    Add-Violation "resolved command is '$($dry.command)', expected '$Mode'"
}
foreach ($rule in @('draw', 'resign', 'max_moves')) {
    if ($null -ne $resolved.adjudication.$rule) {
        Add-Violation "adjudication.$rule is set; Rarog plays games out (RAR-M15, RAR-M17)"
    }
}
$controls = @()
foreach ($field in @('engine_a_time_control', 'engine_b_time_control', 'engine_time_control')) {
    if ($resolved.PSObject.Properties.Name -contains $field) { $controls += ,@($field, $resolved.$field) }
}
if ($controls.Count -eq 0) { Add-Violation "the resolved configuration states no time control" }
foreach ($pair in $controls) {
    $name = $pair[0]; $control = $pair[1]
    if ([int]$control.control.Increment.base_ms -ne $BaseMs) {
        Add-Violation "$name base is $($control.control.Increment.base_ms) ms, expected $BaseMs"
    }
    if ([int]$control.control.Increment.inc_ms -ne $IncrementMs) {
        Add-Violation "$name increment is $($control.control.Increment.inc_ms) ms, expected $IncrementMs"
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
    if ([int]$options.Threads.value -ne $Threads) { Add-Violation "$name Threads is $($options.Threads.value), expected $Threads" }
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
if ("$($resolved.execution.placement_policy.mode)" -ne "auto") {
    Add-Violation "placement is '$($resolved.execution.placement_policy.mode)', expected auto (RAR-M48: unpinned games carry a hidden per-run offset)"
}
if ([int]$resolved.execution.placement_policy.headroom_physical_cores -lt 1) {
    Add-Violation "placement leaves no physical core of headroom"
}
if ([int]$resolved.master_seed -ne $Seed) {
    Add-Violation "resolved seed is $($resolved.master_seed), expected $Seed"
}
if ($Mode -eq "sprt") {
    if ("$($resolved.design.parameters.model)" -ne "normalized") {
        Add-Violation "SPRT model is '$($resolved.design.parameters.model)', expected normalized"
    }
    if ([int]$resolved.design.max_pairs -ne $MaxPairs) {
        Add-Violation "cap is $($resolved.design.max_pairs) pairs, expected $MaxPairs"
    }
}
if ($Mode -eq "spsa") {
    if ([int]$resolved.total_games -ne $TotalGames) {
        Add-Violation "tune budget is $($resolved.total_games) games, expected $TotalGames"
    }
    if ([int]$resolved.settings.iterations -ne $Iterations) {
        Add-Violation "tune horizon is $($resolved.settings.iterations) iterations, expected $Iterations"
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

if ($violations.Count -gt 0) {
    Write-Host ""
    foreach ($violation in $violations) { Write-Host "  POLICY: $violation" -ForegroundColor Red }
    throw ("The resolved configuration is not Rarog's policy ($($violations.Count) field(s) above). " +
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
$lines.Add("book:             $Book")
$lines.Add("book_sha256:      $(Get-HarnessSha256 $Book)")
$lines.Add("opening_order:    $($resolved.openings.order)")
$lines.Add("opening_seed:     $Seed")
$lines.Add("time_control:     ${BaseMs}ms + ${IncrementMs}ms; margin ${MarginMs}ms")
$lines.Add("adjudication:     none (games play to a rules result)")
$lines.Add("hash_mb:          $Hash")
$lines.Add("threads:          $Threads")
$lines.Add("concurrency:      $($resolved.execution.concurrency)")
$lines.Add("placement:        $($resolved.execution.placement_policy.mode), headroom $($resolved.execution.placement_policy.headroom_physical_cores) physical core(s)")
$lines.Add("slot_cpus:        $($slotCpus -join ',')")
if ($Mode -eq "sprt") {
    $design = $resolved.design.parameters
    $lines.Add("test_design:      SPRT elo0=$($design.elo0) elo1=$($design.elo1) alpha=$($design.alpha) beta=$($design.beta) model=$($design.model)")
    $lines.Add("game_budget:      $($resolved.design.max_pairs) pairs = $([int]$resolved.design.max_pairs * 2) games")
}
if ($Mode -eq "spsa") {
    $lines.Add("tune_surface:     $ConfigGroup, $($resolved.tune.parameters.Count) coordinates")
    $lines.Add("tune_horizon:     $($resolved.settings.iterations) iterations x $($resolved.settings.games_per_iteration) games = $($resolved.total_games) games")
    $lines.Add("tune_r_end:       $($resolved.r_end)")
}
$lines.Add("host_busy_percent: $(if ($null -eq $hostState.BusyPercent) { 'unreadable' } else { '{0:N1}' -f $hostState.BusyPercent })")
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
    Write-Host "  Cap: $($resolved.design.max_pairs) pairs; no boundary at the cap is unresolved, not an acceptance"
}
Write-Host "  TC: ${BaseMs}+${IncrementMs}ms  Margin: ${MarginMs}ms  Hash: ${Hash}  Threads: $Threads  Conc: $($resolved.execution.concurrency)"
Write-Host "  Adjudication: none    Book: $(Split-Path $Book -Leaf) (random, seed $Seed)"
Write-Host "  Runner: $($cli.Version), sha256 $($cli.Sha256.Substring(0,8))..., pinned at $($cli.Pin.revision.Substring(0,7))"
Write-Host "  Manifest: $manifestPath"
Write-Host "  Run dir:  $Dir"
Write-Host "======================================================="
Write-Host ""

if ($DryRun) {
    Write-Host "Dry run only; no game was played. Resolved configuration: $dryPath"
    return
}

& $cli.Path @commandArgs 2>&1 | Tee-Object -FilePath $logPath
$runExit = $LASTEXITCODE

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

$completion = [System.Collections.Generic.List[string]]::new()
$completion.Add("completed_utc:    $((Get-Date).ToUniversalTime().ToString('u'))")
$completion.Add("exit_code:        $runExit")
$completion.Add("run_status:       $($record.status)")
$completion.Add("scored_games:     $scored")
$completion.Add("completed_pairs:  $($record.official_sample.completed_pairs)")
$completion.Add("pentanomial:      $($record.official_sample.pentanomial -join ', ')")
$completion.Add("faults:           $faults")
foreach ($artifact in @('result.json', 'run-record.json', 'games.pgn', 'resolved-config.json')) {
    $path = Join-Path $Dir $artifact
    if (Test-Path -LiteralPath $path) { $completion.Add("$($artifact)_sha256: $(Get-HarnessSha256 $path)") }
}
$completion | ForEach-Object { Add-Content -LiteralPath $manifestPath -Encoding utf8 -Value $_ }

if ($runExit -ne 0) {
    throw "colosseum-cli exited $runExit. Status '$($record.status)'; see $logPath and $Dir."
}

# Zero tolerance for a crash, a dropped engine or an illegal move; a rate
# ceiling for time losses, because a small background rate is a property of
# running fourteen concurrent games, not of the candidate (RAR-M14, RAR-E06).
if ($faults -match 'engine\s+(?<engine>\d+)/(?<engineCap>\d+),\s*time losses\s+(?<time>\d+)/(?<timeCap>\d+)') {
    $engineFaults = [int]$Matches['engine']
    $timeLosses = [int]$Matches['time']
    $otherFaults = $engineFaults - $timeLosses
    if ($otherFaults -gt 0) {
        throw ("The run recorded $otherFaults non-time engine fault(s) - a crash, an illegal move or a " +
               "dropped engine is never normal on this harness. The result is invalid. See $Dir.")
    }
    if ($scored -gt 0) {
        $rate = 100.0 * $timeLosses / $scored
        if ($rate -gt $TimeLossRateCeiling) {
            throw ("Time-loss rate {0:N3}% ({1}/{2}) exceeds the {3}% ceiling; the run is invalid. See {4}." `
                -f $rate, $timeLosses, $scored, $TimeLossRateCeiling, $Dir)
        }
        if ($timeLosses -gt 0) {
            Write-Host ("  Time losses: {0}/{1} = {2:N3}% (under the {3}% ceiling; recorded, not fatal)" `
                -f $timeLosses, $scored, $rate, $TimeLossRateCeiling) -ForegroundColor Yellow
        }
    }
} else {
    Write-Warning "Could not parse the fault counters from run-record.json; check $Dir by hand."
}

Write-Host ""
Write-Host "Run finished: status '$($record.status)', $scored scored games."
Write-Host "  Manifest: $manifestPath"
Write-Host "  Log:      $logPath"
Write-Host "  Evidence: $Dir"
if ($Mode -eq "sprt") {
    Write-Host "  Only an H1 boundary promotes the candidate; a cap stop is unresolved and must be parked or reverted."
}
if ($Mode -eq "match") {
    Write-Host "  A fixed match reports an interval and decides nothing."
}
