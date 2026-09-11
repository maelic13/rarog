// `clippy::too_many_arguments` accepted crate-wide — see Cargo.toml. Here it
// covers `gen_pawn_moves`, which receives position facts its siblings already
// computed once rather than recomputing them.

use super::attacks::ATTACKS;
use super::bitboard::Bitboard;
use super::board::Board;
use super::moves::{
    CAPTURE, CASTLE_KINGSIDE, CASTLE_QUEENSIDE, DOUBLE_PUSH, EN_PASSANT, Move, MoveList,
    PROMO_BISHOP, PROMO_CAPTURE_BISHOP, PROMO_CAPTURE_KNIGHT, PROMO_CAPTURE_QUEEN,
    PROMO_CAPTURE_ROOK, PROMO_KNIGHT, PROMO_QUEEN, PROMO_ROOK, QUIET,
};
use super::piece::{CastlingRights, Color, Piece};
use super::square::{Rank, Square};
/// Legal move generation and perft.
///
/// Strategy:
///   1. Find the king square, compute checkers and pinned pieces.
///   2. In double check: generate only king evasions.
///   3. In single check: generate king moves + interpositions/captures of checker.
///   4. No check: generate all moves for each piece, respecting pins.
use crate::infra;

// -----------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------

/// Generate all legal moves for the current position.
pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(48);
    gen_moves::<true, true, _>(board, &mut moves);
    crate::diag_count!(board_gen_vec_calls);
    crate::diag_add!(board_gen_vec_moves, moves.len() as u64);
    moves
}

/// Generate all legal moves into a caller-owned fixed-capacity move list.
///
/// The out-parameter form is the primary one. Returning a `MoveList` by value
/// costs a 520-byte `memcpy` on the normal return path of the fat-LTO build:
/// RVO does not apply there, and RAR-M44 measured the copy at +11.2% on legal
/// generation and +40.5% on captures once removed. The list is cleared first,
/// so a caller may reuse one list across generations.
pub fn generate_legal_into(board: &Board, moves: &mut MoveList) {
    moves.clear();
    gen_moves::<true, true, _>(board, moves);
    crate::diag_count!(board_gen_full_calls);
    crate::diag_add!(board_gen_full_moves, moves.len() as u64);
}

/// [`generate_legal_into`] returning a fresh list, for callers that are not on
/// a hot path and prefer the value form.
pub fn generate_legal_movelist(board: &Board) -> MoveList {
    let mut moves = MoveList::new();
    generate_legal_into(board, &mut moves);
    moves
}

/// Generate only legal quiet moves.
pub fn generate_quiets(board: &Board) -> MoveList {
    let mut moves = MoveList::new();
    gen_moves::<false, true, _>(board, &mut moves);
    moves
}

/// [`generate_quiets`] into a caller-owned list, reusing a pinned set computed
/// earlier at the same node (10.3 speed pass — see [`gen_moves_pinned`]).
pub fn generate_quiets_pinned_into(board: &Board, pinned: Bitboard, moves: &mut MoveList) {
    moves.clear();
    gen_moves_pinned::<false, true, _>(board, pinned, moves);
    crate::diag_count!(board_gen_staged_quiet_calls);
    crate::diag_add!(board_gen_staged_quiet_moves, moves.len() as u64);
}

/// [`generate_quiets_pinned_into`] returning a fresh list.
pub fn generate_quiets_pinned(board: &Board, pinned: Bitboard) -> MoveList {
    let mut moves = MoveList::new();
    generate_quiets_pinned_into(board, pinned, &mut moves);
    moves
}

/// Generate only legal captures (for quiescence search).
///
/// Captures-only terminal callers (qsearch, ProbCut) keep the
/// [`has_pseudo_capture`] pre-scan: nothing else runs at this node, so a
/// negative answer really does save the pin computation and the whole
/// generation pass. 10.3(7) measured the pre-scan firing on **19.1%** of these
/// calls, and the 80.9% that do find a capture exit the scan early (king/pawn
/// tests come first), so the pre-scan is cheap when it fails and pays a full
/// generation when it succeeds.
pub fn generate_captures_into(board: &mut Board, moves: &mut MoveList) {
    moves.clear();
    let us = board.side_to_move;
    let them = !us;

    if !has_pseudo_capture(board, us, them) {
        crate::diag_count!(board_gen_capture_calls);
        return;
    }

    let king_sq = board.king_sq(us);
    let pinned = compute_pinned(board, king_sq, us, them);
    gen_captures_with_pin(board, us, them, king_sq, pinned, moves);
    crate::diag_count!(board_gen_capture_calls);
    crate::diag_add!(board_gen_capture_moves, moves.len() as u64);
}

