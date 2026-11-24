/// Board representation.
///
/// Uses 12 bitboards (one per color×piece), two occupancy bitboards, and an
/// incremental Zobrist hash.  Make/unmake are performed in-place with an
/// internal history stack — no full-struct copies needed.
use crate::infra;
use std::fmt;

use super::attacks::{ATTACKS, knight_attacks_of, pawn_attacks_of};
use super::bitboard::Bitboard;
use super::cuckoo::CUCKOO;
use super::geometry::{KING_DISTANCE, between};
use super::movegen::generate_legal_moves;
use super::moves::{
    CAPTURE, CASTLE_KINGSIDE, CASTLE_QUEENSIDE, DOUBLE_PUSH, EN_PASSANT, Move, MoveList,
    PROMO_BISHOP, PROMO_CAPTURE_BISHOP, PROMO_CAPTURE_KNIGHT, PROMO_CAPTURE_QUEEN,
    PROMO_CAPTURE_ROOK, PROMO_KNIGHT, PROMO_QUEEN, PROMO_ROOK, QUIET, UciMove,
};
use super::piece::{CastlingRights, Color, Piece};
use super::square::{Rank, Square};
use super::zobrist::ZOBRIST;

/// FEN fullmove storage is deliberately bounded to `u16`. Zero is normalized
/// to one on input; a black real or null move at the maximum saturates so the
/// public counter is defined identically in debug and release.
const MAX_FULLMOVE: u16 = u16::MAX;

/// Material scale used by static exchange evaluation.
///
/// This is a value object so benchmarks and later fitting can exercise the
/// same SEE implementation without changing evaluation or playing defaults.
/// Production callers use [`PRODUCTION_SEE_VALUES`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SeeValues {
    values: [i32; 6],
}

impl SeeValues {
    pub const fn new(
        pawn: i32,
        knight: i32,
        bishop: i32,
        rook: i32,
        queen: i32,
        king: i32,
    ) -> Self {
        Self {
            values: [pawn, knight, bishop, rook, queen, king],
        }
    }

    pub const fn as_array(self) -> [i32; 6] {
        self.values
    }

    #[inline(always)]
    const fn value(self, piece: Piece) -> i32 {
        self.values[piece as usize]
    }
}

/// The shipped SEE scale. Its king value is an internal sentinel; legal SEE
/// never captures a king. It deliberately remains separate from HCE values.
pub const PRODUCTION_SEE_VALUES: SeeValues = SeeValues::new(100, 320, 330, 500, 900, 20_000);

/// What a threshold exchange test does with a move that captures nothing.
#[derive(Clone, Copy)]
enum QuietSee {
    /// Answered by its immediate gain: nothing, or a promotion's gain.
    Gain,
    /// Played through the exchange, the moved piece at risk, a promotion push
    /// included.
    Exchange,
}

/// Frozen `cross-engine-board-v1` comparison scale.
pub const CROSS_ENGINE_SEE_VALUES: SeeValues = SeeValues::new(100, 300, 300, 500, 900, 20_000);

// -----------------------------------------------------------------------
// Unmake info — everything needed to undo a move
// -----------------------------------------------------------------------

#[derive(Copy, Clone)]
struct UnmakeInfo {
    /// Captured piece, if any.  255 = no capture.
    captured: u8, // piece index: color*6 + piece, or 255
    castling: CastlingRights,
    ep_sq: u8, // 255 = no EP
    halfmove_clock: u8,
    fullmove: u16,
    plies_from_null: u16,
    hash: u64,
    checkers: Bitboard,
}

