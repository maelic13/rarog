<#
.SYNOPSIS
    Set up AND run a repo-local weather-factory SPSA tune — one command, from
    the repo root, no manual cd.

.DESCRIPTION
    Merges the old setup_spsa.ps1 + run_spsa.ps1. By default it does BOTH:
      1. Setup — populate tools\weather-factory\tuner\ (engine, book, fastchess)
         and write the three config files (cutechess.json, spsa.json,
         config.json). Old tuner state is archived first (unless -Resume).
      2. Launch — run `python main.py` with tools\weather-factory as the working
         dir, piped through watch.ps1 (clean console: per-game lines go to the
         log, only param/report blocks show). Returns you to the repo root on
         Ctrl-C. fastchess is resolved via PATH (weather-factory calls a bare
         "fastchess", which Python 3.11's Popen won't find in the CWD on
         Windows).

    Prerequisites:
      - ./tools/setup_tools.ps1 once if tools\bin\fastchess.exe or
        tools\weather-factory\main.py is missing (this script also auto-clones
        weather-factory if absent).
      - Build the tune binary: ./tools/build_test.ps1 -Suffix <s> -Tune

.PARAMETER ConfigGroup
    Which parameter group to tune (selects tools\spsa_configs\config_<g>.json):
    pruning · lmr · histcov · corr · probcut · futility · tm · lazymargin · history · see.

.PARAMETER Iterations
    Planned total iterations (sets A = Iterations / 10 in spsa.json).
    Default 5000. State is saved every 10 iterations to tuner\state.json.

