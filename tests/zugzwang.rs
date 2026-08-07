//! Phase 4.4 zugzwang and null-move soundness suite.
//!
//! Null-move pruning assumes passing is never better than moving. Zugzwang is
//! exactly where that is false, so an unsound null contract surfaces here and
//! almost nowhere else — `bench` fingerprints and tactical suites are both blind
//! to it, because a wrong null cutoff usually yields a *plausible* move rather
//! than a crash or an illegal one.
//!
//! These are the guard rail for the 4.4 switches
//! (`NmpSuppressNullInVerification`, `NmpDecisiveGuard`, `NmpRequireCutNode`,
//! `NmpUseStaticEval`, `SingularMaxExtension`): a switch that breaks null
//! soundness should fail a test rather than quietly cost Elo in a 16,000-game
//! gate.
//!
//! # Two design rules, both learned the hard way
//!
//! **Assert bounds and legality, not exact moves or scores.** The KQvK
//! post-mortem in `search_strength.rs` is the precedent: a zero-margin
//! assertion becomes a tripwire that fires on any legitimate eval change
//! instead of on real breakage. The only exact-move assertion here is on a
//! position with exactly one legal reply.
//!
//! **Every premise below was measured, not eyed.** The first draft of this file
//! asserted properties of positions that did not have them — a "one legal move"
//! position that was actually stalemate, a "blocked draw" that the engine
//! correctly scored at −1352 because the kings were not symmetric, and two
//! "in check" positions that were checkmate. Each FEN's legal-move count, check
//! status and score are stated in comments as observed at depth 12.

use rarog::board::{Board, Move};
use rarog::search::{SearchEvent, Searcher};
use rarog::search_options::SearchOptions;

fn search(fen: &str, depth: u32) -> (Move, i32) {
    search_with(fen, depth, |_| {})
}

/// Search with the 4.4 switches under caller control.
///
/// `SearchParams` fields are public independently of the `tune` feature, so a
/// test can exercise a switch that production leaves off. This matters most for
/// `NmpDecisiveGuard`, which has **zero population on `bench 13`** — the
/// fingerprint therefore cannot verify it either way, and only a targeted test
/// can show that enabling it does not break null soundness.
fn search_with(
    fen: &str,
    depth: u32,
    configure: impl FnOnce(&mut rarog::params::SearchParams),
) -> (Move, i32) {
    let board = Board::from_fen(fen).expect("valid FEN");
    let mut searcher = Searcher::default();
    let mut options = SearchOptions::default();
    options.position.board = board.clone();
    options.limits.depth = Some(depth);
    configure(&mut options.engine.search_params);
    let result = searcher.search(board, &options, false, || SearchEvent::None);
    (result.bestmove, result.score)
}

fn assert_legal(fen: &str, mv: Move, context: &str) {
    let board = Board::from_fen(fen).expect("valid FEN");
    let legal = board.generate_legal_moves();
    assert!(legal.contains(&mv), "{context}: {mv} is not legal in {fen}");
}

/// Colour-symmetric blocked positions. Symmetry is what makes these a real
/// null-move test: material, structure AND king placement mirror, so the true
/// score is a tempo at most. A null cutoff manufactures the tempo White cannot
/// actually keep and reports a decisive edge.
///
/// The mirror matters. An earlier draft used blocked pawns with kings on a1/h1,
/// which the engine scored at −914 — correctly, because Black's king was far
/// more active. Blocked pawns alone do not make a draw.
///
/// Observed at depth 12: all three score exactly 0, 5 legal moves each.
#[test]
fn symmetric_blocked_positions_are_not_scored_as_decisive() {
    for (fen, label) in [
        (
            "4k3/8/8/p1p1p1p1/P1P1P1P1/8/8/4K3 w - - 0 1",
            "4 pawns, white",
        ),
        (
            "4k3/8/8/p1p1p1p1/P1P1P1P1/8/8/4K3 b - - 0 1",
            "4 pawns, black",
        ),
        ("4k3/8/8/p1p1p3/P1P1P3/8/8/4K3 w - - 0 1", "3 pawns, white"),
    ] {
        let (mv, score) = search(fen, 12);
        assert_legal(fen, mv, label);
        assert!(
            score.abs() < 200,
            "{label}: {score} cp in a colour-symmetric position, where the true \
             value is a tempo at most — the signature of a null cutoff \
             manufacturing tempo"
        );
    }
}

