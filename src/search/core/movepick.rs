//! The staged move picker and move scoring.

use std::mem::MaybeUninit;
use std::slice;

use crate::board::{ATTACKS, Bitboard, Board, Color, Move, MoveList, Piece, Threats};
use crate::eval::piece_value;

use super::Searcher;
use super::history::{CONT_PLY_BACK, HistoryTables};

/// Sentinel SEE for a TT move emitted before its SEE was computed.
pub(super) const SEE_UNKNOWN: i16 = i16::MIN;

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
/// A non-capturing promotion is generated with the quiets; it is ordered
/// ahead of every quiet by this offset on its noisy score.
const QUIET_PROMOTION_OFFSET: i32 = 1_000_000;
/// Good-noisy SEE threshold: `-score / DIVISOR + OFFSET`, in SEE units.
const GOOD_NOISY_SEE_DIVISOR: i32 = 63;
const GOOD_NOISY_SEE_OFFSET: i32 = 87;

#[derive(Copy, Clone, Default)]
pub(crate) struct ScoredMove {
    pub mv: Move,
    pub score: i32,
    /// `0` for a move ordered as good, `-1` for a bad noisy move,
    /// [`SEE_UNKNOWN`] for a TT capture.
    pub see: i16,
    /// The history pruning and reductions read for a quiet move.
    pub(super) quiet_history: i32,
    /// A quiet-list move the picker still emits after quiets are skipped: a
    /// direct check or a non-capturing promotion.
    pub(super) survives_skip: bool,
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
    fn push(&mut self, mv: Move, score: i32, see: i16, quiet_history: i32) {
        self.push_quiet(mv, score, see, quiet_history, false);
    }

    #[inline(always)]
    fn push_quiet(
        &mut self,
        mv: Move,
        score: i32,
        see: i16,
        quiet_history: i32,
        survives_skip: bool,
    ) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len].write(ScoredMove {
            mv,
            score,
            see,
            quiet_history,
            survives_skip,
        });
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
        if candidate.score > best_score {
            best = offset;
            best_score = candidate.score;
        }
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

/// A noisy move is a capture or a promotion.
#[inline(always)]
pub(super) fn is_noisy(mv: Move) -> bool {
    mv.is_capture() || mv.is_promo()
}

/// Picker stages, visited in declaration order and never revisited.
///
/// Contract, covered by the tests below:
///   ORDER      staged: TtMove, GoodNoisy, Quiets, BadNoisy. The noisy
///              stages hold captures; the generator puts non-capturing
///              promotions with the quiets, where they are ordered first.
///              full (root): TtMove, AllRemaining.
///   DUPLICATES the TT move is emitted at most once and every later stage
///              filters it out.
///   LEGALITY   both paths emit only generated legal moves; the TT move is
///              validated by the caller.
///   SKIP       once the caller asks to skip quiets, the only quiet-list
///              moves still emitted are direct checks and non-capturing
///              promotions; bad noisy moves still are.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Stage {
    TtMove,
    /// Noisy moves whose SEE clears a threshold that falls as their score
    /// rises, best first.
    GoodNoisy,
    /// Generate and score the quiets, once.
    GenerateQuiets,
    Quiets,
    /// Noisy moves that failed the good-noisy threshold, in the order they
    /// failed it.
    BadNoisy,
    /// Full path: everything except the TT move, best-first.
    AllRemaining,
    Done,
}

pub(super) enum MovePicker {
    Full {
        scored: ScoredMoveList,
        index: usize,
        tt_move: Move,
        stage: Stage,
    },
    /// One buffer: noisy moves in `[0, noisy_len)`, quiets appended behind
    /// them. As good noisy moves are selected, a move that fails its SEE
    /// threshold is swapped down into `[0, bad_len)`, which the emitted good
    /// moves no longer need, so the bad moves end up in failure order.
    Staged {
        moves: ScoredMoveList,
        noisy_len: usize,
        bad_len: usize,
        noisy_index: usize,
        quiet_index: usize,
        bad_index: usize,
        good_noisy_emitted: usize,
        /// Pinned set from capture generation, reused for the quiets.
        pinned: Bitboard,
        tt_move: Move,
        stage: Stage,
        ply: usize,
    },
}

impl MovePicker {
    pub(super) fn full(scored: ScoredMoveList, tt_move: Move) -> Self {
        Self::Full {
            scored,
            index: 0,
            tt_move,
            stage: Stage::TtMove,
        }
    }

