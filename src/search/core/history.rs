//! Move-ordering histories: threat-aware quiet and noisy tables, pawn-structure
//! and continuation tables, their gravity update and the interim update rules.

use crate::board::{Bitboard, Board, Color, Move, Piece, Square};

use crate::infra;

use super::Searcher;
use super::movepick::is_noisy;

/// `(colour, piece type, square)` slots: the second index of every
/// piece-to table.
pub(super) const PIECE_TO_SIZE: usize = 12 * 64;
/// Continuation contexts: whether the earlier node was in check, whether its
/// move was noisy, and that move's coloured piece and destination.
pub(super) const CONT_CONTEXTS: usize = 2 * 2 * PIECE_TO_SIZE;
pub(super) const CONT_SIZE: usize = CONT_CONTEXTS * PIECE_TO_SIZE;
/// How many plies back each continuation term looks.
pub(super) const CONT_PLY_BACK: [usize; 4] = [1, 2, 4, 6];

const QUIET_MAX: i32 = 8_192;
const NOISY_MAX: i32 = 12_800;
const PAWN_MAX: i32 = 8_192;
const CONT_MAX: i32 = 15_320;
const PAWN_HISTORY_SIZE: usize = 512;
/// Captured-type slots of the noisy table: the six piece types and "none",
/// for a non-capturing promotion.
const CAPTURED_SLOTS: usize = 7;

/// Index of a coloured piece standing on `sq`.
#[inline(always)]
pub(super) fn piece_to(color: Color, piece: Piece, sq: Square) -> usize {
    (color as usize * 6 + piece as usize) * 64 + sq.index()
}

/// The continuation context a move made at a node selects, as a row base
/// into a `CONT_SIZE` table.
#[inline(always)]
pub(super) fn cont_context(
    in_check: bool,
    noisy: bool,
    color: Color,
    piece: Piece,
    to: Square,
) -> usize {
    ((usize::from(in_check) * 2 + usize::from(noisy)) * PIECE_TO_SIZE + piece_to(color, piece, to))
        * PIECE_TO_SIZE
}

/// Gravity update. The bonus is clamped to `±max` and the entry moves toward
/// its sign by `bonus - |bonus| * entry / max`, so an entry never leaves
/// `[-max, max]` and a saturated entry moves little.
#[inline(always)]
pub(super) fn apply_bonus(entry: &mut i16, bonus: i32, max: i32) {
    let bonus = bonus.clamp(-max, max);
    let current = i32::from(*entry);
    *entry = crate::infra::saturating_i16(current + bonus - bonus.abs() * current / max);
}

/// Heap-allocate a zeroed table without building it on the stack first.
fn zeroed<const N: usize>() -> Box<[i16; N]> {
    vec![0i16; N]
        .into_boxed_slice()
        .try_into()
        .unwrap_or_else(|_| unreachable!("length is N by construction"))
}

/// `[side to move][from threatened][to threatened][from][to]`.
type QuietTable = [[[[[i16; 64]; 64]; 2]; 2]; 2];

