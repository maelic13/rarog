use super::*;
use crate::eval::Evaluator;

fn board(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("bad test FEN {fen}: {e}"))
}

fn static_eval(fen: &str) -> i32 {
    Evaluator::default().evaluate(&board(fen))
}

/// The same position with the colours swapped: ranks reversed, piece
/// letters' case swapped, the other side to move, castling rights swapped
/// and an en-passant square moved to the other side's third rank.
fn mirror(fen: &str) -> String {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    let swap_case = |c: char| {
        if c.is_ascii_uppercase() {
            c.to_ascii_lowercase()
        } else {
            c.to_ascii_uppercase()
        }
    };
    let placement = fields[0]
        .split('/')
        .rev()
        .map(|rank| rank.chars().map(swap_case).collect::<String>())
        .collect::<Vec<_>>()
        .join("/");
    let side = if fields[1] == "w" { "b" } else { "w" };
    let castling = if fields[2] == "-" {
        String::from("-")
    } else {
        let swapped: String = fields[2].chars().map(swap_case).collect();
        "KQkq".chars().filter(|c| swapped.contains(*c)).collect()
    };
    let ep = match fields[3].as_bytes() {
        [file, rank] => format!("{}{}", char::from(*file), char::from(b'9' - rank + b'0')),
        _ => String::from("-"),
    };
    format!(
        "{placement} {side} {castling} {ep} {} {}",
        fields[4], fields[5]
    )
}

/// Every position these tests use, each with White the stronger side.
const RECOGNISED: [&str; 40] = [
    "k7/4p3/4K3/8/2N5/N7/8/8 w - - 0 1",
    "K7/8/n7/2n5/4k3/4P3/8/8 b - - 0 1",
    "k7/4p3/4K3/8/2N5/N3P3/8/8 w - - 0 1",
    "k7/4p1p1/4K3/8/2N5/N7/8/8 w - - 0 1",
    "k7/4p3/4K3/8/2N5/8/8/8 w - - 0 1",
    "k7/8/4K3/8/2N5/N7/8/8 w - - 0 1",
    "8/8/n7/2n5/8/4k3/4P3/K7 b - - 0 1",
    "k7/8/4K3/4p3/4N3/N7/8/8 w - - 0 1",
    "8/8/n7/4n3/4P3/4k3/8/K7 b - - 0 1",
    "k7/8/4K3/4p3/2N5/N7/8/8 w - - 0 1",
    "8/8/4K3/4p3/2Nk4/N7/8/8 w - - 0 1",
    "k7/8/4K3/8/2N5/N3p3/8/8 w - - 0 1",
    "k7/8/4K3/4p3/8/N1N5/8/8 w - - 0 1",
    "k7/8/4K3/4p3/8/N7/N7/8 w - - 0 1",
    "8/8/6K1/8/8/8/1kp5/7Q w - - 0 1",
    "8/8/6K1/8/8/8/kp6/7Q w - - 0 1",
    "8/8/6K1/8/8/8/1kp5/7R w - - 0 1",
    "8/1kb5/8/P7/8/8/8/6KR w - - 0 1",
    "8/2kb4/8/1P6/8/8/8/6KR w - - 0 1",
    "8/3k4/r7/3PK3/8/8/8/7R w - - 0 1",
    "R7/P6k/8/8/8/8/8/r5K1 w - - 0 1",
    "1R6/1P6/8/8/8/7k/r7/6K1 w - - 0 1",
    "8/8/1PK5/8/8/7k/r7/1R6 w - - 0 1",
    "8/8/8/1PK5/8/7k/r7/1R6 w - - 0 1",
    "8/8/6KQ/8/3k4/8/8/3r4 w - - 0 1",
    "4k3/p7/8/3b4/8/8/P7/2B1K3 w - - 0 1",
    "4k3/7p/P7/3b4/8/8/8/2B1K3 w - - 0 1",
    "4k3/p7/8/3b4/8/8/PP6/2B1K3 w - - 0 1",
    "4k3/7p/P7/3b4/8/8/P7/2B1K3 w - - 0 1",
    "r3k3/p7/8/3b4/8/8/PP6/2B1K2R w - - 0 1",
    "8/8/4k3/8/8/4K3/8/8 w - - 0 1",
    "8/8/4k3/8/8/4K3/8/5N2 w - - 0 1",
    "8/8/4k3/8/3n4/4K3/8/5B2 w - - 0 1",
    "8/8/4k3/8/8/4K3/8/4NN2 w - - 0 1",
    "7k/8/6KP/8/8/8/8/5B2 w - - 0 1",
    "7k/8/8/8/8/8/7P/7K w - - 0 1",
    "8/1rk5/8/8/8/8/3P1K1R/8 w - - 0 1",
    "4k3/8/4K3/4P3/8/8/8/8 w - - 0 1",
    "8/8/8/4k3/8/8/8/2B1KN2 w - - 0 1",
    "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
];

