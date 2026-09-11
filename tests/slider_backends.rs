//! Coordinate-ray oracle for both slider lookup backends.
//!
//! The same integration test compiles against the magic default and the PEXT
//! configuration (`--cfg rarog_pext` on x86_64). It enumerates every relevant
//! blocker subset for every square, so table construction, index extraction
//! and lookup are checked against a coordinate implementation outside the
//! magic/PEXT machinery.

use rarog::board::{ATTACKS, Bitboard, Square};

#[test]
fn every_slider_occupancy_matches_coordinate_rays() {
    let attacks = &*ATTACKS;
    let backend = if cfg!(all(rarog_pext, target_arch = "x86_64")) {
        "pext"
    } else {
        "magic"
    };

    for square in 0..64u8 {
        let sq = Square(square);
        for (name, directions, actual) in [
            ("rook", &ORTHOGONAL[..], attacks.rook(sq, Bitboard::EMPTY)),
            ("bishop", &DIAGONAL[..], attacks.bishop(sq, Bitboard::EMPTY)),
        ] {
            assert_eq!(
                actual.0,
                coordinate_rays(sq, 0, directions),
                "{backend} {name} empty {sq}"
            );
            let relevant = relevant_mask(sq, directions);
            let mut subset = relevant;
            loop {
                let expected = coordinate_rays(sq, subset, directions);
                let got = if name == "rook" {
                    attacks.rook(sq, Bitboard(subset)).0
                } else {
                    attacks.bishop(sq, Bitboard(subset)).0
                };
                assert_eq!(
                    got, expected,
                    "{backend} {name} square {sq} occupancy {subset:#018x}"
                );
                if subset == 0 {
                    break;
                }
                subset = (subset - 1) & relevant;
            }
        }
    }
}

const ORTHOGONAL: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const DIAGONAL: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

fn relevant_mask(square: Square, directions: &[(i8, i8)]) -> u64 {
    let rank = i8::try_from(square.0 / 8).expect("rank is in 0..8");
    let file = i8::try_from(square.0 % 8).expect("file is in 0..8");
    let mut mask = 0;
    for &(dr, df) in directions {
        let mut r = rank + dr;
        let mut f = file + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let next_r = r + dr;
            let next_f = f + df;
            if !(0..8).contains(&next_r) || !(0..8).contains(&next_f) {
                break;
            }
            mask |= 1u64 << (r * 8 + f);
            r = next_r;
            f = next_f;
        }
    }
    mask
}

fn coordinate_rays(square: Square, occupancy: u64, directions: &[(i8, i8)]) -> u64 {
    let rank = i8::try_from(square.0 / 8).expect("rank is in 0..8");
    let file = i8::try_from(square.0 % 8).expect("file is in 0..8");
    let mut attacks = 0;
    for &(dr, df) in directions {
        let mut r = rank + dr;
        let mut f = file + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if occupancy & bit != 0 {
                break;
            }
            r += dr;
            f += df;
        }
    }
    attacks
}
