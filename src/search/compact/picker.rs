//! The compact kernel's staged move picker: the TT move, then the noisy
//! moves (captures and queen pushes) whose exchange clears a bar, then the
//! quiets, then the noisy moves that failed the bar, each stage generated
//! only when it is reached. Skipping quiets skips them all.

use crate::board::{Bitboard, Board, Move, MoveList, Piece, Threats};

use super::super::Searcher;
use super::super::history::HistoryTables;
use super::super::movepick::{QuietContext, ScoredMoveList, pick_next};
use super::KERNEL_SEE_VALUES;
use super::rules::{
    NoisyOrderInput, QuietOrderInput, SEEDS, good_noisy_threshold, noisy_order, quiet_order,
};

/// The picker's stages, visited in this order and never revisited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    TtMove,
    GenerateNoisy,
    /// Noisy moves whose exchange clears their bar, best first.
    GoodNoisy,
    GenerateQuiets,
    Quiets,
    /// Noisy moves that failed their bar, in the order they failed it.
    BadNoisy,
    Done,
}

/// One node's picker. Noisy moves sit at the front of one list and quiets
/// behind them; a noisy move that fails its bar is swapped down below the
/// noisy moves not yet looked at, so the failures end up in order at the
/// list's front.
pub(super) struct CompactPicker {
    moves: ScoredMoveList,
    noisy_len: usize,
    noisy_index: usize,
    bad_len: usize,
    bad_index: usize,
    quiet_index: usize,
    good_emitted: i32,
    pinned: Bitboard,
    tt_move: Move,
    /// A fixed exchange bar for every noisy move, in place of the scored one.
    threshold: Option<i32>,
    stage: Stage,
    ply: usize,
}

impl CompactPicker {
    /// A picker that hands over `tt_move` first, if any, and holds noisy
    /// moves to `threshold` if one is given.
    #[inline(always)]
    pub(super) fn new(tt_move: Move, threshold: Option<i32>, ply: usize) -> Self {
        Self {
            moves: ScoredMoveList::new(),
            noisy_len: 0,
            noisy_index: 0,
            bad_len: 0,
            bad_index: 0,
            quiet_index: 0,
            good_emitted: 0,
            pinned: Bitboard::EMPTY,
            tt_move,
            threshold,
            stage: Stage::TtMove,
            ply,
        }
    }

    /// The stage of the move last handed over.
    pub(super) fn stage(&self) -> Stage {
        self.stage
    }

