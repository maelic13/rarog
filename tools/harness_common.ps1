# Shared preflight for clock-based fastchess harnesses.

$script:MinimumAffinityFastchessVersion = [version]"1.7.0"
$script:HarnessIsWindows = [Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT

# One named source of truth for result adjudication in strength measurements.
#
# Both profiles are 600/3 TWO-SIDED as of 2026-08-18, by maintainer decision:
# one rule everywhere is worth more than the small saving one-sided bought.
# Two-sided requires both engines to agree before a game is called, so this is
# the CONSERVATIVE direction -- fewer adjudications, more games played out,
# marginally more wall time.
#
# The 2026-08-02 calibration that had split them is retained here as evidence,
# not as policy: over 69,350 Rarog games completed under two-sided 600/3,
# one-sided 600/3 produced no chess-result reversals (three apparent reversals
# were later time forfeits) and changed 71 results to wins that later drew,
# 0.20% of its 35,486 triggers. That is what makes unifying cheap -- the two
# rules were measured to differ on 0.20% of triggers and never on a final
# chess result -- and it is also why unifying is safe rather than merely tidy.
#
# Historical note for anyone reading old ledger rows: strength results before
# 2026-08-18 were adjudicated one-sided. The 0.20% figure above is the measured
# size of that discontinuity.
function Get-StrengthTestProfile {
    [pscustomobject]@{
        Name               = "strength-v2"
        ResignMoveCount    = 3
        ResignScore        = 600
        ResignTwoSided     = $true
        DrawMoveNumber     = 40
        DrawMoveCount      = 8
        DrawScore          = 10
    }
}

function Get-StrengthTestResignArgs {
    $profile = Get-StrengthTestProfile
    $args = @(
        '-resign'
        "movecount=$($profile.ResignMoveCount)"
        "score=$($profile.ResignScore)"
    )
    if ($profile.ResignTwoSided) { $args += 'twosided=true' }
    $args
}

# Datagen keeps its own named profile even though the values now match
# strength-v2 exactly. The reason is ownership, not arithmetic: a false
# resignation assigns the wrong target to every position sampled from that
# game, so if strength adjudication is ever loosened again, labels must not
# follow it silently. Same numbers today, different owner and different
# justification.
function Get-DatagenProfile {
    [pscustomobject]@{
        Name               = "datagen-v1"
        ResignMoveCount    = 3
        ResignScore        = 600
        ResignTwoSided     = $true
        DrawMoveNumber     = 40
        DrawMoveCount      = 8
        DrawScore          = 10
    }
}

# datagen-v2 (2026-09-01, RAR-M17): no adjudication at all. The label-quality
# argument for dropping it in datagen is stronger than the verdict argument
# that dropped it in sprt.ps1, and it is not about mislabeling -- resign at
# 600/3 two-sided almost never calls a game wrong. It is SAMPLE DEPLETION.
# RAR-M15 measured adjudication ending 52.7% of all endgames before they are
# reached, so an adjudicated corpus is systematically short of exactly the
# positions the endgame families need, and the phase-balanced extraction then
# draws its endgame reservoir from a truncated distribution. Basilisk hit the
# same shape from the other side: adjudicated data left its corpus without
# mating material, which made king safety free to destroy mating behavior.
#
# datagen-v1 is retained by name, not edited, because `hce-v2` and every
# manifest already written cite it and must keep meaning what they said.
function Get-DatagenProfileV2 {
    [pscustomobject]@{
        Name               = "datagen-v2"
        Adjudication       = $false
        ResignMoveCount    = $null
        ResignScore        = $null
        ResignTwoSided     = $false
        DrawMoveNumber     = $null
        DrawMoveCount      = $null
        DrawScore          = $null
    }
}

# datagen-v3 (2026-09-01): no EVAL-based adjudication, plus Syzygy tablebase
# adjudication. This is the label contract to prefer, and it resolves a real
# tension that datagen-v2 alone does not.
#
# Removing eval adjudication does not by itself make labels truthful. It makes
# them reflect what the DATAGEN ENGINE can actually convert at its node budget,
# and 4.9a.1 measured that at 60,000 nodes: KBN-K converts at 7%, KRP-KR at
# 52%, KBB-K at 86%. At datagen's 8,000 nodes it is worse. So a theoretically
# won endgame is played out and recorded as a DRAW, which mislabels every
# position sampled from that game -- the exact failure eval adjudication was
# accused of, arriving from the other direction.
#
# Tablebase adjudication is not lossy in that way because it is not an opinion:
# on reaching 6 men it ends the game with the position's true value. The fifty
# move rule is deliberately NOT disabled (`-tbignore50` is not passed), because
# the label must be the result the game would really have had; a cursed win is
# a draw and should be labelled one.
#
# This is for DATAGEN ONLY. A strength gate must never use it: a gate measures
# realized conversion skill, and adjudicating on tablebase truth would credit
# both arms equally for an endgame only one of them can actually win.
function Get-DatagenProfileV3 {
    param([Parameter(Mandatory)][string]$SyzygyPath, [int]$Pieces = 6)
    [pscustomobject]@{
        Name               = "datagen-v3"
        Adjudication       = $false
        TablebaseAdjudication = $true
        TablebasePath      = $SyzygyPath
        TablebasePieces    = $Pieces
        TablebaseIgnore50  = $false
        ResignMoveCount    = $null
        ResignScore        = $null
        ResignTwoSided     = $false
        DrawMoveNumber     = $null
        DrawMoveCount      = $null
        DrawScore          = $null
    }
}

function Get-DatagenResignArgs {
    $profile = Get-DatagenProfile
    $args = @(
        '-resign'
        "movecount=$($profile.ResignMoveCount)"
        "score=$($profile.ResignScore)"
    )
    if ($profile.ResignTwoSided) { $args += 'twosided=true' }
    $args
}

function Get-HarnessPhysicalCpus {
    if ($script:HarnessIsWindows) {
        if (-not ('RarogHarness.CpuTopology' -as [type])) {
            Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.InteropServices;

namespace RarogHarness {
    public sealed class CpuCore {
        public int Cpu { get; set; }
        public int EfficiencyClass { get; set; }
    }

    public static class CpuTopology {
        private const int RelationProcessorCore = 0;

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetLogicalProcessorInformationEx(
            int relationship, IntPtr buffer, ref uint returnedLength);

        public static CpuCore[] PhysicalCpus() {
            uint length = 0;
            GetLogicalProcessorInformationEx(RelationProcessorCore, IntPtr.Zero, ref length);
            if (length == 0) throw new Win32Exception(Marshal.GetLastWin32Error());

            IntPtr buffer = Marshal.AllocHGlobal((int)length);
            try {
                if (!GetLogicalProcessorInformationEx(RelationProcessorCore, buffer, ref length))
                    throw new Win32Exception(Marshal.GetLastWin32Error());

                var result = new List<CpuCore>();
                int offset = 0;
                int groupAffinitySize = IntPtr.Size + 8;
                while (offset < length) {
                    IntPtr entry = IntPtr.Add(buffer, offset);
                    int relationship = Marshal.ReadInt32(entry, 0);
                    int size = Marshal.ReadInt32(entry, 4);
                    if (size <= 0 || offset + size > length)
                        throw new InvalidOperationException("Invalid Windows CPU-topology record.");

                    if (relationship == RelationProcessorCore) {
                        int efficiencyClass = Marshal.ReadByte(entry, 9);
                        int groupCount = (ushort)Marshal.ReadInt16(entry, 30);
                        var logical = new List<int>();
                        for (int groupIndex = 0; groupIndex < groupCount; ++groupIndex) {
                            int gaOffset = 32 + groupIndex * groupAffinitySize;
                            ulong mask = IntPtr.Size == 8
                                ? unchecked((ulong)Marshal.ReadInt64(entry, gaOffset))
                                : unchecked((uint)Marshal.ReadInt32(entry, gaOffset));
                            int group = (ushort)Marshal.ReadInt16(entry, gaOffset + IntPtr.Size);
                            for (int bit = 0; bit < IntPtr.Size * 8; ++bit)
                                if ((mask & (1UL << bit)) != 0) logical.Add(group * 64 + bit);
                        }
                        if (logical.Count == 0)
                            throw new InvalidOperationException("A physical core has no logical processors.");
                        result.Add(new CpuCore {
                            Cpu = logical.Min(),
                            EfficiencyClass = efficiencyClass
                        });
                    }
                    offset += size;
                }

                return result
                    .OrderByDescending(c => c.EfficiencyClass)
                    .ThenBy(c => c.Cpu)
                    .ToArray();
            } finally {
                Marshal.FreeHGlobal(buffer);
            }
        }
    }
}
'@
        }
        return [RarogHarness.CpuTopology]::PhysicalCpus()
    }

    if (Get-Command lscpu -ErrorAction SilentlyContinue) {
        $seen = @{}
        $cores = foreach ($line in (& lscpu '-p=CPU,CORE,SOCKET' 2>$null)) {
            if (-not $line -or $line.StartsWith('#')) { continue }
            $cpu, $core, $socket = $line.Split(',')
            $key = "$socket,$core"
            if (-not $seen.ContainsKey($key)) {
                $seen[$key] = $true
                [pscustomobject]@{ Cpu = [int]$cpu; EfficiencyClass = 0 }
            }
        }
        return @($cores | Sort-Object Cpu)
    }

    return @(0..([Environment]::ProcessorCount - 1) |
        ForEach-Object { [pscustomobject]@{ Cpu = $_; EfficiencyClass = 0 } })
}

function Get-FastchessVersion {
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) { throw "fastchess not found: $Path" }

    $line = (& $Path --version 2>&1 | Select-Object -First 1)
    if (-not $line) { throw "Could not query fastchess version at '$Path'." }

    $match = [regex]::Match("$line", '(?<major>\d+)\.(?<minor>\d+)\.(?<patch>\d+)')
    if (-not $match.Success) { throw "Unrecognized fastchess version string: '$line'." }

    [pscustomobject]@{
        Text    = "$line".Trim()
        Version = [version]::new(
            [int]$match.Groups['major'].Value,
            [int]$match.Groups['minor'].Value,
            [int]$match.Groups['patch'].Value)
    }
}