/// One thread's move-ordering tables. None of them age between searches.
pub(super) struct HistoryTables {
    quiet: Box<QuietTable>,
    /// `[coloured piece and destination][captured type][destination threatened]`.
    noisy: Box<[[[i16; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE]>,
    /// `[pawn-key slot][coloured piece and destination]`, flat.
    pawn: Box<[i16; PAWN_HISTORY_SIZE * PIECE_TO_SIZE]>,
    /// `[context][coloured piece and destination]`, flat; a context is a row
    /// base from [`cont_context`].
    cont: Box<[i16; CONT_SIZE]>,
}

impl Default for HistoryTables {
    fn default() -> Self {
        Self {
            quiet: Box::new([[[[[0; 64]; 64]; 2]; 2]; 2]),
            noisy: Box::new([[[0; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE]),
            pawn: zeroed(),
            cont: zeroed(),
        }
    }
}

impl HistoryTables {
    /// Forget everything, for a new game.
    pub(super) fn clear(&mut self) {
        *self.quiet = [[[[[0; 64]; 64]; 2]; 2]; 2];
        *self.noisy = [[[0; 2]; CAPTURED_SLOTS]; PIECE_TO_SIZE];
        self.pawn.fill(0);
        self.cont.fill(0);
    }

    /// Between searches the tables keep their values: gravity alone bounds
    /// them.
    #[expect(
        clippy::unused_self,
        reason = "the search calls age() on either arm's tables"
    )]
    pub(super) fn age(&mut self) {}

    #[inline(always)]
    pub(super) fn quiet(&self, threats: Bitboard, stm: Color, mv: Move) -> i32 {
        let (from, to) = (mv.from_sq(), mv.to_sq());
        i32::from(
            self.quiet[stm as usize][usize::from(threats.contains(from))]
                [usize::from(threats.contains(to))][from.index()][to.index()],
        )
    }

    #[inline(always)]
    pub(super) fn update_quiet(&mut self, threats: Bitboard, stm: Color, mv: Move, bonus: i32) {
        let (from, to) = (mv.from_sq(), mv.to_sq());
        apply_bonus(
            &mut self.quiet[stm as usize][usize::from(threats.contains(from))]
                [usize::from(threats.contains(to))][from.index()][to.index()],
            bonus,
            QUIET_MAX,
        );
    }

    #[inline(always)]
    pub(super) fn noisy(
        &self,
        threats: Bitboard,
        color: Color,
        piece: Piece,
        to: Square,
        captured: Option<Piece>,
    ) -> i32 {
        i32::from(
            self.noisy[piece_to(color, piece, to)][captured.map_or(6, |p| p as usize)]
                [usize::from(threats.contains(to))],
        )
    }

    #[inline(always)]
    pub(super) fn update_noisy(
        &mut self,
        threats: Bitboard,
        color: Color,
        piece: Piece,
        to: Square,
        captured: Option<Piece>,
        bonus: i32,
    ) {
        apply_bonus(
            &mut self.noisy[piece_to(color, piece, to)][captured.map_or(6, |p| p as usize)]
                [usize::from(threats.contains(to))],
            bonus,
            NOISY_MAX,
        );
    }

    /// Row base of the pawn table for this pawn structure.
    #[inline(always)]
    pub(super) fn pawn_row(pawn_key: u64) -> usize {
        (crate::infra::index(pawn_key) & (PAWN_HISTORY_SIZE - 1)) * PIECE_TO_SIZE
    }

    #[inline(always)]
    pub(super) fn pawn(&self, pawn_row: usize, color: Color, piece: Piece, to: Square) -> i32 {
        i32::from(self.pawn[pawn_row + piece_to(color, piece, to)])
    }

    #[inline(always)]
    pub(super) fn update_pawn(
        &mut self,
        pawn_row: usize,
        color: Color,
        piece: Piece,
        to: Square,
        bonus: i32,
    ) {
        apply_bonus(
            &mut self.pawn[pawn_row + piece_to(color, piece, to)],
            bonus,
            PAWN_MAX,
        );
    }

    #[inline(always)]
    pub(super) fn cont(&self, context: usize, color: Color, piece: Piece, to: Square) -> i32 {
        i32::from(self.cont[context + piece_to(color, piece, to)])
    }

    #[inline(always)]
    pub(super) fn update_cont(
        &mut self,
        context: usize,
        color: Color,
        piece: Piece,
        to: Square,
        bonus: i32,
    ) {
        apply_bonus(
            &mut self.cont[context + piece_to(color, piece, to)],
            bonus,
            CONT_MAX,
        );
    }
}

impl Searcher {
    /// The continuation context `back` plies before `ply`, or `None` when no
    /// move was made there.
    #[inline(always)]
    pub(super) fn cont_context_back(&self, ply: usize, back: usize) -> Option<usize> {
        let entry = self.td.stack.back(ply, back);
        (!entry.mv.is_null()).then_some(entry.cont_key)
    }

    /// The history a pruning or reduction decision reads for a quiet move:
    /// the quiet table plus the one- and two-ply continuations.
    pub(super) fn quiet_pruning_history(
        &self,
        board: &Board,
        threats: Bitboard,
        ply: usize,
        mv: Move,
    ) -> i32 {
        let stm = board.side_to_move();
        let piece = board.moving_piece(mv);
        let mut history = self.td.hist.quiet(threats, stm, mv);
        for back in [1, 2] {
            if let Some(context) = self.cont_context_back(ply, back) {
                history += self.td.hist.cont(context, stm, piece, mv.to_sq());
            }
        }
        history
    }

    /// Update the continuation histories of the move `piece` to `to`, made
    /// at `ply` by `color`, in every context it follows.
    pub(super) fn update_continuations(
        &mut self,
        ply: usize,
        color: Color,
        piece: Piece,
        to: Square,
        bonus: i32,
    ) {
        for back in CONT_PLY_BACK {
            if let Some(context) = self.cont_context_back(ply, back) {
                self.td.hist.update_cont(context, color, piece, to, bonus);
            }
        }
    }

    /// The update when a node's best move raised alpha: a bonus to that move,
    /// a malus to the moves searched before it without raising alpha (quiet
    /// maluses fade with their position in the search order), a continuation
    /// malus to the parent's quiet move when it was among the parent's first
    /// two, and a continuation bonus to a quiet best move that failed high
    /// only after a re-search.
    pub(super) fn update_best_move_histories(&mut self, board: &Board, u: &BestMoveUpdate<'_>) {
        let p = &self.cfg.core;
        let depth = u.depth;
        let cut = i32::from(u.cut_node);
        let quiet_count = infra::to_i32(u.quiets.len());
        let noisy_count = infra::to_i32(u.noisies.len());
        let noisy_bonus = (96 * depth).min(p.hist_noisy_bonus_cap) - 43 - 87 * cut;
        let noisy_malus = (175 * depth).min(1_252) - 58 - 16 * noisy_count;
        let quiet_bonus =
            (p.hist_quiet_bonus_slope * depth).min(p.hist_quiet_bonus_cap) - 72 - 42 * cut;
        let quiet_malus =
            (p.hist_quiet_malus_slope * depth).min(p.hist_quiet_malus_cap) - 46 - 31 * quiet_count;
        let cont_bonus = (97 * depth).min(p.hist_cont_bonus_cap) - 74 - 48 * cut;
        let cont_malus = (414 * depth).min(949) - 49 - 17 * quiet_count;
        let index_scale = p.hist_malus_index_scale;
        let stm = board.side_to_move();
        let threats = u.threats;
        let pawn_row = HistoryTables::pawn_row(board.pawn_key());
        let best = u.best;
        let best_piece = board.moving_piece(best);

        if is_noisy(best) {
            self.td.hist.update_noisy(
                threats,
                stm,
                best_piece,
                best.to_sq(),
                board.captured_piece(best),
                noisy_bonus,
            );
        } else {
            self.td.hist.update_quiet(threats, stm, best, quiet_bonus);
            self.td
                .hist
                .update_pawn(pawn_row, stm, best_piece, best.to_sq(), quiet_bonus);
            self.update_continuations(u.ply, stm, best_piece, best.to_sq(), cont_bonus);
            for (index, &mv) in u.quiets.iter().enumerate() {
                let denominator = 1024 + index_scale * infra::to_i32(index);
                let scale = 1024 * 1024 / (denominator * denominator / 1024);
                let piece = board.moving_piece(mv);
                self.td
                    .hist
                    .update_quiet(threats, stm, mv, -quiet_malus * scale / 1024);
                self.td.hist.update_pawn(
                    pawn_row,
                    stm,
                    piece,
                    mv.to_sq(),
                    -quiet_malus * scale / 1024,
                );
                self.update_continuations(
                    u.ply,
                    stm,
                    piece,
                    mv.to_sq(),
                    -cont_malus * scale / 1024,
                );
            }
        }
        for &mv in u.noisies {
            self.td.hist.update_noisy(
                threats,
                stm,
                board.moving_piece(mv),
                mv.to_sq(),
                board.captured_piece(mv),
                -noisy_malus,
            );
        }

        if u.ply > 0 {
            let parent = *self.td.stack.back(u.ply, 1);
            if !parent.mv.is_null() && !is_noisy(parent.mv) && parent.move_count < 2 {
                let malus = (93 * depth - 52).min(935);
                self.update_continuations(u.ply - 1, !stm, parent.piece, parent.mv.to_sq(), -malus);
            }
        }
        if u.search_count > 1 && u.fail_high && !is_noisy(best) {
            let bonus = (233 * depth - 86).min(1_550);
            self.update_continuations(u.ply, stm, best_piece, best.to_sq(), bonus);
        }
    }

    /// A node that failed low was a refutation of its parent's move: reward
    /// that move. A quiet one gets a quiet bonus scaled by how late the parent
    /// tried it, whether it was the parent's TT move, and how far this node
    /// fell below its own eval and the parent's, plus a continuation bonus;
    /// a noisy one gets a noisy bonus.
    pub(super) fn reward_parent_after_fail_low(
        &mut self,
        board: &Board,
        ply: usize,
        depth: i32,
        best_score: i32,
        static_eval: i32,
        in_check: bool,
    ) {
        let parent = *self.td.stack.back(ply, 1);
        if parent.mv.is_null() {
            return;
        }
        let mover = !board.side_to_move();
        let p = &self.cfg.core;
        if is_noisy(parent.mv) {
            let piece = if parent.mv.is_promo() {
                parent.mv.promo_piece()
            } else {
                parent.piece
            };
            let bonus = (50 * depth).min(654);
            self.td.hist.update_noisy(
                parent.threats,
                mover,
                piece,
                parent.mv.to_sq(),
                parent.captured,
                bonus,
            );
            return;
        }
        let factor = p.hist_fail_low_base
            + (17 * parent.move_count).min(229)
            + 110 * i32::from(parent.mv == parent.tt_move)
            + 144 * i32::from(!in_check && best_score <= static_eval - 44)
            + 306
                * i32::from(
                    parent.static_eval != crate::eval::VALUE_NONE
                        && best_score <= -parent.static_eval - 62,
                );
        let bonus = factor * (180 * depth - 37).min(2_414) / 128;
        self.td
            .hist
            .update_quiet(parent.threats, mover, parent.mv, bonus);
        if let Some(context) = self.cont_context_back(ply, 2) {
            let bonus = (152 * depth - 47).min(1_379);
            self.td
                .hist
                .update_cont(context, mover, parent.piece, parent.mv.to_sq(), bonus);
        }
    }
}

/// A node's best-move history update: see
/// [`Searcher::update_best_move_histories`].
pub(super) struct BestMoveUpdate<'a> {
    pub(super) threats: Bitboard,
    pub(super) ply: usize,
    pub(super) depth: i32,
    pub(super) cut_node: bool,
    pub(super) best: Move,
    /// Quiet and noisy moves searched before the best move without raising
    /// alpha.
    pub(super) quiets: &'a [Move],
    pub(super) noisies: &'a [Move],
    /// Searches of the best move: more than one means a reduced search or a
    /// scout was followed by a re-search.
    pub(super) search_count: u32,
    pub(super) fail_high: bool,
}

/// Up to 32 moves of one class a node searched without raising alpha.
pub(super) struct SearchedMoves {
    moves: [Move; 32],
    len: usize,
}

impl SearchedMoves {
    pub(super) const fn new() -> Self {
        Self {
            moves: [Move::NULL; 32],
            len: 0,
        }
    }