/// Each recogniser reads a position the same with the colours swapped:
/// the same scale factor, and the same evaluation for the side to move.
#[test]
fn every_recogniser_position_reads_the_same_with_the_colours_swapped() {
    for fen in RECOGNISED {
        let mirrored = mirror(fen);
        let (ours, theirs) = (board(fen), board(&mirrored));
        assert_eq!(
            specialized_endgame_scale(&ours),
            specialized_endgame_scale(&theirs),
            "{fen} against {mirrored}"
        );
        assert_eq!(
            opposite_bishop_scale(&ours),
            opposite_bishop_scale(&theirs),
            "{fen} against {mirrored}"
        );
        assert_eq!(
            static_eval(fen),
            static_eval(&mirrored),
            "{fen} against {mirrored}"
        );
    }
}

#[test]
fn knnkp_fires_only_on_exact_material() {
    assert!(knnkp_adjustment(&board("k7/4p3/4K3/8/2N5/N7/8/8 w - - 0 1")).is_some());
    assert!(knnkp_adjustment(&board("K7/8/n7/2n5/4k3/4P3/8/8 b - - 0 1")).is_some());
    // An extra pawn on either side, a missing knight, no pawn at all.
    assert!(knnkp_adjustment(&board("k7/4p3/4K3/8/2N5/N3P3/8/8 w - - 0 1")).is_none());
    assert!(knnkp_adjustment(&board("k7/4p1p1/4K3/8/2N5/N7/8/8 w - - 0 1")).is_none());
    assert!(knnkp_adjustment(&board("k7/4p3/4K3/8/2N5/8/8/8 w - - 0 1")).is_none());
    assert!(knnkp_adjustment(&board("k7/8/4K3/8/2N5/N7/8/8 w - - 0 1")).is_none());
}

#[test]
fn knnkp_is_colour_symmetric() {
    // The same position with colours swapped and the board mirrored.
    let white = knnkp_adjustment(&board("k7/4p3/4K3/8/2N5/N7/8/8 w - - 0 1"));
    let black = knnkp_adjustment(&board("8/8/n7/2n5/8/4k3/4P3/K7 b - - 0 1"));
    assert_eq!(white.map(|v| -v), black);
    assert!(white.is_some_and(|v| v > 0));
    // With the pawn blockaded, so the term that reads the pawn's front
    // square is mirrored too.
    let white = knnkp_adjustment(&board("k7/8/4K3/4p3/4N3/N7/8/8 w - - 0 1"));
    let black = knnkp_adjustment(&board("8/8/n7/4n3/4P3/4k3/8/K7 b - - 0 1"));
    assert_eq!(white.map(|v| -v), black);
}

#[test]
fn knnkp_drives_the_king_to_a_corner_and_holds_the_pawn_back() {
    let cornered = knnkp_adjustment(&board("k7/8/4K3/4p3/2N5/N7/8/8 w - - 0 1")).unwrap();
    let central = knnkp_adjustment(&board("8/8/4K3/4p3/2Nk4/N7/8/8 w - - 0 1")).unwrap();
    assert!(cornered > central, "{cornered} <= {central}");
    let held = knnkp_adjustment(&board("k7/4p3/4K3/8/2N5/N7/8/8 w - - 0 1")).unwrap();
    let run = knnkp_adjustment(&board("k7/8/4K3/8/2N5/N3p3/8/8 w - - 0 1")).unwrap();
    assert!(held > run, "{held} <= {run}");
}

