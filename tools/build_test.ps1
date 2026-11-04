<#
.SYNOPSIS
    Build a Rarog test binary and copy it to the test-engines folder.

.DESCRIPTION
    Three modes:

    Normal (default): runs `cargo xtask build --tier pext` — produces a
    PGO-optimised pext binary.  Use for SPRT and gauntlet testing.  PGO changes
    hot-path timing enough to affect measured Elo comparisons; always use this
    for match testing.

    Native (-Native switch): runs `cargo xtask build --native` — produces a
    PGO-optimised binary of the host's best tier built with
    `-C target-cpu=native` for the exact host CPU (e.g. znver3 on a 5950X), instead of the portable
    x86-64-v3 baseline.  It is not the fast build: on the 5950X it read 3.8%
    slower than the normal mode (RAR-P51), so match and pool binaries use the
    normal mode.  Do NOT distribute this binary, since it is not guaranteed to
    run on other CPUs.

    Tune (-Tune switch): runs `cargo xtask build --tier pext --nopgo --features tune`
    — produces a non-PGO pext binary with search-parameter UCI options exposed.  Use ONLY
    for weather-factory SPSA runs.  PGO is skipped because SPSA accuracy does
    not depend on absolute NPS — both sides of each mini-match use the same
    binary.  With -Features (a candidate arm compiled behind a flag) it builds
    `--features tune,<Features>` and records the flavor `<Features>-tune`, so
    the manifest's bench fingerprint is that arm's own.

    Output always goes to tools\test_engines\ (repo-local and separate from
    released engines).

    A schema-versioned sidecar binds the copied binary to its exact SHA-256,
    source tree, compiler, build command and bench verification. Use
    -SourceRoot to build a frozen worktree with this same checked wrapper.

.PARAMETER Suffix
    Short label for the output file.
    Normal:  rarog-<Suffix>-pext-pgo.exe
    Native:  rarog-<Suffix>-pext-native-pgo.exe
    Tune:    rarog-<Suffix>-tune.exe

.PARAMETER Native
    Build with `--native` instead of `--tier pext`: the host's best tier (pext
    on a 5950X, so the same PEXT code path) with `target-cpu=native` in place
    of the portable baseline.  Local-only.

.PARAMETER Tune
    Build with --features tune instead of PGO.  Use for SPSA binaries only.
    Combine with -Features to tune a candidate arm compiled behind a flag.

.PARAMETER Features
    Cargo features for a candidate arm compiled behind a flag.
    For a PGO build they pass to xtask for both PGO builds; with -Tune they
    join `tune` in the cargo build and name the flavor (`<Features>-tune`). Either
    way they are recorded in the build command and the verified bench
    fingerprint is the arm's own. `tune` and `texel` are refused: the first is
    implied by -Tune, the second bypasses the evaluation caches.

.PARAMETER TestEnginesDir
    Destination directory.  Default: tools\test_engines

.PARAMETER SourceRoot
    Rarog worktree to build. Default: the repository containing this script.

.PARAMETER BuildOnly
    Write a hash-bound manifest without launching bench. Match and datagen
    launchers reject this verification state; it is only for staged builds.

.EXAMPLE
    # Normal SPRT binary
    ./tools/build_test.ps1 -Suffix phase1-lmr

.EXAMPLE
    # Native (-march=native-equivalent) binary for local-only testing
    ./tools/build_test.ps1 -Suffix phase292-native -Native

.EXAMPLE
    # SPSA tuning binary (exposes UCI options)
    ./tools/build_test.ps1 -Suffix phase1-lmr -Tune
#>
param(
    [Parameter(Mandatory)][string]$Suffix,
    [switch]$Tune,
    [switch]$Native,
    [switch]$BuildOnly,
    [string]$Features = "",
    [int]$BenchDepth = 13,
    [string]$TestEnginesDir = "$PSScriptRoot\test_engines",
    [string]$SourceRoot = ""
)

if ($Tune -and $Native) {
    throw "-Tune and -Native are mutually exclusive."
}
if ($Features -match '(^|,)\s*(tune|texel)\s*(,|$)') {
    throw "-Features may not name 'tune' (implied by -Tune) or 'texel' (it bypasses the evaluation caches)."
}
if ($BenchDepth -lt 1) { throw "-BenchDepth must be positive." }

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. "$PSScriptRoot\harness_common.ps1"