/// Hot-struct footprint, pinned.
///
/// The 12 colour-piece bitboards were kept over six type boards plus colours,
/// and the compact `UnmakeInfo` over copying per-ply state. Both decisions are
/// footprint arguments, so the footprint is guarded rather than left to drift:
///
/// - `Board` is 264 bytes. The six-board variant would save 48 and neither
///   figure is near any cache boundary that matters, while the extra mask would
///   land on every `pieces()` call site, over a hundred of them in evaluation.
/// - `UnmakeInfo` is 24 bytes, so a 240-ply search stack costs under 6 KiB
///   and stays comfortably in L1. Copying whole board state per ply instead
///   would cost 240 x 264 = 62 KiB and leave it.
///
/// Upper bounds, not equalities: padding may differ between supported targets,
/// and only growth would invalidate the decision.
const _: () = assert!(
    core::mem::size_of::<Board>() <= 264,
    "Board grew past the 264 bytes 4.11b.14 measured; re-open the PLAN 4.11b.14 representation comparison before accepting the growth"
);
const _: () = assert!(
    core::mem::size_of::<UnmakeInfo>() <= 24,
    "UnmakeInfo grew past 24 bytes; a 240-ply stack no longer costs under 6 KiB and PLAN 4.11b.14's per-ply restoration argument needs re-checking"
);

/// By piece type, the masks of the derived keys a piece's key enters: the
/// pawn key, the minor-piece key and its side's non-pawn key.
const DERIVED_KEY_MASKS: [[u64; 3]; 6] = [
    [u64::MAX, 0, 0],
    [0, u64::MAX, u64::MAX],
    [0, u64::MAX, u64::MAX],
    [0, 0, u64::MAX],
    [0, 0, u64::MAX],
    [0, 0, u64::MAX],
];

const NO_PIECE: u8 = 255;
// Padded 12 → 16 so the hot mailbox decode can index with `& 15` — the
// bounds check elides and no unsafe is needed. Entries 12–15 are unreachable
// filler (the mailbox only stores 0..=11 for occupied squares; callers assert
// occupancy), kept as Pawn so even a broken input stays defined, never UB.
const PIECE_FROM_ENCODED: [Piece; 16] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
    Piece::Pawn,
    Piece::Pawn,
    Piece::Pawn,
    Piece::Pawn,
];

// -----------------------------------------------------------------------
// Board
// -----------------------------------------------------------------------

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameResult {
    WhiteCheckmates,
    BlackCheckmates,
    Stalemate,
    Draw,
}

pub struct Board {
    /// `pieces[color * 6 + piece_type]`
    pieces: [Bitboard; 12],
    /// `occupancy[color]`
    occupancy: [Bitboard; 2],
    /// Union of both occupancy bitboards.
    all_occ: Bitboard,
    /// Encoded piece on each square, or 255 for empty.
    mailbox: [u8; 64],
    /// Side to move.
    side_to_move: Color,
    castling: CastlingRights,
    /// En passant target square (the square a capturing pawn moves *to*).
    /// `255` encodes "no EP".
    ep_sq: u8,
    halfmove_clock: u8,
    fullmove: u16,
    /// Moves made since the last null move or since the position was set
    /// up: how far back the key history holds positions a move sequence
    /// reached.
    plies_from_null: u16,
    /// Incrementally updated Zobrist hash.
    hash: u64,
    pawn_hash: u64,
    minor_hash: u64,
    non_pawn_hash: [u64; 2],
    checkers: Bitboard,
    history: Vec<UnmakeInfo>,
}

/// The squares the side not to move attacks, by attacking piece type and in
/// total. See [`Board::threats`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Threats {
    /// Every square at least one enemy piece attacks.
    pub all: Bitboard,
    /// `by_piece[piece]`: the squares enemy pieces of that type attack.
    pub by_piece: [Bitboard; 6],
}

/// Per-node masks for O(1) direct-check tests — see [`Board::check_info`].
pub(crate) struct CheckInfo {
    /// `check_squares[piece]`: squares from which OUR `piece` delivers a
    /// direct check, under pre-move occupancy. King entry is empty.
    check_squares: [Bitboard; 6],
}

