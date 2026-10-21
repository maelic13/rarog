# Build N INDEPENDENT PGO binaries of one ISA tier, for pooled NPS comparison.
#
# Why this exists: a single PGO build carries a fixed per-binary offset of about
# 0.4% (RAR-P17, RAR-P18), so `nps_read.py` needs several builds per arm to
# resolve a sub-1% effect. Producing them by hand invites the two failure
# modes this file exists to prevent:
#
#   1. Measuring the same bytes twice. `cargo xtask build --pgo` overwrites one
#      asset path, so copies must be taken between builds. If two copies hash
#      the same, the PGO profile did not actually vary and pooling them is
#      meaningless - this script FAILS instead of silently averaging duplicates.
#   2. Measuring the wrong binary. Every copy is launched and must reproduce the
#      expected bench fingerprint before it is accepted into the pool.
#
# Output: $OutDir\<tier>-<n>.exe plus a manifest.txt of hashes and fingerprints.
#
# -NoPgo builds the plain tier instead, to separate a compiler's code
# generation from its use of the profile. A plain build has no profile to
# vary, so its copies must be IDENTICAL: each later build first touches
# src/lib.rs to force a real recompile, any difference fails, and the
# manifest keeps the one binary, which `nps_read.py` reads as a single-build
# arm.

