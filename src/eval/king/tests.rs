use super::super::attacks::AttackMaps;
use super::*;

/// The donor's ring-attacker weights for knight, bishop, rook and queen,
/// which the fixture's weight column was printed with.
const DONOR_WEIGHTS: [i32; 4] = [81, 52, 44, 10];

fn pawn_attacks(board: &Board) -> [Bitboard; 2] {
    let white = board.pieces(Color::White, Piece::Pawn);
    let black = board.pieces(Color::Black, Piece::Pawn);
    [
        white.north_east() | white.north_west(),
        black.south_east() | black.south_west(),
    ]
}

/// Both kings' danger inputs from attack maps filled for `board`.
fn inputs(board: &Board) -> [KingDangerInputs; 2] {
    let atk = &ATTACKS;
    let mut maps = AttackMaps::new();
    maps.fill(board, atk, &pawn_attacks(board));
    let color_occ = [board.color_occ(Color::White), board.color_occ(Color::Black)];
    [Color::White, Color::Black].map(|color| {
        let ks = KsMaps::new(&maps, color, board.occupied(), &color_occ);
        Evaluator::king_danger_inputs(board, atk, color, &ks)
    })
}

/// The fixture's eleven columns for one king, in its order.
fn columns(king: &KingDangerInputs) -> [i32; 11] {
    let a = &king.attackers;
    let weight = a.pieces.iter().zip(DONOR_WEIGHTS).map(|(n, w)| n * w).sum();
    let [knight, bishop, rook, queen] = king.safe_checks;
    [
        a.pawn_attacks + a.pieces.iter().sum::<i32>(),
        weight,
        king.weak_ring,
        king.unsafe_checks,
        king.blockers,
        a.king_attacks,
        rook,
        queen,
        bishop,
        knight,
        king.ring_size,
    ]
}

/// Every king-danger component the donor printed for 500 positions, its
/// piece attacks made plain as Rarog's are
/// (`tools/diag/king_danger_fixture.py`), is reproduced exactly.
#[test]
fn king_danger_inputs_reproduce_the_donor_fixture() {
    let fixture = include_str!("../../../tests/data/king-danger-9587eeeb-plain.tsv");
    let mut lines = fixture.lines();
    let header = lines.next().expect("header");
    assert_eq!(header.split('\t').count(), 23, "fixture columns: {header}");
    let mut rows = 0;
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let board = Board::from_fen(cells[0]).expect("fixture FEN");
        let expected: Vec<i32> = cells[1..]
            .iter()
            .map(|c| c.parse().expect("integer cell"))
            .collect();
        let [white, black] = inputs(&board);
        let actual: Vec<i32> = columns(&white).into_iter().chain(columns(&black)).collect();
        assert_eq!(actual, expected, "{}", cells[0]);
        rows += 1;
    }
    assert_eq!(rows, 500);
}

/// An enemy slider in front of another on the king's line is not the
/// rear one's blocker; a piece in front of both is, once. The fixture
/// holds no such line.
#[test]
fn king_blockers_lift_the_other_sliders_on_the_line() {
    let atk = &ATTACKS;
    let rook_before_queen = Board::from_fen("4q2k/8/8/8/4r3/8/8/4K3 w - - 0 1").unwrap();
    assert_eq!(
        king_blockers(&rook_before_queen, atk, Color::White).count(),
        0
    );
    let knight_before_both = Board::from_fen("4q2k/8/8/4r3/8/8/4N3/4K3 w - - 0 1").unwrap();
    assert_eq!(
        king_blockers(&knight_before_both, atk, Color::White),
        knight_before_both.pieces(Color::White, Piece::Knight)
    );
}

/// A king with every input saturated reads an index far beyond the cap,
/// and its penalty is the cap's, inside the bound the const assertion
/// ties to the decisive band, at any map scale the evaluation accepts.
#[test]
fn a_saturated_king_is_held_at_the_cap() {
    let board = Board::from_fen("6k1/8/8/8/3b4/4nq2/3r4/qr4K1 w - - 0 1").unwrap();
    let [white, _] = inputs(&board);
    let mut ev = Evaluator::default();
    let index = ev.king_danger_index(&white, false, false, 0, 0);
    // A refit of the danger weights moves the fixture's index; the
    // property needs only an index past the cap.
    assert!(
        index > KS_INDEX_CAP,
        "the fixture must saturate the index: {index}"
    );
    let capped = ev.king_danger_penalty(index.min(KS_INDEX_CAP));
    assert_eq!(capped, ev.king_danger_penalty(KS_INDEX_CAP));
    ev.params.ks_map_mg[0] = i32::MAX;
    ev.params.ks_map_eg[0] = i32::MAX;
    let (mg, eg) = ev.king_danger_penalty(KS_INDEX_CAP);
    assert_eq!(mg, KS_MAP_MG_MAX);
    assert!(eg < mg);
}