impl CheckInfo {
    /// Squares from which the side to move's `piece` gives direct check.
    pub(crate) fn direct_check_squares(&self, piece: Piece) -> Bitboard {
        self.check_squares[piece as usize]
    }
}

impl Clone for Board {
    /// Clones preserve history CAPACITY, not just contents.
    ///
    /// This is the mechanism by which a search-time reservation reaches every
    /// helper: `search_impl` reserves on the root once, and each worker's
    /// `root.clone()` inherits that capacity, so no thread reallocates in its
    /// hot path. Dropping the capacity here would silently reintroduce
    /// mid-search growth on helpers only, which is exactly the kind of
    /// thread-asymmetric behaviour that does not show up in a single-threaded
    /// bench.
    fn clone(&self) -> Self {
        let mut history = Vec::with_capacity(self.history.capacity().max(128));
        history.extend_from_slice(&self.history);
        Self {
            pieces: self.pieces,
            occupancy: self.occupancy,
            all_occ: self.all_occ,
            mailbox: self.mailbox,
            side_to_move: self.side_to_move,
            castling: self.castling,
            ep_sq: self.ep_sq,
            halfmove_clock: self.halfmove_clock,
            fullmove: self.fullmove,
            plies_from_null: self.plies_from_null,
            hash: self.hash,
            pawn_hash: self.pawn_hash,
            minor_hash: self.minor_hash,
            non_pawn_hash: self.non_pawn_hash,
            checkers: self.checkers,
            history,
        }
    }
}

impl Board {
    // -----------------------------------------------------------------------
    // Piece accessors
    // -----------------------------------------------------------------------

