<#
.SYNOPSIS
    One-shot setup for a repo-local weather-factory SPSA run.

.DESCRIPTION
    Populates tools\weather-factory\tuner\ and writes the three config files
    (cutechess.json, spsa.json, config.json). After this script completes, run:

        cd tools\weather-factory
        python main.py

    Stop it with Ctrl-C whenever parameter values look stable. State is saved
    every 10 iterations to tuner\state.json so you can resume later.

    Prerequisites:
      - Run ./tools/setup_tools.ps1 once if tools\bin\fastchess.exe or
        tools\weather-factory\main.py is missing.
      - Build the tune binary you want to use:
        ./tools/build_test.ps1 -Suffix phase2-probcut -Tune

.PARAMETER ConfigGroup
    Which parameter group to tune.
    "pruning" - pruning / margin constants.
    "lmr"     - LMR weighted adjustments.
    "probcut" - Phase 2 ProbCut margins.

.PARAMETER Iterations
    Planned total iterations (used to set A = Iterations / 10 in spsa.json).
    Default: 5000.

.PARAMETER EngineSuffix
    Suffix of the binary in tools\test_engines.
    If omitted, defaults to phase1-lmr for LMR, phase1-pruning for pruning,
    and phase2-probcut for ProbCut.
    Examples:
      phase1-lmr       -> tools\test_engines\rarog-phase1-lmr-tune.exe
      phase1-lmr-tune  -> tools\test_engines\rarog-phase1-lmr-tune.exe
      custom.exe       -> tools\test_engines\custom.exe

.PARAMETER Resume
    Keep existing weather-factory tuner state. Use only when resuming the same
    SPSA run. By default, old state/games/graphs are archived before a new run.

.EXAMPLE
    ./tools/build_test.ps1 -Suffix phase2-probcut -Tune
    ./tools/setup_spsa.ps1 -ConfigGroup probcut -EngineSuffix phase2-probcut
#>
param(
    [ValidateSet("pruning","lmr","probcut")][string]$ConfigGroup = "lmr",
    [int]$Iterations = 5000,
    [string]$EngineSuffix = "",
    [switch]$Resume
)

$ErrorActionPreference = "Stop"

$repoRoot  = Split-Path -Parent $PSScriptRoot
$wfRoot    = Join-Path $PSScriptRoot "weather-factory"
$configs   = Join-Path $PSScriptRoot "spsa_configs"
$fastchess = Join-Path $PSScriptRoot "bin\fastchess.exe"
$book      = Join-Path $PSScriptRoot "books\SuperGM_4mvs.pgn"

if ($EngineSuffix -eq "") {
    $EngineSuffix = switch ($ConfigGroup) {
        "lmr" { "phase1-lmr" }
        "pruning" { "phase1-pruning" }
        "probcut" { "phase2-probcut" }
    }
}

if ($EngineSuffix.EndsWith(".exe")) {
    $engineFile = $EngineSuffix
} elseif ($EngineSuffix.EndsWith("-tune") -or $EngineSuffix.EndsWith("-pext-pgo")) {
    $engineFile = "rarog-$EngineSuffix.exe"
} else {
    $engineFile = "rarog-$EngineSuffix-tune.exe"
}
$engine = Join-Path $PSScriptRoot "test_engines\$engineFile"

if (-not (Test-Path (Join-Path $wfRoot "main.py"))) {
    Write-Host "weather-factory missing; cloning into tools\weather-factory..."
    git clone https://github.com/jnlt3/weather-factory $wfRoot
    if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
}

foreach ($f in @($fastchess, $engine, $book)) {
    if (-not (Test-Path $f)) { throw "Required file not found: $f" }
}

Write-Host "Installing matplotlib (weather-factory dependency)..."
pip install matplotlib --quiet
if ($LASTEXITCODE -ne 0) { Write-Warning "pip install matplotlib failed; run it manually if needed." }

$tuner = Join-Path $wfRoot "tuner"
New-Item -ItemType Directory -Force -Path $tuner | Out-Null

if (-not $Resume) {
    $stateFiles = @("state.json", "games.pgn", "graph.png", "fastchess_config.json")
    $existingState = $stateFiles |
        ForEach-Object { Join-Path $tuner $_ } |
        Where-Object { Test-Path $_ }

    if ($existingState) {
        $archive = Join-Path $tuner ("archive_" + (Get-Date -Format "yyyyMMdd_HHmmss"))
        New-Item -ItemType Directory -Force -Path $archive | Out-Null
        foreach ($f in $existingState) {
            Move-Item $f (Join-Path $archive (Split-Path $f -Leaf)) -Force
        }
        Write-Host "Archived previous tuner state -> $archive"
    }
}

$engineName = Split-Path $engine -Leaf
Write-Host "Copying engine    -> $tuner\$engineName"
Copy-Item $engine (Join-Path $tuner $engineName) -Force
Write-Host "Copying book      -> $tuner\$(Split-Path $book -Leaf)"
Copy-Item $book (Join-Path $tuner (Split-Path $book -Leaf)) -Force

Write-Host "Copying fastchess -> $wfRoot\fastchess.exe"
try {
    Copy-Item $fastchess (Join-Path $wfRoot "fastchess.exe") -Force
} catch {
    Write-Host "  skipped; fastchess.exe appears to be in use, existing copy will be used"
}

$cutechessJson = @{
    engine        = $engineName
    book          = "SuperGM_4mvs.pgn"
    games         = 32
    tc            = 1
    hash          = 64
    threads       = 15
    save_rate     = 10
    pgnout        = "file=tuner/games.pgn"
    use_fastchess = $true
} | ConvertTo-Json
$cutechessJson | Out-File (Join-Path $wfRoot "cutechess.json") -Encoding utf8 -NoNewline
Write-Host "Wrote cutechess.json"

$A = [int]([Math]::Floor($Iterations / 10))
$spsaJson = "{`n    ""a"": 1.0,`n    ""c"": 1.0,`n    ""A"": $A,`n    ""alpha"": 0.601,`n    ""gamma"": 0.102`n}"
$spsaJson | Out-File (Join-Path $wfRoot "spsa.json") -Encoding utf8 -NoNewline
Write-Host "Wrote spsa.json (A=$A for $Iterations planned iterations)"

$srcConfig = Join-Path $configs "config_$ConfigGroup.json"
Copy-Item $srcConfig (Join-Path $wfRoot "config.json") -Force
Write-Host "Wrote config.json (group: $ConfigGroup)"

Write-Host ""
Write-Host "============================================================"
Write-Host "  Setup complete."
Write-Host ""
Write-Host "  Run SPSA:"
Write-Host "    cd tools\weather-factory"
Write-Host "    python main.py"
Write-Host ""
Write-Host "  Stop with Ctrl-C when values stabilise."
Write-Host "  State saved every 10 iterations -> tuner\state.json"
Write-Host "  Resume the same run with: ./tools/setup_spsa.ps1 -Resume"
Write-Host "============================================================"