/// [`generate_captures_into`] returning a fresh list.
pub fn generate_captures(board: &mut Board) -> MoveList {
    let mut moves = MoveList::new();
    generate_captures_into(board, &mut moves);
    moves
}

/// [`generate_captures`], also handing back the pinned set it computed so a
/// staged move picker can feed it to [`generate_quiets_pinned`] later at the
/// same node (10.3 speed pass).
///
/// Deliberately does NOT run the [`has_pseudo_capture`] pre-scan, unlike its
/// captures-only sibling (10.3(7)). A staged node usually goes on to generate
/// quiets, and the pre-scan's saving is then illusory: `compute_pinned` is four
/// slider lookups plus a short sniper walk, while a *failing* pre-scan is a
/// full attack pass over every one of our pieces — and 78.5% of the nodes where
/// it fired then paid for the pins anyway in `generate_quiets`. Computing pins
/// unconditionally is therefore less work in the common case and lets every
/// stage at this node share one pinned set.
pub fn generate_captures_pinned_into(board: &mut Board, moves: &mut MoveList) -> Bitboard {
    moves.clear();
    let us = board.side_to_move;
    let them = !us;
    let king_sq = board.king_sq(us);
    let pinned = compute_pinned(board, king_sq, us, them);

    gen_captures_with_pin(board, us, them, king_sq, pinned, moves);
    crate::diag_count!(board_gen_staged_capture_calls);
    crate::diag_add!(board_gen_staged_capture_moves, moves.len() as u64);
    pinned
}

/// [`generate_captures_pinned_into`] returning a fresh list beside the pinned
/// set.
pub fn generate_captures_pinned(board: &mut Board) -> (MoveList, Bitboard) {
    let mut moves = MoveList::new();
    let pinned = generate_captures_pinned_into(board, &mut moves);
    (moves, pinned)
}

fn gen_captures_with_pin(
    board: &Board,
    us: Color,
    them: Color,
    king_sq: Square,
    pinned: Bitboard,
    moves: &mut MoveList,
) {
    if pinned.is_empty() {
        gen_unpinned_captures(board, us, them, king_sq, board.checkers(), moves);
    } else {
        gen_moves_pinned::<true, false, _>(board, pinned, moves);
    }
}

/// Conservative "is there anything to capture at all" pre-scan: it may only
/// return `false` when capture generation would produce an empty list (EP and
/// promotion-captures are pawn attacks, so both are covered). Skipping
/// generation on `false` is therefore behavior-preserving.
fn has_pseudo_capture(board: &Board, us: Color, them: Color) -> bool {
    let atk = &*ATTACKS;
    let their_occ = board.color_occ(them) & !board.pieces(them, Piece::King);

    if (atk.king(board.king_sq(us)) & their_occ).any() {
        return true;
    }

    let pawns = board.pieces(us, Piece::Pawn);
    let pawn_attacks = if us == Color::White {
        pawns.north_east() | pawns.north_west()
    } else {
        pawns.south_east() | pawns.south_west()
    };
    if (pawn_attacks & their_occ).any() {
        return true;
    }
    if let Some(ep_sq) = board.ep_square() {
        let ep_attackers = atk.pawn(them, ep_sq) & pawns;
        if ep_attackers.any() {
            return true;
        }
    }

    let mut knights = board.pieces(us, Piece::Knight);
    while knights.any() {
        if (atk.knight(knights.pop_lsb()) & their_occ).any() {
            return true;
        }
    }

    let all_occ = board.all_occ;
    let mut bishops = board.pieces(us, Piece::Bishop);
    while bishops.any() {
        if (atk.bishop(bishops.pop_lsb(), all_occ) & their_occ).any() {
            return true;
        }
    }

    let mut rooks = board.pieces(us, Piece::Rook);
    while rooks.any() {
        if (atk.rook(rooks.pop_lsb(), all_occ) & their_occ).any() {
            return true;
        }
    }

    let mut queens = board.pieces(us, Piece::Queen);
    while queens.any() {
        if (atk.queen(queens.pop_lsb(), all_occ) & their_occ).any() {
            return true;
        }
    }

    false
}

/// Recursive perft — counts leaf nodes at depth `depth`.
pub fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut moves = MoveList::new();
    generate_legal_into(board, &mut moves);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut nodes = 0u64;
    for &mv in moves.as_slice() {
        board.make_move(mv);
        nodes += perft(board, depth - 1);
        board.unmake_move(mv);
    }
    nodes
}

// -----------------------------------------------------------------------
// Core generation
// -----------------------------------------------------------------------

trait MoveSink {
    fn push_move(&mut self, mv: Move);
}

impl MoveSink for Vec<Move> {
    #[inline(always)]
    fn push_move(&mut self, mv: Move) {
        self.push(mv);
    }
}

