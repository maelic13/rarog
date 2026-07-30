<#
.SYNOPSIS
    One-shot setup: download fastchess and clone weather-factory into tools/.

.DESCRIPTION
    Makes the Rarog tuning toolchain self-contained inside the repo. Run this
    once after cloning if tools/bin/fastchess.exe or tools/weather-factory is
    missing.

    After this script:
      - tools/bin/fastchess.exe
      - tools/weather-factory/
      - matplotlib installed for Python

    The opening books belong in tools/books/ (git-ignored; source:
    D:\chess\books\): UHO_Lichess_4852_v1.epd (SPRT/SPSA/gauntlet default)
    and IM_4mvs.pgn (balanced fallback / CCRL-comparable gauntlets). Copy
    them there before running SPRT or SPSA.

.PARAMETER FastchessTag
    GitHub release tag to download. Default v1.8.0-alpha, a pinned release
    containing the Windows process-affinity fix introduced before v1.7.0.

.EXAMPLE
    ./tools/setup_tools.ps1
#>
param(
    [string]$FastchessTag = "v1.8.0-alpha"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "harness_common.ps1")

$binDir = Join-Path $PSScriptRoot "bin"
$wfDir  = Join-Path $PSScriptRoot "weather-factory"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$fastchessExe = Join-Path $binDir "fastchess.exe"
$downloadFastchess = -not (Test-Path $fastchessExe)
if (Test-Path $fastchessExe) {
    try {
        $info = Assert-AffinityFastchess -Path $fastchessExe
        Write-Host "fastchess already present: $($info.Text)"
        Write-Host "  Existing compatible runner retained (version is recorded per match)."
    } catch {
        Write-Warning $_.Exception.Message
        Write-Host "  Replacing incompatible runner with $FastchessTag."
        $downloadFastchess = $true
    }
}
if ($downloadFastchess) {
    Write-Host "Downloading fastchess ($FastchessTag)..."

    $apiUrl = if ($FastchessTag -eq "latest") {
        "https://api.github.com/repos/Disservin/fastchess/releases/latest"
    } else {
        "https://api.github.com/repos/Disservin/fastchess/releases/tags/$FastchessTag"
    }

    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ Accept = "application/vnd.github.v3+json" }
    $asset = $release.assets |
        Where-Object { $_.name -like "*windows-x86-64*" } |
        Select-Object -First 1

    if (-not $asset) {
        throw "No windows-x86-64 asset found in fastchess release $($release.tag_name). Download manually to tools/bin/fastchess.exe."
    }

    $zipPath = Join-Path $binDir "fastchess.zip"
    Write-Host "  Downloading $($asset.name) from $($release.tag_name)..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath
    if ($asset.digest -match '^sha256:(?<hash>[0-9a-fA-F]{64})$') {
        $actualHash = (Get-FileHash $zipPath -Algorithm SHA256).Hash
        if ($actualHash -ne $Matches['hash']) {
            throw "fastchess archive SHA-256 mismatch: expected $($Matches['hash']), got $actualHash"
        }
        Write-Host "  Archive SHA-256 verified."
    }
    Write-Host "  Extracting..."
    $extractDir = Join-Path ([System.IO.Path]::GetTempPath()) ("rarog-fastchess-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $extractDir | Out-Null
    try {
        Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force
        $extracted = @(Get-ChildItem -LiteralPath $extractDir -Recurse -Filter "fastchess.exe" -File)
        if ($extracted.Count -ne 1) {
            throw "Expected one fastchess.exe in $($asset.name), found $($extracted.Count)."
        }
        Copy-Item -LiteralPath $extracted[0].FullName -Destination $fastchessExe -Force
    } finally {
        Remove-Item -LiteralPath $extractDir -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $zipPath -Force -ErrorAction SilentlyContinue
    }

    if (-not (Test-Path $fastchessExe)) {
        throw "fastchess.exe not found in tools/bin after extraction. Check zip contents and extract manually."
    }

    $ver = & $fastchessExe --version 2>&1 | Select-Object -First 1
    Write-Host "  Done: $ver"
    Assert-AffinityFastchess -Path $fastchessExe | Out-Null
}

if (Test-Path (Join-Path $wfDir "main.py")) {
    Write-Host "weather-factory already present at tools/weather-factory/; skipping clone."
} else {
    Write-Host "Cloning weather-factory -> tools/weather-factory/ ..."
    git clone https://github.com/jnlt3/weather-factory $wfDir
    if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
    Write-Host "  Done."
}