    /// The next move, or `None` when every stage is spent. Once
    /// `skip_quiets` is set, no quiet is handed over again.
    pub(super) fn next(
        &mut self,
        searcher: &Searcher,
        board: &mut Board,
        threats: &Threats,
        skip_quiets: bool,
    ) -> Option<Move> {
        loop {
            match self.stage {
                Stage::TtMove => {
                    self.stage = Stage::GenerateNoisy;
                    if !self.tt_move.is_null() {
                        return Some(self.tt_move);
                    }
                }
                Stage::GenerateNoisy => {
                    let mut noisy = MoveList::new();
                    self.pinned = board.generate_legal_noisy_staged_into(threats, &mut noisy);
                    for &mv in noisy.as_slice() {
                        if mv != self.tt_move {
                            let score = searcher.compact_noisy_score(board, threats, mv);
                            self.moves.push(mv, score, 0, 0);
                        }
                    }
                    self.noisy_len = self.moves.len();
                    self.stage = Stage::GoodNoisy;
                }
                Stage::GoodNoisy => {
                    let tt_quiet = !self.tt_move.is_null() && self.tt_move.is_quiet();
                    while self.noisy_index < self.noisy_len {
                        let picked = pick_next(
                            &mut self.moves.as_mut_slice()[..self.noisy_len],
                            self.noisy_index,
                        );
                        let bar = self.threshold.unwrap_or_else(|| {
                            good_noisy_threshold(SEEDS, picked.score, tt_quiet, self.good_emitted)
                        });
                        if board.see_ge_with_values(picked.mv, bar, KERNEL_SEE_VALUES) {
                            self.noisy_index += 1;
                            self.good_emitted += 1;
                            return Some(picked.mv);
                        }
                        self.moves
                            .as_mut_slice()
                            .swap(self.noisy_index, self.bad_len);
                        self.bad_len += 1;
                        self.noisy_index += 1;
                    }
                    self.stage = if skip_quiets {
                        Stage::BadNoisy
                    } else {
                        Stage::GenerateQuiets
                    };
                }
                Stage::GenerateQuiets => {
                    let mut quiets = MoveList::new();
                    board.generate_legal_quiets_staged_into(self.pinned, threats, &mut quiets);
                    searcher.append_compact_quiets(
                        board,
                        threats,
                        quiets.as_slice(),
                        self.tt_move,
                        self.ply,
                        &mut self.moves,
                    );
                    self.stage = Stage::Quiets;
                }
                Stage::Quiets => {
                    let quiets = &mut self.moves.as_mut_slice()[self.noisy_len..];
                    if !skip_quiets && self.quiet_index < quiets.len() {
                        let picked = pick_next(quiets, self.quiet_index);
                        self.quiet_index += 1;
                        return Some(picked.mv);
                    }
                    self.stage = Stage::BadNoisy;
                }
                Stage::BadNoisy => {
                    if self.bad_index < self.bad_len {
                        let mv = self.moves.as_mut_slice()[self.bad_index].mv;
                        self.bad_index += 1;
                        return Some(mv);
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => return None,
            }
        }
    }
}

impl Searcher {
    /// A noisy move's ordering score in the compact kernel.
    fn compact_noisy_score(&self, board: &Board, threats: &Threats, mv: Move) -> i32 {
        probe!(self, PickNoisy, true);
        let piece = board.moving_piece(mv);
        let captured = board.captured_piece(mv);
        noisy_order(
            SEEDS,
            &NoisyOrderInput {
                victim: captured.map_or(0, |p| KERNEL_SEE_VALUES.as_array()[p as usize]),
                history: self.td.hist.noisy(
                    threats.all,
                    board.side_to_move(),
                    piece,
                    mv.to_sq(),
                    captured,
                ),
                queen_promotion: mv.is_promo() && mv.promo_piece() == Piece::Queen,
                in_check: board.is_in_check(),
                piece: piece as i32,
            },
        )
    }

    /// Score and append the quiets: every legal move that is not noisy, the
    /// rook, bishop and knight pushes among them.
    fn append_compact_quiets(
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
        let ctx = self.quiet_context(board, threats, ply, &mut None);
        let hist = &self.td.hist;
        for &mv in quiets {
            if mv == tt_move || (mv.is_promo() && mv.promo_piece() == Piece::Queen) {
                continue;
            }
            probe!(self, PickQuiet, true);
            out.push(
                mv,
                compact_quiet_score(board, threats, &ctx, hist, mv),
                0,
                0,
            );
        }
    }
}

/// A quiet move's ordering score in the compact kernel.
fn compact_quiet_score(
    board: &Board,
    threats: &Threats,
    ctx: &QuietContext,
    hist: &HistoryTables,
    mv: Move,
) -> i32 {
    let piece = board.moving_piece(mv);
    let index = piece as usize;
    let (from, to) = (mv.from_sq(), mv.to_sq());
    let mut cont = [0; 4];
    for (value, context) in cont.iter_mut().zip(ctx.cont) {
        if let Some(context) = context {
            *value = hist.cont(context, ctx.stm, piece, to);
        }
    }
    quiet_order(
        SEEDS,
        &QuietOrderInput {
            quiet: hist.quiet(threats.all, ctx.stm, mv),
            pawn: hist.pawn(ctx.pawn_row, ctx.stm, piece, to),
            cont,
            piece: index,
            escape: ctx.threatened[index].contains(from),
            check_square: ctx.check_squares[index].contains(to),
            threatened_to: ctx.threatened[index].contains(to),
            offense: ctx.offense[index].contains(to),
            wall: ctx.wall_pawns.contains(from),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    fn drain(board: &mut Board, tt_move: Move, skip_quiets: bool) -> Vec<(Move, Stage)> {
        let searcher = Searcher::default();
        let threats = board.threats();
        let mut picker = CompactPicker::new(tt_move, None, 0);
        let mut out = Vec::new();
        while let Some(mv) = picker.next(&searcher, board, &threats, skip_quiets) {
            out.push((mv, picker.stage()));
        }
        out
    }

    #[test]
    fn every_legal_move_is_handed_over_once_in_stage_order() {
        for fen in [
            crate::board::STARTING_FEN,
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2P1k3/8/8/8/8/4Kp2/8 w - - 0 1",
            "4k3/8/8/8/8/8/3q4/4K3 w - - 0 1",
        ] {
            let mut board = Board::from_fen(fen).expect("a legal FEN");
            let mut legal: Vec<Move> = board.generate_legal_moves();
            let tt_move = legal[legal.len() / 2];
            let mut seen: Vec<Move> = drain(&mut board, tt_move, false)
                .into_iter()
                .map(|(mv, _)| mv)
                .collect();
            assert_eq!(seen[0], tt_move, "{fen}: the TT move first");
            legal.sort_unstable_by_key(|mv| mv.0);
            seen.sort_unstable_by_key(|mv| mv.0);
            assert_eq!(seen, legal, "{fen}: every legal move once");
        }
    }

    #[test]
    fn a_skip_hands_over_no_quiet_and_queen_pushes_are_noisy() {
        let mut board = Board::from_fen("8/2P1k3/8/8/8/8/4Kp2/8 w - - 0 1").expect("legal");
        let moves = drain(&mut board, Move::NULL, true);
        assert!(moves.iter().all(|(mv, _)| mv.is_noisy()));
        assert!(
            moves
                .iter()
                .any(|(mv, _)| mv.is_promo() && mv.promo_piece() == Piece::Queen),
            "the queen push is noisy"
        );
        assert!(
            !moves
                .iter()
                .any(|(mv, _)| mv.is_promo() && mv.promo_piece() != Piece::Queen),
            "an under-promotion push is quiet"
        );
    }

    /// A TT move as the probe hands it over, from the key: none, a legal
    /// move, or bits a collision returns, through the legality test.
    fn tt_move(board: &Board) -> Move {
        let mix = board.hash().wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32;
        let legal = board.generate_legal_movelist();
        let legal = legal.as_slice();
        let candidate = match mix % 3 {
            _ if legal.is_empty() => Move::NULL,
            0 => Move::NULL,
            1 => legal[usize::try_from(mix >> 2).expect("32 bits") % legal.len()],
            _ => Move(u16::try_from((mix >> 8) & 0xFFFF).expect("16 bits")),
        };
        board.legal_move(candidate).unwrap_or(Move::NULL)
    }

    /// Perft through the compact picker, with the TT move it is handed, and
    /// the search's make and unmake, checking the board at every node.
    fn picker_perft(searcher: &Searcher, board: &mut Board, depth: u32, ply: usize) -> u64 {
        if depth == 0 {
            return 1;
        }
        let threats = board.threats();
        let mut picker = CompactPicker::new(tt_move(board), None, ply);
        let mut seen = Vec::new();
        let mut leaves = 0;
        while let Some(mv) = picker.next(searcher, board, &threats, false) {
            assert_eq!(board.legal_move(mv), Some(mv), "{}: {mv}", board.to_fen());
            assert!(!seen.contains(&mv), "{}: {mv} twice", board.to_fen());
            seen.push(mv);
            board.make_move(mv);
            board
                .check_consistency()
                .expect("consistent after the make");
            leaves += picker_perft(searcher, board, depth - 1, ply + 1);
            board.unmake_move(mv);
        }
        leaves
    }

    #[test]
    fn the_picker_path_perft_matches_the_reference_counts() {
        let cases: &[(&str, u32, u64)] = if cfg!(debug_assertions) {
            &[
                (crate::board::STARTING_FEN, 3, 8_902),
                (
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                    2,
                    2_039,
                ),
                ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 3, 2_812),
                (
                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                    3,
                    9_467,
                ),
            ]
        } else {
            &[
                (crate::board::STARTING_FEN, 5, 4_865_609),
                (
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                    4,
                    4_085_603,
                ),
                ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, 674_624),
                (
                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                    4,
                    422_333,
                ),
                (
                    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                    4,
                    2_103_487,
                ),
            ]
        };
        let searcher = Searcher::default();
        for &(fen, depth, expected) in cases {
            let mut board = Board::from_fen(fen).expect("a legal FEN");
            assert_eq!(
                picker_perft(&searcher, &mut board, depth, 0),
                expected,
                "picker perft({depth}) of {fen}"
            );
        }
    }
}