/// The index sums its inputs at their weights, takes the multiple-check
/// weight from two safe checks of a type, and applies the reductions and
/// the shelter feedback; the fixture compares counts only, so this pins
/// the arithmetic.
#[test]
fn the_index_weighs_each_input() {
    let ev = Evaluator::default();
    let p = &ev.params;
    let inputs = KingDangerInputs {
        attackers: RingAttackers {
            pawn_attacks: 1,
            pieces: [1, 0, 2, 1],
            king_attacks: 3,
        },
        weak_ring: 2,
        unsafe_checks: 1,
        blockers: 1,
        safe_checks: [0, 1, 2, 0],
        ring_size: 9,
    };
    let count = 1 + 1 + 2 + 1;
    let weight = p.kd_attacker_weight[0] + 2 * p.kd_attacker_weight[2] + p.kd_attacker_weight[3];
    let expected = count * weight
        + 2 * p.kd_weak_ring[0]
        + p.kd_unsafe_check[0]
        + p.kd_blockers[0]
        + 3 * p.kd_king_attacks[0]
        + p.kd_mobility[0] * 150 / 100
        - p.kd_shelter[0] * 200 / 100
        + p.kd_constant[0]
        + p.kd_safe_check[2]
        + p.kd_safe_check[5]
        - p.kd_no_queen[0]
        - p.kd_knight_defender[0];
    assert_eq!(
        ev.king_danger_index(&inputs, true, true, 150, 200),
        expected
    );
    let without_reductions = expected + p.kd_no_queen[0] + p.kd_knight_defender[0];
    assert_eq!(
        ev.king_danger_index(&inputs, false, false, 150, 200),
        without_reductions
    );
}

/// The donor's shelter and storm tables in its own units, so the
/// shelter fixture's scores can be compared exactly.
fn donor_shelter_evaluator() -> Evaluator {
    let mut ev = Evaluator::default();
    let p = &mut ev.params;
    p.shelter_strength = [
        -6, 81, 93, 58, 39, 18, 25, -43, 61, 35, -49, -29, -11, -63, -10, 75, 23, -2, 32, 3, -45,
        -39, -13, -29, -52, -48, -67, -166,
    ];
    p.unblocked_storm = [
        85, -289, -166, 97, 50, 45, 50, 46, -25, 122, 45, 37, -10, 20, -6, 51, 168, 34, -2, -22,
        -14, -15, -11, 101, 4, 11, -15, -29,
    ];
    p.blocked_storm_mg = [0, 0, 76, -10, -7, -4, -1];
    p.blocked_storm_eg = [0, 0, 78, 15, 10, 6, 2];
    p.shelter_constant_mg = [5];
    p.shelter_constant_eg = [5];
    ev
}

fn shelter_of(ev: &Evaluator, board: &Board, color: Color) -> (i32, i32) {
    let attacks = pawn_attacks(board);
    let shelter = ev.best_shelter(board, color, attacks[(!color) as usize]);
    (shelter.mg, shelter.eg)
}

/// The donor's shelter and storm score (its units, the castling squares
/// included) and its pawnless-flank flag on the 500 fixture positions,
/// reproduced exactly from its own table values.
#[test]
fn king_shelter_reproduces_the_donor_fixture() {
    let ev = donor_shelter_evaluator();
    let fixture = include_str!("../../../tests/data/king-shelter-9587eeeb.tsv");
    let mut lines = fixture.lines();
    assert_eq!(lines.next().expect("header").split('\t').count(), 7);
    let mut rows = 0;
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let board = Board::from_fen(cells[0]).expect("fixture FEN");
        let expected: Vec<i32> = cells[1..]
            .iter()
            .map(|c| c.parse().expect("integer cell"))
            .collect();
        let mut actual = Vec::new();
        for color in [Color::White, Color::Black] {
            let (mg, eg) = shelter_of(&ev, &board, color);
            actual.extend([mg, eg, i32::from(pawnless_flank(&board, color))]);
        }
        assert_eq!(actual, expected, "{}", cells[0]);
        rows += 1;
    }
    assert_eq!(rows, 500);
}