impl MoveSink for MoveList {
    #[inline(always)]
    fn push_move(&mut self, mv: Move) {
        self.push(mv);
    }
}

fn gen_moves<const CAPTURES: bool, const QUIETS: bool, S: MoveSink>(board: &Board, moves: &mut S) {
    let us = board.side_to_move;
    let king_sq = board.king_sq(us);
    let pinned = compute_pinned(board, king_sq, us, !us);
    gen_moves_pinned::<CAPTURES, QUIETS, S>(board, pinned, moves);
}

/// [`gen_moves`] for callers that already hold the pinned set for this
/// position (10.3 speed pass).
///
/// `compute_pinned` is four slider lookups plus a per-sniper `between` scan, and
/// it was being repeated: `generate_captures` computed it and then handed off
/// to `gen_moves`, which computed the very same thing again; and a staged node
/// paid for it once more when quiets were finally generated. Pins are a pure
/// function of the position, so one computation per node serves every stage.
fn gen_moves_pinned<const CAPTURES: bool, const QUIETS: bool, S: MoveSink>(
    board: &Board,
    pinned: Bitboard,
    moves: &mut S,
) {
    debug_assert_eq!(
        pinned,
        compute_pinned(
            board,
            board.king_sq(board.side_to_move),
            board.side_to_move,
            !board.side_to_move
        ),
        "stale pinned set handed to gen_moves_pinned"
    );
    let us = board.side_to_move;
    let them = !us;
    let atk = &*ATTACKS;

    let our_occ = board.color_occ(us);
    let their_occ = board.color_occ(them);
    let all_occ = board.all_occ;

    let king_sq = board.king_sq(us);

    // Cached by Board; refreshed lazily after make/unmake.
    let checkers = board.checkers();
    let in_double_check = checkers.more_than_one();

    // --- King moves (always generated) ---
    let king_targets = atk.king(king_sq) & !our_occ;
    let targets = filter_targets::<CAPTURES, QUIETS>(king_targets, their_occ);
    let mut targets = targets;
    while targets.any() {
        let to = targets.pop_lsb();
        // Make sure the destination isn't attacked (use occ without the king)
        let occ_no_king = all_occ ^ Bitboard::from(king_sq);
        if !board.is_attacked_with_occ(to, them, occ_no_king) {
            add_move(king_sq, to, their_occ, moves);
        }
    }

    // In double check only king moves are legal
    if in_double_check {
        return;
    }

    // Compute mask of target squares for non-king pieces.
    // In single check: must block or capture the checker.
    let check_mask = if checkers.any() {
        let checker_sq = checkers.lsb();
        // Captures of the checker + squares between king and checker
        Bitboard::from(checker_sq) | between(king_sq, checker_sq)
    } else {
        Bitboard::FULL // no check: any target is fine
    };

    // --- Pawns ---
    gen_pawn_moves::<CAPTURES, QUIETS, _>(
        board, us, them, their_occ, all_occ, pinned, king_sq, check_mask, moves,
    );

    // --- Knights ---
    let mut knights = board.pieces(us, Piece::Knight) & !pinned;
    while knights.any() {
        let from = knights.pop_lsb();
        let raw = atk.knight(from) & !our_occ & check_mask;
        let targets = filter_targets::<CAPTURES, QUIETS>(raw, their_occ);
        let mut targets = targets;
        while targets.any() {
            let to = targets.pop_lsb();
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Bishops ---
    let mut bishops = board.pieces(us, Piece::Bishop);
    while bishops.any() {
        let from = bishops.pop_lsb();
        let raw = atk.bishop(from, all_occ) & !our_occ & check_mask;
        let targets = filter_targets::<CAPTURES, QUIETS>(raw, their_occ);
        let mut targets = filter_pinned(from, targets, pinned, king_sq);
        while targets.any() {
            let to = targets.pop_lsb();
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Rooks ---
    let mut rooks = board.pieces(us, Piece::Rook);
    while rooks.any() {
        let from = rooks.pop_lsb();
        let raw = atk.rook(from, all_occ) & !our_occ & check_mask;
        let targets = filter_targets::<CAPTURES, QUIETS>(raw, their_occ);
        let mut targets = filter_pinned(from, targets, pinned, king_sq);
        while targets.any() {
            let to = targets.pop_lsb();
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Queens ---
    let mut queens = board.pieces(us, Piece::Queen);
    while queens.any() {
        let from = queens.pop_lsb();
        let raw = atk.queen(from, all_occ) & !our_occ & check_mask;
        let targets = filter_targets::<CAPTURES, QUIETS>(raw, their_occ);
        let mut targets = filter_pinned(from, targets, pinned, king_sq);
        while targets.any() {
            let to = targets.pop_lsb();
            add_move(from, to, their_occ, moves);
        }
    }

    // --- Castling (only when not in check, not captures_only) ---
    if QUIETS && !checkers.any() {
        gen_castling(board, us, them, all_occ, moves);
    }
}

#[inline(always)]
fn filter_targets<const CAPTURES: bool, const QUIETS: bool>(
    raw: Bitboard,
    their_occ: Bitboard,
) -> Bitboard {
    if CAPTURES && QUIETS {
        raw
    } else if CAPTURES {
        raw & their_occ
    } else {
        raw & !their_occ
    }
}

fn gen_unpinned_captures(
    board: &Board,
    us: Color,
    them: Color,
    king_sq: Square,
    checkers: Bitboard,
    moves: &mut MoveList,
) {
    let atk = &*ATTACKS;
    let their_occ = board.color_occ(them) & !board.pieces(them, Piece::King);
    let all_occ = board.all_occ;
    let king_bb = Bitboard::from(king_sq);

    let mut targets = atk.king(king_sq) & their_occ;
    while targets.any() {
        let to = targets.pop_lsb();
        if !board.is_attacked_with_occ(to, them, all_occ ^ king_bb) {
            moves.push_move(Move::new(king_sq, to, CAPTURE));
        }
    }

    if checkers.more_than_one() {
        return;
    }

    let target_mask = if checkers.any() { checkers } else { their_occ };

    let mut pawns = board.pieces(us, Piece::Pawn);
    while pawns.any() {
        let from = pawns.pop_lsb();
        let mut targets = atk.pawn(us, from) & target_mask;
        while targets.any() {
            let to = targets.pop_lsb();
            push_pawn_move_flags(from, to, true, moves);
        }

        if let Some(ep_sq) = board.ep_square()
            && (atk.pawn(us, from) & Bitboard::from(ep_sq)).any()
        {
            let ep_cap_sq = if us == Color::White {
                Square(ep_sq.0 - 8)
            } else {
                Square(ep_sq.0 + 8)
            };
            let captures_checker =
                checkers.is_empty() || (checkers & Bitboard::from(ep_cap_sq)).any();

            if captures_checker && ep_capture_is_legal(board, us, them, from, ep_sq, ep_cap_sq) {
                moves.push_move(Move::new(from, ep_sq, EN_PASSANT));
            }
        }
    }

    let mut knights = board.pieces(us, Piece::Knight);
    while knights.any() {
        let from = knights.pop_lsb();
        let mut targets = atk.knight(from) & target_mask;
        while targets.any() {
            let to = targets.pop_lsb();
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }

    let mut bishops = board.pieces(us, Piece::Bishop);
    while bishops.any() {
        let from = bishops.pop_lsb();
        let mut targets = atk.bishop(from, all_occ) & target_mask;
        while targets.any() {
            let to = targets.pop_lsb();
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }

    let mut rooks = board.pieces(us, Piece::Rook);
    while rooks.any() {
        let from = rooks.pop_lsb();
        let mut targets = atk.rook(from, all_occ) & target_mask;
        while targets.any() {
            let to = targets.pop_lsb();
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }

    let mut queens = board.pieces(us, Piece::Queen);
    while queens.any() {
        let from = queens.pop_lsb();
        let mut targets = atk.queen(from, all_occ) & target_mask;
        while targets.any() {
            let to = targets.pop_lsb();
            moves.push_move(Move::new(from, to, CAPTURE));
        }
    }
}

#[inline]
fn ep_capture_is_legal(
    board: &Board,
    us: Color,
    them: Color,
    from: Square,
    ep_sq: Square,
    ep_cap_sq: Square,
) -> bool {
    let atk = &*ATTACKS;
    let king_sq = board.king_sq(us);
    let occ_after =
        board.all_occ ^ Bitboard::from(from) ^ Bitboard::from(ep_sq) ^ Bitboard::from(ep_cap_sq);
    let exposed_rook = (board.pieces(them, Piece::Rook) | board.pieces(them, Piece::Queen))
        & atk.rook(king_sq, occ_after);
    let exposed_diag = (board.pieces(them, Piece::Bishop) | board.pieces(them, Piece::Queen))
        & atk.bishop(king_sq, occ_after);

    exposed_rook.is_empty() && exposed_diag.is_empty()
}

// -----------------------------------------------------------------------
// Pawn move generation
// -----------------------------------------------------------------------

fn gen_pawn_moves<const CAPTURES: bool, const QUIETS: bool, S: MoveSink>(
    board: &Board,
    us: Color,
    them: Color,
    their_occ: Bitboard,
    all_occ: Bitboard,
    pinned: Bitboard,
    king_sq: Square,
    check_mask: Bitboard,
    moves: &mut S,
) {
    let atk = &*ATTACKS;
    let pawns = board.pieces(us, Piece::Pawn);
    let free_pawns = pawns & !pinned;
    let pinned_pawns = pawns & pinned;

    if QUIETS {
        gen_unpinned_pawn_quiets(us, free_pawns, all_occ, check_mask, moves);
    }
    if CAPTURES {
        gen_unpinned_pawn_captures(us, free_pawns, their_occ, check_mask, moves);
    }

    let rank2 = match us {
        Color::White => Bitboard::RANK_2,
        Color::Black => Bitboard::RANK_7,
    };
    let push_one = |bb: Bitboard| match us {
        Color::White => bb.north(),
        Color::Black => bb.south(),
    };

    let mut pawn_bb = pinned_pawns;
    while pawn_bb.any() {
        let from = pawn_bb.pop_lsb();
        let from_bb = Bitboard::from(from);

        // --- Quiet pushes ---
        if QUIETS {
            let single_dest = push_one(from_bb) & !all_occ;
            if single_dest.any() {
                let single_sq = single_dest.lsb();
                let in_mask = (single_dest & check_mask).any();
                if in_mask && on_same_ray(from, single_sq, king_sq) {
                    push_pawn_move_flags(from, single_sq, false, moves);
                }

                // Double push from starting rank
                if (from_bb & rank2).any() {
                    let double_dest = push_one(single_dest) & !all_occ & check_mask;
                    if double_dest.any() {
                        let to = double_dest.lsb();
                        if on_same_ray(from, to, king_sq) {
                            moves.push_move(Move::new(from, to, DOUBLE_PUSH));
                        }
                    }
                }
            }
        }

        // --- Captures ---
        if CAPTURES {
            let capture_targets = atk.pawn(us, from) & their_occ & check_mask;
            let mut capture_targets = capture_targets;
            while capture_targets.any() {
                let to = capture_targets.pop_lsb();
                if !on_same_ray(from, to, king_sq) {
                    continue;
                }
                push_pawn_move_flags(from, to, true, moves);
            }
        }
    }

    if CAPTURES && let Some(ep_sq) = board.ep_square() {
        let ep_cap_sq = if us == Color::White {
            Square(ep_sq.0 - 8)
        } else {
            Square(ep_sq.0 + 8)
        };
        let ep_resolves = (check_mask & Bitboard::from(ep_sq)).any()
            || (check_mask & Bitboard::from(ep_cap_sq)).any()
            || check_mask.0 == u64::MAX;

        if ep_resolves {
            let mut attackers = atk.pawn(them, ep_sq) & pawns;
            while attackers.any() {
                let from = attackers.pop_lsb();
                if (pinned & Bitboard::from(from)).any() && !on_same_ray(from, ep_sq, king_sq) {
                    continue;
                }
                if ep_capture_is_legal(board, us, them, from, ep_sq, ep_cap_sq) {
                    moves.push_move(Move::new(from, ep_sq, EN_PASSANT));
                }
            }
        }
    }
}

fn gen_unpinned_pawn_quiets<S: MoveSink>(
    us: Color,
    pawns: Bitboard,
    all_occ: Bitboard,
    check_mask: Bitboard,
    moves: &mut S,
) {
    let empty = !all_occ;
    let (push_from, promo_from, push_offset, double_offset) = match us {
        Color::White => (
            pawns & !Bitboard::RANK_7,
            pawns & Bitboard::RANK_7,
            8i16,
            16i16,
        ),
        Color::Black => (
            pawns & !Bitboard::RANK_2,
            pawns & Bitboard::RANK_2,
            -8i16,
            -16i16,
        ),
    };

    let single = match us {
        Color::White => push_from.north(),
        Color::Black => push_from.south(),
    } & empty;
    let mut targets = single & check_mask;
    while targets.any() {
        let to = targets.pop_lsb();
        moves.push_move(Move::new(
            Square(infra::to_u8(to.0 as i16 - push_offset)),
            to,
            QUIET,
        ));
    }

    let double = match us {
        Color::White => (single & Bitboard::RANK_3).north(),
        Color::Black => (single & Bitboard::RANK_6).south(),
    } & empty
        & check_mask;
    let mut targets = double;
    while targets.any() {
        let to = targets.pop_lsb();
        moves.push_move(Move::new(
            Square(infra::to_u8(to.0 as i16 - double_offset)),
            to,
            DOUBLE_PUSH,
        ));
    }

    let promo = match us {
        Color::White => promo_from.north(),
        Color::Black => promo_from.south(),
    } & empty
        & check_mask;
    let mut targets = promo;
    while targets.any() {
        let to = targets.pop_lsb();
        push_pawn_move_flags(
            Square(infra::to_u8(to.0 as i16 - push_offset)),
            to,
            false,
            moves,
        );
    }
}

fn gen_unpinned_pawn_captures<S: MoveSink>(
    us: Color,
    pawns: Bitboard,
    their_occ: Bitboard,
    check_mask: Bitboard,
    moves: &mut S,
) {
    let (east_targets, west_targets, east_offset, west_offset) = match us {
        Color::White => (pawns.north_east(), pawns.north_west(), 9i16, 7i16),
        Color::Black => (pawns.south_east(), pawns.south_west(), -7i16, -9i16),
    };

    let mut targets = east_targets & their_occ & check_mask;
    while targets.any() {
        let to = targets.pop_lsb();
        push_pawn_move_flags(
            Square(infra::to_u8(to.0 as i16 - east_offset)),
            to,
            true,
            moves,
        );
    }

    let mut targets = west_targets & their_occ & check_mask;
    while targets.any() {
        let to = targets.pop_lsb();
        push_pawn_move_flags(
            Square(infra::to_u8(to.0 as i16 - west_offset)),
            to,
            true,
            moves,
        );
    }
}

/// Emit pawn move(s) — either simple or promotion set.
#[inline]
fn push_pawn_move_flags<S: MoveSink>(from: Square, to: Square, is_capture: bool, moves: &mut S) {
    let is_promo = to.rank() == Rank::R8 || to.rank() == Rank::R1;
    if is_promo {
        if is_capture {
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_QUEEN));
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_ROOK));
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_BISHOP));
            moves.push_move(Move::new(from, to, PROMO_CAPTURE_KNIGHT));
        } else {
            moves.push_move(Move::new(from, to, PROMO_QUEEN));
            moves.push_move(Move::new(from, to, PROMO_ROOK));
            moves.push_move(Move::new(from, to, PROMO_BISHOP));
            moves.push_move(Move::new(from, to, PROMO_KNIGHT));
        }
    } else if is_capture {
        moves.push_move(Move::new(from, to, CAPTURE));
    } else {
        moves.push_move(Move::new(from, to, QUIET));
    }
}

/// Add a single non-pawn move (capture or quiet).
#[inline(always)]
fn add_move<S: MoveSink>(from: Square, to: Square, their_occ: Bitboard, moves: &mut S) {
    if (Bitboard::from(to) & their_occ).any() {
        moves.push_move(Move::new(from, to, CAPTURE));
    } else {
        moves.push_move(Move::new(from, to, QUIET));
    }
}

// -----------------------------------------------------------------------
// Castling
// -----------------------------------------------------------------------

fn gen_castling<S: MoveSink>(
    board: &Board,
    us: Color,
    them: Color,
    all_occ: Bitboard,
    moves: &mut S,
) {
    let (ks_flag, qs_flag, king_sq, ks_rook, qs_rook, ks_empty, qs_empty, ks_safe, qs_safe) = if us
        == Color::White
    {
        (
            CastlingRights::WHITE_KINGSIDE,
            CastlingRights::WHITE_QUEENSIDE,
            Square::E1,
            Square::H1,
            Square::A1,
            // Squares that must be empty for KS / QS
            Bitboard::from(Square::F1) | Bitboard::from(Square::G1),
            Bitboard::from(Square::B1) | Bitboard::from(Square::C1) | Bitboard::from(Square::D1),
            // Squares that must not be attacked for KS / QS (king path)
            [Square::F1, Square::G1],
            [Square::C1, Square::D1],
        )
    } else {
        (
            CastlingRights::BLACK_KINGSIDE,
            CastlingRights::BLACK_QUEENSIDE,
            Square::E8,
            Square::H8,
            Square::A8,
            Bitboard::from(Square::F8) | Bitboard::from(Square::G8),
            Bitboard::from(Square::B8) | Bitboard::from(Square::C8) | Bitboard::from(Square::D8),
            [Square::F8, Square::G8],
            [Square::C8, Square::D8],
        )
    };

    // Verify the rook is actually present (handles FEN edge cases)
    if board.castling.has(ks_flag)
        && (all_occ & ks_empty).is_empty()
        && (board.pieces(us, Piece::Rook) & Bitboard::from(ks_rook)).any()
        && !board.is_attacked(ks_safe[0], them)
        && !board.is_attacked(ks_safe[1], them)
    {
        moves.push_move(Move::new(king_sq, ks_safe[1], CASTLE_KINGSIDE));
    }

    if board.castling.has(qs_flag)
        && (all_occ & qs_empty).is_empty()
        && (board.pieces(us, Piece::Rook) & Bitboard::from(qs_rook)).any()
        && !board.is_attacked(qs_safe[0], them)
        && !board.is_attacked(qs_safe[1], them)
    {
        moves.push_move(Move::new(king_sq, qs_safe[0], CASTLE_QUEENSIDE));
    }
}

// -----------------------------------------------------------------------
// Pin detection
// -----------------------------------------------------------------------

/// Compute the bitboard of our pieces that are pinned to our king.
fn compute_pinned(board: &Board, king_sq: Square, us: Color, them: Color) -> Bitboard {
    crate::diag_count!(board_compute_pinned_calls);
    let our_occ = board.color_occ(us);
    let atk = &*ATTACKS;
    let mut pinned = Bitboard::EMPTY;

    // X-ray diagonal: see through our own pieces to find diagonal pinners
    let bishop_vision = atk.bishop(king_sq, board.all_occ);
    let xray_bishop = atk.bishop(king_sq, board.all_occ ^ (bishop_vision & our_occ));
    let diag_pinners =
        (board.pieces(them, Piece::Bishop) | board.pieces(them, Piece::Queen)) & xray_bishop;
    let mut diag_pinners = diag_pinners;
    while diag_pinners.any() {
        let pinner_sq = diag_pinners.pop_lsb();
        let ray = between(king_sq, pinner_sq);
        let blockers = ray & our_occ;
        if blockers.any() && !blockers.more_than_one() {
            pinned |= blockers;
        }
    }

    // X-ray orthogonal: see through our own pieces to find orthogonal pinners
    let rook_vision = atk.rook(king_sq, board.all_occ);
    let xray_rook = atk.rook(king_sq, board.all_occ ^ (rook_vision & our_occ));
    let ortho_pinners =
        (board.pieces(them, Piece::Rook) | board.pieces(them, Piece::Queen)) & xray_rook;
    let mut ortho_pinners = ortho_pinners;
    while ortho_pinners.any() {
        let pinner_sq = ortho_pinners.pop_lsb();
        let ray = between(king_sq, pinner_sq);
        let blockers = ray & our_occ;
        if blockers.any() && !blockers.more_than_one() {
            pinned |= blockers;
        }
    }

    pinned
}

/// Filter target squares for a pinned piece — it may only move along the pin ray.
#[inline]
fn filter_pinned(from: Square, targets: Bitboard, pinned: Bitboard, king_sq: Square) -> Bitboard {
    if (pinned & Bitboard::from(from)).any() {
        targets & ray_through(from, king_sq)
    } else {
        targets
    }
}

// -----------------------------------------------------------------------
// Geometry helpers
// -----------------------------------------------------------------------

/// Bitboard of squares strictly between `a` and `b` on a rank, file, or diagonal.
/// Returns `EMPTY` if they are not aligned.
pub fn between(a: Square, b: Square) -> Bitboard {
    // Use precomputed table
    BETWEEN[a.index()][b.index()]
}

/// Full ray through both `a` and `b` (including both endpoints).
pub fn ray_through(a: Square, b: Square) -> Bitboard {
    LINE[a.index()][b.index()]
}

/// Returns true if `from`, `to`, and `king` are all on the same rank/file/diagonal.
#[inline]
pub fn on_same_ray(from: Square, to: Square, king: Square) -> bool {
    (ray_through(from, to) & Bitboard::from(king)).any()
}

// -----------------------------------------------------------------------
// Precomputed between / line tables
// -----------------------------------------------------------------------

static BETWEEN: [[Bitboard; 64]; 64] = init_between();
static LINE: [[Bitboard; 64]; 64] = init_line();

const fn abs_i8(v: i8) -> i8 {
    if v < 0 { -v } else { v }
}

const fn signum_i8(v: i8) -> i8 {
    if v > 0 {
        1
    } else if v < 0 {
        -1
    } else {
        0
    }
}

const fn aligned(ar: i8, af: i8, br: i8, bf: i8) -> bool {
    ar == br || af == bf || abs_i8(br - ar) == abs_i8(bf - af)
}

// Const-evaluated init — helpers are non-const; any out-of-range would fail
// the compile-time evaluation itself, so plain casts are sound here.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const fn init_between() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let mut b = 0usize;
        while b < 64 {
            if a != b {
                let ar = (a / 8) as i8;
                let af = (a % 8) as i8;
                let br = (b / 8) as i8;
                let bf = (b % 8) as i8;

                if aligned(ar, af, br, bf) {
                    let sr = signum_i8(br - ar);
                    let sf = signum_i8(bf - af);
                    let mut r = ar + sr;
                    let mut f = af + sf;
                    let mut bits = 0u64;
                    while r != br || f != bf {
                        bits |= 1u64 << ((r as u8) * 8 + f as u8);
                        r += sr;
                        f += sf;
                    }
                    table[a][b] = Bitboard(bits);
                }
            }
            b += 1;
        }
        a += 1;
    }
    table
}

// Const-evaluated init — helpers are non-const; any out-of-range would fail
// the compile-time evaluation itself, so plain casts are sound here.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const fn init_line() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];
    let mut a = 0usize;
    while a < 64 {
        let mut b = 0usize;
        while b < 64 {
            if a != b {
                let ar = (a / 8) as i8;
                let af = (a % 8) as i8;
                let br = (b / 8) as i8;
                let bf = (b % 8) as i8;

                if aligned(ar, af, br, bf) {
                    let sr = signum_i8(br - ar);
                    let sf = signum_i8(bf - af);
                    let mut bits = (1u64 << a) | (1u64 << b);

                    let mut r = ar + sr;
                    let mut f = af + sf;
                    while r >= 0 && r < 8 && f >= 0 && f < 8 {
                        bits |= 1u64 << ((r as u8) * 8 + f as u8);
                        r += sr;
                        f += sf;
                    }

                    let mut r = ar - sr;
                    let mut f = af - sf;
                    while r >= 0 && r < 8 && f >= 0 && f < 8 {
                        bits |= 1u64 << ((r as u8) * 8 + f as u8);
                        r -= sr;
                        f -= sf;
                    }

                    table[a][b] = Bitboard(bits);
                }
            }
            b += 1;
        }
        a += 1;
    }
    table
}
// -----------------------------------------------------------------------
// Extend Board with occ-parameterized attack check (needed for EP)
// -----------------------------------------------------------------------