    pub(super) fn staged(
        searcher: &Searcher,
        board: &mut Board,
        threats: &Threats,
        tt_move: Move,
        ply: usize,
    ) -> Self {
        let mut noisy = MoveList::new();
        let pinned = board.generate_legal_captures_pinned_into(&mut noisy);
        let mut moves = ScoredMoveList::new();
        for &mv in noisy.as_slice() {
            if mv != tt_move {
                moves.push(mv, searcher.noisy_score(board, threats, mv), 0, 0);
            }
        }
        Self::Staged {
            noisy_len: moves.len(),
            moves,
            bad_len: 0,
            noisy_index: 0,
            quiet_index: 0,
            bad_index: 0,
            good_noisy_emitted: 0,
            pinned,
            tt_move,
            stage: Stage::TtMove,
            ply,
        }
    }

    /// The stage the picker is in: the stage of the last emitted move, except
    /// after the TT move, which reports the stage that follows it.
    pub(super) fn stage(&self) -> Stage {
        match self {
            Self::Full { stage, .. } | Self::Staged { stage, .. } => *stage,
        }
    }

    /// Yield the next move. With `skip_quiets` set no further quiet is
    /// emitted; the picker goes straight to the bad noisy moves.
    pub(super) fn next(
        &mut self,
        searcher: &Searcher,
        board: &mut Board,
        threats: &Threats,
        skip_quiets: bool,
    ) -> Option<ScoredMove> {
        match self {
            Self::Full {
                scored,
                index,
                tt_move,
                stage,
            } => loop {
                match *stage {
                    Stage::TtMove => {
                        *stage = Stage::AllRemaining;
                        if !tt_move.is_null() {
                            return Some(tt_scored_move(*tt_move));
                        }
                    }
                    Stage::AllRemaining => {
                        while *index < scored.len() {
                            let picked = pick_next(scored.as_mut_slice(), *index);
                            *index += 1;
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::Done;
                    }
                    _ => return None,
                }
            },
            Self::Staged {
                moves,
                noisy_len,
                bad_len,
                noisy_index,
                quiet_index,
                bad_index,
                good_noisy_emitted,
                pinned,
                tt_move,
                stage,
                ply,
            } => loop {
                match *stage {
                    Stage::TtMove => {
                        *stage = Stage::GoodNoisy;
                        if !tt_move.is_null() {
                            return Some(tt_scored_move(*tt_move));
                        }
                    }
                    Stage::GoodNoisy => {
                        while *noisy_index < *noisy_len {
                            let picked =
                                pick_next(&mut moves.as_mut_slice()[..*noisy_len], *noisy_index);
                            // After three good noisy moves under a quiet TT
                            // move, only material-winning ones stay good.
                            let threshold = if !tt_move.is_null()
                                && !is_noisy(*tt_move)
                                && *good_noisy_emitted > 2
                            {
                                1
                            } else {
                                -picked.score / GOOD_NOISY_SEE_DIVISOR + GOOD_NOISY_SEE_OFFSET
                            };
                            if board.see_ge(picked.mv, threshold) {
                                *noisy_index += 1;
                                *good_noisy_emitted += 1;
                                return Some(picked);
                            }
                            moves.as_mut_slice().swap(*noisy_index, *bad_len);
                            *bad_len += 1;
                            *noisy_index += 1;
                        }
                        // Quiets are generated even when already skipped:
                        // direct checks and promotions among them survive.
                        *stage = Stage::GenerateQuiets;
                    }
                    Stage::GenerateQuiets => {
                        let mut quiets = MoveList::new();
                        board.generate_legal_quiets_pinned_into(*pinned, &mut quiets);
                        searcher.append_quiet_moves(
                            board,
                            threats,
                            quiets.as_slice(),
                            *tt_move,
                            *ply,
                            moves,
                        );
                        *stage = Stage::Quiets;
                    }
                    Stage::Quiets => {
                        while *noisy_len + *quiet_index < moves.len() {
                            let picked =
                                pick_next(&mut moves.as_mut_slice()[*noisy_len..], *quiet_index);
                            *quiet_index += 1;
                            if !skip_quiets || picked.survives_skip {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::BadNoisy;
                    }
                    Stage::BadNoisy => {
                        if *bad_index < *bad_len {
                            let mut picked = moves.as_mut_slice()[*bad_index];
                            *bad_index += 1;
                            picked.see = -1;
                            return Some(picked);
                        }
                        *stage = Stage::Done;
                    }
                    _ => return None,
                }
            },
        }
    }
}

fn tt_scored_move(mv: Move) -> ScoredMove {
    let see = if mv.is_capture() { SEE_UNKNOWN } else { 0 };
    ScoredMove {
        mv,
        score: 30_000_000,
        see,
        quiet_history: 0,
        survives_skip: false,
    }
}

/// Attacks of every square in `sources` by a piece moving like `attacks`.
fn attacks_from(sources: Bitboard, attacks: impl Fn(crate::board::Square) -> Bitboard) -> Bitboard {
    sources.fold(Bitboard::EMPTY, |all, sq| all | attacks(sq))
}

/// Squares attacked by pawns of `color` standing on `pawns`.
fn pawn_attacks(pawns: Bitboard, color: Color) -> Bitboard {
    attacks_from(pawns, |sq| ATTACKS.pawn(color, sq))
}

/// Node-invariant inputs of quiet scoring, resolved once per node.
struct QuietContext {
    stm: Color,
    /// Squares attacked by enemy pieces cheaper than each moving piece type.
    threatened: [Bitboard; 6],
    /// Safe squares from which each piece type would attack an enemy piece.
    offense: [Bitboard; 6],
    check_squares: [Bitboard; 6],
    wall_pawns: Bitboard,
    pawn_row: usize,
    cont: [Option<usize>; 4],
}

impl Searcher {
    fn quiet_context(&self, board: &Board, threats: &Threats, ply: usize) -> QuietContext {
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
        let their = |piece| board.pieces(them, piece);
        let knight_targets =
            (their(Piece::Bishop) & !all) | their(Piece::Rook) | their(Piece::Queen);
        let pawn_offense = (pawn_attacks(board.color_occ(them), them) & !all)
            | (by[Piece::Pawn as usize] & LEVER_RANKS[stm as usize] & !non_pawn);
        let offense = [
            pawn_offense,
            attacks_from(knight_targets, |sq| ATTACKS.knight(sq)) & !all,
            attacks_from(their(Piece::Rook), |sq| ATTACKS.bishop(sq, occ)) & !all,
            (Bitboard::FILE_A << u32::from(board.king_sq(them).0 % 8)) & !all,
            (attacks_from(their(Piece::Bishop) & !all, |sq| ATTACKS.rook(sq, occ))
                | attacks_from(their(Piece::Rook) & !all, |sq| ATTACKS.bishop(sq, occ)))
                & !all,
            Bitboard::EMPTY,
        ];
        let king = board.king_sq(stm);
        let wall_pawns = if HOME_RANKS[stm as usize].contains(king) {
            ATTACKS.king(king) & (board.pieces(stm, Piece::Pawn) | board.pieces(them, Piece::Pawn))
        } else {
            Bitboard::EMPTY
        };
        let check_info = board.check_info();
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

    /// Score and append quiet moves: histories, continuation terms, and the
    /// threat, check, offense and king-wall terms of the moving piece.
    fn append_quiet_moves(
        &self,
        board: &Board,
        threats: &Threats,
        quiets: &[Move],
        tt_move: Move,
        ply: usize,
        out: &mut ScoredMoveList,
    ) {
        if quiets.is_empty() {
            return;
        }
        let ctx = self.quiet_context(board, threats, ply);
        let hist = &self.td.hist;
        for &mv in quiets {
            if mv == tt_move {
                continue;
            }
            if mv.is_promo() {
                let score = QUIET_PROMOTION_OFFSET + self.noisy_score(board, threats, mv);
                out.push_quiet(mv, score, 0, 0, true);
                continue;
            }
            let (score, pruning_history) = self.quiet_score(board, threats, &ctx, hist, mv);
            let direct_check =
                ctx.check_squares[board.moving_piece(mv) as usize].contains(mv.to_sq());
            out.push_quiet(mv, score, 0, pruning_history, direct_check);
        }
    }

    /// Ordering score and pruning history of one quiet move.
    fn quiet_score(
        &self,
        board: &Board,
        threats: &Threats,
        ctx: &QuietContext,
        hist: &HistoryTables,
        mv: Move,
    ) -> (i32, i32) {
        let piece = board.moving_piece(mv);
        let pt = piece as usize;
        let (from, to) = (mv.from_sq(), mv.to_sq());
        let quiet = hist.quiet(threats.all, ctx.stm, mv);
        let mut score = QUIET_WEIGHT * quiet / 1024 + hist.pawn(ctx.pawn_row, ctx.stm, piece, to);
        let mut pruning_history = quiet;
        for (slot, context) in ctx.cont.iter().enumerate() {
            if let Some(context) = *context {
                let value = hist.cont(context, ctx.stm, piece, to);
                score += CONT_WEIGHTS[slot] * value / 1024;
                if slot < 2 {
                    pruning_history += value;
                }
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
        (score, pruning_history)
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

    /// Score a whole move list for the full picker: TT move, good noisy
    /// moves, quiets, bad noisy moves.
    pub(super) fn score_moves(
        &self,
        board: &Board,
        threats: &Threats,
        moves: &[Move],
        tt_move: Move,
        ply: usize,
    ) -> ScoredMoveList {
        let mut scored = ScoredMoveList::new();
        let mut ctx = None;
        for &mv in moves {
            if mv == tt_move {
                scored.push(mv, 30_000_000, 0, 0);
            } else if is_noisy(mv) {
                let noisy = self.noisy_score(board, threats, mv);
                let threshold = -noisy / GOOD_NOISY_SEE_DIVISOR + GOOD_NOISY_SEE_OFFSET;
                if board.see_ge(mv, threshold) {
                    scored.push(mv, 20_000_000 + noisy, 0, 0);
                } else {
                    scored.push(mv, -2_000_000 + noisy, -1, 0);
                }
            } else {
                let ctx = ctx.get_or_insert_with(|| self.quiet_context(board, threats, ply));
                let (score, history) = self.quiet_score(board, threats, ctx, &self.td.hist, mv);
                scored.push(mv, score, 0, history);
            }
        }
        scored
    }

    /// Score captures and promotions for quiescence and ProbCut: SEE sign
    /// first, then the noisy score.
    pub(super) fn score_tactical_moves(
        &self,
        board: &Board,
        threats: &Threats,
        moves: &[Move],
        tt_move: Move,
    ) -> ScoredMoveList {
        let mut scored = ScoredMoveList::new();
        for &mv in moves {
            let good = !mv.is_capture() || board.see_ge(mv, 0);
            let noisy = self.noisy_score(board, threats, mv);
            let (score, see) = if mv == tt_move {
                (30_000_000, if good { 0 } else { -1 })
            } else if good {
                (20_000_000 + noisy, 0)
            } else {
                (-2_000_000 + noisy, -1)
            };
            scored.push(mv, score, see, 0);
        }
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(
        picker: &mut MovePicker,
        searcher: &Searcher,
        board: &mut Board,
        threats: &Threats,
    ) -> Vec<(Move, Stage)> {
        let mut out = Vec::new();
        while let Some(picked) = picker.next(searcher, board, threats, false) {
            out.push((picked.mv, picker.stage()));
        }
        out
    }

    const FENS: [&str; 6] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1",
        // In check: the staged picker serves evasions too.
        "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
        "4k3/8/8/8/8/5n2/8/3BK3 w - - 0 1",
        // Promotions, with and without capture.
        "1r2k3/2P5/8/8/8/8/1p6/2N1K3 b - - 0 1",
    ];

    /// Every legal move exactly once, from both pickers, with and without a
    /// TT move, including in check and with promotions.
    #[test]
    fn pickers_emit_every_legal_move_exactly_once() {
        let searcher = Searcher::default();
        for fen in FENS {
            let mut board = Board::from_fen(fen).expect("valid FEN");
            let threats = board.threats();
            let mut legal: Vec<Move> = board.generate_legal_movelist().as_slice().to_vec();
            legal.sort_unstable_by_key(|m| m.0);
            let tt_candidates = [Move::NULL, legal[0], legal[legal.len() - 1]];
            for tt in tt_candidates {
                let mut staged = MovePicker::staged(&searcher, &mut board, &threats, tt, 0);
                let mut got: Vec<Move> = drain(&mut staged, &searcher, &mut board, &threats)
                    .into_iter()
                    .map(|(mv, _)| mv)
                    .collect();
                got.sort_unstable_by_key(|m| m.0);
                assert_eq!(got, legal, "staged picker, fen {fen}, tt {tt}");

                let scored = searcher.score_moves(&board, &threats, &legal, tt, 0);
                let mut full = MovePicker::full(scored, tt);
                let mut got: Vec<Move> = drain(&mut full, &searcher, &mut board, &threats)
                    .into_iter()
                    .map(|(mv, _)| mv)
                    .collect();
                got.sort_unstable_by_key(|m| m.0);
                assert_eq!(got, legal, "full picker, fen {fen}, tt {tt}");
            }
        }
    }

    /// Staged order: the TT move first, then good noisy, quiets, bad noisy,
    /// never returning to an earlier stage.
    #[test]
    fn staged_picker_visits_stages_in_order() {
        let searcher = Searcher::default();
        let order = |stage| match stage {
            Stage::TtMove => 0,
            Stage::GoodNoisy => 1,
            Stage::Quiets => 2,
            Stage::BadNoisy => 3,
            other => panic!("move emitted from {other:?}"),
        };
        for fen in FENS {
            let mut board = Board::from_fen(fen).expect("valid FEN");
            let threats = board.threats();
            let legal = board.generate_legal_moves();
            let mut picker = MovePicker::staged(&searcher, &mut board, &threats, legal[0], 0);
            let emitted = drain(&mut picker, &searcher, &mut board, &threats);
            assert_eq!(emitted[0].0, legal[0], "fen {fen}");
            let emitted = &emitted[1..];
            for pair in emitted.windows(2) {
                assert!(order(pair[0].1) <= order(pair[1].1), "fen {fen}: {pair:?}");
            }
            for &(mv, stage) in emitted {
                match stage {
                    Stage::GoodNoisy | Stage::BadNoisy => {
                        assert!(mv.is_capture(), "{mv} in {stage:?}");
                    }
                    Stage::Quiets => assert!(!mv.is_capture(), "{mv} in Quiets"),
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn losing_capture_comes_after_the_quiets() {
        let searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let threats = board.threats();
        let losing = board.parse_move("c3d5").expect("legal capture");
        assert!(!board.see_ge(losing, 0));
        let mut picker = MovePicker::staged(&searcher, &mut board, &threats, Move::NULL, 0);
        let emitted = drain(&mut picker, &searcher, &mut board, &threats);
        let position = emitted
            .iter()
            .position(|(mv, _)| *mv == losing)
            .expect("emitted");
        assert_eq!(emitted[position].1, Stage::BadNoisy);
        assert!(
            emitted[..position]
                .iter()
                .any(|(_, stage)| *stage == Stage::Quiets)
        );
    }

    /// Skipping quiets drops every not-yet-emitted quiet and keeps every bad
    /// noisy move.
    #[test]
    fn skipping_quiets_keeps_the_bad_noisy_moves() {
        let searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let threats = board.threats();
        let losing = board.parse_move("c3d5").expect("legal capture");
        let mut picker = MovePicker::staged(&searcher, &mut board, &threats, Move::NULL, 0);
        let mut emitted = Vec::new();
        let mut skip = false;
        while let Some(picked) = picker.next(&searcher, &mut board, &threats, skip) {
            if picker.stage() == Stage::Quiets {
                skip = true;
            }
            emitted.push((picked.mv, picker.stage()));
        }
        let quiets = emitted.iter().filter(|(_, s)| *s == Stage::Quiets).count();
        assert_eq!(quiets, 1, "one quiet before the skip");
        assert!(
            emitted
                .iter()
                .any(|(mv, s)| *mv == losing && *s == Stage::BadNoisy)
        );
        assert!(picker.next(&searcher, &mut board, &threats, true).is_none());
        assert!(
            picker
                .next(&searcher, &mut board, &threats, false)
                .is_none()
        );
    }

    /// After the skip, direct checks and non-capturing promotions still come
    /// out of the quiet stage; ordinary quiets do not.
    #[test]
    fn skipping_quiets_still_emits_direct_checks_and_promotions() {
        let searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/1P6/8/8/8/8/8/R3K3 w - - 0 1").expect("valid FEN");
        let threats = board.threats();
        let check_info = board.check_info();
        let mut picker = MovePicker::staged(&searcher, &mut board, &threats, Move::NULL, 0);
        let first = picker
            .next(&searcher, &mut board, &threats, false)
            .expect("a quiet first");
        let mut after_skip = Vec::new();
        while let Some(picked) = picker.next(&searcher, &mut board, &threats, true) {
            after_skip.push(picked.mv);
        }
        let legal = board.generate_legal_moves();
        for mv in legal {
            if mv == first.mv || mv.is_capture() {
                continue;
            }
            let survives = mv.is_promo()
                || check_info
                    .direct_check_squares(board.moving_piece(mv))
                    .contains(mv.to_sq());
            assert_eq!(after_skip.contains(&mv), survives, "{mv}");
        }
        assert!(after_skip.iter().any(|mv| mv.is_promo()));
        assert!(
            after_skip.iter().any(|mv| !mv.is_promo()),
            "a rook check survives"
        );
    }

    #[test]
    fn checking_and_escaping_quiets_are_ordered_first() {
        let searcher = Searcher::default();
        // The white knight on d4 is attacked by the c5 pawn; Nf5 escapes to a
        // safe square, Rh8 checks along the rank, Kb1 does neither.
        let board = Board::from_fen("4k3/8/8/2p5/3N4/8/8/K6R w - - 0 1").expect("valid FEN");
        let threats = board.threats();
        let legal = board.generate_legal_moves();
        let scored_list = searcher.score_moves(&board, &threats, &legal, Move::NULL, 0);
        let mut scored = scored_list;
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