    pub(super) fn push(&mut self, mv: Move) {
        if self.len < self.moves.len() {
            self.moves[self.len] = mv;
            self.len += 1;
        }
    }

    pub(super) fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each table's entries stay inside `±max` under any sequence of
    /// clamped bonuses, and the update is gravity: repeated identical bonuses
    /// converge on the bound without crossing it, and a saturated entry moves
    /// less than an empty one.
    #[test]
    fn every_table_is_bounded_and_updates_by_gravity() {
        for max in [QUIET_MAX, NOISY_MAX, PAWN_MAX, CONT_MAX] {
            let mut entry = 0i16;
            let mut previous = 0;
            for _ in 0..200 {
                apply_bonus(&mut entry, 3 * max, max);
                let value = i32::from(entry);
                assert!(value <= max && value >= previous, "max {max}: {value}");
                previous = value;
            }
            assert!(i32::from(entry) >= max - max / 64, "max {max} converges");
            for _ in 0..200 {
                apply_bonus(&mut entry, -3 * max, max);
                assert!(i32::from(entry) >= -max, "max {max}: {entry}");
            }
            let mut empty = 0i16;
            let mut high = i16::try_from(max * 3 / 4).expect("fits");
            apply_bonus(&mut empty, 1_000, max);
            let before = high;
            apply_bonus(&mut high, 1_000, max);
            assert!(high - before < empty, "saturated entry moves less");
        }
    }