    /// Bitboard for a specific color + piece type.
    #[inline(always)]
    pub fn pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.pieces[color as usize * 6 + piece as usize]
    }

    /// Bitboard for all pieces of a given color.
    #[inline(always)]
    pub fn color_occ(&self, color: Color) -> Bitboard {
        self.occupancy[color as usize]
    }

    /// Piece type and color at a given square, or `None` if empty.
    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Option<(Color, Piece)> {
        decode_piece(self.mailbox[sq.index()])
    }

    /// King square for a given color.
    #[inline(always)]
    pub fn king_sq(&self, color: Color) -> Square {
        self.pieces(color, Piece::King).lsb()
    }

    /// En passant target square, if any.
    #[inline(always)]
    pub fn ep_square(&self) -> Option<Square> {
        if self.ep_sq == 255 {
            None
        } else {
            Some(Square(self.ep_sq))
        }
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    pub fn castling(&self) -> CastlingRights {
        self.castling
    }

    /// Plies since the last capture or pawn move.
    #[inline(always)]
    pub fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    #[inline(always)]
    pub fn fullmove(&self) -> u16 {
        self.fullmove
    }

    /// Incrementally updated Zobrist key of the position.
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// The key the position will have after `mv`, exact for a plain move or
    /// capture that changes no castling right and sets no en-passant square,
    /// and only a guess otherwise. A hint for prefetching, nothing else.
    #[inline(always)]
    pub(crate) fn key_after_hint(&self, mv: Move) -> u64 {
        let zob = &ZOBRIST;
        let us = self.side_to_move;
        let (from, to) = (mv.from_sq(), mv.to_sq());
        let piece = self.occupant_type(from);
        let mut key =
            self.hash ^ zob.side() ^ zob.piece(us, piece, from) ^ zob.piece(us, piece, to);
        if self.ep_sq != 255 {
            key ^= zob.ep(Square(self.ep_sq).file());
        }
        if mv.flags() == CAPTURE {
            key ^= zob.piece(!us, self.occupant_type(to), to);
        }
        key
    }

    #[inline(always)]
    pub fn occupied_count(&self) -> u32 {
        self.all_occ.count()
    }

    #[inline(always)]
    pub fn occupied(&self) -> Bitboard {
        self.all_occ
    }

    #[inline(always)]
    fn color_on(&self, sq: Square) -> Option<Color> {
        self.piece_at(sq).map(|(color, _)| color)
    }

    #[inline(always)]
    pub fn moving_piece(&self, mv: Move) -> Piece {
        debug_assert!(self.mailbox[mv.from_sq().index()] < 12);
        self.occupant_type(mv.from_sq())
    }

    pub fn parse_move(&self, input: &str) -> Option<Move> {
        self.legal_uci_move(UciMove::parse(input)?)
    }

    /// The legal move `mv` names, with the flags this position gives it: the
    /// only way from UCI text to a move that can be played.
    pub fn legal_uci_move(&self, mv: UciMove) -> Option<Move> {
        self.legal_move(mv.0)
    }

    fn pseudo_legal_move(&self, mv: Move) -> Option<Move> {
        if mv.is_null() {
            return None;
        }

        let from = mv.from_sq();
        let to = mv.to_sq();
        let us = self.side_to_move;
        let them = !us;
        let (color, piece) = self.piece_at(from)?;
        if color != us {
            return None;
        }
        match self.piece_at(to) {
            Some((target_color, _)) if target_color == us => return None,
            Some((_, Piece::King)) => return None,
            _ => {}
        }

        let promotion = mv.promotion();
        let atk = &ATTACKS;
        let to_bb = Bitboard::from(to);
        let canonical = match piece {
            Piece::Pawn => {
                let delta = to.0 as i16 - from.0 as i16;
                let forward = if us == Color::White { 8 } else { -8 };
                let start_rank = if us == Color::White {
                    Rank::R2
                } else {
                    Rank::R7
                };
                let promotion_rank = if us == Color::White {
                    Rank::R8
                } else {
                    Rank::R1
                };
                let reaches_promotion = to.rank() == promotion_rank;
                if reaches_promotion != promotion.is_some() {
                    return None;
                }

                let is_pawn_attack = (atk.pawn(us, from) & to_bb).any();
                if is_pawn_attack {
                    if self.color_on(to) == Some(them) {
                        pawn_move_with_promotion(from, to, true, promotion)
                    } else if self.ep_square() == Some(to) && self.color_on(to).is_none() {
                        let cap_sq = if us == Color::White {
                            Square(to.0 - 8)
                        } else {
                            Square(to.0 + 8)
                        };
                        (self.piece_at(cap_sq) == Some((them, Piece::Pawn)))
                            .then_some(Move::new(from, to, EN_PASSANT))
                    } else {
                        None
                    }
                } else if delta == forward && self.color_on(to).is_none() {
                    pawn_move_with_promotion(from, to, false, promotion)
                } else if delta == 2 * forward
                    && from.rank() == start_rank
                    && self.color_on(to).is_none()
                {
                    let mid = Square(infra::to_u8(from.0 as i16 + forward));
                    (self.color_on(mid).is_none()).then_some(Move::new(from, to, DOUBLE_PUSH))
                } else {
                    None
                }
            }
            Piece::Knight => {
                if promotion.is_some() || (atk.knight(from) & to_bb).is_empty() {
                    None
                } else {
                    Some(Move::new(
                        from,
                        to,
                        capture_or_quiet(self.color_on(to), them),
                    ))
                }
            }
            Piece::Bishop => {
                if promotion.is_some() || (atk.bishop(from, self.all_occ) & to_bb).is_empty() {
                    None
                } else {
                    Some(Move::new(
                        from,
                        to,
                        capture_or_quiet(self.color_on(to), them),
                    ))
                }
            }
            Piece::Rook => {
                if promotion.is_some() || (atk.rook(from, self.all_occ) & to_bb).is_empty() {
                    None
                } else {
                    Some(Move::new(
                        from,
                        to,
                        capture_or_quiet(self.color_on(to), them),
                    ))
                }
            }
            Piece::Queen => {
                if promotion.is_some() || (atk.queen(from, self.all_occ) & to_bb).is_empty() {
                    None
                } else {
                    Some(Move::new(
                        from,
                        to,
                        capture_or_quiet(self.color_on(to), them),
                    ))
                }
            }
            Piece::King => {
                if promotion.is_some() {
                    None
                } else if (atk.king(from) & to_bb).any() {
                    Some(Move::new(
                        from,
                        to,
                        capture_or_quiet(self.color_on(to), them),
                    ))
                } else {
                    self.castling_move(from, to, us, them)
                }
            }
        }?;

        Some(canonical)
    }

    #[inline(always)]
    pub fn is_pseudo_legal(&self, mv: impl Into<UciMove>) -> bool {
        self.pseudo_legal_move(mv.into().0).is_some()
    }

    pub(crate) fn legal_move(&self, mv: Move) -> Option<Move> {
        let canonical = self.pseudo_legal_move(mv)?;
        self.king_safe_after(canonical).then_some(canonical)
    }

    /// Legality as a plain boolean, of a move or of UCI text's move. It
    /// discards the canonical move [`Board::legal_move`] and
    /// [`Board::legal_uci_move`] return, the only form that carries the flags
    /// a played move needs.
    #[inline(always)]
    pub fn is_legal(&self, mv: impl Into<UciMove>) -> bool {
        self.legal_move(mv.into().0).is_some()
    }

    /// Reserve room for `plies` further make/unmake pairs.
    ///
    /// One `UnmakeInfo` is pushed per ply, so the peak depth of the history
    /// vector is the game length already played plus the deepest line the
    /// search walks. Reserving that headroom before the hot path is entered
    /// means no `push` can reallocate mid-search.
    ///
    /// Deliberately a reservation on the existing `Vec` and not a fixed array:
    /// game history length is unbounded in principle, and a clamped array
    /// would silently drop repetition evidence in a long game.
    pub(crate) fn reserve_history(&mut self, plies: usize) {
        self.history.reserve(plies);
    }

    pub fn play_uci(&mut self, input: &str) -> bool {
        if let Some(mv) = self.parse_move(input) {
            self.make_move(mv);
            true
        } else {
            false
        }
    }

    pub fn generate_legal_moves(&self) -> Vec<Move> {
        generate_legal_moves(self)
    }

    pub fn generate_legal_movelist(&self) -> MoveList {
        super::movegen::generate_legal_movelist(self)
    }

    /// [`Board::generate_legal_movelist`] into a caller-owned list, which is
    /// the form the search uses: the value form pays a 520-byte return copy.
    pub fn generate_legal_movelist_into(&self, moves: &mut MoveList) {
        super::movegen::generate_legal_into(self, moves);
    }

    /// The legal captures, by the generator the search's noisy stage uses.
    pub fn generate_legal_captures(&mut self) -> MoveList {
        let mut moves = MoveList::new();
        super::movegen::generate_captures_pinned_into(self, None, &mut moves);
        moves
    }

    pub fn generate_legal_quiets(&self) -> MoveList {
        super::movegen::generate_quiets(self)
    }

    /// Capture generation into a caller-owned list that also yields the pinned
    /// set, for a staged picker that will generate quiets at the same node.
    pub fn generate_legal_captures_pinned_into(&mut self, moves: &mut MoveList) -> Bitboard {
        super::movegen::generate_captures_pinned_into(self, None, moves)
    }

    /// Quiet generation into a caller-owned list, reusing the pinned set from
    /// [`Board::generate_legal_captures_pinned_into`] at the same node.
    pub fn generate_legal_quiets_pinned_into(&self, pinned: Bitboard, moves: &mut MoveList) {
        super::movegen::generate_quiets_pinned_into(self, pinned, None, moves);
    }

    /// [`Board::generate_legal_captures_pinned_into`] for a caller that holds
    /// this node's [`Board::threats`], with the legal non-capturing queen
    /// promotions after the captures: its `all` decides every king move.
    pub(crate) fn generate_legal_noisy_staged_into(
        &mut self,
        threats: &Threats,
        moves: &mut MoveList,
    ) -> Bitboard {
        super::movegen::generate_noisy_pinned_into(self, Some(threats.all), moves)
    }

    /// [`Board::generate_legal_quiets_pinned_into`] for a caller that holds
    /// this node's [`Board::threats`]: its `all` decides every king move.
    pub(crate) fn generate_legal_quiets_staged_into(
        &self,
        pinned: Bitboard,
        threats: &Threats,
        moves: &mut MoveList,
    ) {
        super::movegen::generate_quiets_pinned_into(self, pinned, Some(threats.all), moves);
    }

    pub fn perft(&mut self, depth: u32) -> u64 {
        super::movegen::perft(self, depth)
    }

    pub fn captured_piece(&self, mv: Move) -> Option<Piece> {
        let flags = mv.flags();
        if flags == EN_PASSANT {
            Some(Piece::Pawn)
        } else if flags == CAPTURE || flags >= PROMO_CAPTURE_KNIGHT {
            debug_assert!(self.mailbox[mv.to_sq().index()] < 12);
            Some(self.occupant_type(mv.to_sq()))
        } else {
            None
        }
    }

    pub(crate) fn has_non_pawn_material(&self, color: Color) -> bool {
        (self.pieces(color, Piece::Knight)
            | self.pieces(color, Piece::Bishop)
            | self.pieces(color, Piece::Rook)
            | self.pieces(color, Piece::Queen))
        .any()
    }

    #[inline(always)]
    pub fn pawn_key(&self) -> u64 {
        self.pawn_hash
    }

    #[inline(always)]
    pub fn minor_key(&self) -> u64 {
        self.minor_hash
    }

    /// `color`'s non-pawn key: the key over that colour's knights, bishops,
    /// rooks, queens and king, kept as the pieces move.
    #[inline(always)]
    pub fn non_pawn_key(&self, color: Color) -> u64 {
        self.non_pawn_hash[color as usize]
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::starting_position()
    }
}

