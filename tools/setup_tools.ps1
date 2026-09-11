<#
.SYNOPSIS
    Stage fastchess, the UHO book and the patched weather-factory toolchain.

.DESCRIPTION
    Makes the Rarog tuning toolchain self-contained inside the repo. Run this
    once after cloning if tools/bin/fastchess.exe or tools/weather-factory is
    missing.

    After this script:
      - tools/bin/fastchess.exe
      - tools/books/UHO_Lichess_4852_v1.epd (when present in -BookSource)
      - tools/weather-factory/
      - matplotlib installed for Python

    Opening books are git-ignored. The UHO strength-test book is copied from
    -BookSource when it is not already staged; IM_4mvs.pgn remains an optional
    balanced fallback.

.PARAMETER FastchessTag
    GitHub release tag to download. Default v1.8.0-alpha, a pinned release
    containing the Windows process-affinity fix introduced before v1.7.0.

.PARAMETER BookSource
    Directory containing UHO_Lichess_4852_v1.epd. Default D:\chess\books.

.EXAMPLE
    ./tools/setup_tools.ps1
#>
param(
    [string]$FastchessTag = "v1.8.0-alpha",
    [string]$BookSource = "D:\chess\books"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "harness_common.ps1")

# Every textual patch below is written against this exact upstream revision.
# A floating clone can accept an anchor while changing surrounding semantics,
# which is unacceptable for a runner that will consume 160,000 games.
$weatherFactoryRevision = "19b4805c9a2372955c29666118070269f34aa2eb"

$binDir   = Join-Path $PSScriptRoot "bin"
$booksDir = Join-Path $PSScriptRoot "books"
$wfDir    = Join-Path $PSScriptRoot "weather-factory"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
New-Item -ItemType Directory -Force -Path $booksDir | Out-Null

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

$bookName = "UHO_Lichess_4852_v1.epd"
$bookDest = Join-Path $booksDir $bookName
if (-not (Test-Path -LiteralPath $bookDest -PathType Leaf)) {
    $bookSourcePath = Join-Path $BookSource $bookName
    if (Test-Path -LiteralPath $bookSourcePath -PathType Leaf) {
        Write-Host "Copying $bookName -> tools/books/ ..."
        Copy-Item -LiteralPath $bookSourcePath -Destination $bookDest -Force
        Write-Host "  SHA-256: $(Get-HarnessSha256 $bookDest)"
    } else {
        Write-Warning "$bookName was not found in '$BookSource'; stage it before SPRT or SPSA."
    }
}

if (Test-Path (Join-Path $wfDir "main.py")) {
    Write-Host "weather-factory already present at tools/weather-factory/; skipping clone."
} else {
    Write-Host "Cloning weather-factory -> tools/weather-factory/ ..."
    git clone https://github.com/jnlt3/weather-factory $wfDir
    if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
    git -C $wfDir checkout --detach $weatherFactoryRevision
    if ($LASTEXITCODE -ne 0) { throw "Could not checkout pinned weather-factory revision $weatherFactoryRevision" }
    Write-Host "  Done."
}

$actualWeatherFactoryRevision = (git -C $wfDir rev-parse HEAD).Trim()
if ($actualWeatherFactoryRevision -ne $weatherFactoryRevision) {
    throw ("weather-factory is at $actualWeatherFactoryRevision, expected pinned revision " +
        "$weatherFactoryRevision. Preserve any tuner state, recreate tools/weather-factory, and rerun setup.")
}
Write-Host "weather-factory revision verified: $weatherFactoryRevision"

# Literal multi-line patch anchors must not depend on core.autocrlf. Normalize
# the pinned Python sources once so setup is reproducible across worktrees.
foreach ($pyFile in @("cutechess.py", "spsa.py", "main.py")) {
    $pyPath = Join-Path $wfDir $pyFile
    if (-not (Test-Path -LiteralPath $pyPath)) { continue }
    $text = (Get-Content $pyPath -Raw) -replace "`r`n", "`n"
    Set-Content -LiteralPath $pyPath -Value ($text.TrimEnd() + "`n") -Encoding utf8 -NoNewline
}
Write-Host "  Normalized weather-factory sources to LF for deterministic patching."

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

