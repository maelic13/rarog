# Shared preflight for clock-based fastchess harnesses.

$script:MinimumAffinityFastchessVersion = [version]"1.7.0"
$script:HarnessIsWindows = [Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT

# One named source of truth for result adjudication in strength measurements.
# Calibrated 2026-08-02 on 69,350 Rarog games completed under the stricter
# 600/3 two-sided rule: one-sided 600/3 produced no chess-result reversals
# (three apparent reversals were later time forfeits) and changed 71 results
# to wins that later drew, 0.20% of its 35,486 triggers. Datagen intentionally
# keeps a separate, stricter training-label profile because one wrong result
# labels many positions.
function Get-StrengthTestProfile {
    [pscustomobject]@{
        Name               = "strength-v1"
        ResignMoveCount    = 3
        ResignScore        = 600
        ResignTwoSided     = $false
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

# Training labels need a stricter result contract than strength tests.  A
# false resignation assigns the wrong target to every sampled position in that
# game, so datagen keeps the historical two-sided 600/3 rule deliberately.
# Keep this separate from strength-v1: SPRT uses one-sided resignation because
# its only job is to decide a game result quickly and safely.
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
    param([int]$Requested, [int]$ReservePhysicalCores = 2, [int]$ThreadsPerGame = 1)

    if ($ThreadsPerGame -lt 1) { throw "ThreadsPerGame must be >= 1 (got $ThreadsPerGame)." }
    $physical = Get-PhysicalCoreCount
    $budget = [Math]::Max(1, $physical - $ReservePhysicalCores)
    $recommended = [Math]::Max(1, [Math]::Floor($budget / $ThreadsPerGame))
    $resolved = if ($Requested -gt 0) { $Requested } else { $recommended }
    $needed = $resolved * $ThreadsPerGame
    if ($needed -gt $physical) {
        throw ("Concurrency $resolved x Threads $ThreadsPerGame = $needed cores, " +
               "which exceeds the detected $physical physical cores.")
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

function Assert-NoAffinityFailure {
    param([Parameter(Mandatory)][string]$LogPath)

    $failure = Select-String -LiteralPath $LogPath `
        -Pattern '(?i)(failed to set cpu affinity|no cores available)' `
        -ErrorAction SilentlyContinue
    if ($failure) {
        throw "fastchess reported an affinity failure; the match is invalid. See '$LogPath'."
    }
}