#[test]
fn knnkp_values_the_blockade_above_reaching_it_above_neither() {
    // Pawn e5, front square e4: a knight on e4; a knight on c3, which can
    // reach e4; knights on a3 and a2, which cannot.
    let blockaded = knnkp_adjustment(&board("k7/8/4K3/4p3/4N3/N7/8/8 w - - 0 1")).unwrap();
    let reachable = knnkp_adjustment(&board("k7/8/4K3/4p3/8/N1N5/8/8 w - - 0 1")).unwrap();
    let neither = knnkp_adjustment(&board("k7/8/4K3/4p3/8/N7/N7/8 w - - 0 1")).unwrap();
    assert!(blockaded > reachable, "{blockaded} <= {reachable}");
    assert!(reachable >= neither, "{reachable} < {neither}");
}

/// FEN of a piece list (`(square, piece char)`, a1 = 0) with the given
/// side to move.
fn fen_of(pieces: &[(usize, char)], white_to_move: bool) -> String {
    let mut grid = [None; 64];
    for &(sq, piece) in pieces {
        grid[sq] = Some(piece);
    }
    let mut fen = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            if let Some(piece) = grid[rank * 8 + file] {
                if empty > 0 {
                    fen.push_str(&empty.to_string());
                    empty = 0;
                }
                fen.push(piece);
            } else {
                empty += 1;
            }
        }
        if empty > 0 {
            fen.push_str(&empty.to_string());
        }
        if rank > 0 {
            fen.push('/');
        }
    }
    fen.push_str(if white_to_move {
        " w - - 0 1"
    } else {
        " b - - 0 1"
    });
    fen
}

/// The adjustment must never push KNN-KP below a position one capture
/// away (either knight gone, KN-KP; the pawn gone, KNN-K at 0) that the
/// generic evaluation keeps it above. A score that loses the knights'
/// material value makes the search give a knight away.
#[test]
fn knnkp_stays_above_its_one_capture_neighbours() {
    let mut state: u64 = 0x5EED_2026_1012;
    let mut next_square = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from((state >> 33) % 64).expect("below 64")
    };
    let mut evaluator = Evaluator::default();
    let mut generated = 0;
    let mut active = 0;
    while generated < 2000 {
        let squares = [
            next_square(),
            next_square(),
            next_square(),
            next_square(),
            next_square(),
        ];
        let [wk, n1, n2, bk, bp] = squares;
        let distinct = (0..5).all(|i| (i + 1..5).all(|j| squares[i] != squares[j]));
        if !distinct || !(8..56).contains(&bp) || KING_DISTANCE[wk][bk] < 2 {
            continue;
        }
        // A pawn one push from a promotion square that no knight holds or
        // covers cannot be kept by the knights, so the position is not one
        // they can win and its order against "the pawn gone" means nothing.
        if bp < 16 {
            let promotion = bp - 8;
            let covers = |knight: usize| {
                let files = (knight % 8).abs_diff(promotion % 8);
                let ranks = (knight / 8).abs_diff(promotion / 8);
                knight == promotion || (files, ranks) == (1, 2) || (files, ranks) == (2, 1)
            };
            if !covers(n1) && !covers(n2) {
                continue;
            }
        }
        // A knight checking the side not to move is illegal. With only
        // knights and kings there are no discovered checks, so removing a
        // piece keeps a legal position legal, and mirroring keeps it too.
        let original = [(wk, 'K'), (n1, 'N'), (n2, 'N'), (bk, 'k'), (bp, 'p')];
        if Board::from_fen(&fen_of(&original, true)).is_err() {
            continue;
        }
        generated += 1;
        for mirrored in [false, true] {
            // Mirroring swaps the colours and flips the ranks, so the
            // knights' side is Black and Black is to move.
            let place = |sq: usize, piece: char| {
                if mirrored {
                    let swapped = if piece.is_ascii_uppercase() {
                        piece.to_ascii_lowercase()
                    } else {
                        piece.to_ascii_uppercase()
                    };
                    (sq ^ 56, swapped)
                } else {
                    (sq, piece)
                }
            };
            let full = [
                place(wk, 'K'),
                place(n1, 'N'),
                place(n2, 'N'),
                place(bk, 'k'),
                place(bp, 'p'),
            ];
            let mut score =
                |pieces: &[(usize, char)]| evaluator.evaluate(&board(&fen_of(pieces, !mirrored)));
            let fen = fen_of(&full, !mirrored);
            let adjusted = score(&full);
            let sign = if mirrored { -1 } else { 1 };
            let adjustment = sign * knnkp_adjustment(&board(&fen)).expect("exact KNN-KP");
            let generic = adjusted - adjustment;
            let neighbours = [
                score(&[full[0], full[2], full[3], full[4]]),
                score(&[full[0], full[1], full[3], full[4]]),
                0,
            ];
            for neighbour in neighbours {
                if generic > neighbour {
                    active += 1;
                    assert!(
                        adjusted > neighbour,
                        "{fen}: generic {generic} > {neighbour}, adjusted {adjusted}"
                    );
                }
            }
        }
    }
    assert!(active > 9000, "only {active} comparisons were active");
}

