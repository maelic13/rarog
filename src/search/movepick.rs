//! The root's move picker and its move scoring.

use std::mem::MaybeUninit;
use std::slice;

use crate::board::attacks::{knight_attacks_of, pawn_attacks_of};
use crate::board::{ATTACKS, Bitboard, Board, CheckInfo, Color, Move, Piece, Threats};
use crate::eval::piece_value;

use super::Searcher;
use super::history::{CONT_PLY_BACK, HistoryTables};

/// Squares on which a pawn lever of the side to move stands advanced:
/// ranks 5 and 6 for White, 3 and 4 for Black.
const LEVER_RANKS: [Bitboard; 2] = [
    Bitboard(0x0000_FFFF_0000_0000),
    Bitboard(0x0000_0000_FFFF_0000),
];
const HOME_RANKS: [Bitboard; 2] = [Bitboard(0xFF), Bitboard(0xFF00_0000_0000_0000)];

// Quiet ordering weights, in history units.
const QUIET_WEIGHT: i32 = 1_763;
const CONT_WEIGHTS: [i32; 4] = [1_614, 1_066, 1_086, 1_051];
/// Leaving a square attacked by a cheaper piece, per moving piece type.
const ESCAPE_BONUS: [i32; 6] = [0, 8_854, 8_170, 14_051, 20_357, 0];
const CHECK_SQUARE_BONUS: i32 = 10_723;
const THREATENED_TO_MALUS: i32 = 8_875;
const OFFENSE_BONUS: i32 = 3_446;
const KING_WALL_PAWN_MALUS: i32 = 4_494;
// Noisy ordering: the victim's value in SEE units scaled to history units,
// a queen-promotion bonus, and in check an ordering by moving piece so
// evasions by cheaper pieces come first.
const VICTIM_WEIGHT: i32 = 18_976;
const QUEEN_PROMOTION_BONUS: i32 = 4_558;
const IN_CHECK_BASE: i32 = 200_000;
const IN_CHECK_PER_PIECE: i32 = 20_000;
/// Good-noisy SEE threshold: `-score / DIVISOR + OFFSET`, in SEE units.
const GOOD_NOISY_SEE_DIVISOR: i32 = 63;
const GOOD_NOISY_SEE_OFFSET: i32 = 87;

#[derive(Copy, Clone, Default)]
pub(crate) struct ScoredMove {
    pub mv: Move,
    pub score: i32,
}

// See MoveList in board/moves.rs: plain initialized arrays cost -10% NPS.
// Unsafe confined to the slice accessors with a local prefix-initialization
// invariant.
pub(super) struct ScoredMoveList {
    moves: [MaybeUninit<ScoredMove>; 256],
    len: usize,
}

impl ScoredMoveList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            moves: [const { MaybeUninit::uninit() }; 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub(super) fn push(&mut self, mv: Move, score: i32) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len].write(ScoredMove { mv, score });
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub(crate) fn as_mut_slice(&mut self) -> &mut [ScoredMove] {
        // SAFETY: only the initialized prefix below `len` is exposed.
        unsafe { slice::from_raw_parts_mut(self.moves.as_mut_ptr().cast::<ScoredMove>(), self.len) }
    }
}

/// Selection step: move the highest-scored entry of `moves[index..]` into
/// `moves[index]` and return it. Ties resolve to the earliest entry.
pub(super) fn pick_next(moves: &mut [ScoredMove], index: usize) -> ScoredMove {
    let tail = &mut moves[index..];
    let mut best = 0;
    let mut best_score = tail[0].score;
    for (offset, candidate) in tail.iter().enumerate().skip(1) {
        // A new maximum turns up at no predictable point in the scan, so a
        // branch here mispredicts at every one; a conditional move does not.
        let better = candidate.score > best_score;
        best = std::hint::select_unpredictable(better, offset, best);
        best_score = std::hint::select_unpredictable(better, candidate.score, best_score);
    }
    tail.swap(0, best);
    tail[0]
}

pub(super) fn diversify_root_scores(moves: &mut [ScoredMove], offset: usize) {
    moves.sort_unstable_by_key(|m| std::cmp::Reverse(m.score));
    if offset < moves.len() {
        moves[offset].score = moves[0].score.saturating_add(1_000_000);
    }
}