# Some experiments freeze discrete architecture outside SPSA and tune only
# continuous consumers. Teach the runner to apply those fixed UCI options identically to
# both perturbed engines; they are deliberately absent from config.json, so
# they cannot receive another parameter's noisy gradient.
if (Test-Path $wfCute) {
    $c = Get-Content $wfCute -Raw
    if ($c -match 'RAROG_FIXED_OPTIONS_V1') {
        Write-Host "  weather-factory fixed-option support already present."
    } else {
        $signatureAnchor = '        use_fastchess: bool = True'
        if (-not $c.Contains($signatureAnchor)) {
            throw "weather-factory/cutechess.py fixed-option signature anchor not found; upstream changed."
        }
        $c = $c.Replace($signatureAnchor, "        use_fastchess: bool = True,`n        fixed_options: dict | None = None")

        $fieldAnchor = '        self.use_fastchess = use_fastchess'
        if (-not $c.Contains($fieldAnchor)) {
            throw "weather-factory/cutechess.py fixed-option field anchor not found; upstream changed."
        }
        $c = $c.Replace($fieldAnchor,
            $fieldAnchor + "`n        self.fixed_options = fixed_options or {}  # RAROG_FIXED_OPTIONS_V1")

        $commandAnchor = "        return (`n            f`"{command} `""
        if (-not $c.Contains($commandAnchor)) {
            throw "weather-factory/cutechess.py fixed-option command anchor not found; upstream changed."
        }
        $commandReplacement = "        fixed = ' '.join(f'option.{name}={value}' for name, value in self.fixed_options.items())`n" +
            "        fixed = (fixed + ' ') if fixed else ''`n`n" + $commandAnchor
        $c = $c.Replace($commandAnchor, $commandReplacement)
        $c = $c.Replace('f"option.Hash={self.hash_size} {'' ''.join(params_a)} "',
            'f"option.Hash={self.hash_size} {fixed}{'' ''.join(params_a)} "')
        $c = $c.Replace('f"option.Hash={self.hash_size} {'' ''.join(params_b)} "',
            'f"option.Hash={self.hash_size} {fixed}{'' ''.join(params_b)} "')

        Set-Content -Path $wfCute -Value $c -Encoding utf8
        python -m py_compile $wfCute
        if ($LASTEXITCODE -ne 0) {
            throw "weather-factory fixed-option patch failed Python syntax validation: $wfCute"
        }
        Write-Host "  weather-factory fixed-option support verified."
    }
}

# ADJUDICATION REMOVED FROM THE TUNER, 2026-09-01 (RAR-M17). weather-factory
# ships `-resign movecount=3 score=400` and a `-draw` rule; both go, so SPSA
# matches sprt.ps1, gauntlet.ps1 and datagen.ps1, all of which now play games
# to a rules result.
#
# The history this replaces is worth keeping, because it is what makes
# dropping the rule cheap rather than reckless. V1-V3 aligned the tuner's
# RESIGN rule to 600/3 two-sided after a retrospective 69,350-game
# calibration: 600/3 produced no chess-result reversals (three apparent ones
# were later time forfeits) while 400/3 changed 1,533 outcomes and included 80
# eventual opposite winners. So resign at 600/3 was already close to free,
# which means removing it costs close to nothing either.
#
# The DRAW rule was the one nobody had priced, and V1-V3 never touched it.
# RAR-M15 measured it ending 52.7% of all endgames before they are reached.
# That is the line that mattered, and it is gone too.
$wfCuteAdj = Join-Path $wfDir "cutechess.py"
if (Test-Path $wfCuteAdj) {
    # V4, 2026-09-01 (RAR-M17): SPSA runs with NO adjudication, like every
    # other instrument here. Earlier versions aligned the tuner's resign rule
    # to strength-v2; the alignment argument is now moot because there is
    # nothing left to align to. Both the resign and draw lines are removed --
    # V1-V3 only ever rewrote resign and left the draw rule standing, which is
    # the line RAR-M15 measured ending 52.7% of endgames.
    #
    # This is a bigger change for SPSA than for a gate, and it is deliberate:
    # SPSA runs far more games, so it pays the ~10% throughput cost the most,
    # but a tuner blind to conversion tunes toward positions it never has to
    # convert. Do not switch a RUNNING tune to it; spsa.ps1 exempts resumes.
    $a = Get-Content $wfCuteAdj -Raw
    if ($a -match 'RAROG_ADJUDICATION_PATCH_V4') {
        Write-Host "  weather-factory adjudication patch already present (V4: none)."
    } else {
        $resignPattern = '(?m)^\s*"-resign[^"]*"(\s*#\s*RAROG_ADJUDICATION_PATCH_V[0-9][^
]*)?
?
'
        $drawPattern = '(?m)^\s*"-draw[^"]*"(\s*#[^
]*)?
?
'
        if ($a -notmatch $resignPattern) {
            throw ("weather-factory/cutechess.py resign anchor not found; upstream changed. " +
                "Inspect it before assuming the tuner runs without adjudication.")
        }
        if ($a -notmatch $drawPattern) {
            throw ("weather-factory/cutechess.py draw anchor not found; upstream changed. " +
                "Inspect it before assuming the tuner runs without adjudication.")
        }
        $marker = '            ""  # RAROG_ADJUDICATION_PATCH_V4: adjudication off (RAR-M17)' + "`n"
        $a = [regex]::Replace($a, $resignPattern, $marker)
        $a = [regex]::Replace($a, $drawPattern, '')
        Set-Content -Path $wfCuteAdj -Value $a -Encoding utf8

        python -m py_compile $wfCuteAdj
        if ($LASTEXITCODE -ne 0) {
            throw "weather-factory adjudication patch failed Python syntax validation: $wfCuteAdj"
        }
        if ((Get-Content $wfCuteAdj -Raw) -match '(?m)^\s*"-(resign|draw)') {
            throw "weather-factory/cutechess.py still passes adjudication after the V4 patch."
        }
        Write-Host "  weather-factory adjudication removed (V4) and Python syntax verified."
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