.PARAMETER REnd
Learning rate at the END of the planned run (fishtest's `r_end`). The gain
`a` is DERIVED from this and -Iterations, so the schedule always lands on
the same end-state whatever horizon you pick — changing -Iterations can
never silently change how hot the tune finishes. Default 0.0031, from a
simulation validated against 8.5's real trajectory; fishtest's own default
is 0.002, the same order. Larger = hotter = more late wander.

.PARAMETER Concurrency
    Parallel games per SPSA mini-match. Default 0 auto-detects physical cores
    and leaves two free. Engines remain single-threaded.

.PARAMETER EngineSuffix
    Suffix of the tune binary in tools\test_engines. If omitted, a per-group
    default is used (e.g. history -> p81-history). Accepts a bare suffix
    (rarog-<s>-tune.exe), a "-tune"/"-pext-pgo" suffix, or a full "*.exe".

.PARAMETER Resume
    Preserve the existing tuner state (state.json/games/graph) instead of
    archiving it — continues an interrupted run rather than starting fresh.

.PARAMETER SetupOnly
    Do the setup and stop (do not launch). Prints the launch command.

.PARAMETER LaunchOnly
    Skip setup and just launch (tuner must already be populated). Ignores
    -Iterations / -EngineSuffix / -Resume.

.PARAMETER LogFile
    Override the full-log path. Default tools\results\spsa_<ConfigGroup>.log.

.EXAMPLE
    # Fresh setup + run, one command:
    ./tools/build_test.ps1 -Suffix p81-history -Tune
    ./tools/spsa.ps1 -ConfigGroup history -Iterations 2500

.EXAMPLE
    # Continue an interrupted run:
    ./tools/spsa.ps1 -ConfigGroup history -Resume

.EXAMPLE
    # Set up now, launch later:
    ./tools/spsa.ps1 -ConfigGroup history -SetupOnly
    ./tools/spsa.ps1 -ConfigGroup history -LaunchOnly
#>
param(
    [ValidateSet("aspiration","selectivity","pruning","lmr","histcov","corr","probcut","futility","tm","lazymargin","history","see")]
    [string]$ConfigGroup = "lmr",
    [int]$Iterations = 5000,
    [double]$REnd = 0.0031,
    [int]$Concurrency = 0,
    [string]$EngineSuffix = "",
    [switch]$Resume,
    [switch]$SetupOnly,
    [switch]$LaunchOnly,
    [switch]$ShowValues,
    [string]$LogFile = ""
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "harness_common.ps1")

if ($SetupOnly -and $LaunchOnly) { throw "-SetupOnly and -LaunchOnly are mutually exclusive." }

# -ShowValues: print the current tuned values from tuner\state.json and exit.
# weather-factory saves state every ~10 iterations, so after a Ctrl-C (which
# tears down the pipe before Python's exit printout survives) this is the
# reliable way to read the finals — no need to catch the last console line.
if ($ShowValues) {
    $statePath = Join-Path $PSScriptRoot "weather-factory\tuner\state.json"
    if (-not (Test-Path $statePath)) { throw "No state.json at $statePath — nothing to show." }
    # -AsHashtable: state.json has both "a" and "A" (SPSA hyper-params), which
    # trips ConvertFrom-Json's case-insensitive default.
    $state = Get-Content $statePath -Raw | ConvertFrom-Json -AsHashtable
    # $state.t is the SPSA counter in GAMES, not iterations. Each iteration
    # plays `games` games (cutechess.json, default 32), so iters = t / games.
    $ccPath = Join-Path $PSScriptRoot "weather-factory\cutechess.json"
    $gamesPerIter = if (Test-Path $ccPath) {
        [int]((Get-Content $ccPath -Raw | ConvertFrom-Json).games)
    } else { 32 }
    if ($gamesPerIter -le 0) { $gamesPerIter = 32 }
    $iters = [int]($state.t / $gamesPerIter)
    Write-Host "Current SPSA values (tuner\state.json, iter $iters / $($state.t) games):"
    foreach ($p in $state.uci_params) {
        "{0,-16} = {1}" -f $p.name, [int][Math]::Round([double]$p.value) | Write-Host
    }
    return
}

$concurrencyInfo = Resolve-HarnessConcurrency -Requested $Concurrency
$Concurrency = $concurrencyInfo.Concurrency

$wfRoot    = Join-Path $PSScriptRoot "weather-factory"
$configs   = Join-Path $PSScriptRoot "spsa_configs"
$fastchess = Join-Path $PSScriptRoot "bin\fastchess.exe"
$watch     = Join-Path $PSScriptRoot "watch.ps1"
# UHO EPD (2026-07-17): weather-factory auto-detects the book format from the
# extension (cutechess.py: format={book.split('.')[-1]}), so the EPD works
# unmodified — and keeps SPSA on the same book as sprt.ps1 (PLAN principle #7).
$book      = Join-Path $PSScriptRoot "books\UHO_Lichess_4852_v1.epd"
if ($LogFile -eq "") { $LogFile = Join-Path $PSScriptRoot "results\spsa_$ConfigGroup.log" }

# ─── Setup ────────────────────────────────────────────────────────────────
if (-not $LaunchOnly) {
    if ($EngineSuffix -eq "") {
        $EngineSuffix = switch ($ConfigGroup) {
            "aspiration" { "p102a" }
            "selectivity" { "p1046a" }
            "lmr" { "p86-lmr" }
            "histcov" { "p84-histcov" }
            "corr" { "p85-corr" }
            "pruning" { "phase1-pruning" }
            "probcut" { "phase2-probcut" }
            "futility" { "phase2-futility" }
            "tm" { "phase5-tm" }
            "lazymargin" { "phase5-lazymargin" }
            "history" { "p81-history" }
            "see" { "p72-see" }
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
        Write-Host "weather-factory missing; running the pinned toolchain setup..."
        & (Join-Path $PSScriptRoot "setup_tools.ps1")
    }

    foreach ($f in @($fastchess, $engine, $book)) {
        if (-not (Test-Path $f)) { throw "Required file not found: $f" }
    }
    Assert-AffinityFastchess -Path $fastchess | Out-Null

    $wfCute = Join-Path $wfRoot "cutechess.py"
    $expectedAffinityCpus = (Get-HarnessPhysicalCpus).Cpu -join ','
    $wfCuteContent = if (Test-Path $wfCute) { Get-Content $wfCute -Raw } else { "" }
    if ($wfCuteContent -notmatch 'RAROG_AFFINITY_PATCH_V2' -or
        $wfCuteContent -notmatch [regex]::Escape("-use-affinity $expectedAffinityCpus ")) {
        throw "weather-factory is not carrying the verified Rarog affinity patch; run tools/setup_tools.ps1."
    }
    python -m py_compile $wfCute
    if ($LASTEXITCODE -ne 0) { throw "weather-factory Python syntax validation failed: $wfCute" }
    $wfSpsaPy = Join-Path $wfRoot "spsa.py"
    if ((Get-Content $wfSpsaPy -Raw) -notmatch 'RAROG_SCHEDULE_FIX_V1') {
        throw "weather-factory is not carrying the SPSA schedule fix (decay per-iteration); run tools/setup_tools.ps1."
    }
    if ($wfCuteContent -notmatch 'RAROG_ADJUDICATION_PATCH_V2') {
        throw ("weather-factory is not carrying the strength-v1 adjudication alignment (resign 600/3 one-sided, " +
            "matching sprt.ps1); run tools/setup_tools.ps1.")
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
    } else {
        Write-Host "Resume: keeping existing tuner state (state.json preserved)."
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
        book          = (Split-Path $book -Leaf)
        games         = 32
        tc            = 3      # 3+0.03 (weather-factory auto inc = tc/100); UNIFIED
                               # with sprt.ps1's default so SPSA optima transfer to
                               # the confirming SPRT (PLAN.md guiding principle #7).
        hash          = 64
        threads       = $Concurrency
        save_rate     = 10
        pgnout        = "file=tuner/games.pgn"
        use_fastchess = $true
    } | ConvertTo-Json
    $cutechessJson | Out-File (Join-Path $wfRoot "cutechess.json") -Encoding utf8 -NoNewline
    Write-Host "Wrote cutechess.json"

    # ── Gain derived from the END state, the way fishtest does it ────────
    # Stockfish's fishtest does NOT hand-pick `a`. Each parameter carries
    # `c_end` (perturbation size at the END of the run) and `r_end` (learning
    # rate at the end), and the schedule constants are DERIVED from them and
    # the planned horizon:
    #     c     = c_end * N^gamma          -> c_t hits c_end exactly at t=N
    #     a_end = r_end * c_end^2
    #     a     = a_end * (A + N)^alpha    -> a_t hits a_end exactly at t=N
    # That is the design our 2026-07-27 bug could not survive: because both
    # constants are back-solved from the horizon, changing the planned
    # iteration count can never silently change the END behaviour, and `a`
    # can never be left stale when the schedule shape changes.
    #
    # ⚠ ONLY THE `a` HALF IS IMPLEMENTED (clarified 2026-08-04). weather-factory
    # keeps `a`/`c` global and puts the per-parameter scale in each config's
    # `step`. `c` stays 1.0 and is NOT back-solved, and no knob declares a
    # `c_end` — so a config's `step` is the perturbation at iteration 1, NOT at
    # the horizon:
    #     c_t = c / it^gamma ;  c_t(1) = 1.0 ;  c_t(5000) = 0.4195
    #     perturbation(knob, it) = step * c_t(it)
    # The maths below is still self-consistent: substituting c_end = N^-gamma
    # into a = r_end * c_end^2 * (A+N)^alpha gives exactly the expression used
    # here, so every completed tune is valid. Only the NAME misleads — reading
    # `step` as `c_end` gives the wrong answer about whether a knob survives to
    # the horizon.
    #     a = r_end * (A + N)^alpha / N^(2*gamma)
    # Practical consequence, enforced by audit class 6: the engine receives
    # round(value), so an integer knob needs step * c_t(N) >= 0.5, i.e.
    # step >= 2. A step-1 integer knob goes dead at it > 2^(1/gamma) ~= 894.
    # Cross-check on the two calibrations agreeing from independent
    # directions: our simulation (validated against 8.5's real trajectory to
    # within 0.02 steps of observed wander) puts the optimum at a ≈ 0.1 for
    # N=5000, which is r_end ≈ 0.0031 — the same order as fishtest's 0.002
    # default, while the a=1.0 we shipped this morning is r_end ≈ 0.031, ~15x
    # hotter than fishtest has ever defaulted to. Two independent methods
    # agreeing that a=1.0 was far too hot.
    $alpha = 0.601
    $gamma = 0.102
    # ⚠⚠ DO NOT name these `$A` and `$a`. PowerShell variable names are
    # CASE-INSENSITIVE, so `$A` and `$a` are ONE variable: the gain assignment
    # silently overwrote the damping term, and spsa.json shipped
    # `"A": 0.0965` where it needed `"A": 500`. That is A ≈ 0, i.e. NO damping
    # over the first 10% of the run — the exact defect the 2026-07-27 schedule
    # fix existed to remove, reintroduced by a language footgun.
    # Found 2026-07-30 by a -SetupOnly dry run before 10.4.6(a), which is the
    # FIRST tune this parameterization would ever have driven, so no fit was
    # contaminated. The assertion below is what makes it un-shippable again.
    $dampingA = [int]([Math]::Floor($Iterations / 10))
    $gainA = $REnd * [Math]::Pow($dampingA + $Iterations, $alpha) / [Math]::Pow($Iterations, 2 * $gamma)
    $gainFmt = [Math]::Round($gainA, 5)
    $spsaPath = Join-Path $wfRoot "spsa.json"
    $spsaJson = "{`n    ""a"": $gainFmt,`n    ""c"": 1.0,`n    ""A"": $dampingA,`n    ""alpha"": $alpha,`n    ""gamma"": $gamma`n}"
    $spsaJson | Out-File $spsaPath -Encoding utf8 -NoNewline

    # Read back and verify. The schedule is invisible at runtime — a wrong A
    # produces a plausible-looking run that anneals wrongly for 40 hours — so it
    # is checked here, where it is still cheap.
    # ⚠ -AsHashtable is MANDATORY: `a` and `A` differ only in case, and the
    # default ConvertFrom-Json throws on that ("keys with different casing").
    # Same footgun as the variable naming above, one layer down. Index with
    # brackets — property access on the hashtable would be case-insensitive and
    # silently return the wrong one of the two.
    $written = Get-Content $spsaPath -Raw | ConvertFrom-Json -AsHashtable
    if ([int]$written['A'] -ne $dampingA) {
        throw "spsa.json A is $($written['A']), expected $dampingA (damping = 10% of the horizon)."
    }
    if ([int]$written['A'] -le 0) {
        throw "spsa.json A must be positive; got $($written['A']). A=0 means NO damping."
    }
    if ([Math]::Abs([double]$written['a'] - $gainFmt) -gt 1e-9) {
        throw "spsa.json a is $($written['a']), expected $gainFmt."
    }
    Write-Host "Wrote spsa.json (r_end=$REnd over $Iterations iterations -> a=$gainFmt, A=$dampingA)"
    Write-Host "  (a is DERIVED from r_end and the horizon — change -Iterations and it re-solves.)"
    Write-Host "  Verified: A = $($written['A']) (10% of horizon), a = $($written['a'])." -ForegroundColor Green

    $srcConfig = Join-Path $configs "config_$ConfigGroup.json"
    if (-not (Test-Path $srcConfig)) { throw "Config not found: $srcConfig" }
    Copy-Item $srcConfig (Join-Path $wfRoot "config.json") -Force
    Write-Host "Wrote config.json (group: $ConfigGroup)"
    Write-Host ""

    if ($SetupOnly) {
        Write-Host "============================================================"
        Write-Host "  Setup complete (SetupOnly). Launch when ready:"
        Write-Host "    ./tools/spsa.ps1 -ConfigGroup $ConfigGroup -LaunchOnly"
        Write-Host "============================================================"
        return
    }
}

# ─── Launch ───────────────────────────────────────────────────────────────
foreach ($p in @((Join-Path $wfRoot "main.py"), (Join-Path $wfRoot "cutechess.py"),
        (Join-Path $wfRoot "fastchess.exe"), $watch, (Join-Path $wfRoot "config.json"),
        (Join-Path $wfRoot "cutechess.json"))) {
    if (-not (Test-Path $p)) {
        throw "Not found: $p — run ./tools/spsa.ps1 -ConfigGroup $ConfigGroup (setup) first."
    }
}
$launchFastchess = Join-Path $wfRoot "fastchess.exe"
Assert-AffinityFastchess -Path $launchFastchess | Out-Null
$launchCute = Join-Path $wfRoot "cutechess.py"
$expectedAffinityCpus = (Get-HarnessPhysicalCpus).Cpu -join ','
$launchCuteContent = Get-Content $launchCute -Raw
if ($launchCuteContent -notmatch 'RAROG_AFFINITY_PATCH_V2' -or
    $launchCuteContent -notmatch [regex]::Escape("-use-affinity $expectedAffinityCpus ")) {
    throw "weather-factory is not carrying the verified Rarog affinity patch; run tools/setup_tools.ps1."
}
python -m py_compile $launchCute
if ($LASTEXITCODE -ne 0) { throw "weather-factory Python syntax validation failed: $launchCute" }
$launchSpsaPy = Join-Path $wfRoot "spsa.py"
if ((Get-Content $launchSpsaPy -Raw) -notmatch 'RAROG_SCHEDULE_FIX_V1') {
    throw "weather-factory is not carrying the SPSA schedule fix (decay per-iteration); run tools/setup_tools.ps1."
}
# Adjudication alignment is required to START a tune, but NEVER blocks a RESUME.
# A run that began under the old resign rule must finish under it: switching
# game-termination rules mid-tune makes the early iterations incomparable with
# the late ones, which is strictly worse than completing under the old rule.
# So the check keys on whether this is a fresh run, not on -LaunchOnly.
if (-not (Test-Path (Join-Path $wfRoot "tuner\state.json")) -and
    $launchCuteContent -notmatch 'RAROG_ADJUDICATION_PATCH_V2') {
    throw ("weather-factory is not carrying the strength-v1 adjudication alignment (resign 600/3 one-sided, " +
        "matching sprt.ps1); run tools/setup_tools.ps1 before starting a new tune.")
}

$launchConfigPath = Join-Path $wfRoot "cutechess.json"
$launchConfig = Get-Content $launchConfigPath -Raw | ConvertFrom-Json
if ([int]$launchConfig.threads -ne $Concurrency) {
    throw "cutechess.json concurrency is $($launchConfig.threads), but this launch resolved to $Concurrency. " +
          "Run setup again, or pass -Concurrency $($launchConfig.threads) explicitly to resume that run."
}

# ─── Multi-session bookkeeping ────────────────────────────────────────────
# A 5,000-iteration tune is ~40 h (28.57 s/iter measured), so it ALWAYS spans
# several sessions. Three things make that safe, and each was broken before
# 2026-07-27:
#   1. the log must APPEND on resume (it truncated — 8.5 lost 1,086 of its
#      3,670 iterations, and the trajectory is what the bake filter reads);
#   2. the run must STOP ITSELF at the target (main.py was `while True:`, so
#      the target existed only in the operator's head);
#   3. `A` is FROZEN at first launch — main.py restores spsa_params from
#      state.json, so re-passing -Iterations on a resume silently does
#      nothing. Get it right the first time or archive and restart.
$resuming = $LaunchOnly -or $Resume
$statePathLaunch = Join-Path $wfRoot "tuner\state.json"
$doneIters = 0
if (Test-Path $statePathLaunch) {
    $st = Get-Content $statePathLaunch -Raw | ConvertFrom-Json -AsHashtable
    $doneIters = [int]($st.t / $launchConfig.games)
    $stateA = $st.spsa_params.A
    if ($resuming -and $stateA -ne [int]([Math]::Floor($Iterations / 10))) {
        Write-Host ("NOTE: this run's schedule was fixed at first launch (A=$stateA => " +
            "$($stateA * 10) planned iterations). -Iterations $Iterations is IGNORED on a resume; " +
            "A comes from state.json. To change it, archive and start fresh.") -ForegroundColor Yellow
    }
}
$env:RAROG_MAX_ITERS = "$Iterations"

Write-Host "SPSA ($ConfigGroup): python main.py | watch.ps1"
Write-Host "  Log: $LogFile$(if ($resuming) { '  (APPENDING — previous sessions preserved)' })"
Write-Host "  State saved every 10 iterations -> tuner\state.json"
if ($doneIters -gt 0) {
    $remain = [Math]::Max(0, $Iterations - $doneIters)
    $eta = [TimeSpan]::FromSeconds($remain * 28.57)
    Write-Host ("  Progress: iteration $doneIters / $Iterations " +
        "($([Math]::Round(100.0 * $doneIters / $Iterations, 1))%) — " +
        "$remain to go, ETA $([int]$eta.TotalHours) h $($eta.Minutes) m") -ForegroundColor Cyan
} else {
    $eta = [TimeSpan]::FromSeconds($Iterations * 28.57)
    Write-Host ("  Target: $Iterations iterations, ETA $([int]$eta.TotalHours) h " +
        "at 28.57 s/iter — expect to resume across sessions:") -ForegroundColor Cyan
    Write-Host "    ./tools/spsa.ps1 -ConfigGroup $ConfigGroup -LaunchOnly -Iterations $Iterations"
}
Write-Host "  Stops itself at the target; Ctrl-C any time (state is saved, then resume)."
Write-Host ""

# weather-factory launches fastchess as a bare "fastchess" command, but on
# Windows + Python 3.11 subprocess.Popen does NOT search the current directory,
# so it fails with FileNotFoundError even though fastchess.exe is in $wfRoot.
# Prepending $wfRoot to PATH lets Popen resolve it (CreateProcess searches PATH).
# This is the committed safety net; the vendored cutechess.py is also patched to
# prefer ./fastchess.exe, but PATH covers a fresh weather-factory re-clone.
$savedPath = $env:PATH
$env:PATH = "$wfRoot;$env:PATH"
Push-Location $wfRoot
try {
    # 2>&1 folds Python's stderr into the stream so watch.ps1 can tee/filter it.
    # Use `& $watch` (an in-session pipeline stage), NOT `pwsh $watch` (a child
    # process): a script's process{} block only receives piped input when it runs
    # in THIS session — a separate pwsh silently drops the stream (empty console
    # + empty log).
    python main.py 2>&1 | & $watch -LogFile $LogFile -Append:$resuming
} finally {
    Pop-Location
    $env:PATH = $savedPath
}


