//! Checked numeric conversions (Phase 9.0b).
//!
//! Rarog is 64-bit-only (compile-guarded in `lib.rs`/`main.rs`), and its
//! quantities are domain-bounded: plies ≤ 128, depths ≤ 100, move counts
//! ≤ 256, piece counts ≤ 10, squares < 64. The ~240 bare `as` casts this
//! module replaces were each individually harmless, but nothing *checked*
//! that, and cast #241 would have been on its own. Every narrowing in the
//! crate now goes through one of these functions: the conversion is named,
//! `debug_assert!`ed (exercised — the debug suite runs since 9.0a revived
//! it), and the only `as` casts live in this one annotated block.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

/// `u64 → usize` for hash/key indexing. Lossless: the crate compiles only on
/// 64-bit targets, so `usize` is exactly `u64`-wide.
#[inline(always)]
pub fn index(x: u64) -> usize {
    x as usize
}

/// Domain-bounded narrowing to `i32` (plies, depths, counts, bit indices).
#[inline(always)]
pub fn to_i32<T: SmallInt>(x: T) -> i32 {
    x.to_i32()
}

/// Non-negative, domain-bounded `i32 → usize` (table indices).
#[inline(always)]
pub fn to_usize(x: i32) -> usize {
    debug_assert!(x >= 0, "negative value used as an index: {x}");
    x as usize
}

/// Domain-bounded narrowing to `u8` (squares, files, ranks).
#[inline(always)]
pub fn to_u8<T: SmallInt>(x: T) -> u8 {
    let v = x.to_i32();
    debug_assert!((0..=255).contains(&v), "value out of u8 range: {v}");
    v as u8
}

/// Integers that are small by domain. Each impl narrows through `i32` with a
/// debug-time range check; implement it only for source types that actually
/// appear in the engine.
pub trait SmallInt: Copy {
    fn to_i32(self) -> i32;
}

impl SmallInt for usize {
    #[inline(always)]
    fn to_i32(self) -> i32 {
        debug_assert!(self <= i32::MAX as usize, "usize too large for i32: {self}");
        self as i32
    }
}

impl SmallInt for u64 {
    #[inline(always)]
    fn to_i32(self) -> i32 {
        debug_assert!(self <= i32::MAX as u64, "u64 too large for i32: {self}");
        self as i32
    }
}

impl SmallInt for u32 {
    #[inline(always)]
    fn to_i32(self) -> i32 {
        debug_assert!(self <= i32::MAX as u32, "u32 too large for i32: {self}");
        self as i32
    }
}

impl SmallInt for i32 {
    #[inline(always)]
    fn to_i32(self) -> i32 {
        self
    }
}

impl SmallInt for i16 {
    #[inline(always)]
    fn to_i32(self) -> i32 {
        i32::from(self)
    }
}

impl SmallInt for i8 {
    #[inline(always)]
    fn to_i32(self) -> i32 {
        i32::from(self)
    }
}

impl SmallInt for u8 {
    #[inline(always)]
    fn to_i32(self) -> i32 {
        i32::from(self)
    }
}

/// Narrows an `i32` to `i16`, saturating at the bounds.
///
/// 9.0b: replaces the `v.clamp(i16::MIN as i32, i16::MAX as i32) as i16`
/// idiom that appeared verbatim in four hot places (SEE scoring, history
/// updates, TT score and static-eval packing). Expressed with `try_from` so
/// there is **no cast at all** — the saturation is explicit and the compiler
/// Clamp an `i64` into `i32` range.
///
/// The texel tuner's `reconstruct` accumulates `weight * count` products in
/// `i64` to keep the tapered sum exact, then hands back an ordinary eval
/// score. Real positions land nowhere near the boundary, but saturating is
/// the honest narrowing: a runaway weight during a fit should peg the score,
/// not wrap it to the opposite sign and silently teach the tuner nonsense.
#[inline(always)]
pub fn saturating_i32(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
}

/// still emits the same compare-and-select.
#[inline(always)]
pub fn saturating_i16(value: i32) -> i16 {
    i16::try_from(value).unwrap_or(if value < 0 { i16::MIN } else { i16::MAX })
}

/// Narrows an `i32` to `i8`, saturating at `lo`/`i8::MAX`.
///
/// 9.0b: used for TT depth packing, where `-1` is the meaningful floor.
#[inline(always)]
pub fn saturating_i8(value: i32, lo: i8) -> i8 {
    i8::try_from(value)
        .unwrap_or(if value < 0 { lo } else { i8::MAX })
        .max(lo)
}

pub fn capitalize_first_letter(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod narrow_tests {
    use super::{saturating_i8, saturating_i16, saturating_i32};

    #[test]
    fn saturating_i32_clamps_both_ends() {
        assert_eq!(saturating_i32(0), 0);
        assert_eq!(saturating_i32(i64::from(i32::MAX)), i32::MAX);
        assert_eq!(saturating_i32(i64::from(i32::MIN)), i32::MIN);
        assert_eq!(saturating_i32(i64::from(i32::MAX) + 1), i32::MAX);
        assert_eq!(saturating_i32(i64::from(i32::MIN) - 1), i32::MIN);
        assert_eq!(saturating_i32(i64::MAX), i32::MAX);
        assert_eq!(saturating_i32(i64::MIN), i32::MIN);
    }

    #[test]
    fn saturating_i16_matches_clamp_then_truncate() {
        // The idiom it replaced, checked across the boundaries and beyond.
        for v in [
            0,
            1,
            -1,
            i16::MAX as i32,
            i16::MIN as i32,
            i16::MAX as i32 + 1,
            i16::MIN as i32 - 1,
            i32::MAX,
            i32::MIN,
            100_000,
            -100_000,
        ] {
            let expected = v.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            assert_eq!(saturating_i16(v), expected, "value {v}");
        }
    }

    #[test]
    fn saturating_i8_respects_floor_and_ceiling() {
        assert_eq!(saturating_i8(5, -1), 5);
        assert_eq!(saturating_i8(-1, -1), -1);
        assert_eq!(
            saturating_i8(-50, -1),
            -1,
            "below the floor saturates to it"
        );
        assert_eq!(saturating_i8(i32::MAX, -1), i8::MAX);
        assert_eq!(saturating_i8(i8::MAX as i32, -1), i8::MAX);
    }
}