// -----------------------------------------------------------------------
// FEN helper
// -----------------------------------------------------------------------

fn validate_castling_rights(board: &Board, rights: CastlingRights) -> Result<(), String> {
    let required = [
        (
            CastlingRights::WHITE_KINGSIDE,
            Square::E1,
            Square::H1,
            Color::White,
        ),
        (
            CastlingRights::WHITE_QUEENSIDE,
            Square::E1,
            Square::A1,
            Color::White,
        ),
        (
            CastlingRights::BLACK_KINGSIDE,
            Square::E8,
            Square::H8,
            Color::Black,
        ),
        (
            CastlingRights::BLACK_QUEENSIDE,
            Square::E8,
            Square::A8,
            Color::Black,
        ),
    ];

    for (right, king_sq, rook_sq, color) in required {
        if rights.has(right)
            && (board.piece_at(king_sq) != Some((color, Piece::King))
                || board.piece_at(rook_sq) != Some((color, Piece::Rook)))
        {
            return Err(format!(
                "castling right {} does not match king/rook placement",
                right.as_str()
            ));
        }
    }

    Ok(())
}

fn ep_capture_square(capturer: Color, ep_sq: Square) -> Option<Square> {
    match capturer {
        Color::White => ep_sq.0.checked_sub(8).map(Square),
        Color::Black => ep_sq.0.checked_add(8).filter(|sq| *sq < 64).map(Square),
    }
}