/// The chess rule, not a hardcoded constant: KQ-vs-KP is a fortress draw
/// only for rook/bishop pawns — knight and centre pawns are wins and must
/// not be scaled.
#[test]
fn kqkp_fortress_only_draws_rook_and_bishop_pawns() {
    // Bishop pawn (c) on the 7th, king guarding c1, queen's king far: drawn.
    assert_eq!(
        kqkp_fortress_scale(&board("8/8/6K1/8/8/8/1kp5/7Q w - - 0 1")),
        Some(0)
    );
    // Knight pawn (b) on the 7th — a win for the queen — must NOT scale.
    assert_eq!(
        kqkp_fortress_scale(&board("8/8/6K1/8/8/8/kp6/7Q w - - 0 1")),
        None
    );
}

/// KR-vs-KP draw zone gets a conservative *partial* scale, never a forced 0.
#[test]
fn krkp_partial_scale_in_the_draw_zone() {
    assert_eq!(
        krkp_drawish_scale(&board("8/8/6K1/8/8/8/1kp5/7R w - - 0 1")),
        Some(16)
    );
}

/// KRP-vs-KB rook-pawn fortresses. Both FENs are Syzygy DRAW. The partial
/// scales leave a +350 score above 100 cp even where they fire, so an
/// aggregate overclaim rate cannot show the recogniser is live; this
/// asserts the mechanism directly.
#[test]
fn krpkb_scales_rook_pawn_fortresses() {
    // Pawn a5, bishop on the pawn's own colour, defending king next to the
    // queening square: the reference's moderate fortress reduction.
    assert_eq!(
        krpkb_scale(&board("8/1kb5/8/P7/8/8/8/6KR w - - 0 1")),
        Some(24)
    );
    // A non-rook pawn is outside the reference's case analysis entirely,
    // and must fall through rather than be scaled on a guess.
    assert_eq!(
        krpkb_scale(&board("8/2kb4/8/1P6/8/8/8/6KR w - - 0 1")),
        None
    );
}

/// KRP-vs-KR draw recognition. Every FEN here was probed against
/// Syzygy before it was written down, so these assert chess truth rather
/// than the recognizer's own opinion of itself.
#[test]
fn krpkr_recognizes_textbook_draws() {
    // Philidor third-rank defence: defending rook on the 6th cuts the
    // attacking king while the pawn is still short of it. Syzygy: DRAW.
    assert_eq!(
        krpkr_scale(&board("8/3k4/r7/3PK3/8/8/8/7R w - - 0 1")),
        Some(0)
    );
    // Pawn a7 with the rook in front on a8 and the defending king boxed on
    // h7, its rook behind the pawn. Syzygy: DRAW.
    assert_eq!(
        krpkr_scale(&board("R7/P6k/8/8/8/8/8/r5K1 w - - 0 1")),
        Some(0)
    );
}

/// The failure mode that matters: a hard `Some(0)` on a position that is
/// actually WON would turn wins into claimed draws. The reference calls its
/// own case analysis "far from perfect", so this is the guard that earns
/// porting it. All three are Syzygy WIN, so the endgame scaling must also
/// leave a winning score at least half its size. The scaling is tested on
/// a fixed score, not the evaluation's, whose size every refit moves.
#[test]
fn krpkr_never_zeroes_a_won_position() {
    const WINNING: i32 = 300;
    for fen in [
        "1R6/1P6/8/8/8/7k/r7/6K1 w - - 0 1",
        "8/8/1PK5/8/8/7k/r7/1R6 w - - 0 1",
        "8/8/8/1PK5/8/7k/r7/1R6 w - - 0 1",
    ] {
        let position = board(fen);
        assert_ne!(
            krpkr_scale(&position),
            Some(0),
            "won KRP-KR must not be scaled to a forced draw: {fen}"
        );
        let scaled = scale_endgame(&position, WINNING);
        assert!(
            scaled >= WINNING / 2,
            "won KRP-KR should stay clearly winning, scaled {WINNING} to {scaled} for {fen}"
        );
    }
}

