//! The x86-64 tier ladder: which tier a CPU can run, and which it should.
//!
//! Shared, by `#[path]`, between the engine (its start-up advice) and `xtask`
//! (which builds the host's best tier by default), so the two cannot disagree
//! about what a CPU is. Standard library only.
//!
//! A tier is one x86-64 microarchitecture level plus, for `pext`, the PEXT
//! slider path. A flag can say an instruction is legal; it cannot say it is
//! fast, so the one model-based rule is the slow-PEXT one below.

/// One x86-64 build tier, cheapest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// `x86-64`, the baseline every x86-64 CPU runs.
    Base,
    /// `x86-64-v2`: SSE4.2, SSSE3, POPCNT (Nehalem, AMD Bulldozer onward).
    Sse42,
    /// `x86-64-v3`: AVX2, BMI1, BMI2, FMA, LZCNT, MOVBE (Haswell, Excavator).
    Avx2,
    /// `x86-64-v3` with the PEXT slider path.
    Pext,
}

impl Tier {
    /// Every tier, cheapest first.
    pub const ALL: [Tier; 4] = [Tier::Base, Tier::Sse42, Tier::Avx2, Tier::Pext];

    /// The tier's name in file names, options and messages.
    pub const fn name(self) -> &'static str {
        match self {
            Tier::Base => "base",
            Tier::Sse42 => "sse42",
            Tier::Avx2 => "avx2",
            Tier::Pext => "pext",
        }
    }

    /// The microarchitecture level a CPU needs to run this tier at all.
    pub const fn level(self) -> Level {
        match self {
            Tier::Base => Level::V1,
            Tier::Sse42 => Level::V2,
            Tier::Avx2 | Tier::Pext => Level::V3,
        }
    }
}

/// The x86-64 microarchitecture level a CPU supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    V1,
    V2,
    V3,
}

/// The fastest tier for a CPU at `level`.
///
/// Pure, so it is tested across CPUs this machine is not.
pub const fn best_tier(level: Level, slow_pext: bool) -> Tier {
    match level {
        Level::V1 => Tier::Base,
        Level::V2 => Tier::Sse42,
        Level::V3 if slow_pext => Tier::Avx2,
        Level::V3 => Tier::Pext,
    }
}

/// Does this vendor and family implement `pdep`/`pext` in microcode?
///
/// AMD family 15h (Bulldozer through Excavator) and 17h (Zen, Zen+, Zen 2):
/// an order of magnitude slower than the hardware path, which makes the
/// `pext` build the slowest one there. Family 19h (Zen 3) onward and every
/// Intel CPU with BMI2 are fast. Stockfish carries this same exception and
/// no other model-based rule.
pub const fn has_slow_pext(is_amd: bool, family: u32) -> bool {
    is_amd && (family == 0x15 || family == 0x17)
}

/// The level of the CPU this runs on; `None` off x86-64.
///
/// `is_x86_feature_detected!` folds to `true` for a feature the build enables
/// statically, so in a tier compiled for a level this reports at least that
/// level. That is correct rather than a blind spot: a CPU running the build
/// has those features. The detection also checks the operating system saves
/// the AVX state. LAHF/SAHF, part of v2, is not detectable through `std`;
/// every CPU with SSE4.2 has it.
pub fn host_level() -> Option<Level> {
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::is_x86_feature_detected as has;
        let v2 = has!("sse3")
            && has!("ssse3")
            && has!("sse4.1")
            && has!("sse4.2")
            && has!("popcnt")
            && has!("cmpxchg16b");
        let v3 = v2
            && has!("avx")
            && has!("avx2")
            && has!("bmi1")
            && has!("bmi2")
            && has!("fma")
            && has!("f16c")
            && has!("lzcnt")
            && has!("movbe");
        Some(if v3 {
            Level::V3
        } else if v2 {
            Level::V2
        } else {
            Level::V1
        })
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        None
    }
}

/// Is this CPU's PEXT microcoded? `false` off x86-64.
pub fn host_has_slow_pext() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        let (is_amd, family) = cpu_vendor_family();
        has_slow_pext(is_amd, family)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// Vendor and display family of the running CPU: `(is_amd, family)`.
#[cfg(target_arch = "x86_64")]
fn cpu_vendor_family() -> (bool, u32) {
    use core::arch::x86_64::__cpuid;

    // No `unsafe`: `__cpuid` needs no target feature, since CPUID predates
    // x86-64, so it is a safe function on the pinned toolchain and the unsafe
    // floor does not move.
    let leaf0 = __cpuid(0);
    let leaf1 = __cpuid(1);
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&leaf0.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&leaf0.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&leaf0.ecx.to_le_bytes());
    let eax = leaf1.eax;

    // Display family: the base family, plus the extended field once the base
    // saturates at 0xF. Intel and AMD document the same encoding.
    let base_family = (eax >> 8) & 0xF;
    let family = if base_family == 0xF {
        base_family + ((eax >> 20) & 0xFF)
    } else {
        base_family
    };
    (&vendor == b"AuthenticAMD", family)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_pext_covers_excavator_and_zen1_and_zen2_only() {
        assert!(has_slow_pext(true, 0x15), "Bulldozer to Excavator");
        assert!(has_slow_pext(true, 0x17), "Zen, Zen+ and Zen 2");
        assert!(!has_slow_pext(true, 0x19), "Zen 3 uses the hardware path");
        assert!(!has_slow_pext(true, 0x1A), "Zen 5 uses the hardware path");
        // Intel families collide numerically with AMD's; the vendor gate keeps
        // a Core i7 (family 6, or 0x15/0x17 by accident) out.
        assert!(!has_slow_pext(false, 0x15));
        assert!(!has_slow_pext(false, 0x17));
    }

    #[test]
    fn the_best_tier_follows_the_level() {
        assert_eq!(best_tier(Level::V1, false), Tier::Base);
        assert_eq!(best_tier(Level::V2, false), Tier::Sse42);
        assert_eq!(best_tier(Level::V3, false), Tier::Pext);
        // A v3 CPU whose PEXT is microcoded stops at avx2.
        assert_eq!(best_tier(Level::V3, true), Tier::Avx2);
        // Slow PEXT says nothing about a CPU below v3.
        assert_eq!(best_tier(Level::V2, true), Tier::Sse42);
        assert_eq!(best_tier(Level::V1, true), Tier::Base);
    }

    #[test]
    fn every_best_tier_runs_at_its_level() {
        for level in [Level::V1, Level::V2, Level::V3] {
            for slow in [false, true] {
                assert!(best_tier(level, slow).level() <= level);
            }
        }
    }

    #[test]
    fn tiers_are_ordered_and_named_once() {
        assert!(Tier::ALL.windows(2).all(|pair| pair[0] < pair[1]));
        let names: Vec<&str> = Tier::ALL.iter().map(|tier| tier.name()).collect();
        assert_eq!(names, ["base", "sse42", "avx2", "pext"]);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn cpuid_reports_a_plausible_family_on_this_host() {
        let (_is_amd, family) = cpu_vendor_family();
        // Every x86-64 CPU reports a nonzero display family; 0 would mean the
        // decode is wrong rather than that the CPU is unusual.
        assert!(family > 0, "family decoded as 0");
        assert!(family <= 0xFF + 0xF, "family {family:#x} is out of range");
        assert!(host_level().is_some());
    }
}