fn ep_origin_square(capturer: Color, ep_sq: Square) -> Option<Square> {
    match capturer {
        Color::White => ep_sq.0.checked_add(8).filter(|sq| *sq < 64).map(Square),
        Color::Black => ep_sq.0.checked_sub(8).map(Square),
    }
}

fn fen_char_to_piece(c: char) -> Result<(Color, Piece), String> {
    match c {
        'P' => Ok((Color::White, Piece::Pawn)),
        'N' => Ok((Color::White, Piece::Knight)),
        'B' => Ok((Color::White, Piece::Bishop)),
        'R' => Ok((Color::White, Piece::Rook)),
        'Q' => Ok((Color::White, Piece::Queen)),
        'K' => Ok((Color::White, Piece::King)),
        'p' => Ok((Color::Black, Piece::Pawn)),
        'n' => Ok((Color::Black, Piece::Knight)),
        'b' => Ok((Color::Black, Piece::Bishop)),
        'r' => Ok((Color::Black, Piece::Rook)),
        'q' => Ok((Color::Black, Piece::Queen)),
        'k' => Ok((Color::Black, Piece::King)),
        c => Err(format!("invalid FEN piece char: {c}")),
    }
}

#[inline(always)]
fn encode_piece(color: Color, piece: Piece) -> u8 {
    color as u8 * 6 + piece as u8
}