function Assert-AffinityFastchess {
    param([Parameter(Mandatory)][string]$Path)

    $info = Get-FastchessVersion -Path $Path
    if ($script:HarnessIsWindows -and $info.Version -lt $script:MinimumAffinityFastchessVersion) {
        throw "fastchess $($info.Version) is too old for reliable Windows affinity. " +
              "Version 1.7.0 contains the process-affinity fix; run tools/setup_tools.ps1 " +
              "to install the pinned runner. Found: $($info.Text)"
    }
    $info
}

function Get-PhysicalCoreCount {
    $count = @(Get-HarnessPhysicalCpus).Count
    if (-not $count -or $count -lt 1) { $count = 1 }
    [int]$count
}

function Resolve-HarnessConcurrency {
    # 8.13: `ThreadsPerGame` generalises this past the 1-thread assumption.
    # Each concurrent game needs `ThreadsPerGame` physical cores, so the core
    # budget is divided, not handed out one game per core. At Threads=1 the
    # arithmetic is identical to before, so 1-thread runs are unaffected.
    # `AllowOversubscribe` is for NODE-LIMITED work only, and datagen is the
    # only caller that qualifies. The physical-core ceiling exists because a
    # TIMED game mismeasures under contention -- that is the whole affinity and
    # forfeit story, RAR-M14 included. A `go nodes` search has no such exposure:
    # it plays identical moves however slowly it runs, so contention costs wall
    # time and changes nothing else. Holding datagen to 14 of 32 logical
    # processors therefore threw away throughput for a hazard it does not have.
    param([int]$Requested, [int]$ReservePhysicalCores = 2, [int]$ThreadsPerGame = 1,
          [switch]$AllowOversubscribe)

    if ($ThreadsPerGame -lt 1) { throw "ThreadsPerGame must be >= 1 (got $ThreadsPerGame)." }
    $physical = Get-PhysicalCoreCount
    $ceiling = if ($AllowOversubscribe) { [Environment]::ProcessorCount } else { $physical }
    $budget = [Math]::Max(1, $ceiling - $ReservePhysicalCores)
    $recommended = [Math]::Max(1, [Math]::Floor($budget / $ThreadsPerGame))
    $resolved = if ($Requested -gt 0) { $Requested } else { $recommended }
    $needed = $resolved * $ThreadsPerGame
    if ($needed -gt $ceiling) {
        $kind = if ($AllowOversubscribe) { "logical processors" } else { "physical cores" }
        throw ("Concurrency $resolved x Threads $ThreadsPerGame = $needed, " +
               "which exceeds the detected $ceiling $kind.")
    }
    [pscustomobject]@{
        Concurrency   = [int]$resolved
        PhysicalCores = [int]$physical
        CoresUsed     = [int]$needed
        ThreadsPerGame = [int]$ThreadsPerGame
        AutoSelected  = ($Requested -le 0)
    }
}