# --- provenance manifest -----------------------------------------------------
# Every test binary gets a sidecar JSON next to it: git SHA + dirty flag,
# branch, rustc, and a bench fingerprint VERIFIED by running the binary just
# built (which doubles as a smoke test — a broken build fails here, not in an
# SPRT). sprt.ps1 copies both engines' manifests into the result dir, so every
# result is permanently self-describing.
#
# LOCAL-ONLY BY DESIGN (user decision 2026-07-20): manifests exist for
# development provenance. tools/test_engines/ and tools/results/ are
# gitignored, and the release workflow (build.yml) has NO manifest step —
# nothing here can ever appear on the GitHub release page.
function Write-EngineManifest {
    param(
        [Parameter(Mandatory)][string]$BinaryPath,
        [Parameter(Mandatory)][string]$Suffix,
        [Parameter(Mandatory)][string]$Flavor,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string]$BuildCommand,
        [Parameter(Mandatory)][int]$Depth,
        [switch]$SkipBench
    )

    $sha    = (& git -C $RepositoryRoot rev-parse HEAD).Trim()
    $tree   = (& git -C $RepositoryRoot rev-parse 'HEAD^{tree}').Trim()
    $branch = (& git -C $RepositoryRoot rev-parse --abbrev-ref HEAD).Trim()
    $dirty  = [bool](& git -C $RepositoryRoot status --porcelain)
    $rustc  = (rustc -V).Trim()
    $binary = Get-Item -LiteralPath $BinaryPath

    $nodes = $null
    $benchLine = $null
    if (-not $SkipBench) {
        Write-Host "Verifying bench fingerprint of $($binary.Name) (depth $Depth) ..."
        $inputPath = [IO.Path]::GetTempFileName()
        $stdoutPath = [IO.Path]::GetTempFileName()
        $stderrPath = [IO.Path]::GetTempFileName()
        try {
            Set-Content -LiteralPath $inputPath -Value "bench $Depth" -Encoding ascii
            $process = Start-Process -FilePath $BinaryPath -WindowStyle Hidden -Wait -PassThru `
                -RedirectStandardInput $inputPath -RedirectStandardOutput $stdoutPath `
                -RedirectStandardError $stderrPath
            if ($process.ExitCode -ne 0) {
                throw "Bench exited with code $($process.ExitCode); refusing an unverified manifest."
            }
            $benchOut = (Get-Content -LiteralPath $stdoutPath -Raw) +
                (Get-Content -LiteralPath $stderrPath -Raw)
        } finally {
            Remove-Item -LiteralPath $inputPath, $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue
        }
        $benchLine = (($benchOut -split "`n" | Where-Object { $_ -match "Nodes searched" }) | Select-Object -Last 1).Trim()
        if ($benchLine -notmatch "([0-9][0-9,]*)\s*$") {
            throw "Could not parse a bench node count from the built binary - refusing an unverified manifest."
        }
        $nodes = [int64]($Matches[1] -replace ",", "")
        if ($nodes -le 0) { throw "Bench reported $nodes nodes - broken binary." }
    }

    $manifest = [ordered]@{
        schema_version    = 2
        engine            = $binary.Name
        binary_sha256     = Get-HarnessSha256 $BinaryPath
        binary_size_bytes = $binary.Length
        suffix            = $Suffix
        flavor            = $Flavor
        build_command      = $BuildCommand
        git_sha           = $sha
        git_tree          = $tree
        git_branch        = $branch
        git_dirty         = $dirty
        rustc             = $rustc
        verification      = if ($SkipBench) { "build-only" } else { "bench" }
        bench_depth       = if ($SkipBench) { $null } else { $Depth }
        bench_nodes       = $nodes
        bench_line        = $benchLine
        pgo_workload      = if ($Flavor -like "*-pgo") { "bench 13 (xtask default)" } else { $null }
        built_utc         = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    }

    $manifestPath = [IO.Path]::ChangeExtension($BinaryPath, ".json")
    Write-JsonAtomic -Path $manifestPath -Value $manifest
    $verified = if ($SkipBench) { "build-only; engine not launched" } else { "bench $nodes" }
    Write-Host "Manifest: $manifestPath  ($verified$(if ($dirty) { ', DIRTY WORKING TREE' }))"
    if ($dirty) {
        Write-Host "WARNING: built from a DIRTY working tree — this binary is not reproducible from git_sha alone." -ForegroundColor Yellow
    }
}