/// The root picker's stages, visited in declaration order and never
/// revisited.
///
/// Contract, covered by the tests below: the TT move first, at most once
/// (the caller validates it), then every other generated legal move once,
/// best first.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Stage {
    TtMove,
    /// Everything except the TT move, best first.
    AllRemaining,
    Done,
}

/// The root's picker: its whole scored list.
pub(super) struct MovePicker {
    scored: ScoredMoveList,
    index: usize,
    tt_move: Move,
    stage: Stage,
}

impl MovePicker {
    pub(super) fn full(scored: ScoredMoveList, tt_move: Move) -> Self {
        Self {
            scored,
            index: 0,
            tt_move,
            stage: Stage::TtMove,
        }
    }

    /// Yield the next move.
    pub(super) fn next(&mut self) -> Option<ScoredMove> {
        loop {
            match self.stage {
                Stage::TtMove => {
                    self.stage = Stage::AllRemaining;
                    if !self.tt_move.is_null() {
                        return Some(tt_scored_move(self.tt_move));
                    }
                }
                Stage::AllRemaining => {
                    while self.index < self.scored.len() {
                        let picked = pick_next(self.scored.as_mut_slice(), self.index);
                        self.index += 1;
                        if picked.mv != self.tt_move {
                            return Some(picked);
                        }
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => return None,
            }
        }
    }
}

fn tt_scored_move(mv: Move) -> ScoredMove {
    ScoredMove {
        mv,
        score: 30_000_000,
    }
}

/// Attacks of every square in `sources` by a piece moving like `attacks`.
fn attacks_from(sources: Bitboard, attacks: impl Fn(crate::board::Square) -> Bitboard) -> Bitboard {
    sources.fold(Bitboard::EMPTY, |all, sq| all | attacks(sq))
}

/// Node-invariant inputs of quiet scoring, resolved once per node.
pub(super) struct QuietContext {
    pub(super) stm: Color,
    /// Squares attacked by enemy pieces cheaper than each moving piece type.
    pub(super) threatened: [Bitboard; 6],
    /// Safe squares from which each piece type would attack an enemy piece.
    pub(super) offense: [Bitboard; 6],
    pub(super) check_squares: [Bitboard; 6],
    pub(super) wall_pawns: Bitboard,
    pub(super) pawn_row: usize,
    pub(super) cont: [Option<usize>; 4],
}

impl Searcher {
    pub(super) fn quiet_context(
        &self,
        board: &Board,
        threats: &Threats,
        ply: usize,
        node_ci: &mut Option<CheckInfo>,
    ) -> QuietContext {
        let stm = board.side_to_move();
        let them = !stm;
        let by = &threats.by_piece;
        let all = threats.all;
        let occ = board.occupied();
        let minor =
            by[Piece::Pawn as usize] | by[Piece::Knight as usize] | by[Piece::Bishop as usize];
        let rook = minor | by[Piece::Rook as usize];
        let threatened = [
            Bitboard::EMPTY,
            by[Piece::Pawn as usize],
            by[Piece::Pawn as usize],
            minor,
            rook,
            Bitboard::EMPTY,
        ];
        let non_pawn = by[Piece::Knight as usize]
            | by[Piece::Bishop as usize]
            | by[Piece::Rook as usize]
            | by[Piece::Queen as usize]
            | by[Piece::King as usize];
        let atk = &ATTACKS;
        let their = |piece| board.pieces(them, piece);
        let knight_targets =
            (their(Piece::Bishop) & !all) | their(Piece::Rook) | their(Piece::Queen);
        // A pawn of ours attacks their piece from the squares one of their
        // pawns there would attack; the same holds for knights.
        let pawn_offense = (pawn_attacks_of(them, board.color_occ(them)) & !all)
            | (by[Piece::Pawn as usize] & LEVER_RANKS[stm as usize] & !non_pawn);
        let offense = [
            pawn_offense,
            knight_attacks_of(knight_targets) & !all,
            attacks_from(their(Piece::Rook), |sq| atk.bishop(sq, occ)) & !all,
            (Bitboard::FILE_A << u32::from(board.king_sq(them).0 % 8)) & !all,
            (attacks_from(their(Piece::Bishop) & !all, |sq| atk.rook(sq, occ))
                | attacks_from(their(Piece::Rook) & !all, |sq| atk.bishop(sq, occ)))
                & !all,
            Bitboard::EMPTY,
        ];
        let king = board.king_sq(stm);
        let wall_pawns = if HOME_RANKS[stm as usize].contains(king) {
            atk.king(king) & (board.pieces(stm, Piece::Pawn) | board.pieces(them, Piece::Pawn))
        } else {
            Bitboard::EMPTY
        };
        // The node's check masks, shared with the move loop's check tests:
        // the first of the two to need them computes them.
        let check_info = node_ci.get_or_insert_with(|| board.check_info());
        #[cfg(debug_assertions)]
        {
            let fresh = board.check_info();
            debug_assert!(
                Piece::ALL.iter().all(|&piece| {
                    check_info.direct_check_squares(piece) == fresh.direct_check_squares(piece)
                }),
                "the shared check masks are not this position's"
            );
        }
        let mut check_squares = [Bitboard::EMPTY; 6];
        for piece in Piece::ALL {
            check_squares[piece as usize] = check_info.direct_check_squares(piece);
        }
        let mut cont = [None; 4];
        for (slot, back) in CONT_PLY_BACK.into_iter().enumerate() {
            cont[slot] = self.cont_context_back(ply, back);
        }
        QuietContext {
            stm,
            threatened,
            offense,
            check_squares,
            wall_pawns,
            pawn_row: HistoryTables::pawn_row(board.pawn_key()),
            cont,
        }
    }

    /// Ordering score of one quiet move.
    fn quiet_score(
        &self,
        board: &Board,
        threats: &Threats,
        ctx: &QuietContext,
        hist: &HistoryTables,
        mv: Move,
    ) -> i32 {
        let piece = board.moving_piece(mv);
        let pt = piece as usize;
        let (from, to) = (mv.from_sq(), mv.to_sq());
        let quiet = hist.quiet(threats.all, ctx.stm, mv);
        let mut score = QUIET_WEIGHT * quiet / 1024 + hist.pawn(ctx.pawn_row, ctx.stm, piece, to);
        for (slot, context) in ctx.cont.iter().enumerate() {
            if let Some(context) = *context {
                score += CONT_WEIGHTS[slot] * hist.cont(context, ctx.stm, piece, to) / 1024;
            }
        }
        if ctx.threatened[pt].contains(from) {
            score += ESCAPE_BONUS[pt];
        }
        if ctx.check_squares[pt].contains(to) {
            score += CHECK_SQUARE_BONUS;
        }
        if ctx.threatened[pt].contains(to) {
            score -= THREATENED_TO_MALUS;
        }
        if ctx.offense[pt].contains(to) {
            score += OFFENSE_BONUS;
        }
        if ctx.wall_pawns.contains(from) {
            score -= KING_WALL_PAWN_MALUS;
        }
        score
    }

    /// Ordering score of a capture or promotion: the victim, its history,
    /// a queen promotion, and in check the moving piece.
    pub(super) fn noisy_score(&self, board: &Board, threats: &Threats, mv: Move) -> i32 {
        let piece = board.moving_piece(mv);
        let captured = board.captured_piece(mv);
        let victim = captured.map_or(0, piece_value);
        let mut score = VICTIM_WEIGHT * victim / 1024
            + self.td.hist.noisy(
                threats.all,
                board.side_to_move(),
                piece,
                mv.to_sq(),
                captured,
            );
        if mv.is_promo() && mv.promo_piece() == Piece::Queen {
            score += QUEEN_PROMOTION_BONUS;
        }
        if board.is_in_check() {
            score += IN_CHECK_BASE - IN_CHECK_PER_PIECE * piece as i32;
        }
        score
    }

    /// Score a whole move list for the root picker: TT move, good noisy
    /// moves, quiets, bad noisy moves. Appends to the caller's list: the list
    /// is 4 KB, and returning it by value copied it at every node.
    pub(super) fn score_moves(
        &self,
        board: &Board,
        threats: &Threats,
        moves: &[Move],
        tt_move: Move,
        ply: usize,
        scored: &mut ScoredMoveList,
        node_ci: &mut Option<CheckInfo>,
    ) {
        let mut ctx = None;
        for &mv in moves {
            if mv == tt_move {
                scored.push(mv, 30_000_000);
            } else if mv.is_noisy() {
                let noisy = self.noisy_score(board, threats, mv);
                let threshold = -noisy / GOOD_NOISY_SEE_DIVISOR + GOOD_NOISY_SEE_OFFSET;
                if board.see_ge(mv, threshold) {
                    scored.push(mv, 20_000_000 + noisy);
                } else {
                    scored.push(mv, -2_000_000 + noisy);
                }
            } else {
                let ctx =
                    ctx.get_or_insert_with(|| self.quiet_context(board, threats, ply, node_ci));
                let score = self.quiet_score(board, threats, ctx, &self.td.hist, mv);
                scored.push(mv, score);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `as_mut_slice` exposes `..len` as initialized, so `len` must never pass
    /// the last slot a push wrote: the push after a full list panics
    /// instead (the debug assertion or, in release, the checked index).
    #[test]
    #[should_panic]
    fn a_push_past_the_last_slot_panics() {
        let mut list = ScoredMoveList::new();
        let mv = Move::from_uci("e2e4").expect("valid UCI move shape");
        for _ in 0..256 {
            list.push(mv, 0);
        }
        assert_eq!(list.as_mut_slice().len(), 256);
        list.push(mv, 0);
    }

    fn drain(picker: &mut MovePicker) -> Vec<Move> {
        let mut out = Vec::new();
        while let Some(picked) = picker.next() {
            out.push(picked.mv);
        }
        out
    }

    const FENS: [&str; 6] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1",
        // In check: the root picker serves evasions too.
        "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
        "4k3/8/8/8/8/5n2/8/3BK3 w - - 0 1",
        // Promotions, with and without capture.
        "1r2k3/2P5/8/8/8/8/1p6/2N1K3 b - - 0 1",
    ];

    /// Every legal move exactly once, with and without a TT move, including
    /// in check and with promotions.
    #[test]
    fn pickers_emit_every_legal_move_exactly_once() {
        let searcher = Searcher::default();
        for fen in FENS {
            let board = Board::from_fen(fen).expect("valid FEN");
            let threats = board.threats();
            let mut legal: Vec<Move> = board.generate_legal_movelist().as_slice().to_vec();
            legal.sort_unstable_by_key(|m| m.0);
            let tt_candidates = [Move::NULL, legal[0], legal[legal.len() - 1]];
            for tt in tt_candidates {
                let mut scored = ScoredMoveList::new();
                searcher.score_moves(&board, &threats, &legal, tt, 0, &mut scored, &mut None);
                let mut full = MovePicker::full(scored, tt);
                let mut got = drain(&mut full);
                got.sort_unstable_by_key(|m| m.0);
                assert_eq!(got, legal, "full picker, fen {fen}, tt {tt}");
            }
        }
    }

    #[test]
    fn checking_and_escaping_quiets_are_ordered_first() {
        let searcher = Searcher::default();
        // The white knight on d4 is attacked by the c5 pawn; Nf5 escapes to a
        // safe square, Rh8 checks along the rank, Kb1 does neither.
        let board = Board::from_fen("4k3/8/8/2p5/3N4/8/8/K6R w - - 0 1").expect("valid FEN");
        let threats = board.threats();
        let legal = board.generate_legal_moves();
        let mut scored = ScoredMoveList::new();
        searcher.score_moves(
            &board,
            &threats,
            &legal,
            Move::NULL,
            0,
            &mut scored,
            &mut None,
        );
        let score_of = |uci: &str, scored: &mut ScoredMoveList| {
            let mv = board.parse_move(uci).expect("legal");
            scored
                .as_mut_slice()
                .iter()
                .find(|s| s.mv == mv)
                .expect("scored")
                .score
        };
        let check = score_of("h1h8", &mut scored);
        let escape = score_of("d4f5", &mut scored);
        let idle = score_of("a1b1", &mut scored);
        assert!(check >= idle + CHECK_SQUARE_BONUS, "{check} vs {idle}");
        assert!(escape >= idle + ESCAPE_BONUS[Piece::Knight as usize] - THREATENED_TO_MALUS);
    }
}
