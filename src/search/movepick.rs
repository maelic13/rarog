//! The staged move picker and move scoring.

use std::mem::MaybeUninit;
use std::slice;

use crate::board::{Bitboard, Board, Move, MoveList, Piece};
use crate::eval::piece_value;

use super::Searcher;

/// Sentinel SEE for a TT move emitted before its SEE was computed.
pub(super) const SEE_UNKNOWN: i16 = i16::MIN;

#[derive(Copy, Clone, Default)]
pub(crate) struct ScoredMove {
    pub mv: Move,
    pub score: i32,
    pub see: i16,
    pub quiet_history: i32,
}

// 9.0 KEEP-UNSAFE (measured): see MoveList in board/moves.rs — plain
// initialized arrays cost −10% NPS (2026-07-19). Unsafe confined to the
// slice accessors with a local prefix-initialization invariant.
pub(crate) struct ScoredMoveList {
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
    pub fn push(&mut self, mv: Move, score: i32, see: i32) {
        self.push_with_history(mv, score, see, 0);
    }

    #[inline(always)]
    pub fn push_with_history(&mut self, mv: Move, score: i32, see: i32, quiet_history: i32) {
        debug_assert!(self.len < self.moves.len());
        self.moves[self.len].write(ScoredMove {
            mv,
            score,
            see: crate::infra::saturating_i16(see),
            quiet_history,
        });
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [ScoredMove] {
        // SAFETY: only the initialized prefix below `len` is exposed.
        unsafe { slice::from_raw_parts_mut(self.moves.as_mut_ptr().cast::<ScoredMove>(), self.len) }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BadCapture {
    pub attacker: Piece,
    pub to: u8,
    pub captured: Option<Piece>,
}

pub(crate) struct BadCaptureList {
    items: [MaybeUninit<BadCapture>; 256],
    len: usize,
}

impl BadCaptureList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            items: [const { MaybeUninit::uninit() }; 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, attacker: Piece, to: u8, captured: Option<Piece>) {
        debug_assert!(self.len < self.items.len());
        self.items[self.len].write(BadCapture {
            attacker,
            to,
            captured,
        });
        self.len += 1;
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[BadCapture] {
        // SAFETY: only the initialized prefix below `len` is exposed.
        unsafe { slice::from_raw_parts(self.items.as_ptr().cast::<BadCapture>(), self.len) }
    }
}

/// Selection step: move the highest-scored entry of `moves[index..]` into
/// `moves[index]` and return it.
///
/// 10.3(8d): scans a `split_at_mut` tail by iterator and carries the running
/// best SCORE in a local. The old form indexed `moves[current]` and
/// `moves[best]` per iteration — two loads where one suffices, plus an index
/// LLVM cannot bound-check away (`best` is only provably `< len` by induction
/// through the loop). Ties still resolve to the earliest entry: the comparison
/// stays strictly `>`.
pub(crate) fn pick_next(moves: &mut [ScoredMove], index: usize) -> ScoredMove {
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

pub(crate) fn diversify_root_scores(moves: &mut [ScoredMove], offset: usize) {
    moves.sort_unstable_by_key(|m| std::cmp::Reverse(m.score));
    if offset < moves.len() {
        moves[offset].score = moves[0].score.saturating_add(1_000_000);
    }
}

// 9.0: `clippy::large_enum_variant` is deliberately allowed here. The `Full`
// variant embeds a ScoredMoveList (~3 KB) inline, which is the point: a
// MovePicker is constructed at EVERY interior node, and boxing the large
// variant would trade a stack-resident list for a heap allocation per node.
// Measured elsewhere in 9.0: making the move lists heap/initialized cost
// -10% NPS. The enum size is a deliberate space-for-speed trade, and the
// note is kept even though clippy no longer objects: if the variants diverge
// again the lint fires, and this is the reason not to "fix" it by boxing.
/// 4.5.2 MOVE-PICKER STAGE CONTRACT.
///
/// The staged picker's transitions used to live implicitly in three cursor
/// comparisons (`good_index < good_len`, `cap_len + quiet_index < len`,
/// `good_len + bad_index < cap_len`). Reading the order off that required
/// reconstructing the buffer partition in your head, and nothing named the
/// order or made it assertable. This enum is that order.
///
/// Contract, and all three parts are covered by tests below:
///   ORDER      staged: TtMove, GoodCaptures, Quiets, BadCaptures.
///              full (root and in-check): TtMove, AllRemaining.
///   DUPLICATES the TT move is emitted at most once, and every later stage
///              filters it out, so no move is ever emitted twice.
///   LEGALITY   both paths only ever emit moves from a legal generator; the
///              TT move is validated by the caller before construction.
///
/// Stages are visited in declaration order and never revisited. `Done` is
/// terminal: once reached the picker yields `None` forever.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Stage {
    /// Emit the TT move alone, if there is one.
    TtMove,
    /// Staged path: captures with SEE >= 0, best-first.
    GoodCaptures,
    /// Staged path: generate quiets on demand, once.
    GenerateQuiets,
    /// Staged path: quiets, best-first.
    Quiets,
    /// Staged path: SEE-losing captures, best-first, deliberately last.
    BadCaptures,
    /// Full path: everything except the TT move, best-first from one list.
    AllRemaining,
    /// Terminal.
    Done,
}

pub(super) enum MovePicker {
    Full {
        scored: ScoredMoveList,
        index: usize,
        tt_move: Move,
        stage: Stage,
    },
    /// 10.3(4): ONE buffer, partitioned in place, instead of three separate
    /// 3,080-byte lists. Layout — `[0, good_len)` good captures,
    /// `[good_len, cap_len)` bad captures, `[cap_len, len)` quiets. Every
    /// push is sequential, so `ScoredMoveList`'s prefix-initialization
    /// invariant is untouched. Total legal moves in any position (~218) fit
    /// the 256 capacity, so captures and quiets provably coexist.
    Staged {
        moves: ScoredMoveList,
        good_len: usize,
        cap_len: usize,
        /// Cursor within `[0, good_len)`.
        good_index: usize,
        /// Cursor within `[cap_len, len)`, relative to `cap_len`.
        quiet_index: usize,
        /// Cursor within `[good_len, cap_len)`, relative to `good_len`.
        bad_index: usize,
        /// 10.3(5): the pinned set computed by capture generation, reused when
        /// quiets are generated later at this same node. `board` is restored
        /// by `unmake_move` between the two stages, so the position — and
        /// therefore the pin structure — is unchanged. 10.3(7) made this
        /// unconditional: the staged path no longer pre-scans for captures, so
        /// a pinned set always exists to share.
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
        tt_move: Move,
        ply: usize,
    ) -> Self {
        let mut captures = MoveList::new();
        let pinned = board.generate_legal_captures_pinned_into(&mut captures);
        let (moves, good_len, cap_len) =
            searcher.score_staged_captures(board, captures.as_slice(), tt_move);
        Self::Staged {
            moves,
            good_len,
            cap_len,
            good_index: 0,
            quiet_index: 0,
            bad_index: 0,
            pinned,
            tt_move,
            stage: Stage::TtMove,
            ply,
        }
    }

    /// Yield the next move, advancing through `Stage` in declaration order.
    ///
    /// Every stage is a `loop`+`match` step rather than a fallthrough chain, so
    /// a stage that runs dry advances exactly once and the order is readable
    /// without reconstructing the buffer partition.
    pub(super) fn next(&mut self, searcher: &Searcher, board: &mut Board) -> Option<ScoredMove> {
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
                            // Duplicate guarantee: the TT move already went out
                            // in Stage::TtMove.
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
                good_len,
                cap_len,
                good_index,
                quiet_index,
                bad_index,
                pinned,
                tt_move,
                stage,
                ply,
            } => loop {
                match *stage {
                    Stage::TtMove => {
                        *stage = Stage::GoodCaptures;
                        if !tt_move.is_null() {
                            return Some(tt_scored_move(*tt_move));
                        }
                    }
                    Stage::GoodCaptures => {
                        // The selection scan is bounded to the good partition,
                        // so it can never pull a bad capture forward.
                        while *good_index < *good_len {
                            let picked =
                                pick_next(&mut moves.as_mut_slice()[..*good_len], *good_index);
                            *good_index += 1;
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::GenerateQuiets;
                    }
                    Stage::GenerateQuiets => {
                        // Once, on demand, appended after the captures. The
                        // stage exists so "have the quiets been generated" is a
                        // position in the order rather than a bool.
                        let mut quiet_moves = MoveList::new();
                        board.generate_legal_quiets_pinned_into(*pinned, &mut quiet_moves);
                        searcher.append_scored_moves(
                            board,
                            quiet_moves.as_slice(),
                            *tt_move,
                            *ply,
                            moves,
                        );
                        *stage = Stage::Quiets;
                    }
                    Stage::Quiets => {
                        while *cap_len + *quiet_index < moves.len() {
                            let picked =
                                pick_next(&mut moves.as_mut_slice()[*cap_len..], *quiet_index);
                            *quiet_index += 1;
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::BadCaptures;
                    }
                    Stage::BadCaptures => {
                        while *good_len + *bad_index < *cap_len {
                            let picked = pick_next(
                                &mut moves.as_mut_slice()[*good_len..*cap_len],
                                *bad_index,
                            );
                            *bad_index += 1;
                            if picked.mv != *tt_move {
                                return Some(picked);
                            }
                        }
                        *stage = Stage::Done;
                    }
                    _ => return None,
                }
            },
        }
    }
}

pub(super) fn tt_scored_move(mv: Move) -> ScoredMove {
    let see = if mv.is_capture() { SEE_UNKNOWN } else { 0 };
    ScoredMove {
        mv,
        score: 30_000_000,
        see,
        quiet_history: 0,
    }
}

impl Searcher {
    pub(super) fn score_moves(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
        ply: usize,
    ) -> ScoredMoveList {
        let mut scored = ScoredMoveList::new();
        self.append_scored_moves(board, moves, tt_move, ply, &mut scored);
        scored
    }

    /// [`Self::score_moves`] writing into a caller-owned buffer, so the
    /// staged picker can append quiets behind the captures already sitting
    /// in its single partitioned list (10.3(4)).
    pub(super) fn append_scored_moves(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
        ply: usize,
        scored: &mut ScoredMoveList,
    ) {
        let previous = if ply > 0 {
            self.td.stack[ply - 1].mv
        } else {
            Move::NULL
        };
        let counter = if !previous.is_null() {
            self.td.countermove[previous.from_sq().index()][previous.to_sq().index()]
        } else {
            Move::NULL
        };
        // 10.3: check masks computed at most once per node, and only if a
        // quiet actually reaches history scoring — capture-only lists (the
        // common qsearch case) never pay for it. 8.12(g2): the history row
        // bases follow the same lazy once-per-node pattern.
        let mut check_info = None;
        let mut quiet_ctx = None;
        // 9.7.5(d): node-invariant, so read once rather than per move — the
        // killers were being loaded FOUR times per move (twice in the tier
        // chain, twice more in the quiet-history re-test below).
        let killer0 = self.td.killers[ply][0];
        let killer1 = self.td.killers[ply][1];
        let stm = board.side_to_move();

        for &mv in moves {
            let mut see = 0;
            // 9.7.5(d): captured where the quiet branch computes it. The old
            // form re-derived "was this a quiet?" afterwards, repeating six
            // comparisons (capture, promo, tt-move, both killers, countermove)
            // that the tier chain below has already resolved.
            let mut quiet_history = 0;
            let score = if mv == tt_move {
                30_000_000
            } else if mv.is_capture() {
                let attacker = board.moving_piece(mv);
                let victim = board.captured_piece(mv).unwrap_or(Piece::Pawn);
                see = board.see(mv);
                let hist = self.td.cap_history[attacker as usize][mv.to_sq().index()]
                    [victim as usize] as i32;
                if see >= 0 {
                    20_000_000 + 32 * see + 10 * piece_value(victim) - piece_value(attacker) + hist
                } else {
                    -2_000_000 + see + hist
                }
            } else if mv.is_promo() {
                18_000_000 + piece_value(mv.promo_piece())
            } else if mv == killer0 {
                16_000_000
            } else if mv == killer1 {
                15_900_000
            } else if mv == counter {
                15_800_000
            } else {
                let ci = check_info.get_or_insert_with(|| board.check_info());
                let ctx = quiet_ctx.get_or_insert_with(|| self.quiet_history_ctx(board, ply));
                quiet_history = self.quiet_history_score(board, ci, ctx, stm, mv, ply);
                quiet_history
            };
            scored.push_with_history(mv, score, see, quiet_history);
        }
    }

    pub(super) fn score_staged_captures(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
    ) -> (ScoredMoveList, usize, usize) {
        // Two passes so the partition lands in one buffer without moving
        // anything: good captures first, then bad ones appended behind them.
        // Scoring is pure, so scoring twice is only arithmetic — and the
        // second pass runs over the (usually small) bad-capture subset.
        let mut out = ScoredMoveList::new();
        for &mv in moves {
            if mv == tt_move {
                continue;
            }
            let scored = self.score_tactical_move(board, mv, tt_move);
            if scored.see >= 0 || mv.is_promo() {
                out.push(scored.mv, scored.score, scored.see as i32);
            }
        }
        let good_len = out.len();
        for &mv in moves {
            if mv == tt_move {
                continue;
            }
            let scored = self.score_tactical_move(board, mv, tt_move);
            if !(scored.see >= 0 || mv.is_promo()) {
                out.push(scored.mv, scored.score, scored.see as i32);
            }
        }
        let cap_len = out.len();
        (out, good_len, cap_len)
    }

    pub(super) fn score_tactical_moves(
        &self,
        board: &Board,
        moves: &[Move],
        tt_move: Move,
    ) -> ScoredMoveList {
        let mut scored = ScoredMoveList::new();
        for &mv in moves {
            let scored_move = self.score_tactical_move(board, mv, tt_move);
            scored.push(scored_move.mv, scored_move.score, scored_move.see as i32);
        }
        scored
    }

    pub(super) fn score_tactical_move(&self, board: &Board, mv: Move, tt_move: Move) -> ScoredMove {
        let mut see = 0;
        let score = if mv == tt_move {
            if mv.is_capture() && !board.see_ge(mv, 0) {
                see = -1;
            }
            30_000_000
        } else if mv.is_capture() {
            let attacker = board.moving_piece(mv);
            let victim = board.captured_piece(mv).unwrap_or(Piece::Pawn);
            let promo_gain = if mv.is_promo() {
                piece_value(mv.promo_piece()) - piece_value(Piece::Pawn)
            } else {
                0
            };
            let hist =
                self.td.cap_history[attacker as usize][mv.to_sq().index()][victim as usize] as i32;
            if board.see_ge(mv, 0) {
                20_000_000 + 16 * (piece_value(victim) + promo_gain) - piece_value(attacker) + hist
            } else {
                see = -1;
                -2_000_000 + 16 * (piece_value(victim) + promo_gain) - piece_value(attacker) + hist
            }
        } else if mv.is_promo() {
            18_000_000 + piece_value(mv.promo_piece())
        } else {
            0
        };

        ScoredMove {
            mv,
            score,
            see: crate::infra::saturating_i16(see),
            quiet_history: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_direct_checks_receive_ordering_bonus() {
        let searcher = Searcher::default();
        let board = Board::from_fen("4k3/8/8/8/8/8/8/R6K w - - 0 1").expect("valid FEN");
        let checking = board.parse_move("a1e1").expect("legal checking move");
        let quiet = board.parse_move("a1a2").expect("legal quiet move");
        assert!(board.gives_check(checking));
        assert!(!board.gives_check(quiet));

        let mut scored = searcher.score_moves(&board, &[checking, quiet], Move::NULL, 0);
        let moves = scored.as_mut_slice();
        let checking_score = moves
            .iter()
            .find(|scored| scored.mv == checking)
            .expect("checking move scored")
            .score;
        let quiet_score = moves
            .iter()
            .find(|scored| scored.mv == quiet)
            .expect("quiet move scored")
            .score;

        // 4.6c: read the live parameter, not a duplicate constant, so this
        // test cannot drift from the value ordering actually uses.
        let bonus = crate::search::params::SearchParams::default().check_bonus_safe;
        assert!(checking_score >= quiet_score + bonus);
    }

    #[test]
    fn staged_picker_emits_valid_quiet_tt_move_first() {
        let searcher = Searcher::default();
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
                .expect("valid FEN");
        let tt_move = board
            .legal_move(Move::from_uci("g1f3").expect("valid UCI move shape"))
            .expect("quiet TT move must be legal");

        let mut picker = MovePicker::staged(&searcher, &mut board, tt_move, 0);
        let picked = picker.next(&searcher, &mut board).expect("first move");

        assert_eq!(picked.mv, tt_move);
        assert!(!picked.mv.is_capture());
    }

    #[test]
    fn staged_picker_delays_bad_captures_until_after_quiets() {
        let searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1").expect("valid FEN");
        let losing_capture = board
            .parse_move("c3d5")
            .expect("knight capture must be legal");
        assert!(losing_capture.is_capture());
        assert!(!board.see_ge(losing_capture, 0));

        let mut picker = MovePicker::staged(&searcher, &mut board, Move::NULL, 0);
        let mut quiet_seen = false;
        let mut losing_capture_seen = false;

        while let Some(picked) = picker.next(&searcher, &mut board) {
            if picked.mv == losing_capture {
                assert!(
                    quiet_seen,
                    "losing captures should be staged after quiet moves"
                );
                losing_capture_seen = true;
                break;
            }
            if board.is_quiet_move(picked.mv) {
                quiet_seen = true;
            }
        }

        assert!(
            quiet_seen,
            "test position must have at least one quiet move"
        );
        assert!(
            losing_capture_seen,
            "test position must include the losing capture"
        );
    }

    /// Collect everything a picker yields, for the contract tests below.
    fn drain_picker(picker: &mut MovePicker, searcher: &Searcher, board: &mut Board) -> Vec<Move> {
        let mut out = Vec::new();
        while let Some(picked) = picker.next(searcher, board) {
            out.push(picked.mv);
        }
        out
    }

    /// 4.5.2 CONTRACT — staged path: every legal move, exactly once.
    ///
    /// This is the legality and duplicate guarantee in one assertion. It holds
    /// with a TT move set, which is the case that can double-emit: the TT move
    /// goes out in `Stage::TtMove` and every later stage must filter it.
    #[test]
    fn staged_picker_emits_every_legal_move_exactly_once() {
        let searcher = Searcher::default();
        for fen in [
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "4k3/8/4p3/3p4/8/2N5/8/4K3 w - - 0 1",
        ] {
            let mut board = Board::from_fen(fen).expect("valid FEN");
            let mut legal: Vec<Move> = board.generate_legal_movelist().as_slice().to_vec();
            for tt in [Move::NULL, legal[0], legal[legal.len() - 1]] {
                let mut picker = MovePicker::staged(&searcher, &mut board, tt, 0);
                let mut got = drain_picker(&mut picker, &searcher, &mut board);
                got.sort_unstable_by_key(|m| m.0);
                legal.sort_unstable_by_key(|m| m.0);
                assert_eq!(
                    got, legal,
                    "staged picker must emit every legal move exactly once                      (fen {fen}, tt {tt})"
                );
            }
        }
    }

    /// 4.5.2 CONTRACT — full path (root and in-check): same guarantee.
    #[test]
    fn full_picker_emits_every_legal_move_exactly_once() {
        let searcher = Searcher::default();
        // Second FEN is a check position — rook on e2 checking Ke1, with
        // Kxe2/Kd1/Kf1 legal — which is exactly when the search takes the full
        // path. Not a mate: a mated position has no legal moves and would test
        // nothing here.
        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
        ] {
            let mut board = Board::from_fen(fen).expect("valid FEN");
            let mut legal: Vec<Move> = board.generate_legal_movelist().as_slice().to_vec();
            assert!(!legal.is_empty(), "test FEN must have legal moves: {fen}");
            for tt in [Move::NULL, legal[0]] {
                let scored = searcher.score_moves(&board, legal.as_slice(), tt, 0);
                let mut picker = MovePicker::full(scored, tt);
                let mut got = drain_picker(&mut picker, &searcher, &mut board);
                got.sort_unstable_by_key(|m| m.0);
                legal.sort_unstable_by_key(|m| m.0);
                assert_eq!(
                    got, legal,
                    "full picker must emit every legal move exactly once                      (fen {fen}, tt {tt})"
                );
            }
        }
    }

    /// 4.5.2 CONTRACT — `Stage::Done` is terminal and stays terminal.
    #[test]
    fn picker_is_exhausted_permanently() {
        let searcher = Searcher::default();
        let mut board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");
        let mut picker = MovePicker::staged(&searcher, &mut board, Move::NULL, 0);
        while picker.next(&searcher, &mut board).is_some() {}
        for _ in 0..3 {
            assert!(
                picker.next(&searcher, &mut board).is_none(),
                "an exhausted picker must keep returning None"
            );
        }
    }

    #[test]
    fn bad_capture_struct_stays_shrunk() {
        // Phase 2.9.3: `to: usize` (8 bytes) padded BadCapture to 16 bytes;
        // `to: u8` (a 0-63 square fits easily) drops it to ~3-4 bytes. Guard
        // against this creeping back up, since each BadCaptureList is [_; 256]
        // and two are allocated per negamax frame.
        assert!(
            std::mem::size_of::<BadCapture>() <= 4,
            "size_of::<BadCapture>() = {}",
            std::mem::size_of::<BadCapture>()
        );
    }
}