function Get-HarnessAffinityCpuList {
    # 8.13: the pinned set must cover EVERY core the games will use, i.e.
    # Concurrency x ThreadsPerGame — not one core per game. Under-sizing this
    # list silently oversubscribes cores and reintroduces exactly the hidden
    # per-run offset the affinity pinning exists to remove.
    param([Parameter(Mandatory)][int]$Concurrency, [int]$ThreadsPerGame = 1)

    $cores = @(Get-HarnessPhysicalCpus)
    $needed = $Concurrency * $ThreadsPerGame
    if ($needed -gt $cores.Count) {
        throw "Concurrency $Concurrency x Threads $ThreadsPerGame = $needed exceeds $($cores.Count) physical cores."
    }
    (($cores | Select-Object -First $needed).Cpu -join ',')
}

function New-HarnessSeed {
    param([int]$Requested)
    if ($Requested -ne 0) { return $Requested }
    Get-Random -Minimum 1 -Maximum ([int]::MaxValue)
}

function Get-HarnessSha256 {
    param([Parameter(Mandatory)][string]$Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
}

function Get-EngineUciOptions {
    param(
        [Parameter(Mandatory)][string]$Path,
        [int]$TimeoutMs = 15000,
        [switch]$Detailed
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Engine not found: $Path"
    }

    $full = (Resolve-Path -LiteralPath $Path).Path
    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName               = $full
    $psi.WorkingDirectory       = Split-Path -Parent $full
    $psi.RedirectStandardInput  = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError  = $true
    $psi.UseShellExecute        = $false

    $proc = [System.Diagnostics.Process]::Start($psi)
    $text = ""
    try {
        # Read asynchronously before writing. Otherwise a verbose engine can
        # fill the stdout pipe while WaitForExit waits for a process that can
        # no longer make progress.
        $stdout = $proc.StandardOutput.ReadToEndAsync()
        $stderr = $proc.StandardError.ReadToEndAsync()
        $proc.StandardInput.WriteLine("uci")
        $proc.StandardInput.WriteLine("quit")
        $proc.StandardInput.Close()
        if (-not $proc.WaitForExit($TimeoutMs)) {
            throw "Engine '$Path' did not answer 'uci' within ${TimeoutMs} ms."
        }
        $text = $stdout.Result
        $errorText = $stderr.Result
        if ($proc.ExitCode -ne 0) {
            throw "Engine '$Path' exited $($proc.ExitCode) during UCI discovery: $errorText"
        }
    } finally {
        if (-not $proc.HasExited) { $proc.Kill($true) }
        $proc.Dispose()
    }

    if ($text -notmatch '(?m)^\s*uciok\s*$') {
        throw "Engine '$Path' did not emit 'uciok'; it is not a working UCI engine."
    }

    $options = [System.Collections.Generic.List[object]]::new()
    foreach ($line in ($text -split "`r?`n")) {
        $match = [regex]::Match($line, '^\s*option\s+name\s+(?<name>.+?)\s+type\s+(?<type>\S+)(?<tail>.*)$')
        if (-not $match.Success) { continue }
        $tail = $match.Groups['tail'].Value
        $defaultMatch = [regex]::Match($tail, '(?:^|\s)default\s+(?<value>\S+)')
        $minMatch = [regex]::Match($tail, '(?:^|\s)min\s+(?<value>-?\d+)')
        $maxMatch = [regex]::Match($tail, '(?:^|\s)max\s+(?<value>-?\d+)')
        $options.Add([pscustomobject]@{
            Name    = $match.Groups['name'].Value.Trim()
            Type    = $match.Groups['type'].Value
            Default = if ($defaultMatch.Success) { $defaultMatch.Groups['value'].Value } else { $null }
            Min     = if ($minMatch.Success) { [int64]$minMatch.Groups['value'].Value } else { $null }
            Max     = if ($maxMatch.Success) { [int64]$maxMatch.Groups['value'].Value } else { $null }
            Raw     = $line.Trim()
        })
    }
    if ($Detailed) { $options.ToArray(); return }
    $options.Name
}

function Test-EngineSupportsOption {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Name
    )

    $normalize = { param($value) ($value -replace '\s+', ' ').Trim().ToLowerInvariant() }
    $target = & $normalize $Name
    foreach ($advertised in (Get-EngineUciOptions -Path $Path)) {
        if ((& $normalize $advertised) -eq $target) { return $true }
    }
    $false
}