impl Board {
    pub fn is_attacked_with_occ(&self, sq: Square, attacker: Color, occ: Bitboard) -> bool {
        let atk = &*ATTACKS;
        if (atk.pawn(!attacker, sq) & self.pieces(attacker, Piece::Pawn)).any() {
            return true;
        }
        if (atk.knight(sq) & self.pieces(attacker, Piece::Knight)).any() {
            return true;
        }
        if (atk.king(sq) & self.pieces(attacker, Piece::King)).any() {
            return true;
        }
        if (atk.bishop(sq, occ)
            & (self.pieces(attacker, Piece::Bishop) | self.pieces(attacker, Piece::Queen)))
        .any()
        {
            return true;
        }
        if (atk.rook(sq, occ)
            & (self.pieces(attacker, Piece::Rook) | self.pieces(attacker, Piece::Queen)))
        .any()
        {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod pin_tests {
    use super::*;

    // Independent mailbox ray walk: no slider tables, BETWEEN or LINE table.
    fn ray_walk_pins(board: &Board, us: Color) -> Bitboard {
        let king = board.king_sq(us);
        let mut result = Bitboard::EMPTY;
        for (df, dr) in [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ] {
            let mut file = i16::from(king.0 % 8) + df;
            let mut rank = i16::from(king.0 / 8) + dr;
            let mut blocker = None;
            while (0..8).contains(&file) && (0..8).contains(&rank) {
                let sq = Square(u8::try_from(rank * 8 + file).unwrap());
                if let Some((color, piece)) = board.piece_at(sq) {
                    if color == us && blocker.is_none() {
                        blocker = Some(sq);
                    } else {
                        if color != us
                            && (piece == Piece::Queen
                                || (df != 0 && dr != 0 && piece == Piece::Bishop)
                                || ((df == 0 || dr == 0) && piece == Piece::Rook))
                            && let Some(pinned) = blocker
                        {
                            result |= Bitboard::from(pinned);
                        }
                        break;
                    }
                }
                file += df;
                rank += dr;
            }
        }
        result
    }

    #[test]
    fn sniper_pins_match_independent_ray_walk() {
        let profile = include_str!("../../tests/data/board-v2.tsv");
        let mut rng = 0x9e37_79b9_7f4a_7c15u64;
        let mut checked = 0;
        for row in profile
            .lines()
            .filter(|row| !row.starts_with('#') && !row.is_empty())
        {
            let fields: Vec<_> = row.split('|').collect();
            let mut board = Board::from_fen(fields[2]).unwrap();
            for _ in 0..100 {
                for us in [Color::White, Color::Black] {
                    assert_eq!(
                        compute_pinned(&board, board.king_sq(us), us, !us),
                        ray_walk_pins(&board, us),
                        "pins for {us:?} in {}",
                        board.to_fen()
                    );
                    checked += 1;
                }
                let moves = board.generate_legal_movelist();
                if moves.is_empty() {
                    break;
                }
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                let index = usize::try_from(rng % u64::try_from(moves.len()).unwrap()).unwrap();
                board.make_move(moves.as_slice()[index]);
            }
        }
        assert!(
            checked >= 1000,
            "pin oracle walk unexpectedly thin: {checked}"
        );
    }
}
