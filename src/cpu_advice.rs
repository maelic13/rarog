//! Startup advice about which released asset this CPU should be running.
//!
//! A.4.2. Two user-visible problems motivate this, both measured:
//!
//! 1. **Too conservative.** RAR-P20 measured `avx2` at **+4.59%** over `base`
//!    and `pext` at **+2.45%** over `avx2` on an idle 5950X. A user who picks
//!    `base` on a capable CPU gives up real strength for nothing.
//! 2. **Too ambitious.** AMD Excavator (family 15h) and Zen/Zen+/Zen2 (17h)
//!    implement `pdep`/`pext` in microcode, an order of magnitude slower than
//!    the hardware path. On those parts our `pext` asset — the one that looks
//!    like the fast choice — is our *slowest* engine, and nothing said so.
//!
//! **Why the obvious implementation does not work, and what this does instead.**
//! `src/main.rs` records that the previous startup CPU guard never once fired in
//! a shipped binary: `is_x86_feature_detected!` expands to
//! `cfg!(target_feature = "...") || runtime_detect(...)`, so in a tier that
//! *statically requires* the feature the macro folds to a compile-time `true`,
//! the branch becomes dead code and the message string is stripped from the
//! artifact. A runtime check for a feature the build statically requires is
//! `true` by construction.
//!
//! Every check below is deliberately on the working side of that rule:
//!
//! * `base` enables neither `avx2` nor `bmi2`, so detecting them there is a
//!   genuine runtime test.
//! * `avx2` and `pext` both enable `bmi2` statically — so they never ask. They
//!   do not need to: `x86-64-v3` requires BMI2, so a CPU running those assets
//!   has it by construction.
//! * "Is this BMI2 slow?" is a vendor/family question, not a feature question.
//!   `CPUID` cannot be folded away by `target-feature`, so it works in every
//!   tier — including the one that needs it most.
//!
//! The advice is advice: it prints and continues. It never refuses to run,
//! never changes search, and must never move the bench fingerprint.

/// The ISA tier this binary was compiled for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Portable `x86-64` baseline.
    Base,
    /// `x86-64-v3`: AVX2, BMI1, BMI2, POPCNT, LZCNT.
    Avx2,
    /// `x86-64-v3` plus the PEXT slider path (`--cfg rarog_pext`).
    Pext,
}

impl Tier {
    /// The asset-name fragment users actually see on the release page.
    pub const fn asset(self) -> &'static str {
        match self {
            Tier::Base => "x86-64",
            Tier::Avx2 => "avx2",
            Tier::Pext => "pext",
        }
    }
}

/// Which tier this binary is, decided entirely at compile time.
///
/// `rarog_pext` is checked first because the PEXT build also enables every
/// `avx2` feature; asking about `target_feature` first would misreport it.
pub const fn built_tier() -> Tier {
    if cfg!(all(rarog_pext, target_arch = "x86_64")) {
        Tier::Pext
    } else if cfg!(target_feature = "avx2") {
        Tier::Avx2
    } else {
        Tier::Base
    }
}

/// Compile-time guard: `built_tier` must agree with what the compiler enabled.
///
/// This is what catches `built_tier` drifting away from `xtask`'s `rustflags` —
/// most plausibly by testing `target_feature = "avx2"` before `rarog_pext`,
/// which would make every PEXT asset report itself as `avx2`.
///
/// It is a `const` item rather than a `#[test]` so that a mismatch fails the
/// **build**, in whatever configuration is being built, instead of waiting for
/// a test run in one configuration. It is deliberately not a `const` block
/// inside a `match` arm, which is the shape clippy's `assertions_on_constants`
/// hint suggests: those are evaluated even in arms that are not selected, so the
/// `Pext` assertion would fail every non-PEXT build. Here only the selected arm
/// is const-evaluated, which is precisely what makes the guard expressible.
const _: () = {
    let agrees = match built_tier() {
        Tier::Pext => cfg!(all(rarog_pext, target_arch = "x86_64")),
        Tier::Avx2 => cfg!(target_feature = "avx2") && !cfg!(rarog_pext),
        Tier::Base => !cfg!(target_feature = "avx2"),
    };
    assert!(
        agrees,
        "built_tier() disagrees with the active target features"
    );
};

/// The tier this CPU should be running, from what it supports.
///
/// Pure so it can be tested across CPUs this machine is not.
const fn recommended_tier(has_avx2: bool, has_bmi2: bool, slow_pext: bool) -> Tier {
    if !has_avx2 {
        Tier::Base
    } else if !has_bmi2 || slow_pext {
        Tier::Avx2
    } else {
        Tier::Pext
    }
}

/// Does this vendor/family implement `pdep`/`pext` in microcode?
///
/// AMD family 15h (Bulldozer through Excavator) and 17h (Zen, Zen+, Zen2).
/// Family 19h (Zen 3) onward is the hardware path and is fine. Intel has been
/// fast since Haswell. Stockfish carries exactly one model-based exception in
/// its entire source and it is this one — everything else there, all eight
/// tiers and seventeen feature bits, is pure feature testing. A flag can say an
/// instruction is legal; it cannot say it is fast.
const fn has_slow_pext(is_amd: bool, family: u32) -> bool {
    is_amd && (family == 0x15 || family == 0x17)
}