#[inline(always)]
fn capture_or_quiet(target: Option<Color>, them: Color) -> u16 {
    if target == Some(them) { CAPTURE } else { QUIET }
}

#[inline(always)]
fn pawn_move_with_promotion(
    from: Square,
    to: Square,
    is_capture: bool,
    promotion: Option<Piece>,
) -> Option<Move> {
    let flag = match promotion {
        Some(piece) => promotion_flag(piece, is_capture)?,
        None if is_capture => CAPTURE,
        None => QUIET,
    };
    Some(Move::new(from, to, flag))
}

#[inline(always)]
fn promotion_flag(piece: Piece, is_capture: bool) -> Option<u16> {
    match (piece, is_capture) {
        (Piece::Knight, false) => Some(PROMO_KNIGHT),
        (Piece::Bishop, false) => Some(PROMO_BISHOP),
        (Piece::Rook, false) => Some(PROMO_ROOK),
        (Piece::Queen, false) => Some(PROMO_QUEEN),
        (Piece::Knight, true) => Some(PROMO_CAPTURE_KNIGHT),
        (Piece::Bishop, true) => Some(PROMO_CAPTURE_BISHOP),
        (Piece::Rook, true) => Some(PROMO_CAPTURE_ROOK),
        (Piece::Queen, true) => Some(PROMO_CAPTURE_QUEEN),
        _ => None,
    }
}

#[inline(always)]
fn decode_piece(encoded: u8) -> Option<(Color, Piece)> {
    if encoded >= 12 {
        return None;
    }

    let color = if encoded < 6 {
        Color::White
    } else {
        Color::Black
    };
    let piece = PIECE_FROM_ENCODED[encoded as usize];
    Some((color, piece))
}

// -----------------------------------------------------------------------
// Display
// -----------------------------------------------------------------------

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  +-----------------+")?;
        for rank in (0..8).rev() {
            write!(f, "{} | ", rank + 1)?;
            for file in 0..8u8 {
                let sq = Square(rank * 8 + file);
                if let Some((color, piece)) = self.piece_at(sq) {
                    let c = match piece {
                        Piece::Pawn => 'p',
                        Piece::Knight => 'n',
                        Piece::Bishop => 'b',
                        Piece::Rook => 'r',
                        Piece::Queen => 'q',
                        Piece::King => 'k',
                    };
                    let c = if color == Color::White {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    };
                    write!(f, "{c} ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "  +-----------------+")?;
        writeln!(f, "    a b c d e f g h")?;
        writeln!(f, "  Side: {:?}", self.side_to_move)?;
        writeln!(f, "  Castling: {}", self.castling.as_str())?;
        if self.ep_sq != 255 {
            writeln!(f, "  EP: {}", Square(self.ep_sq))?;
        }
        writeln!(f, "  Hash: 0x{:016X}", self.hash)?;
        Ok(())
    }
}

mod checks;
mod draw;
mod fen;
mod make;
mod see;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod repetition_tests;
