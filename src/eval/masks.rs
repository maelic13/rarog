//! Family masks for attribution reads inside an evaluation unit (`tune`
//! builds only, never a release): `KingDangerMask` zeroes the king-danger
//! map's output and `KingShelterMask` zeroes shelter and storm together with
//! their feedback into the danger index. They serve tree reads and fixed
//! game reads, never gates.
//!
//! The masks are process-wide and become fixed at the first evaluation: the
//! evaluation cache and the transposition table hold values of the function
//! in force when they were stored, so a mask that changed afterwards would
//! mix two functions in one search. The pawn cache keeps the unmasked
//! shelter; the mask applies where the score is used.

use std::sync::atomic::{AtomicU8, Ordering};

const KING_DANGER: u8 = 1;
const KING_SHELTER: u8 = 2;
const FIXED: u8 = 0x80;

/// Which king-safety families an evaluation leaves out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FamilyMasks {
    pub king_danger: bool,
    pub king_shelter: bool,
}

impl FamilyMasks {
    fn from_bits(bits: u8) -> Self {
        Self {
            king_danger: bits & KING_DANGER != 0,
            king_shelter: bits & KING_SHELTER != 0,
        }
    }
}

/// One maskable family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskedFamily {
    KingDanger,
    KingShelter,
}

impl MaskedFamily {
    const fn bit(self) -> u8 {
        match self {
            Self::KingDanger => KING_DANGER,
            Self::KingShelter => KING_SHELTER,
        }
    }

    /// The UCI option that sets this mask.
    pub const fn option_name(self) -> &'static str {
        match self {
            Self::KingDanger => "KingDangerMask",
            Self::KingShelter => "KingShelterMask",
        }
    }
}

/// A mask change refused because an evaluation has already read the masks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MasksFixed;

/// The masks, and whether an evaluation has fixed them, in one atomic so a
/// change and the first read are ordered against each other.
pub struct MaskCell {
    bits: AtomicU8,
}

impl MaskCell {
    pub const fn new() -> Self {
        Self {
            bits: AtomicU8::new(0),
        }
    }

    /// Mask or unmask one family. Setting the value already in force is
    /// always accepted; changing it after the first evaluation is refused.
    pub fn set(&self, family: MaskedFamily, on: bool) -> Result<(), MasksFixed> {
        let mut current = self.bits.load(Ordering::Relaxed);
        loop {
            let wanted = if on {
                current | family.bit()
            } else {
                current & !family.bit()
            };
            if wanted == current {
                return Ok(());
            }
            if current & FIXED != 0 {
                return Err(MasksFixed);
            }
            match self.bits.compare_exchange_weak(
                current,
                wanted,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(seen) => current = seen,
            }
        }
    }

    /// The masks in force, fixed from this call on.
    pub fn fix(&self) -> FamilyMasks {
        let bits = self.bits.load(Ordering::Relaxed);
        if bits & FIXED != 0 {
            return FamilyMasks::from_bits(bits);
        }
        FamilyMasks::from_bits(self.bits.fetch_or(FIXED, Ordering::Relaxed))
    }
}

/// The masks the UCI options set and every evaluator reads.
pub static FAMILY_MASKS: MaskCell = MaskCell::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mask_is_set_before_the_first_evaluation_and_fixed_after_it() {
        let cell = MaskCell::new();
        assert_eq!(cell.set(MaskedFamily::KingShelter, true), Ok(()));
        assert_eq!(cell.set(MaskedFamily::KingShelter, false), Ok(()));
        assert_eq!(cell.set(MaskedFamily::KingDanger, true), Ok(()));
        let fixed = cell.fix();
        assert_eq!(
            fixed,
            FamilyMasks {
                king_danger: true,
                king_shelter: false,
            }
        );
        assert_eq!(cell.set(MaskedFamily::KingDanger, false), Err(MasksFixed));
        assert_eq!(cell.set(MaskedFamily::KingShelter, true), Err(MasksFixed));
        // Repeating the value in force is no change.
        assert_eq!(cell.set(MaskedFamily::KingDanger, true), Ok(()));
        assert_eq!(cell.set(MaskedFamily::KingShelter, false), Ok(()));
        assert_eq!(cell.fix(), fixed);
    }
}