    #[test]
    fn tables_index_by_threat_and_colour() {
        let mut tables = HistoryTables::default();
        let mv = Move::from_uci("g1f3").expect("valid move");
        let threatened_to = Bitboard::from(Square::F3);
        tables.update_quiet(threatened_to, Color::White, mv, 500);
        assert_eq!(tables.quiet(threatened_to, Color::White, mv), 500);
        assert_eq!(tables.quiet(Bitboard::EMPTY, Color::White, mv), 0);
        assert_eq!(tables.quiet(threatened_to, Color::Black, mv), 0);

        tables.update_noisy(
            Bitboard::EMPTY,
            Color::Black,
            Piece::Queen,
            Square::D4,
            None,
            700,
        );
        assert_eq!(
            tables.noisy(
                Bitboard::EMPTY,
                Color::Black,
                Piece::Queen,
                Square::D4,
                None
            ),
            700
        );
        assert_eq!(
            tables.noisy(
                Bitboard::EMPTY,
                Color::Black,
                Piece::Queen,
                Square::D4,
                Some(Piece::Pawn)
            ),
            0
        );
        assert_eq!(
            tables.noisy(
                Bitboard::from(Square::D4),
                Color::Black,
                Piece::Queen,
                Square::D4,
                None
            ),
            0
        );

        let row = HistoryTables::pawn_row(0xDEAD_BEEF);
        tables.update_pawn(row, Color::White, Piece::Knight, Square::F3, -300);
        assert_eq!(
            tables.pawn(row, Color::White, Piece::Knight, Square::F3),
            -300
        );
        assert_eq!(tables.pawn(row, Color::Black, Piece::Knight, Square::F3), 0);

        let context = cont_context(true, false, Color::Black, Piece::Pawn, Square::E5);
        tables.update_cont(context, Color::White, Piece::Knight, Square::F3, 900);
        assert_eq!(
            tables.cont(context, Color::White, Piece::Knight, Square::F3),
            900
        );
        let other = cont_context(false, false, Color::Black, Piece::Pawn, Square::E5);
        assert_eq!(
            tables.cont(other, Color::White, Piece::Knight, Square::F3),
            0
        );

        tables.clear();
        assert_eq!(tables.quiet(threatened_to, Color::White, mv), 0);
        assert_eq!(
            tables.cont(context, Color::White, Piece::Knight, Square::F3),
            0
        );
    }