param(
    [Parameter(Mandatory = $true)][ValidateSet("base", "x86-64", "avx2", "pext")][string]$Arch,
    [int]$Builds = 4,
    [Parameter(Mandatory = $true)][string]$OutDir,
    # The source's `bench 13` node count (GUIDE's checkpoint, or the arm's
    # own). No default: a default goes stale with the first integrated
    # behaviour change and then refuses every honest build.
    [Parameter(Mandatory = $true)][int]$ExpectFingerprint,
    # Plain (non-PGO) builds, checked deterministic; see the header.
    [switch]$NoPgo,
    [int]$Depth = 13,
    # Cargo features for a candidate arm compiled behind a flag.
    [string]$Features = "",
    # Local experimentation only. A pool built from an unidentifiable tree must
    # never back a recorded result; `sprt.ps1` carries the same escape hatch.
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
if ($NoPgo -and $Builds -lt 2) { throw "-NoPgo needs -Builds 2 or more: the extra builds prove the plain build deterministic" }
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$sha = (& git rev-parse HEAD).Trim()
$dirty = (& git status --porcelain)
if ($dirty -and -not $AllowDirty) {
    throw "working tree is dirty - refusing to build a pool whose source cannot be identified (-AllowDirty to override locally)"
}
if ($dirty) { Write-Warning "DIRTY TREE: this pool is not reproducible and must not back a recorded result" }

$benchIn = Join-Path $env:TEMP "nps_pool_bench_in.txt"
Set-Content -LiteralPath $benchIn -Value "bench $Depth" -Encoding ascii

$rows = @()
for ($i = 1; $i -le $Builds; $i++) {
    Write-Host "== $Arch build $i/$Builds =="
    $log = Join-Path $OutDir "build-$Arch-$i.log"
    $featureArgs = if ($Features) { @("--features", $Features) } else { @() }
    $pgoArgs = if ($NoPgo) { @() } else { @("--pgo") }
    # A plain build of an unchanged tree is cargo's no-op; the touch makes
    # the determinism check compare two real compiles.
    if ($NoPgo -and $i -gt 1) { (Get-Item src/lib.rs).LastWriteTime = Get-Date }
    & cargo xtask build --arch $Arch @pgoArgs @featureArgs *> $log
    if ($LASTEXITCODE -ne 0) { throw "build $i failed (exit $LASTEXITCODE); see $log" }

    # Take the artifact path from xtask itself rather than reconstructing the
    # asset name here - the naming rules live in one place and must stay there.
    $built = (Select-String -Path $log -Pattern '^Built (.+)$' | Select-Object -Last 1)
    if (-not $built) { throw "no 'Built <path>' line in $log - cannot identify the artifact" }
    $src = $built.Matches[0].Groups[1].Value.Trim()
    if (-not (Test-Path $src)) { throw "xtask reported '$src' but it does not exist" }

    $dst = Join-Path $OutDir "$Arch-$i.exe"
    Copy-Item $src $dst -Force

    $hash = (Get-FileHash -Algorithm SHA256 $dst).Hash
    $outFile = Join-Path $env:TEMP "nps_pool_bench_out.txt"
    $p = Start-Process -FilePath $dst -RedirectStandardInput $benchIn `
        -RedirectStandardOutput $outFile -NoNewWindow -PassThru -Wait
    if ($p.ExitCode -ne 0) { throw "$dst exited $($p.ExitCode) running bench $Depth" }
    $fpLine = Select-String -Path $outFile -Pattern '^Nodes searched\s*:\s*(\d+)' | Select-Object -Last 1
    if (-not $fpLine) { throw "no fingerprint from $dst - it benched nothing" }
    $nodes = [int]$fpLine.Matches[0].Groups[1].Value
    if ($nodes -ne $ExpectFingerprint) {
        throw "$dst benched $nodes, expected $ExpectFingerprint - this is not the binary you think it is"
    }

    "  {0}  {1}  nodes {2}" -f (Split-Path $dst -Leaf), $hash.Substring(0, 16), $nodes | Write-Host
    $rows += [pscustomobject]@{ File = $dst; Hash = $hash; Nodes = $nodes }
}

if ($NoPgo) {
    # The check that makes one plain build stand for its arm: no build luck.
    $distinct = @($rows | Group-Object Hash)
    if ($distinct.Count -ne 1) {
        throw "plain builds of one tree differ ($($distinct.Count) distinct hashes) - one build cannot stand for the arm"
    }
    Get-ChildItem -LiteralPath $OutDir -Filter "$Arch-*.exe" | Where-Object { $_.Name -ne "$Arch-1.exe" } | Remove-Item
    $kept = $rows.Count
    $rows = @($rows[0])
}

# The check that makes pooling meaningful: independent profiles => distinct bytes.
$dupes = $rows | Group-Object Hash | Where-Object { $_.Count -gt 1 }
if ($dupes) {
    $names = ($dupes.Group.File | Split-Path -Leaf) -join ", "
    throw "IDENTICAL binaries in the pool ($names) - the PGO profile did not vary, so pooling them measures nothing"
}

# The manifest names one source for every build, so the tree is read again
# after them: a commit or an edit made while they compiled means the builds
# need not share a source, and such a pool is labelled as one that cannot be
# identified, or refused when it was asked to be clean.
$shaAfter = (& git rev-parse HEAD).Trim()
$dirtyAfter = (& git status --porcelain)
$clean = (-not $dirty) -and (-not $dirtyAfter) -and ($shaAfter -eq $sha)
if (-not $clean -and -not $AllowDirty) {
    throw "the source changed while the pool was built (HEAD $sha, now $shaAfter, or files modified) - its builds cannot be identified"
}
$sourceState = if ($clean) { "(clean)" } else { "(DIRTY: not reproducible, must not back a recorded result)" }

$manifest = Join-Path $OutDir "manifest-$Arch.txt"
@(
    "arch          : $Arch"
    "features      : $(if ($Features) { $Features } else { '(default)' })"
    "source        : $sha $sourceState"
    "builds        : $Builds"
    $(if ($NoPgo) { "pgo           : no (deterministic: $kept builds identical, one kept)" } else { "pgo           : yes" })
    "fingerprint   : $ExpectFingerprint (verified on every copy)"
    "generated     : $(Get-Date -Format o)"
    ""
) + ($rows | ForEach-Object { "{0}  {1}" -f $_.Hash, (Split-Path $_.File -Leaf) }) |
Set-Content -LiteralPath $manifest -Encoding ascii

Write-Host ""
if ($NoPgo) {
    Write-Host "plain OK: $Builds identical $Arch builds at fingerprint $ExpectFingerprint, one kept"
} else {
    Write-Host "pool OK: $Builds distinct $Arch builds, all at fingerprint $ExpectFingerprint"
}
Write-Host "manifest: $manifest"
Write-Host ("set for nps_multibuild: " + (($rows.File | ForEach-Object { "`"$_`"" }) -join ", "))