# weather-factory has no native affinity setting. Patch its generated
# fastchess command with the OS-derived physical-core list. Rebuild this line
# on every setup so moving the clone to other hardware cannot retain stale IDs.
$wfCute = Join-Path $wfDir "cutechess.py"
if (Test-Path $wfCute) {
    $c = Get-Content $wfCute -Raw
    $allPhysicalCpus = (Get-HarnessPhysicalCpus).Cpu -join ','
    $c = $c -replace '(?m)^\s*\+ \("-use-affinity " if self\.use_fastchess else ""\).*\r?\n?', ''
    $c = $c -replace '(?m)^.*RAROG_AFFINITY_PATCH_V2.*\r?\n?', ''
    $anchor = 'f"-concurrency {self.threads} "'
    $patch = $anchor + "`n" + ('            f"{''-use-affinity ' + $allPhysicalCpus + ' '' if self.use_fastchess else ''''}"  # RAROG_AFFINITY_PATCH_V2')
    if (-not $c.Contains($anchor)) {
        throw "weather-factory/cutechess.py affinity patch anchor not found; upstream changed."
    }
    $c = $c.Replace($anchor, $patch)
    Set-Content -Path $wfCute -Value $c -Encoding utf8

    python -m py_compile $wfCute
    if ($LASTEXITCODE -ne 0) {
        throw "weather-factory affinity patch failed Python syntax validation: $wfCute"
    }
    Write-Host "  weather-factory affinity patch and Python syntax verified."
}

# ADJUDICATION ALIGNMENT (2026-07-30). weather-factory ships
# `-resign movecount=3 score=400` with no `twosided`, while sprt.ps1 uses
# `movecount=3 score=600 twosided=true` — so the tuner was optimising under
# game-termination rules the gate did not use, violating the unified-conditions
# principle that puts SPSA and SPRT on the same TC and the same book.
#
# Two independent problems, and the one-sided flag is the worse of them:
#   * score=400 vs 600 — the tune resigned games the gate would have played on.
#   * NO `twosided` — fastchess then adjudicates on the LOSING side's own
#     evaluation alone. Both SPSA arms are the SAME binary with perturbed
#     parameters, so an arm whose parameters produce more extreme scores
#     resigns more readily than its sibling. That is an asymmetry between the
#     two arms of every mini-match, i.e. it lands directly in the gradient the
#     tuner is estimating. `twosided=true` requires both engines to agree the
#     game is decided, which removes the asymmetry by construction.
#
# Verified against fishtest's worker (official-stockfish/fishtest,
# worker/games.py) 2026-07-30: Stockfish uses `-resign movecount=3 score=600`
# and the string "twosided" does not appear in that file at all. So we match
# its THRESHOLD exactly and deliberately go one step stricter on the flag.
# That deviation is defensible: fishtest runs on donated heterogeneous workers
# where one-sided resignation ends games sooner and throughput dominates; we
# run one machine where the correctness of a gradient matters more than a few
# percent of games-per-hour.
#
# NOT aligned to fishtest, deliberately: the draw rule. Ours is
# `movenumber=40 movecount=8 score=10` against fishtest's
# `movenumber=34 movecount=8 score=20` — later AND with a tighter score
# window, i.e. strictly more conservative on both axes, and it already agrees
# between sprt.ps1 and the tuner. Changing it would move the verdict
# instrument and break comparability with the whole existing ledger for no
# correctness gain.
$wfCuteAdj = Join-Path $wfDir "cutechess.py"
if (Test-Path $wfCuteAdj) {
    $a = Get-Content $wfCuteAdj -Raw
    if ($a -match 'RAROG_ADJUDICATION_PATCH_V1') {
        Write-Host "  weather-factory adjudication patch already present."
    } else {
        $anchorResign = '"-resign movecount=3 score=400 "'
        if (-not $a.Contains($anchorResign)) {
            throw ("weather-factory/cutechess.py adjudication anchor not found; upstream changed. " +
                "Expected $anchorResign — check the resign line before assuming it is already aligned.")
        }
        $a = $a.Replace($anchorResign,
            '"-resign movecount=3 score=600 twosided=true "  # RAROG_ADJUDICATION_PATCH_V1: match sprt.ps1')
        Set-Content -Path $wfCuteAdj -Value $a -Encoding utf8

        python -m py_compile $wfCuteAdj
        if ($LASTEXITCODE -ne 0) {
            throw "weather-factory adjudication patch failed Python syntax validation: $wfCuteAdj"
        }
        Write-Host "  weather-factory adjudication patch and Python syntax verified."
    }
}

# weather-factory's SPSA schedule feeds t (GAMES, 32/iteration) into Spall's
# decay, which is designed per-iteration — the gain annealed 32^0.601 ~= 8x
# too fast and every tune froze after a few hundred iterations (PLAN: "SPSA
# `A` is in the wrong units", found 2026-07-23). Patch step() to convert
# units; t/state.json stay in games so old states resume correctly.
$wfSpsa = Join-Path $wfDir "spsa.py"
if (Test-Path $wfSpsa) {
    $s = Get-Content $wfSpsa -Raw
    if ($s -match 'RAROG_SCHEDULE_FIX_V1') {
        Write-Host "  weather-factory SPSA schedule patch already present."
    } else {
        $anchorA = 'a_t = self.spsa.a / (self.t + self.spsa.A) ** self.spsa.alpha'
        $anchorC = 'c_t = self.spsa.c / self.t ** self.spsa.gamma'
        if (-not ($s.Contains($anchorA) -and $s.Contains($anchorC))) {
            throw "weather-factory/spsa.py schedule patch anchors not found; upstream changed."
        }
        $s = $s.Replace($anchorA,
            "it = self.t / self.cutechess.games  # RAROG_SCHEDULE_FIX_V1: Spall decay per-iteration; t/state.json stay in games`n" +
            "        a_t = self.spsa.a / (it + self.spsa.A) ** self.spsa.alpha")
        $s = $s.Replace($anchorC, 'c_t = self.spsa.c / it ** self.spsa.gamma')
        Set-Content -Path $wfSpsa -Value $s -Encoding utf8

        python -m py_compile $wfSpsa
        if ($LASTEXITCODE -ne 0) {
            throw "weather-factory schedule patch failed Python syntax validation: $wfSpsa"
        }
        Write-Host "  weather-factory SPSA schedule patch and Python syntax verified."
    }
}


# weather-factory's main.py loops forever (`while True:`), so a target
# iteration count existed only in the operator's head — unworkable for the
# 5,000-iteration tunes 10.4.6 needs, which always span several sessions.
# Patch it to stop cleanly at $env:RAROG_MAX_ITERS (0/unset = unbounded), and
# guard the finally-block rate prints against a zero-length session (resuming
# an already-complete run would otherwise ZeroDivisionError after saving).
$wfMain = Join-Path $wfDir "main.py"
if (Test-Path $wfMain) {
    $m = Get-Content $wfMain -Raw
    if ($m -match 'RAROG_MAX_ITERS_V1') {
        Write-Host "  weather-factory main.py iteration-target patch already present."
    } else {
        $anchor = '(?m)^(    try:\r?\n)(        while True:\r?\n)(            start = time\.time\(\))'
        if (-not [regex]::IsMatch($m, $anchor)) {
            throw "weather-factory/main.py loop anchor not found; upstream changed."
        }
        $m = [regex]::Replace($m, '(?m)^import dataclasses', "import dataclasses`nimport os")
        $repl = "    max_iters = int(os.environ.get('RAROG_MAX_ITERS', '0'))  # RAROG_MAX_ITERS_V1`n" +
                "    if max_iters:`n        print(f'Target: {max_iters} iterations (set RAROG_MAX_ITERS=0 to run unbounded).')`n" +
                '$1$2' +
                "            if max_iters and spsa.t / cutechess.games >= max_iters:`n" +
                "                print(f'Reached target {max_iters} iterations - stopping cleanly.')`n" +
                "                break`n" + '$3'
        $m = [regex]::Replace($m, $anchor, $repl)
        $m = $m.Replace('(spsa.t - start_t)', 'max(1, spsa.t - start_t)')
        Set-Content -Path $wfMain -Value $m -Encoding utf8

        python -m py_compile $wfMain
        if ($LASTEXITCODE -ne 0) {
            throw "weather-factory main.py patch failed Python syntax validation: $wfMain"
        }
        Write-Host "  weather-factory main.py iteration-target patch and Python syntax verified."
    }
}

Write-Host "Installing matplotlib (weather-factory dependency)..."
pip install matplotlib --quiet
if ($LASTEXITCODE -ne 0) { Write-Warning "pip install matplotlib failed; run manually if needed." }

Write-Host ""
Write-Host "============================================================"
Write-Host "  Toolchain setup complete."
Write-Host ""
Write-Host "  Next steps:"
Write-Host "    1. Build a tune binary:"
Write-Host "         ./tools/build_test.ps1 -Suffix phase1-lmr -Tune"
Write-Host "    2. Configure and start SPSA (setup + launch, one command):"
Write-Host "         ./tools/spsa.ps1 -ConfigGroup lmr -EngineSuffix phase1-lmr"
Write-Host "============================================================"