# Build through xtask, which names the file and checks the built engine
# reports that name, and return the file it reports as built. The path comes
# from xtask's own `Built <path>` line, never from a glob, so the naming rules
# stay in one place and a stale file cannot be picked up.
function Invoke-XtaskBuild {
    param([Parameter(Mandatory)][string[]]$Arguments)
    $log = [IO.Path]::GetTempFileName()
    try {
        & cargo xtask build @Arguments *> $log
        $exit = $LASTEXITCODE
        Get-Content -LiteralPath $log | Write-Host
        if ($exit -ne 0) { throw "cargo xtask build $($Arguments -join ' ') failed (exit $exit)" }
        $built = Select-String -LiteralPath $log -Pattern '^Built (.+)$' | Select-Object -Last 1
        if (-not $built) { throw "xtask printed no 'Built <path>' line; cannot identify the binary." }
        return Get-Item -LiteralPath $built.Matches[0].Groups[1].Value.Trim()
    } finally {
        Remove-Item -LiteralPath $log -Force -ErrorAction SilentlyContinue
    }
}

$repoRoot = if ($SourceRoot) {
    (Resolve-Path -LiteralPath $SourceRoot).Path
} else {
    Split-Path -Parent $PSScriptRoot
}
if (-not (Test-Path -LiteralPath (Join-Path $repoRoot "Cargo.toml") -PathType Leaf)) {
    throw "-SourceRoot is not a Rarog worktree: $repoRoot"
}
$TestEnginesDir = [IO.Path]::GetFullPath($TestEnginesDir)
Push-Location $repoRoot
try {
    if ($Tune) {
        Write-Host ""
        $tuneFeatures = if ($Features) { "tune,$Features" } else { "tune" }
        $tuneFlavor = if ($Features) { "$($Features -replace '\s*,\s*', '+')-tune" } else { "pext-tune" }
        Write-Host "Building pext tune binary (--features $tuneFeatures, no PGO, flavor $tuneFlavor) — suffix: $Suffix"
        Write-Host "NOTE: Use this binary only for SPSA, never for SPRT."
        Write-Host ""

        $tuneArgs = @("--tier", "pext", "--nopgo", "--features", $tuneFeatures)
        $built = Invoke-XtaskBuild -Arguments $tuneArgs

        if (-not (Test-Path $TestEnginesDir)) {
            New-Item -ItemType Directory -Path $TestEnginesDir | Out-Null
        }

        $dest = Join-Path $TestEnginesDir "rarog-$Suffix-tune.exe"
        Copy-Item $built.FullName $dest -Force
        Write-EngineManifest -BinaryPath $dest -Suffix $Suffix -Flavor $tuneFlavor `
            -RepositoryRoot $repoRoot -BuildCommand "cargo xtask build $($tuneArgs -join ' ')" `
            -Depth $BenchDepth -SkipBench:$BuildOnly
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    } else {
        # Gate binaries stay portable, so what we SPRT matches the shipped pext
        # build. -Native builds the host's best tier with `target-cpu=native`.
        $label = if ($Native) { "native" } else { "pext" }
        Write-Host ""
        Write-Host "Building $label+PGO binary (suffix: $Suffix) ..."
        Write-Host ""

        [string[]]$xtaskArgs = if ($Native) { @("--native") } else { @("--tier", "pext") }
        if ($Features) { $xtaskArgs += @("--features", $Features) }
        $dist = Invoke-XtaskBuild -Arguments $xtaskArgs

        if (-not (Test-Path $TestEnginesDir)) {
            New-Item -ItemType Directory -Path $TestEnginesDir | Out-Null
        }

        $fileFlavor = if ($Native) { "pext-native-pgo" } else { "pext-pgo" }
        $dest = Join-Path $TestEnginesDir "rarog-$Suffix-$fileFlavor.exe"
        Copy-Item $dist.FullName $dest -Force
        $buildCommand = "cargo xtask build $($xtaskArgs -join ' ')"
        Write-EngineManifest -BinaryPath $dest -Suffix $Suffix -Flavor $fileFlavor `
            -RepositoryRoot $repoRoot -BuildCommand $buildCommand -Depth $BenchDepth `
            -SkipBench:$BuildOnly
        Write-Host ""
        Write-Host "Done: $dest"
        Write-Host ""
    }
} finally {
    Pop-Location
}