    /// The best move is rewarded; the quiets searched before it are penalised,
    /// the earliest most; nothing else moves.
    #[test]
    fn a_best_quiet_is_rewarded_and_earlier_quiets_fade() {
        let mut searcher = Searcher::default();
        let board = Board::default();
        let threats = board.threats().all;
        let mv = |uci: &str| board.parse_move(uci).expect("legal move");
        let best = mv("g1f3");
        let quiets = [mv("a2a3"), mv("b2b3"), mv("c2c3"), mv("d2d3")];
        searcher.update_best_move_histories(
            &board,
            &BestMoveUpdate {
                threats,
                ply: 0,
                depth: 8,
                cut_node: false,
                best,
                quiets: &quiets,
                noisies: &[],
                search_count: 1,
                fail_high: true,
            },
        );
        let quiet = |m: Move| searcher.td.hist.quiet(threats, Color::White, m);
        assert!(quiet(best) > 0);
        let maluses = quiets.map(quiet);
        assert!(maluses.iter().all(|&v| v < 0), "{maluses:?}");
        assert!(
            maluses.windows(2).all(|w| w[0] <= w[1]),
            "fades: {maluses:?}"
        );
        assert_eq!(quiet(mv("h2h3")), 0, "an unsearched move is untouched");
        let pawn_row = HistoryTables::pawn_row(board.pawn_key());
        assert!(
            searcher
                .td
                .hist
                .pawn(pawn_row, Color::White, Piece::Knight, Square::F3)
                > 0
        );
    }

    /// A fail-low child rewards the parent's quiet move under the parent's
    /// threats, from the parent side's perspective.
    #[test]
    fn a_fail_low_rewards_the_parents_quiet_move() {
        let mut searcher = Searcher::default();
        let mut board = Board::default();
        let parent_threats = board.threats().all;
        let parent_move = board.parse_move("e2e4").expect("legal move");
        searcher.td.stack[0].mv = parent_move;
        searcher.td.stack[0].piece = Piece::Pawn;
        searcher.td.stack[0].move_count = 12;
        searcher.td.stack[0].threats = parent_threats;
        searcher.td.stack[0].static_eval = 30;
        board.make_move(parent_move);
        searcher.reward_parent_after_fail_low(&board, 1, 6, -250, -40, false);
        assert!(
            searcher
                .td
                .hist
                .quiet(parent_threats, Color::White, parent_move)
                > 0
        );
        assert_eq!(
            searcher
                .td
                .hist
                .quiet(parent_threats, Color::Black, parent_move),
            0
        );
    }

    /// The largest context plus the largest piece-to slot is the last entry.
    #[test]
    fn continuation_indexes_cover_the_table_exactly() {
        let last = cont_context(true, true, Color::Black, Piece::King, Square::H8)
            + piece_to(Color::Black, Piece::King, Square::H8);
        assert_eq!(last, CONT_SIZE - 1);
        assert_eq!(
            HistoryTables::pawn_row(u64::MAX) + piece_to(Color::Black, Piece::King, Square::H8),
            PAWN_HISTORY_SIZE * PIECE_TO_SIZE - 1
        );
    }
}