/// A king that can still castle is credited with the better side's
/// cover; without the right, only its own square counts.
#[test]
fn shelter_takes_the_better_castling_square() {
    let ev = donor_shelter_evaluator();
    let can_castle = Board::from_fen("4k3/8/8/8/8/8/5PPP/4K2R w K - 0 1").unwrap();
    let cannot = Board::from_fen("4k3/8/8/8/8/8/5PPP/4K2R w - - 0 1").unwrap();
    let castled = Board::from_fen("4k3/8/8/8/8/8/5PPP/5RK1 w - - 0 1").unwrap();
    assert_eq!(
        shelter_of(&ev, &can_castle, Color::White),
        shelter_of(&ev, &castled, Color::White)
    );
    assert!(shelter_of(&ev, &cannot, Color::White).0 < shelter_of(&ev, &castled, Color::White).0);
}

/// The files are centred on the king's file kept within b–g, so a king
/// on h reads f, g and h like a king on g; a pawn on our pawn is a
/// blocked storm.
#[test]
fn shelter_files_clamp_and_block() {
    let board = Board::from_fen("4k3/8/8/8/8/6p1/6P1/7K w - - 0 1").unwrap();
    let attacks = pawn_attacks(&board);
    let on_h = shelter_files(&board, Color::White, Square(7), attacks[1]);
    let on_g = shelter_files(&board, Color::White, Square(6), attacks[1]);
    assert_eq!(on_h, on_g);
    assert_eq!(on_h.map(|f| f.edge), [2, 1, 0]);
    let g_file = on_h[1];
    assert_eq!((g_file.ours, g_file.theirs), (1, 2));
    assert!(g_file.blocked());
}

/// The pawn entry keeps each king's shelter with the square and rights
/// it was computed for: the same pawns with the king on two squares
/// evaluate as a fresh evaluator does, in either order. The queens give
/// the position a middlegame phase, where the shelters differ.
#[cfg(not(feature = "texel"))]
#[test]
fn the_pawn_entry_recomputes_the_shelter_for_a_moved_king() {
    let on_g1 = Board::from_fen("3q2k1/5ppp/8/8/8/8/5PPP/3Q2K1 w - - 0 1").unwrap();
    let on_e1 = Board::from_fen("3q2k1/5ppp/8/8/8/8/5PPP/3QK2R w - - 0 1").unwrap();
    let on_e1_castling = Board::from_fen("3q2k1/5ppp/8/8/8/8/5PPP/3QK2R w K - 0 1").unwrap();
    assert_eq!(on_g1.pawn_key(), on_e1.pawn_key());
    assert_eq!(on_e1.pawn_key(), on_e1_castling.pawn_key());
    let fresh = |board: &Board| {
        let mut ev = Evaluator::default();
        ev.set_lazy_margin(i32::MAX);
        ev.evaluate(board)
    };
    let mut ev = Evaluator::default();
    ev.set_lazy_margin(i32::MAX);
    for board in [
        &on_g1,
        &on_e1,
        &on_e1_castling,
        &on_e1,
        &on_g1,
        &on_e1_castling,
    ] {
        assert_eq!(ev.evaluate(board), fresh(board));
    }
    let shelter = |board: &Board| {
        let attacks = pawn_attacks(board);
        Evaluator::default()
            .best_shelter(board, Color::White, attacks[1])
            .mg
    };
    assert_ne!(shelter(&on_g1), shelter(&on_e1));
    assert_ne!(shelter(&on_e1), shelter(&on_e1_castling));
}

#[cfg(feature = "tune")]
use super::super::masks::{FamilyMasks, MaskCell, MaskedFamily};

