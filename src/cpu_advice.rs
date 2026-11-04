//! Startup advice about which released build this CPU should be running.
//!
//! Two user-visible problems motivate this, both measured:
//!
//! 1. **Too conservative.** `avx2` measured **+4.59%** over `base`
//!    and `pext` at **+2.45%** over `avx2` on an idle 5950X. A user who picks
//!    `base` on a capable CPU gives up real strength for nothing.
//! 2. **Too ambitious.** AMD Excavator (family 15h) and Zen/Zen+/Zen2 (17h)
//!    implement `pdep`/`pext` in microcode, an order of magnitude slower than
//!    the hardware path. On those parts our `pext` build — the one that looks
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
//! Every check here is on the working side of that rule. A tier detects only
//! the features above its own level, which are genuine runtime tests; the
//! features of its own level fold to `true`, correctly, since a CPU running
//! the build has them. "Is this PEXT slow?" is a vendor and family question,
//! and `CPUID` cannot be folded away by `target-feature`, so it works in every
//! tier, including the one that needs it most. The ladder itself is
//! `cpu_tier`, shared with `xtask`.
//!
//! The advice is advice: it prints and continues. It never refuses to run,
//! never changes search, and must never move the bench fingerprint.

use crate::cpu_tier::{self, Tier};

/// Which tier this binary is, decided entirely at compile time.
///
/// `rarog_pext` is checked first because the PEXT build also enables every
/// `avx2` feature; asking about `target_feature` first would misreport it.
/// Off x86-64 the answer is unused: there is no ladder to advise on.
const fn built_tier() -> Tier {
    if cfg!(all(rarog_pext, target_arch = "x86_64")) {
        Tier::Pext
    } else if cfg!(target_feature = "avx2") {
        Tier::Avx2
    } else if cfg!(all(target_feature = "sse4.2", target_feature = "popcnt")) {
        Tier::Sse42
    } else {
        Tier::Base
    }
}

/// Compile-time guard: `built_tier` must agree with what the compiler enabled.
///
/// This is what catches `built_tier` drifting away from `xtask`'s `rustflags` —
/// most plausibly by testing `target_feature = "avx2"` before `rarog_pext`,
/// which would make every PEXT build report itself as `avx2`.
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
        Tier::Sse42 => {
            cfg!(all(target_feature = "sse4.2", target_feature = "popcnt"))
                && !cfg!(target_feature = "avx2")
        }
        Tier::Base => !cfg!(target_feature = "sse4.2") || !cfg!(target_feature = "popcnt"),
    };
    assert!(
        agrees,
        "built_tier() disagrees with the active target features"
    );
};

/// The advisory line, or `None` when this build is already the right one.
fn advice_for(built: Tier, recommended: Tier, slow_pext: bool) -> Option<String> {
    if built == recommended {
        return None;
    }
    let line = match (built, recommended) {
        // The damaging case: the build that looks fastest is the slowest here.
        (Tier::Pext, _) if slow_pext => format!(
            "CPU advisory: this CPU implements PEXT in microcode, which makes the \
             `pext` build the slowest one for it. Use the `{}` build instead.",
            recommended.name()
        ),
        // Leaving measured speed unclaimed. No figure here on purpose: the
        // gap between builds depends on the CPU, so the advisory names the
        // faster build and the README explains the choice without numbers.
        _ => format!(
            "CPU advisory: this CPU supports the `{}` build, which is faster than \
             the `{}` build you are running.",
            recommended.name(),
            built.name()
        ),
    };
    Some(line)
}

/// One line of advice when this CPU would be better served by another build.
///
/// `None` off x86-64: the ARM64 builds have no tier ladder.
pub fn startup_advice() -> Option<String> {
    let level = cpu_tier::host_level()?;
    let slow_pext = cpu_tier::host_has_slow_pext();
    advice_for(
        built_tier(),
        cpu_tier::best_tier(level, slow_pext),
        slow_pext,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_correctly_chosen_build_says_nothing() {
        for tier in Tier::ALL {
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
    fn an_under_tiered_build_names_the_faster_one() {
        let line = advice_for(Tier::Base, Tier::Pext, false).expect("advice expected");
        assert!(line.contains("`pext`"), "{line}");
        assert!(line.contains("`base`"), "{line}");
        // Not the microcode wording: this user's problem is the opposite one.
        assert!(!line.contains("microcode"), "{line}");

        let line = advice_for(Tier::Base, Tier::Sse42, false).expect("advice expected");
        assert!(line.contains("`sse42`"), "{line}");
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