/// The textbook king-and-pawn zugzwang: whichever side must move concedes.
/// Observed at depth 12: score 0 with 9 legal moves, so the engine already
/// handles it — this pins that it keeps doing so.
#[test]
fn classic_king_pawn_zugzwang_stays_balanced() {
    let fen = "6k1/5p2/6p1/8/6P1/5P2/6K1/8 w - - 0 1";
    let (mv, score) = search(fen, 12);
    assert_legal(fen, mv, "classic zugzwang");
    assert!(
        score.abs() < 200,
        "classic zugzwang scored {score} cp; a null cutoff inflates exactly this"
    );
}

/// A null move must never be attempted while in check — passing would leave the
/// king capturable and any resulting score is meaningless. Swept across depths
/// because NMP is depth-gated at 3, so one depth could miss it.
///
/// All five were verified to be in check WITH legal replies (3–4 each); two
/// candidates from the first draft were dropped because they were checkmate,
/// which tests nothing about null moves.
#[test]
fn in_check_never_returns_a_null_artifact() {
    let fens = [
        "4k3/8/8/8/8/8/8/4K2r w - - 0 1",
        "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
        "3k4/8/8/8/7b/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/3n4/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/8/q3K3 w - - 0 1",
    ];
    for fen in fens {
        let board = Board::from_fen(fen).expect("valid FEN");
        assert!(board.is_in_check(), "{fen} should be in check");
        let legal = board.generate_legal_moves();
        assert!(
            !legal.is_empty(),
            "{fen} is checkmate, so it tests nothing about null moves"
        );
        for depth in [3, 4, 8, 12] {
            let (mv, score) = search(fen, depth);
            assert_legal(fen, mv, &format!("depth {depth}"));
            assert!(
                score.abs() <= 32_000,
                "{fen} at depth {depth}: score {score} is outside the mate range"
            );
        }
    }
}

/// A position with exactly one legal reply. Whatever pruning decides, the search
/// must return that move — the strongest available check that no pruning path can
/// escape a forced continuation.
///
/// Observed: 1 legal move (`h1g2`, capturing the checking queen), in check,
/// score 0. The premise is asserted rather than trusted, because the first draft
/// used a stalemate position by mistake.
#[test]
fn a_single_legal_reply_is_always_returned() {
    let fen = "7k/8/8/8/8/8/6q1/7K w - - 0 1";
    let board = Board::from_fen(fen).expect("valid FEN");
    let legal = board.generate_legal_moves();
    assert_eq!(
        legal.len(),
        1,
        "test premise broken: {} legal moves, not 1",
        legal.len()
    );
    for depth in [1, 4, 8, 12] {
        let (mv, _) = search(fen, depth);
        assert_eq!(
            mv, legal[0],
            "depth {depth} returned {mv}, not the only legal move"
        );
    }
}

/// Pawn-only endgames are refused by NMP's existing `has_non_pawn_material`
/// guard. This pins it from both sides, so the suite catches over-restriction as
/// well as an unsound cutoff.
#[test]
fn pawn_only_endgames_stay_sound_in_both_directions() {
    // Observed: Black is not merely better but mating (score 31,991). A weakened
    // material guard that pruned this would lose the mate entirely.
    let won = "8/8/8/8/8/1k6/1p6/1K6 b - - 0 1";
    let (mv, score) = search(won, 14);
    assert_legal(won, mv, "won pawn ending");
    assert!(
        score > 1_000,
        "the side with the promoting pawn scored only {score}; the NMP material \
         guard may have been weakened into pruning this"
    );
    // Observed: +116, i.e. near-equal rather than dead drawn. The bound is set
    // to catch a decisive claim, not to pin the exact assessment.
    let level = "8/8/8/p7/P7/8/8/K6k w - - 0 1";
    let (mv, score) = search(level, 14);
    assert_legal(level, mv, "level pawn ending");
    assert!(score.abs() < 900, "level pawn ending scored {score} cp");
}