/// The advisory line, or `None` when this asset is already the right one.
fn advice_for(built: Tier, recommended: Tier, slow_pext: bool) -> Option<String> {
    if built == recommended {
        return None;
    }
    let line = match (built, recommended) {
        // The damaging case: the asset that looks fastest is the slowest here.
        (Tier::Pext, _) if slow_pext => format!(
            "CPU advisory: this CPU implements PEXT in microcode, which makes the \
             `pext` build the slowest one for it. Use the `{}` build instead.",
            recommended.asset()
        ),
        // Leaving measured speed unclaimed. No figure here on purpose: the
        // README (A.4.4) owns the numbers, so they can be revised without
        // rebuilding the engine, and base-to-pext is not directly measured yet.
        _ => format!(
            "CPU advisory: this CPU supports the `{}` build, which is faster than \
             the `{}` build you are running.",
            recommended.asset(),
            built.asset()
        ),
    };
    Some(line)
}

/// Vendor and family for the running CPU.
///
/// Returns `(is_amd, family)`.
#[cfg(target_arch = "x86_64")]
fn cpu_vendor_family() -> (bool, u32) {
    use core::arch::x86_64::__cpuid;

    // No `unsafe` here, deliberately, and it is worth saying why rather than
    // leaving a reader to wonder whether one was forgotten: `__cpuid` requires
    // no `target_feature` — CPUID predates x86-64 and is unconditionally
    // present — so on the pinned toolchain it is a safe function. The unsafe
    // floor (PLAN principle #8) is unchanged by this module.
    let leaf0 = __cpuid(0);
    let leaf1 = __cpuid(1);
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&leaf0.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&leaf0.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&leaf0.ecx.to_le_bytes());
    let eax = leaf1.eax;

    // Display family = base family, plus the extended field once base saturates
    // at 0xF. Intel and AMD document the same encoding.
    let base_family = (eax >> 8) & 0xF;
    let family = if base_family == 0xF {
        base_family + ((eax >> 20) & 0xFF)
    } else {
        base_family
    };
    (&vendor == b"AuthenticAMD", family)
}

/// One line of advice when this CPU would be better served by another asset.
///
/// `None` on every non-x86-64 target: the ARM64 assets have no tier ladder.
pub fn startup_advice() -> Option<String> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        None
    }
    #[cfg(target_arch = "x86_64")]
    {
        let (is_amd, family) = cpu_vendor_family();
        let slow_pext = has_slow_pext(is_amd, family);
        // Genuine runtime detection in `base`, where neither feature is static.
        // In `avx2`/`pext` these fold to `true`, which is correct rather than
        // broken: `x86-64-v3` requires both, so a CPU running those assets has
        // them. Nothing downstream depends on distinguishing the two cases.
        let has_avx2 = std::arch::is_x86_feature_detected!("avx2");
        let has_bmi2 = std::arch::is_x86_feature_detected!("bmi2");
        advice_for(
            built_tier(),
            recommended_tier(has_avx2, has_bmi2, slow_pext),
            slow_pext,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_pext_covers_excavator_and_zen1_and_zen2_only() {
        assert!(
            has_slow_pext(true, 0x15),
            "Bulldozer/Excavator is microcoded"
        );
        assert!(
            has_slow_pext(true, 0x17),
            "Zen, Zen+ and Zen2 are microcoded"
        );
        assert!(!has_slow_pext(true, 0x19), "Zen 3 uses the hardware path");
        assert!(!has_slow_pext(true, 0x1A), "Zen 5 uses the hardware path");
        // Intel families collide numerically with AMD's; the vendor gate is
        // what keeps a Core i7 (family 6) out of the microcoded set.
        assert!(!has_slow_pext(false, 0x15));
        assert!(!has_slow_pext(false, 0x17));
    }

    #[test]
    fn recommendation_follows_capability() {
        assert_eq!(recommended_tier(false, false, false), Tier::Base);
        assert_eq!(recommended_tier(false, true, false), Tier::Base);
        assert_eq!(recommended_tier(true, false, false), Tier::Avx2);
        assert_eq!(recommended_tier(true, true, false), Tier::Pext);
        // A capable CPU whose PEXT is microcoded stops at avx2.
        assert_eq!(recommended_tier(true, true, true), Tier::Avx2);
    }

    #[test]
    fn a_correctly_chosen_asset_says_nothing() {
        for tier in [Tier::Base, Tier::Avx2, Tier::Pext] {
            assert_eq!(advice_for(tier, tier, false), None);
            assert_eq!(advice_for(tier, tier, true), None);
        }
    }

    #[test]
    fn the_microcoded_pext_case_names_pext_as_the_problem() {
        let line = advice_for(Tier::Pext, Tier::Avx2, true).expect("advice expected");
        assert!(line.contains("microcode"), "{line}");
        assert!(line.contains("`avx2`"), "{line}");
    }

    #[test]
    fn an_under_tiered_asset_names_the_faster_one() {
        let line = advice_for(Tier::Base, Tier::Pext, false).expect("advice expected");
        assert!(line.contains("`pext`"), "{line}");
        assert!(line.contains("`x86-64`"), "{line}");
        // Not the microcode wording: this user's problem is the opposite one.
        assert!(!line.contains("microcode"), "{line}");
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn cpuid_reports_a_plausible_family_on_this_host() {
        let (_is_amd, family) = cpu_vendor_family();
        // Every x86-64 CPU reports a nonzero display family; 0 would mean the
        // decode is wrong rather than that the CPU is unusual.
        assert!(family > 0, "family decoded as 0");
        assert!(family <= 0xFF + 0xF, "family {family:#x} is out of range");
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn startup_advice_is_silent_or_actionable_but_never_empty() {
        if let Some(line) = startup_advice() {
            assert!(line.starts_with("CPU advisory: "), "{line}");
            assert!(line.contains("build"), "{line}");
        }
    }
}