# Commit the iteration counter only after the mini-match and parameter update
# complete. Upstream advances it before launching fastchess, so Ctrl-C can save
# an unplayed point as completed and resume from the wrong annealing step. Also
# roll back the tiny parameter-update section if an interrupt lands inside it;
# otherwise state could contain half an update paired with the old counter.
if (Test-Path $wfSpsa) {
    $s = Get-Content $wfSpsa -Raw
    if ($s -notmatch 'RAROG_TRANSACTIONAL_STEP_V1') {
        $advanceAnchor = '(?m)^        self\.t \+= self\.cutechess\.games\r?\n        it = self\.t / self\.cutechess\.games  # RAROG_SCHEDULE_FIX_V1: Spall decay per-iteration; t/state\.json stay in games\r?$'
        if (-not [regex]::IsMatch($s, $advanceAnchor)) {
            throw "weather-factory/spsa.py transactional-step advance anchor not found; upstream changed."
        }
        $advanceReplacement = "        next_t = self.t + self.cutechess.games  # RAROG_TRANSACTIONAL_STEP_V1: commit after completed update`n" +
            "        it = next_t / self.cutechess.games  # RAROG_SCHEDULE_FIX_V1: Spall decay per-iteration; t/state.json stay in games"
        $s = [regex]::Replace($s, $advanceAnchor, $advanceReplacement)

        $commitAnchor = '(?m)^            param\.update\(-param_grad \* a_t \* param\.step\)\r?\n\r?\n    @property\r?$'
        if (-not [regex]::IsMatch($s, $commitAnchor)) {
            throw "weather-factory/spsa.py transactional-step commit anchor not found; upstream changed."
        }
        $commitReplacement = "            param.update(-param_grad * a_t * param.step)`n`n        self.t = next_t`n`n    @property"
        $s = [regex]::Replace($s, $commitAnchor, $commitReplacement)
        Set-Content -Path $wfSpsa -Value $s -Encoding utf8
    }

    $s = Get-Content $wfSpsa -Raw
    if ($s -notmatch 'RAROG_TRANSACTIONAL_STEP_V2') {
        $updateAnchor = '(?m)^        for param, delta, param in zip\(self\.uci_params, self\.delta, self\.uci_params\):\r?\n            param_grad = gradient / \(delta \* c_t\)\r?\n            param\.update\(-param_grad \* a_t \* param\.step\)\r?\n\r?\n        self\.t = next_t\r?$'
        if (-not [regex]::IsMatch($s, $updateAnchor)) {
            throw "weather-factory/spsa.py rollback anchor not found; upstream changed."
        }
        $updateReplacement = @'
        old_values = [param.value for param in self.uci_params]
        try:
            for param, delta in zip(self.uci_params, self.delta):
                param_grad = gradient / (delta * c_t)
                param.update(-param_grad * a_t * param.step)
        except BaseException:
            for param, old_value in zip(self.uci_params, old_values):
                param.value = old_value
            raise

        self.t = next_t  # RAROG_TRANSACTIONAL_STEP_V2: params and counter commit together
'@
        $s = [regex]::Replace($s, $updateAnchor, $updateReplacement)
        Set-Content -Path $wfSpsa -Value $s -Encoding utf8
    }

    if ((Get-Content $wfSpsa -Raw) -match 'RAROG_TRANSACTIONAL_STEP_V2') {
        python -m py_compile $wfSpsa
        if ($LASTEXITCODE -ne 0) {
            throw "weather-factory transactional-step patch failed Python syntax validation: $wfSpsa"
        }
        Write-Host "  weather-factory transactional SPSA step verified."
    } else {
        throw "weather-factory transactional SPSA V2 marker missing after patch."
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
Write-Host "         ./tools/build_test.ps1 -Suffix history -Tune"
Write-Host "    2. Configure and start SPSA (setup + launch, one command):"
Write-Host "         ./tools/spsa.ps1 -ConfigGroup history -EngineSuffix history"
Write-Host "============================================================"

