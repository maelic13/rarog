<#
.SYNOPSIS
    One-shot setup for a weather-factory SPSA run.

.DESCRIPTION
    Clones weather-factory, installs its Python dependency, populates the
    tuner\ folder, and writes the three config files.  After this script
    completes, run:

        cd D:\chess\weather-factory
        python main.py

    Stop it with Ctrl-C whenever parameter values look stable (typically
    after 2 000 – 5 000 iterations).  State is saved every 10 iterations
    to tuner\state.json so you can resume at any time.

.PARAMETER ConfigGroup
    Which parameter group to tune.
    "pruning"  (default) – 13 pruning / margin constants, ready to tune now.
    "lmr"                – LMR weighted terms (blocked: needs Phase 3 port first).

.PARAMETER Iterations
    Planned total iterations (used to set A = Iterations / 10 in spsa.json).
    Default: 5000.  At ~12 s/iteration this is roughly 17 hours; stop
    earlier with Ctrl-C if values stabilise sooner.

.EXAMPLE
    # Standard first run — pruning group, 5 000 iterations
    ./tools/setup_spsa.ps1

.EXAMPLE
    # Shorter overnight run (~7 hours)
    ./tools/setup_spsa.ps1 -Iterations 2000
#>
param(
    [ValidateSet("pruning","lmr")][string]$ConfigGroup = "pruning",
    [int]$Iterations = 5000
)

$ErrorActionPreference = "Stop"

$wfRoot    = "D:\chess\weather-factory"
$repoRoot  = Split-Path -Parent $PSScriptRoot
$configs   = Join-Path $repoRoot "tools\spsa_configs"
$fastchess = "D:\chess\fastchess\fastchess.exe"
$engine    = "D:\chess\engines\test_engines\rarog-phase1-defaults-pext-pgo.exe"
$book      = "D:\chess\books\SuperGM_4mvs.pgn"

# ── 1. Validate prerequisites ────────────────────────────────────────────────
foreach ($f in @($fastchess, $engine, $book)) {
    if (-not (Test-Path $f)) { throw "Required file not found: $f" }
}

# ── 2. Clone weather-factory (skip if already present) ──────────────────────
if (-not (Test-Path $wfRoot)) {
    Write-Host "Cloning weather-factory..."
    git clone https://github.com/jnlt3/weather-factory $wfRoot
    if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
} else {
    Write-Host "weather-factory already present at $wfRoot — skipping clone."
}

# ── 3. Install Python dependency ─────────────────────────────────────────────
Write-Host "Installing matplotlib (weather-factory dependency)..."
pip install matplotlib --quiet
if ($LASTEXITCODE -ne 0) { throw "pip install failed" }

# ── 4. Populate tuner\ folder ────────────────────────────────────────────────
$tuner = Join-Path $wfRoot "tuner"
New-Item -ItemType Directory -Force -Path $tuner | Out-Null

$engineName = Split-Path $engine -Leaf
Write-Host "Copying engine  → $tuner\$engineName"
Copy-Item $engine  (Join-Path $tuner $engineName) -Force
Write-Host "Copying book    → $tuner\$(Split-Path $book -Leaf)"
Copy-Item $book    (Join-Path $tuner (Split-Path $book -Leaf)) -Force

# weather-factory calls fastchess as just "fastchess" (no path), so it
# must be findable via the CWD when running from the weather-factory root.
Write-Host "Copying fastchess → $wfRoot\fastchess.exe"
Copy-Item $fastchess (Join-Path $wfRoot "fastchess.exe") -Force

# ── 5. Write cutechess.json ──────────────────────────────────────────────────
$cutechessJson = @{
    engine       = $engineName
    book         = "SuperGM_4mvs.pgn"
    games        = 32
    tc           = 1
    hash         = 64
    threads      = 15
    save_rate    = 10
    # weather-factory builds the arg as "-pgnout $pgnout" (no file= prefix),
    # so the value itself must carry "file=" for fastchess to accept it.
    pgnout       = "file=tuner/games.pgn"
    use_fastchess = $true
} | ConvertTo-Json
$cutechessJson | Out-File (Join-Path $wfRoot "cutechess.json") -Encoding utf8 -NoNewline
Write-Host "Wrote cutechess.json"

# ── 6. Write spsa.json (A = Iterations / 10) ─────────────────────────────────
# ConvertTo-Json can't have both "a" and "A" as keys (case-insensitive in PS),
# so write the JSON directly as a string.
$A = [int]([Math]::Floor($Iterations / 10))
$spsaJson = "{`n    ""a"": 1.0,`n    ""c"": 1.0,`n    ""A"": $A,`n    ""alpha"": 0.601,`n    ""gamma"": 0.102`n}"
$spsaJson | Out-File (Join-Path $wfRoot "spsa.json") -Encoding utf8 -NoNewline
Write-Host "Wrote spsa.json  (A=$A for $Iterations planned iterations)"

# ── 7. Write config.json from the chosen parameter group ─────────────────────
$srcConfig = Join-Path $configs "config_$ConfigGroup.json"
Copy-Item $srcConfig (Join-Path $wfRoot "config.json") -Force
Write-Host "Wrote config.json (group: $ConfigGroup)"

# ── Done ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "============================================================"
Write-Host "  Setup complete."
Write-Host ""
Write-Host "  Run SPSA:"
Write-Host "    cd $wfRoot"
Write-Host "    python main.py"
Write-Host ""
Write-Host "  Stop with Ctrl-C when values stabilise."
Write-Host "  State saved every 10 iterations → tuner\state.json"
Write-Host "  Resume after a stop: just run python main.py again"
Write-Host "  (it auto-reads tuner\state.json)"
Write-Host "============================================================"
