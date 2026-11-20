//! The compact kernel's staged move picker: the TT move, then the noisy
//! moves (captures and queen pushes) whose exchange clears a bar, then the
//! quiets, then the noisy moves that failed the bar, each stage generated
//! only when it is reached. Skipping quiets skips them all.

use crate::board::{Bitboard, Board, Move, MoveList, Piece, Threats};

use super::super::Searcher;
use super::super::history::HistoryTables;
use super::super::movepick::{QuietContext, ROOT_TT_SCORE, ScoredMoveList, pick_next};
use super::rules::{
    NoisyOrderInput, QuietOrderInput, Seeds, good_noisy_threshold, noisy_order, quiet_order,
};
use super::{KERNEL_SEE_VALUES, is_quiet_move};

/// The root's tiers below the table move and the pool's searched moves
/// (`apply_shared_root_scores`, from 25,000,000): the good noisy moves,
/// the quiets round zero, the bad noisy moves. Every ordering score lies
/// within `ROOT_ORDER_SPAN` of zero, so no tier reaches the next.
const ROOT_GOOD_NOISY: i32 = 20_000_000;
const ROOT_BAD_NOISY: i32 = -20_000_000;
const ROOT_ORDER_SPAN: i32 = 4_000_000;

/// The picker's stages, visited in this order and never revisited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in crate::search) enum Stage {
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
pub(in crate::search) struct CompactPicker {
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
    pub(in crate::search) fn new(tt_move: Move, threshold: Option<i32>, ply: usize) -> Self {
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
    pub(in crate::search) fn stage(&self) -> Stage {
        self.stage
    }

    /// The moves a stage-ending decision skips with the one it prices: the
    /// quiets not yet handed over, or the bad noisy moves.
    #[cfg(feature = "pricing")]
    pub(super) fn moves_left_behind(&self) -> usize {
        match self.stage {
            Stage::Quiets => (self.moves.len() - self.noisy_len).saturating_sub(self.quiet_index),
            Stage::BadNoisy => self.bad_len - self.bad_index,
            _ => 0,
        }
    }

    /// Whether the table's move is quiet by the kernel's partition, in
    /// which an under-promotion push is quiet.
    fn tt_move_quiet(&self) -> bool {
        is_quiet_move(self.tt_move)
    }

    /// The next move, or `None` when every stage is spent. Once
    /// `skip_quiets` is set, no quiet is handed over again.
    pub(in crate::search) fn next(
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
                            self.moves.push_scored(mv, score);
                        }
                    }
                    self.noisy_len = self.moves.len();
                    self.stage = Stage::GoodNoisy;
                }
                Stage::GoodNoisy => {
                    let tt_quiet = self.tt_move_quiet();
                    while self.noisy_index < self.noisy_len {
                        let picked = pick_next(
                            &mut self.moves.as_mut_slice()[..self.noisy_len],
                            self.noisy_index,
                        );
                        let bar = self.threshold.unwrap_or_else(|| {
                            good_noisy_threshold(
                                searcher.seeds(),
                                picked.score,
                                tt_quiet,
                                self.good_emitted,
                            )
                        });
                        if board.see_ge_exchange_with_values(picked.mv, bar, KERNEL_SEE_VALUES) {
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
    /// The root's moves, written into `out` in the order the staged picker
    /// hands them over at any other node, ties included: the table move, the
    /// noisy moves that clear the good-noisy bar, the quiets, the noisy moves
    /// that failed it. The moves come from the staged picker's generators and
    /// are put in order by its selection step, so a tie falls as it does
    /// there, and the bar is decided in that order, so its count of good
    /// moves so far is the same. The whole list is ordered and then narrowed
    /// to the moves `keep` admits, as the donor's root skips a move outside
    /// its lines once its picker hands it over. The scores never rise along
    /// the list, so the root's picker takes it in place until the pool's
    /// scores reorder it.
    pub(in crate::search) fn score_root_moves(
        &self,
        board: &mut Board,
        threats: &Threats,
        keep: impl Fn(Move) -> bool,
        tt_move: Move,
        out: &mut ScoredMoveList,
    ) {
        debug_assert_eq!(out.len(), 0, "the root's list starts empty");
        if !tt_move.is_null() && keep(tt_move) {
            out.push_scored(tt_move, ROOT_TT_SCORE);
        }
        let mut generated = MoveList::new();
        let pinned = board.generate_legal_noisy_staged_into(threats, &mut generated);
        let mut noisy_list = ScoredMoveList::new();
        for &mv in generated.as_slice() {
            if mv != tt_move {
                noisy_list.push_scored(mv, self.compact_noisy_score(board, threats, mv));
            }
        }
        let tt_quiet = is_quiet_move(tt_move);
        let mut good = 0;
        let noisy = noisy_list.as_mut_slice();
        for index in 0..noisy.len() {
            let picked = pick_next(noisy, index);
            debug_assert!(picked.score.abs() < ROOT_ORDER_SPAN, "{}", picked.score);
            let bar = good_noisy_threshold(self.seeds(), picked.score, tt_quiet, good);
            let tier = if board.see_ge_exchange_with_values(picked.mv, bar, KERNEL_SEE_VALUES) {
                good += 1;
                ROOT_GOOD_NOISY
            } else {
                ROOT_BAD_NOISY
            };
            noisy[index].score = tier + picked.score;
        }
        let good_tier = |score: i32| score > ROOT_GOOD_NOISY - ROOT_ORDER_SPAN;
        for entry in noisy
            .iter()
            .filter(|entry| good_tier(entry.score) && keep(entry.mv))
        {
            out.push_scored(entry.mv, entry.score);
        }
        generated = MoveList::new();
        board.generate_legal_quiets_staged_into(pinned, threats, &mut generated);
        // The quiet generator's queen pushes are the noisy generator's too.
        let mut quiets = generated
            .iter()
            .copied()
            .filter(|&mv| mv != tt_move && is_quiet_move(mv))
            .peekable();
        if quiets.peek().is_some() {
            let ctx = self.quiet_context(board, threats, 0, &mut None);
            let mut quiet_list = ScoredMoveList::new();
            for mv in quiets {
                probe!(self, PickQuiet, true);
                let score =
                    compact_quiet_score(self.seeds(), board, threats, &ctx, &self.td.hist, mv);
                debug_assert!(score.abs() < ROOT_ORDER_SPAN, "{score}");
                quiet_list.push_scored(mv, score);
            }
            let quiet = quiet_list.as_mut_slice();
            for index in 0..quiet.len() {
                let picked = pick_next(quiet, index);
                if keep(picked.mv) {
                    out.push_scored(picked.mv, picked.score);
                }
            }
        }
        for entry in noisy
            .iter()
            .filter(|entry| !good_tier(entry.score) && keep(entry.mv))
        {
            out.push_scored(entry.mv, entry.score);
        }
    }

    /// A noisy move's ordering score in the compact kernel.
    fn compact_noisy_score(&self, board: &Board, threats: &Threats, mv: Move) -> i32 {
        probe!(self, PickNoisy, true);
        let piece = board.moving_piece(mv);
        let captured = board.captured_piece(mv);
        noisy_order(
            self.seeds(),
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
            out.push_scored(
                mv,
                compact_quiet_score(self.seeds(), board, threats, &ctx, hist, mv),
            );
        }
    }
}

/// A quiet move's ordering score in the compact kernel.
fn compact_quiet_score(
    seeds: &Seeds,
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
        seeds,
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
    use super::super::super::movepick::MovePicker;
    use super::*;
    use crate::board::Board;
    use crate::search::{EngineOptions, SearchEvent, SearchLimits};

    fn drain(board: &mut Board, tt_move: Move, skip_quiets: bool) -> Vec<(Move, Stage)> {
        drain_with(&Searcher::default(), board, tt_move, skip_quiets)
    }

    fn drain_with(
        searcher: &Searcher,
        board: &mut Board,
        tt_move: Move,
        skip_quiets: bool,
    ) -> Vec<(Move, Stage)> {
        let threats = board.threats();
        let mut picker = CompactPicker::new(tt_move, None, 0);
        let mut out = Vec::new();
        while let Some(mv) = picker.next(searcher, board, &threats, skip_quiets) {
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
    fn the_tt_move_is_quiet_by_the_kernels_partition() {
        let board = Board::from_fen("1n2k3/2P5/8/8/8/8/8/4K2R w K - 0 1").expect("legal");
        let quiet = |uci: &str| {
            let mv = board.parse_move(uci).expect("legal");
            CompactPicker::new(mv, None, 0).tt_move_quiet()
        };
        assert!(quiet("c7c8n"), "an under-promotion push is quiet");
        assert!(quiet("c7c8r"));
        assert!(!quiet("c7c8q"), "a queen push is noisy");
        assert!(!quiet("c7b8n"), "a capturing promotion is noisy");
        assert!(quiet("h1h7"));
        assert!(quiet("e1g1"), "castling is quiet");
        assert!(!CompactPicker::new(Move::NULL, None, 0).tt_move_quiet());
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

    const ROOT_FENS: [&str; 7] = [
        crate::board::STARTING_FEN,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
        // In check: the root serves evasions too.
        "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
        // Promotions, with and without capture.
        "1r2k3/2P5/8/8/8/8/1p6/2N1K3 b - - 0 1",
        "1n2k3/2P5/8/8/8/8/8/4K2R w K - 0 1",
        // Two rooks and a bishop hang, and the knights trade evenly: under a
        // quiet table move the bar rises once three good captures are out,
        // and the trade fails it.
        "4k3/8/r3p2r/1b1n4/8/2N5/8/R3K2R w - - 0 1",
    ];

    /// A searcher whose histories a short search of `board` has filled.
    fn warmed(board: &Board) -> Searcher {
        let limits = SearchLimits {
            depth: Some(6),
            ..SearchLimits::default()
        };
        let mut searcher = Searcher::default();
        searcher.reset_search_state(
            &limits,
            &EngineOptions::default(),
            board.side_to_move(),
            0,
            true,
        );
        let legal = board.generate_legal_moves();
        searcher.search_root(board.clone(), &legal, false, &mut || SearchEvent::None);
        searcher
    }

    /// The root's moves that `keep` admits, best first, each with the stage
    /// its score's tier stands for.
    fn root_order(
        searcher: &Searcher,
        board: &Board,
        tt_move: Move,
        keep: impl Fn(Move) -> bool,
    ) -> Vec<(Move, Stage)> {
        let mut board = board.clone();
        let threats = board.threats();
        let mut picker = MovePicker::new(tt_move);
        searcher.score_root_moves(&mut board, &threats, keep, tt_move, picker.scored_mut());
        std::iter::from_fn(|| picker.next())
            .map(|picked| {
                let stage = match picked.score {
                    _ if picked.mv == tt_move => Stage::TtMove,
                    s if s > ROOT_GOOD_NOISY - ROOT_ORDER_SPAN => Stage::GoodNoisy,
                    s if s < ROOT_BAD_NOISY + ROOT_ORDER_SPAN => Stage::BadNoisy,
                    _ => Stage::Quiets,
                };
                (picked.mv, stage)
            })
            .collect()
    }

    /// Every legal move once and the table move first, with and without
    /// one, in check and with promotions.
    #[test]
    fn the_root_hands_over_every_legal_move_once_the_table_move_first() {
        let searcher = Searcher::default();
        for fen in ROOT_FENS {
            let board = Board::from_fen(fen).expect("a legal FEN");
            let mut legal = board.generate_legal_moves();
            legal.sort_unstable_by_key(|mv| mv.0);
            for tt_move in [Move::NULL, legal[0], legal[legal.len() - 1]] {
                let order = root_order(&searcher, &board, tt_move, |_| true);
                if !tt_move.is_null() {
                    assert_eq!(order[0].0, tt_move, "{fen}: the table move first");
                }
                let mut seen: Vec<Move> = order.iter().map(|&(mv, _)| mv).collect();
                seen.sort_unstable_by_key(|mv| mv.0);
                assert_eq!(seen, legal, "{fen}, {tt_move}: every legal move once");
            }
        }
    }

    /// A narrowed root list (MultiPV's later lines, `searchmoves`) hands over
    /// exactly the moves it admits, in the order of the whole list.
    #[test]
    fn a_narrowed_root_keeps_the_whole_lists_order() {
        let searcher = Searcher::default();
        for fen in ROOT_FENS {
            let board = Board::from_fen(fen).expect("a legal FEN");
            let full = root_order(&searcher, &board, Move::NULL, |_| true);
            let admitted: Vec<Move> = full.iter().step_by(2).map(|&(mv, _)| mv).collect();
            let narrowed = root_order(&searcher, &board, Move::NULL, |mv| admitted.contains(&mv));
            let expected: Vec<(Move, Stage)> = full
                .iter()
                .copied()
                .filter(|(mv, _)| admitted.contains(mv))
                .collect();
            assert_eq!(narrowed, expected, "{fen}");
        }
    }

    /// The root's order is the staged picker's, move for move and ties
    /// included, each move in the staged picker's stage. Read on empty
    /// histories, where a capture's score is its victim's and many quiets
    /// tie, and on histories a search has filled.
    #[test]
    fn the_roots_order_is_the_staged_pickers() {
        for fen in ROOT_FENS {
            let mut board = Board::from_fen(fen).expect("a legal FEN");
            for warm in [false, true] {
                let searcher = if warm {
                    warmed(&board)
                } else {
                    Searcher::default()
                };
                let legal = board.generate_legal_moves();
                let quiet = *legal
                    .iter()
                    .find(|&&mv| is_quiet_move(mv))
                    .expect("a quiet");
                for tt_move in [Move::NULL, legal[0], legal[legal.len() - 1], quiet] {
                    let staged: Vec<(Move, Stage)> =
                        drain_with(&searcher, &mut board, tt_move, false)
                            .into_iter()
                            // The picker has moved past the table move's stage
                            // when it hands the move over.
                            .map(|(mv, stage)| {
                                (mv, if mv == tt_move { Stage::TtMove } else { stage })
                            })
                            .collect();
                    let root = root_order(&searcher, &board, tt_move, |_| true);
                    assert_eq!(root, staged, "{fen}, warm {warm}, table move {tt_move}");
                }
            }
        }
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