/// Won endings (knight-pawn KQKP, KQ-vs-KR) keep a clearly winning static
/// score and are not scaled toward draw.
#[test]
fn won_endings_are_not_scaled_toward_draw() {
    let kqkp_knight_pawn = static_eval("8/8/6K1/8/8/8/kp6/7Q w - - 0 1");
    assert!(
        kqkp_knight_pawn > 300,
        "won KQ vs knight-pawn should stay clearly winning, got {kqkp_knight_pawn}"
    );
    let kqkr = static_eval("8/8/6KQ/8/3k4/8/8/3r4 w - - 0 1");
    assert!(
        kqkr > 150,
        "won KQ vs KR should stay clearly winning, got {kqkr}"
    );
}

/// Below three pawns a pure bishop ending takes the `32 + ...` rule, which
/// passed pawns relax upward.
#[test]
fn opposite_bishop_scale_relaxed_by_passers() {
    assert_eq!(
        opposite_bishop_scale(&board("4k3/p7/8/3b4/8/8/P7/2B1K3 w - - 0 1")),
        Some(40) // 2 pawns, no passers: 32 + 2*4
    );
    assert_eq!(
        opposite_bishop_scale(&board("4k3/7p/P7/3b4/8/8/8/2B1K3 w - - 0 1")),
        Some(48) // 2 passed pawns relax to the /48 cap
    );
}

/// From three pawns a pure bishop ending takes the fitted rule; other
/// pieces on the board keep the `32 + ...` rule at any pawn count.
#[test]
fn opposite_bishop_scale_pure_endings_from_three_pawns() {
    assert_eq!(
        opposite_bishop_scale(&board("4k3/p7/8/3b4/8/8/PP6/2B1K3 w - - 0 1")),
        Some(7) // 3 pawns, no passers: 1 + 3*2
    );
    assert_eq!(
        opposite_bishop_scale(&board("4k3/7p/P7/3b4/8/8/P7/2B1K3 w - - 0 1")),
        Some(37) // 3 pawns, all passed: 1 + 3*2 + 3*10
    );
    assert_eq!(
        opposite_bishop_scale(&board("r3k3/p7/8/3b4/8/8/PP6/2B1K2R w - - 0 1")),
        Some(44) // rooks on: 32 + 3*4
    );
}

/// `is_draw_verdict` is true exactly where `scale_endgame` returns 0
/// whatever the score it is given, for every verdict family and for the
/// near misses beside them.
#[test]
fn draw_verdicts_are_exactly_scale_endgames_zeros() {
    let cases = [
        ("8/8/4k3/8/8/4K3/8/8 w - - 0 1", true),      // KK
        ("8/8/4k3/8/8/4K3/8/5N2 w - - 0 1", true),    // KNK
        ("8/8/4k3/8/3n4/4K3/8/5B2 w - - 0 1", true),  // minor against minor
        ("8/8/4k3/8/8/4K3/8/4NN2 w - - 0 1", true),   // KNNK
        ("7k/8/6KP/8/8/8/8/5B2 w - - 0 1", true),     // wrong-corner KBP
        ("7k/8/8/8/8/8/7P/7K w - - 0 1", true),       // drawn KPK
        ("8/1rk5/8/8/8/8/3P1K1R/8 w - - 0 1", true),  // a KRPKR zero scale
        ("4k3/8/4K3/4P3/8/8/8/8 w - - 0 1", false),   // won KPK
        ("8/8/8/4k3/8/8/8/2B1KN2 w - - 0 1", false),  // KBNK, a win
        ("k7/4p3/4K3/8/2N5/N7/8/8 w - - 0 1", false), // KNN-KP's adjustment
        (
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            false,
        ),
    ];
    for (fen, verdict) in cases {
        let position = board(fen);
        let zero = scale_endgame(&position, 977) == 0 && scale_endgame(&position, -613) == 0;
        assert_eq!(zero, verdict, "{fen}: scale_endgame's zero");
        assert_eq!(is_draw_verdict(&position), verdict, "{fen}");
    }
}