function Write-JsonAtomic {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][object]$Value,
        [int]$Depth = 8
    )

    $temporary = "$Path.tmp"
    try {
        $Value | ConvertTo-Json -Depth $Depth |
            Set-Content -LiteralPath $temporary -Encoding utf8
        Move-Item -LiteralPath $temporary -Destination $Path -Force
    } finally {
        if (Test-Path -LiteralPath $temporary) {
            Remove-Item -LiteralPath $temporary -Force
        }
    }
}

function Assert-NoMatchAnomaly {
    param(
        [Parameter(Mandatory)][string]$LogPath,
        [double]$TimeoutRateCeiling = 0.5
    )

    # ZERO TOLERANCE. A crash, an illegal move, a dropped engine or a protocol
    # error is never normal on this harness: across every stored gate log in
    # tools/results, not one Rarog match has produced any of them. If one
    # appears, the match is describing a broken engine or a broken runner.
    $hard = Select-String -LiteralPath $LogPath `
        -Pattern '(?i)(crashed:\s*[1-9]|disconnect|illegal move|protocol error)' `
        -ErrorAction SilentlyContinue
    if ($hard) {
        throw ("Match contained a crash/illegal-move/disconnect/protocol anomaly " +
               "and is invalid. See '$LogPath'.")
    }

    # RATE-LIMITED. Time forfeits are different: a small background rate is a
    # property of running 14 concurrent games on 14 physical cores, not of the
    # candidate. Measured across the stored logs, healthy matches -- including
    # two null calibrations running the SAME binary on both sides -- sit at
    # 0.03%-0.33%, while the two genuinely poisoned runs sit at 5.52%
    # (p810-mopup, 423/7,665) and 34.85% (VarA-pooled at Threads=4, starved by
    # fastchess 1.8.0 affinity pinning, 184/528). Two orders of magnitude
    # separate them, so a ceiling discriminates and zero tolerance does not.
    #
    # This threshold replaces a zero-tolerance test added in d2c7788 that no
    # match had ever run under. RAR-E06 was the first to reach it: it hit H1
    # at 3,914 games with 3 forfeits (0.077%) and was declared invalid, even
    # though all three flagged sides were already lost by 5-9 pawns and the
    # worst-case reversal of all three moves the estimate ~0.3 Elo against a
    # +22.04 result. Applied to history the old test voided nearly every
    # accepted gate in the project.
    #
    # The count is taken from the per-game 'loses on time' lines rather than
    # the 'Timeouts:' summary, because the summary is per player and would
    # double-count the denominator.
    $forfeits = @(Select-String -LiteralPath $LogPath -Pattern '(?i)loses on time' `
        -ErrorAction SilentlyContinue).Count
    $games = @(Select-String -LiteralPath $LogPath -Pattern '^Finished game \d' `
        -ErrorAction SilentlyContinue).Count
    if ($games -le 0) {
        if ($forfeits -gt 0) {
            throw "Match log records $forfeits time forfeit(s) but no finished games. See '$LogPath'."
        }
        return
    }
    $rate = 100.0 * $forfeits / $games
    if ($rate -gt $TimeoutRateCeiling) {
        # One format string, not a concatenation: `+` binds tighter than `-f`,
        # so "a" + "b" -f $x formats only "b" and throws the placeholders.
        $template = "Time-forfeit rate {0:N3}% ({1}/{2}) exceeds the {3}% ceiling; " +
                    "the match is invalid. See '{4}'."
        throw ($template -f $rate, $forfeits, $games, $TimeoutRateCeiling, $LogPath)
    }
    if ($forfeits -gt 0) {
        Write-Host ("  Time forfeits: {0}/{1} = {2:N3}% (under the {3}% ceiling; recorded, not fatal)" `
            -f $forfeits, $games, $rate, $TimeoutRateCeiling) -ForegroundColor Yellow
    }
}

function Assert-NoAffinityFailure {
    param([Parameter(Mandatory)][string]$LogPath)

    $failure = Select-String -LiteralPath $LogPath `
        -Pattern '(?i)(failed to set cpu affinity|no cores available)' `
        -ErrorAction SilentlyContinue
    if ($failure) {
        throw "fastchess reported an affinity failure; the match is invalid. See '$LogPath'."
    }
}