#[cfg(feature = "tune")]
const UNMASKED: FamilyMasks = FamilyMasks {
    king_danger: false,
    king_shelter: false,
};
#[cfg(feature = "tune")]
const DANGER_MASKED: FamilyMasks = FamilyMasks {
    king_danger: true,
    king_shelter: false,
};
#[cfg(feature = "tune")]
const SHELTER_MASKED: FamilyMasks = FamilyMasks {
    king_danger: false,
    king_shelter: true,
};

/// An evaluator with the lazy gate out of reach, its own masks and its
/// parameters edited by `edit`.
#[cfg(feature = "tune")]
fn masked_evaluator(masks: FamilyMasks, edit: impl Fn(&mut super::super::EvalParams)) -> Evaluator {
    let cell: &'static MaskCell = Box::leak(Box::new(MaskCell::new()));
    cell.set(MaskedFamily::KingDanger, masks.king_danger)
        .unwrap();
    cell.set(MaskedFamily::KingShelter, masks.king_shelter)
        .unwrap();
    let mut ev = Evaluator::default();
    ev.set_lazy_margin(i32::MAX);
    ev.mask_source = cell;
    edit(&mut ev.params);
    ev
}

/// `KingDangerMask` evaluates as the map at zero scale, and
/// `KingShelterMask` as zero shelter and storm tables, constants and
/// feedback, over random playouts; each changes many of those positions.
#[cfg(feature = "tune")]
#[test]
fn each_mask_evaluates_as_its_family_at_zero_weight() {
    let mut plain = masked_evaluator(UNMASKED, |_| {});
    let mut danger_masked = masked_evaluator(DANGER_MASKED, |_| {});
    let mut danger_zero = masked_evaluator(UNMASKED, |p| {
        p.ks_map_mg = [0];
        p.ks_map_eg = [0];
    });
    let mut shelter_masked = masked_evaluator(SHELTER_MASKED, |_| {});
    let mut shelter_zero = masked_evaluator(UNMASKED, |p| {
        p.shelter_strength = [0; 28];
        p.unblocked_storm = [0; 28];
        p.blocked_storm_mg = [0; 7];
        p.blocked_storm_eg = [0; 7];
        p.shelter_constant_mg = [0];
        p.shelter_constant_eg = [0];
        p.kd_shelter = [0];
    });

    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let (mut checked, mut danger_moved, mut shelter_moved) = (0, 0, 0);
    for _ in 0..200 {
        let mut board = Board::starting_position();
        for _ in 0..60 {
            let base = plain.evaluate(&board);
            let danger = danger_masked.evaluate(&board);
            let shelter = shelter_masked.evaluate(&board);
            assert_eq!(danger, danger_zero.evaluate(&board), "{}", board.to_fen());
            assert_eq!(shelter, shelter_zero.evaluate(&board), "{}", board.to_fen());
            checked += 1;
            danger_moved += usize::from(danger != base);
            shelter_moved += usize::from(shelter != base);

            let moves = board.generate_legal_movelist();
            if moves.is_empty() {
                break;
            }
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            board.make_move(moves.as_slice()[infra::index(state) % moves.len()]);
        }
    }
    assert!(checked > 5000, "only {checked} positions");
    assert!(
        danger_moved * 10 > checked,
        "the danger mask moved {danger_moved}"
    );
    assert!(
        shelter_moved * 2 > checked,
        "the shelter mask moved {shelter_moved}"
    );
}

/// The masks change a middlegame evaluation and leave a pawn ending, where
/// the king family is idle, as it was.
#[cfg(feature = "tune")]
#[test]
fn the_masks_reach_the_middlegame_and_not_a_pawn_ending() {
    let middlegame =
        Board::from_fen("r1bq1rk1/pppp1ppp/2n2n2/2b1p1N1/2B1P2Q/8/PPPP1PPP/RNB1K2R w KQ - 0 1")
            .unwrap();
    let pawn_ending = Board::from_fen("8/5pk1/6p1/8/8/6P1/5PK1/8 w - - 0 1").unwrap();
    let plain = |board: &Board| masked_evaluator(UNMASKED, |_| {}).evaluate(board);
    for masks in [DANGER_MASKED, SHELTER_MASKED] {
        let masked = |board: &Board| masked_evaluator(masks, |_| {}).evaluate(board);
        assert_ne!(masked(&middlegame), plain(&middlegame), "{masks:?}");
        assert_eq!(masked(&pawn_ending), plain(&pawn_ending), "{masks:?}");
    }
}
