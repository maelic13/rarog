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

    The opening book belongs at tools/books/SuperGM_4mvs.pgn. If it is missing,
    copy it there before running SPRT or SPSA.

.PARAMETER FastchessTag
    GitHub release tag to download. Default "latest" fetches the newest
    release. Pin a tag for reproducibility.

.EXAMPLE
    ./tools/setup_tools.ps1
#>
param(
    [string]$FastchessTag = "latest"
)

$ErrorActionPreference = "Stop"

$binDir = Join-Path $PSScriptRoot "bin"
$wfDir  = Join-Path $PSScriptRoot "weather-factory"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$fastchessExe = Join-Path $binDir "fastchess.exe"
if (Test-Path $fastchessExe) {
    $ver = & $fastchessExe --version 2>&1 | Select-Object -First 1
    Write-Host "fastchess already present: $ver"
    Write-Host "  Delete tools/bin/fastchess.exe to re-download."
} else {
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
    Write-Host "  Extracting..."
    Expand-Archive -Path $zipPath -DestinationPath $binDir -Force
    Remove-Item $zipPath

    if (-not (Test-Path $fastchessExe)) {
        throw "fastchess.exe not found in tools/bin after extraction. Check zip contents and extract manually."
    }

    $ver = & $fastchessExe --version 2>&1 | Select-Object -First 1
    Write-Host "  Done: $ver"
}

if (Test-Path (Join-Path $wfDir "main.py")) {
    Write-Host "weather-factory already present at tools/weather-factory/; skipping clone."
} else {
    Write-Host "Cloning weather-factory -> tools/weather-factory/ ..."
    git clone https://github.com/jnlt3/weather-factory $wfDir
    if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
    Write-Host "  Done."
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
Write-Host "    2. Configure and start SPSA:"
Write-Host "         ./tools/setup_spsa.ps1 -ConfigGroup lmr -EngineSuffix phase1-lmr"
Write-Host "         cd tools/weather-factory"
Write-Host "         python main.py"
Write-Host "============================================================"