/// Mate-range windows are where a null cutoff is least defensible: a pass cannot
/// refute a forced mate. `NmpDecisiveGuard` targets this, and the property must
/// already hold with the guard off.
#[test]
fn forced_mates_are_still_proven() {
    // Observed: 31,999 via a1a8, i.e. mate in one is found and proven.
    let fen = "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1";
    let (mv, score) = search(fen, 8);
    assert_legal(fen, mv, "back-rank mate");
    assert!(
        score > 30_000,
        "mate in one scored only {score}; the search did not prove it"
    );
}

/// Every 4.4 switch, turned ON, must preserve the soundness properties above.
///
/// This is the half of the safety argument the bench fingerprint cannot make.
/// A switch that lands inert is only safe to enable later if enabling it has
/// been shown not to break anything — and for `NmpDecisiveGuard`, whose bench
/// population is exactly zero, this test is the *only* evidence that exists.
///
/// Asserted with the switches on individually and all together, because the
/// combination is what 4.4's bundle will actually ship.
#[test]
fn every_4_4_switch_preserves_null_soundness() {
    type Setter = fn(&mut rarog::params::SearchParams);
    let arms: [(&str, Setter); 10] = [
        ("baseline", |_p| {}),
        ("nmp subtree suppression", |p| {
            p.nmp_suppress_null_in_verification = 1;
        }),
        ("nmp decisive guard", |p| p.nmp_decisive_guard = 1),
        ("nmp require cut node", |p| p.nmp_require_cut_node = 1),
        ("nmp static eval", |p| p.nmp_use_static_eval = 1),
        ("singular cap 1", |p| p.singular_max_extension = 1),
        ("singular rejects speculative", |p| {
            p.singular_reject_speculative = 1;
        }),
        ("rfp at tt_pv", |p| p.rfp_allow_tt_pv = 1),
        ("all tt_pv", |p| {
            p.rfp_allow_tt_pv = 1;
            p.razor_allow_tt_pv = 1;
            p.nmp_allow_tt_pv = 1;
            p.probcut_allow_tt_pv = 1;
        }),
        ("everything on", |p| {
            p.nmp_suppress_null_in_verification = 1;
            p.nmp_decisive_guard = 1;
            p.nmp_require_cut_node = 1;
            p.nmp_use_static_eval = 1;
            p.singular_max_extension = 1;
            p.singular_reject_speculative = 1;
            p.rfp_allow_tt_pv = 1;
            p.razor_allow_tt_pv = 1;
            p.nmp_allow_tt_pv = 1;
            p.probcut_allow_tt_pv = 1;
        }),
    ];

    for (label, configure) in arms {
        // Symmetric blocked: must stay near level.
        let sym = "4k3/8/8/p1p1p1p1/P1P1P1P1/8/8/4K3 w - - 0 1";
        let (mv, score) = search_with(sym, 12, configure);
        assert_legal(sym, mv, label);
        assert!(
            score.abs() < 200,
            "{label}: symmetric blocked position scored {score} cp"
        );

        // Forced mate must still be proven — the case the decisive guard touches.
        let mate = "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1";
        let (mv, score) = search_with(mate, 8, configure);
        assert_legal(mate, mv, label);
        assert!(
            score > 30_000,
            "{label}: mate in one scored only {score}, so the switch lost a proven mate"
        );

        // In check with legal replies: the reply must be legal at every depth.
        let check = "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1";
        for depth in [3, 8, 12] {
            let (mv, _) = search_with(check, depth, configure);
            assert_legal(check, mv, &format!("{label} at depth {depth}"));
        }

        // The only legal reply must still be returned.
        let forced = "7k/8/8/8/8/8/6q1/7K w - - 0 1";
        let legal = Board::from_fen(forced)
            .expect("valid FEN")
            .generate_legal_moves();
        let (mv, _) = search_with(forced, 8, configure);
        assert_eq!(mv, legal[0], "{label}: lost the only legal move");
    }
}
