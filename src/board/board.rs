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
    // Construction
    // -----------------------------------------------------------------------

    pub fn starting_position() -> Self {
        Self::from_fen(STARTING_FEN).expect("starting FEN is valid")
    }

    /// A position from its piece placement, as another engine's search hands
    /// it over: twelve bitboards in `[WP..WK, BP..BK]` order, the side to
    /// move, the castling rights and the rule-50 clock, with no en-passant
    /// square and move number 1. It applies [`Board::from_fen`]'s checks
    /// without writing or parsing text, and reserves no move history, so a
    /// board built here is for evaluation: making moves on it grows the
    /// history on the heap.
    pub fn from_placement(
        pieces: [Bitboard; 12],
        side_to_move: Color,
        castling: CastlingRights,
        halfmove_clock: u8,
    ) -> Result<Self, String> {
        let mut board = Self {
            pieces: [Bitboard::EMPTY; 12],
            occupancy: [Bitboard::EMPTY; 2],
            all_occ: Bitboard::EMPTY,
            mailbox: [NO_PIECE; 64],
            side_to_move,
            castling: CastlingRights::NONE,
            ep_sq: 255,
            halfmove_clock,
            fullmove: 1,
            plies_from_null: 0,
            hash: 0,
            pawn_hash: 0,
            minor_hash: 0,
            non_pawn_hash: [0; 2],
            checkers: Bitboard::EMPTY,
            history: Vec::new(),
        };
        let back_ranks = Bitboard(0xFF00_0000_0000_00FF);
        for color in [Color::White, Color::Black] {
            for piece in Piece::ALL {
                let set = pieces[color as usize * 6 + piece as usize];
                if (set & board.all_occ).any() {
                    return Err("two pieces on one square".to_string());
                }
                if piece == Piece::Pawn && (set & back_ranks).any() {
                    return Err("pawns are not legal on the first or eighth rank".to_string());
                }
                for sq in set {
                    board.add_piece(color, piece, sq);
                    board.hash ^= ZOBRIST.piece(color, piece, sq);
                }
            }
        }
        if side_to_move == Color::Black {
            board.hash ^= ZOBRIST.side();
        }
        validate_castling_rights(&board, castling)?;
        board.castling = castling;
        board.hash ^= ZOBRIST.castling(castling);
        board.validate_position()?;
        board.checkers = board.calculate_checkers();
        Ok(board)
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Self {
            pieces: [Bitboard::EMPTY; 12],
            occupancy: [Bitboard::EMPTY; 2],
            all_occ: Bitboard::EMPTY,
            mailbox: [NO_PIECE; 64],
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            ep_sq: 255,
            halfmove_clock: 0,
            fullmove: 1,
            plies_from_null: 0,
            hash: 0,
            pawn_hash: 0,
            minor_hash: 0,
            non_pawn_hash: [0; 2],
            checkers: Bitboard::EMPTY,
            history: Vec::with_capacity(128),
        };

        let parts = fen.split_whitespace().collect::<Vec<_>>();
        if !(4..=6).contains(&parts.len()) {
            return Err("FEN must contain 4 to 6 fields".to_string());
        }

        // 1. Piece placement
        let placement = parts[0];
        let mut rank = 7u8;
        let mut file = 0u8;
        for ch in placement.chars() {
            match ch {
                '/' => {
                    if file != 8 {
                        return Err(format!("incomplete FEN rank before '/' in {placement}"));
                    }
                    if rank == 0 {
                        return Err(format!("too many FEN ranks in {placement}"));
                    }
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => {
                    file += ch as u8 - b'0';
                    if file > 8 {
                        return Err(format!("too many squares in FEN rank {rank}"));
                    }
                }
                c => {
                    let (color, piece) = fen_char_to_piece(c)?;
                    if file >= 8 {
                        return Err(format!("too many squares in FEN rank {rank}"));
                    }
                    if piece == Piece::Pawn && (rank == 0 || rank == 7) {
                        return Err("pawns are not legal on the first or eighth rank".to_string());
                    }
                    let sq = Square(rank * 8 + file);
                    board.add_piece(color, piece, sq);
                    board.hash ^= ZOBRIST.piece(color, piece, sq);
                    file += 1;
                }
            }
        }
        if rank != 0 || file != 8 {
            return Err(format!(
                "piece placement must contain 8 complete ranks: {placement}"
            ));
        }

        // 2. Side to move
        match parts[1] {
            "w" => board.side_to_move = Color::White,
            "b" => {
                board.side_to_move = Color::Black;
                board.hash ^= ZOBRIST.side();
            }
            s => return Err(format!("invalid side to move: {s}")),
        }

        // 3. Castling rights
        let castling_str = parts[2];
        let mut cr = CastlingRights::NONE;
        if castling_str.contains('-') && castling_str.len() > 1 {
            return Err(format!("invalid castling rights: {castling_str}"));
        }
        for c in castling_str.chars() {
            match c {
                'K' if !cr.has(CastlingRights::WHITE_KINGSIDE) => {
                    cr.0 |= CastlingRights::WHITE_KINGSIDE.0;
                }
                'Q' if !cr.has(CastlingRights::WHITE_QUEENSIDE) => {
                    cr.0 |= CastlingRights::WHITE_QUEENSIDE.0;
                }
                'k' if !cr.has(CastlingRights::BLACK_KINGSIDE) => {
                    cr.0 |= CastlingRights::BLACK_KINGSIDE.0;
                }
                'q' if !cr.has(CastlingRights::BLACK_QUEENSIDE) => {
                    cr.0 |= CastlingRights::BLACK_QUEENSIDE.0;
                }
                '-' => {}
                c => return Err(format!("invalid castling char: {c}")),
            }
        }
        validate_castling_rights(&board, cr)?;
        board.castling = cr;
        board.hash ^= ZOBRIST.castling(cr);

        // 4. En passant
        let ep_str = parts[3];
        let ep_candidate = if ep_str != "-" {
            let sq = Square::from_algebraic(ep_str)
                .ok_or_else(|| format!("invalid ep square: {ep_str}"))?;
            Some(sq)
        } else {
            None
        };

        // 5. Halfmove clock
        if let Some(s) = parts.get(4) {
            board.halfmove_clock = s
                .parse::<u8>()
                .map_err(|_| format!("invalid halfmove clock: {s}"))?;
        }

        // 6. Fullmove number
        if let Some(s) = parts.get(5) {
            board.fullmove = s
                .parse::<u16>()
                .map_err(|_| format!("invalid fullmove number: {s}"))?
                .max(1);
        }

        board.validate_position()?;
        if let Some(ep_sq) = ep_candidate {
            board.set_legal_ep_square(ep_sq)?;
        }
        board.checkers = board.calculate_checkers();
        Ok(board)
    }

    /// Serialize the board to a FEN string.
    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(80);

        // Piece placement (rank 8 down to rank 1)
        for rank in (0..8).rev() {
            let mut empty = 0u8;
            for file in 0..8u8 {
                let sq = Square(rank * 8 + file);
                if let Some((color, piece)) = self.piece_at(sq) {
                    if empty > 0 {
                        fen.push((b'0' + empty) as char);
                        empty = 0;
                    }
                    let c = match piece {
                        Piece::Pawn => 'p',
                        Piece::Knight => 'n',
                        Piece::Bishop => 'b',
                        Piece::Rook => 'r',
                        Piece::Queen => 'q',
                        Piece::King => 'k',
                    };
                    fen.push(if color == Color::White {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    });
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push((b'0' + empty) as char);
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(if self.side_to_move == Color::White {
            'w'
        } else {
            'b'
        });
        fen.push(' ');
        fen.push_str(self.castling.as_str());
        fen.push(' ');
        if self.ep_sq == 255 {
            fen.push('-');
        } else {
            fen.push_str(&Square(self.ep_sq).to_string());
        }
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove.to_string());
        fen
    }

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
    pub fn key_after_hint(&self, mv: Move) -> u64 {
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

    pub fn legal_move(&self, mv: Move) -> Option<Move> {
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

    /// Per-node direct-check masks, computed once per node, so a move's
    /// direct-check test is one bitboard membership test.
    pub(crate) fn check_info(&self) -> CheckInfo {
        crate::diag_count!(board_check_info_calls);
        let us = self.side_to_move;
        let them = !us;
        let ksq = self.king_sq(them);
        let atk = &ATTACKS;
        let occ = self.all_occ;
        let bishop_from_k = atk.bishop(ksq, occ);
        let rook_from_k = atk.rook(ksq, occ);
        let mut check_squares = [Bitboard::EMPTY; 6];
        // Squares from which OUR pawn attacks their king = the squares a
        // pawn of THEIR colour on ksq would attack (attack reciprocity).
        check_squares[Piece::Pawn as usize] = atk.pawn(them, ksq);
        check_squares[Piece::Knight as usize] = atk.knight(ksq);
        check_squares[Piece::Bishop as usize] = bishop_from_k;
        check_squares[Piece::Rook as usize] = rook_from_k;
        check_squares[Piece::Queen as usize] = bishop_from_k | rook_from_k;
        // King entry stays EMPTY: a king never gives direct check.
        CheckInfo { check_squares }
    }

    /// The squares the side not to move attacks, by piece type and in total.
    ///
    /// The side to move's king is removed from the occupancy first, so a
    /// slider attacking the king also attacks the squares behind it on the
    /// same line. A threat set answers "would a piece of ours stand attacked
    /// there", and for the king stepping back along the checking line the
    /// answer is yes.
    pub fn threats(&self) -> Threats {
        let us = self.side_to_move;
        let them = !us;
        let atk = &ATTACKS;
        let occ = self.all_occ ^ self.pieces(us, Piece::King);
        let mut by_piece = [Bitboard::EMPTY; 6];
        by_piece[Piece::Pawn as usize] = pawn_attacks_of(them, self.pieces(them, Piece::Pawn));
        by_piece[Piece::Knight as usize] = knight_attacks_of(self.pieces(them, Piece::Knight));
        for sq in self.pieces(them, Piece::Bishop) {
            by_piece[Piece::Bishop as usize] |= atk.bishop(sq, occ);
        }
        for sq in self.pieces(them, Piece::Rook) {
            by_piece[Piece::Rook as usize] |= atk.rook(sq, occ);
        }
        for sq in self.pieces(them, Piece::Queen) {
            by_piece[Piece::Queen as usize] |= atk.queen(sq, occ);
        }
        by_piece[Piece::King as usize] = atk.king(self.king_sq(them));
        let all = by_piece
            .iter()
            .fold(Bitboard::EMPTY, |all, &attacks| all | attacks);
        Threats { all, by_piece }
    }

    pub fn gives_check(&self, mv: Move) -> bool {
        crate::diag_count!(board_gives_check_full_calls);
        if mv.is_castling() {
            // After castling, the only piece that can give check is the rook
            // from its post-castle square. The king never gives check (kings
            // can't be adjacent), and no discovered check is possible: the
            // squares the king and rook vacate (e1/e8 and the corner rook
            // squares a1/h1/a8/h8) are all on the board edge/corner, so they
            // can never lie strictly between one of our sliders and the enemy
            // king. Use the post-castle occupancy so the rook's ray is
            // correctly blocked by our own pieces — notably the king on c1/c8
            // after a queenside castle blocks the d1/d8 rook toward a1/a8.
            let us = self.side_to_move;
            let them = !us;
            let king_from = mv.from_sq();
            let king_to = mv.to_sq();
            let (rook_from, rook_to) = if mv.flags() == CASTLE_KINGSIDE {
                if us == Color::White {
                    (Square::H1, Square::F1)
                } else {
                    (Square::H8, Square::F8)
                }
            } else if us == Color::White {
                (Square::A1, Square::D1)
            } else {
                (Square::A8, Square::D8)
            };
            let occ_after = (self.all_occ ^ Bitboard::from(king_from) ^ Bitboard::from(rook_from))
                | Bitboard::from(king_to)
                | Bitboard::from(rook_to);
            let their_king_bb = Bitboard::from(self.king_sq(them));
            return (ATTACKS.rook(rook_to, occ_after) & their_king_bb).any();
        }

        let us = self.side_to_move;
        let them = !us;
        let from = mv.from_sq();
        let to = mv.to_sq();
        let from_bb = Bitboard::from(from);
        let to_bb = Bitboard::from(to);
        let their_king = self.king_sq(them);
        let their_king_bb = Bitboard::from(their_king);
        let atk = &ATTACKS;

        let moving_piece = if mv.is_promo() {
            mv.promo_piece()
        } else {
            self.moving_piece(mv)
        };

        let mut occ = (self.all_occ ^ from_bb) | to_bb;
        if mv.is_en_passant() {
            let cap_sq = if us == Color::White {
                Square(to.0 - 8)
            } else {
                Square(to.0 + 8)
            };
            occ ^= Bitboard::from(cap_sq);
        }

        let direct = match moving_piece {
            Piece::Pawn => (atk.pawn(us, to) & their_king_bb).any(),
            Piece::Knight => (atk.knight(to) & their_king_bb).any(),
            Piece::Bishop => (atk.bishop(to, occ) & their_king_bb).any(),
            Piece::Rook => (atk.rook(to, occ) & their_king_bb).any(),
            Piece::Queen => (atk.queen(to, occ) & their_king_bb).any(),
            Piece::King => false,
        };
        if direct {
            return true;
        }

        let diagonal_sliders =
            (self.pieces(us, Piece::Bishop) | self.pieces(us, Piece::Queen)) & !from_bb;
        if (atk.bishop(their_king, occ) & diagonal_sliders).any() {
            return true;
        }

        let orthogonal_sliders =
            (self.pieces(us, Piece::Rook) | self.pieces(us, Piece::Queen)) & !from_bb;
        (atk.rook(their_king, occ) & orthogonal_sliders).any()
    }

    fn castling_move(&self, from: Square, to: Square, us: Color, them: Color) -> Option<Move> {
        if self.is_in_check() {
            return None;
        }

        let (king_sq, ks_flag, qs_flag, ks_rook, qs_rook, ks_empty, qs_empty, ks_safe, qs_safe) =
            if us == Color::White {
                (
                    Square::E1,
                    CastlingRights::WHITE_KINGSIDE,
                    CastlingRights::WHITE_QUEENSIDE,
                    Square::H1,
                    Square::A1,
                    Bitboard::from(Square::F1) | Bitboard::from(Square::G1),
                    Bitboard::from(Square::B1)
                        | Bitboard::from(Square::C1)
                        | Bitboard::from(Square::D1),
                    [Square::F1, Square::G1],
                    [Square::D1, Square::C1],
                )
            } else {
                (
                    Square::E8,
                    CastlingRights::BLACK_KINGSIDE,
                    CastlingRights::BLACK_QUEENSIDE,
                    Square::H8,
                    Square::A8,
                    Bitboard::from(Square::F8) | Bitboard::from(Square::G8),
                    Bitboard::from(Square::B8)
                        | Bitboard::from(Square::C8)
                        | Bitboard::from(Square::D8),
                    [Square::F8, Square::G8],
                    [Square::D8, Square::C8],
                )
            };

        if from != king_sq {
            return None;
        }
        if to == ks_safe[1] {
            if self.castling.has(ks_flag)
                && (self.all_occ & ks_empty).is_empty()
                && (self.pieces(us, Piece::Rook) & Bitboard::from(ks_rook)).any()
                && !self.is_attacked(ks_safe[0], them)
                && !self.is_attacked(ks_safe[1], them)
            {
                return Some(Move::new(from, to, CASTLE_KINGSIDE));
            }
        } else if to == qs_safe[1]
            && self.castling.has(qs_flag)
            && (self.all_occ & qs_empty).is_empty()
            && (self.pieces(us, Piece::Rook) & Bitboard::from(qs_rook)).any()
            && !self.is_attacked(qs_safe[0], them)
            && !self.is_attacked(qs_safe[1], them)
        {
            return Some(Move::new(from, to, CASTLE_QUEENSIDE));
        }
        None
    }

    fn king_safe_after(&self, mv: Move) -> bool {
        let us = self.side_to_move;
        let them = !us;
        let from = mv.from_sq();
        let to = mv.to_sq();
        let from_bb = Bitboard::from(from);
        let to_bb = Bitboard::from(to);
        let captured = if self.piece_at(to).is_some() {
            to_bb
        } else {
            Bitboard::EMPTY
        };

        if self.moving_piece(mv) == Piece::King {
            return mv.is_castling()
                || !self.attacked_by(to, them, self.all_occ ^ from_bb, !captured);
        }

        let king_sq = self.king_sq(us);
        let (occ_after, captured) = if mv.is_en_passant() {
            let cap_sq = if us == Color::White {
                Square(to.0 - 8)
            } else {
                Square(to.0 + 8)
            };
            (
                (self.all_occ ^ from_bb ^ Bitboard::from(cap_sq)) | to_bb,
                Bitboard::from(cap_sq),
            )
        } else {
            ((self.all_occ ^ from_bb) | to_bb, captured)
        };
        !self.attacked_by(king_sq, them, occ_after, !captured)
    }

    /// Rule-50 draw with mate precedence (the FIDE Laws' mate-first rule):
    /// at clock >= 100 the game is drawn UNLESS the position is checkmate —
    /// a mate delivered by the 100th-clock move wins. Stalemate at the
    /// boundary is a draw either way, so only the mated case needs the
    /// (rare) legal-move probe.
    #[inline(always)]
    fn is_rule50_draw(&self) -> bool {
        self.halfmove_clock >= 100 && (!self.is_in_check() || self.has_legal_move())
    }

    /// The search asks this per node at the rule-50 boundary, so the moves go
    /// to a stack list, and the list is kept out of line and off the search's
    /// frame.
    #[cold]
    #[inline(never)]
    fn has_legal_move(&self) -> bool {
        let mut moves = MoveList::new();
        self.generate_legal_movelist_into(&mut moves);
        !moves.is_empty()
    }

    pub fn can_declare_draw(&self) -> bool {
        self.is_rule50_draw() || self.has_insufficient_material() || self.is_threefold_repetition()
    }

    pub fn has_repeated_position(&self) -> bool {
        self.halfmove_clock >= 4 && self.is_repetition(2)
    }

    /// The search's draw at `ply` plies from the root: the fifty-move
    /// rule (mate first), material neither side can mate with, or a
    /// repetition, which needs one earlier occurrence inside the search or
    /// two anywhere since the last irreversible or null move.
    pub fn is_draw_at(&self, ply: usize) -> bool {
        self.is_dead_material() || self.is_rule50_draw() || self.repetition_draw_at(ply)
    }

    /// No pawn, rook or queen, and: three men or fewer; a minor each, unless
    /// a king stands in a corner, where a mate can still be forced; or two
    /// bishops on one side that stand on squares of one colour.
    pub fn is_dead_material(&self) -> bool {
        let both = |piece| self.pieces(Color::White, piece) | self.pieces(Color::Black, piece);
        if (both(Piece::Pawn) | both(Piece::Rook) | both(Piece::Queen)).any() {
            return false;
        }
        let men = self.occupied_count();
        if men != 4 {
            return men < 4;
        }
        let stm = self.side_to_move;
        if (self.pieces(stm, Piece::Knight) | self.pieces(stm, Piece::Bishop)).count() == 1 {
            const CORNERS: Bitboard = Bitboard(0x8100_0000_0000_0081);
            return (both(Piece::King) & CORNERS).is_empty();
        }
        if both(Piece::Knight).any() {
            return false;
        }
        const ONE_SQUARE_COLOUR: Bitboard = Bitboard(0x55AA_55AA_55AA_55AA);
        (both(Piece::Bishop) & ONE_SQUARE_COLOUR).count() != 1
    }

    /// How many plies back the key history holds positions this one could
    /// repeat: none past an irreversible or a null move.
    fn repetition_window(&self) -> usize {
        usize::from(self.halfmove_clock)
            .min(usize::from(self.plies_from_null))
            .min(self.history.len())
    }

    /// The key of the position `plies` plies back, inside the history.
    #[inline(always)]
    fn key_back(&self, plies: usize) -> u64 {
        self.history[self.history.len() - plies].hash
    }

    /// Whether the position `plies` back occurred again earlier inside
    /// `window`.
    fn repeats_back(&self, plies: usize, window: usize) -> bool {
        let key = self.key_back(plies);
        (plies + 4..=window)
            .step_by(2)
            .any(|older| self.key_back(older) == key)
    }

    /// A repetition draws when the nearest earlier occurrence lies inside the
    /// search, fewer than `ply` plies back, or had itself occurred before.
    fn repetition_draw_at(&self, ply: usize) -> bool {
        let window = self.repetition_window();
        (4..=window)
            .step_by(2)
            .find(|&plies| self.key_back(plies) == self.hash)
            .is_some_and(|nearest| nearest < ply || self.repeats_back(nearest, window))
    }

    /// Whether the side to move has a move that repeats a position that
    /// [`Board::is_draw_at`] would score as a draw: a reversible move joins
    /// this position to one an odd number of plies back, the opponent's moves
    /// since then cancel out, nothing stands between the move's squares, and
    /// the earlier position lies inside the search or had itself occurred
    /// before.
    pub fn upcoming_repetition(&self, ply: usize) -> bool {
        let window = self.repetition_window();
        if window < 3 {
            return false;
        }
        let side = ZOBRIST.side();
        // The piece-square change of the opponent's moves since `plies` back.
        let mut theirs = self.hash ^ self.key_back(1) ^ side;
        let mut plies = 3;
        while plies <= window {
            theirs ^= self.key_back(plies - 1) ^ self.key_back(plies) ^ side;
            if theirs == 0
                && let Some((from, to)) = CUCKOO.reversible_move(self.hash ^ self.key_back(plies))
                && (between(from, to) & self.all_occ).is_empty()
                && (ply > plies || self.repeats_back(plies, window))
            {
                return true;
            }
            plies += 2;
        }
        false
    }

    /// The arbiter's draws a move can reach at the tablebase root: a
    /// claimable threefold, or with `rule50` a rule-50 draw. The search's
    /// aggressive twofold does not count here, because a root move ranked as
    /// a draw is one the game would score as a draw.
    pub fn is_arbiter_draw(&self, rule50: bool) -> bool {
        (rule50 && self.is_rule50_draw()) || self.is_threefold_repetition()
    }

    /// Whether any position since the last zeroing move, at least four plies
    /// back from the oldest, repeats an earlier one in that window. A winning
    /// side that has already repeated must make progress by distance to
    /// zeroing, not merely keep the win.
    pub fn has_repetition_since_zeroing(&self) -> bool {
        let window = usize::from(self.halfmove_clock).min(self.history.len());
        let hash_at = |plies_back: usize| {
            if plies_back == 0 {
                self.hash
            } else {
                self.history[self.history.len() - plies_back].hash
            }
        };
        (0..=window.saturating_sub(4)).any(|newer| {
            (newer + 4..=window)
                .step_by(2)
                .any(|older| hash_at(older) == hash_at(newer))
        }) && window >= 4
    }

    /// Whether distance to zeroing is distance to mate here: no pawns, and
    /// three men, or four with neither queen nor rook.
    pub fn dtz_is_dtm(&self) -> bool {
        let pawns = self.pieces(Color::White, Piece::Pawn) | self.pieces(Color::Black, Piece::Pawn);
        let heavy = self.pieces(Color::White, Piece::Queen)
            | self.pieces(Color::Black, Piece::Queen)
            | self.pieces(Color::White, Piece::Rook)
            | self.pieces(Color::Black, Piece::Rook);
        let men = self.occupied_count();
        !pawns.any() && (men == 3 || (men == 4 && !heavy.any()))
    }

    pub fn has_non_pawn_material(&self, color: Color) -> bool {
        (self.pieces(color, Piece::Knight)
            | self.pieces(color, Piece::Bishop)
            | self.pieces(color, Piece::Rook)
            | self.pieces(color, Piece::Queen))
        .any()
    }

    /// Rebuild every derived field from the mailbox and compare against
    /// what make/unmake has been maintaining incrementally.
    ///
    /// `Board` keeps five redundant representations of the same position —
    /// the 12 piece bitboards, the two occupancies, `all_occ`, the mailbox,
    /// and five Zobrist keys. `to_fen()` reads the mailbox, so a FEN
    /// comparison validates the mailbox and nothing else. A desync in the
    /// rest is silent: the auxiliary keys are CACHE KEYS for the pawn cache,
    /// correction history and pawn history, so a drifted key returns another
    /// position's cached evaluation rather than crashing.
    ///
    /// Returns `Err` with the first mismatch rather than panicking, so tests
    /// can report instead of aborting. Not on any hot path.
    pub fn check_consistency(&self) -> Result<(), String> {
        let mut pieces = [Bitboard::EMPTY; 12];
        let mut occupancy = [Bitboard::EMPTY; 2];
        let mut hash = 0u64;
        let mut pawn_hash = 0u64;
        let mut minor_hash = 0u64;
        let mut non_pawn_hash = [0u64; 2];

        for index in 0..64u8 {
            let sq = Square(index);
            let Some((color, piece)) = decode_piece(self.mailbox[sq.index()]) else {
                continue;
            };
            let bb = Bitboard::from(sq);
            pieces[color as usize * 6 + piece as usize] |= bb;
            occupancy[color as usize] |= bb;
            let piece_key = ZOBRIST.piece(color, piece, sq);
            hash ^= piece_key;
            match piece {
                Piece::Pawn => pawn_hash ^= piece_key,
                Piece::Knight | Piece::Bishop => {
                    minor_hash ^= piece_key;
                    non_pawn_hash[color as usize] ^= piece_key;
                }
                Piece::Rook | Piece::Queen | Piece::King => {
                    non_pawn_hash[color as usize] ^= piece_key;
                }
            }
        }

        if self.side_to_move == Color::Black {
            hash ^= ZOBRIST.side();
        }
        hash ^= ZOBRIST.castling(self.castling);
        if self.ep_sq != 255 {
            hash ^= ZOBRIST.ep(Square(self.ep_sq).file());
        }

        for (i, expected) in pieces.iter().enumerate() {
            if self.pieces[i] != *expected {
                return Err(format!(
                    "pieces[{i}] desynced from mailbox: have {:?}, mailbox implies {:?}",
                    self.pieces[i], expected
                ));
            }
        }
        for (i, expected) in occupancy.iter().enumerate() {
            if self.occupancy[i] != *expected {
                return Err(format!("occupancy[{i}] desynced from mailbox"));
            }
        }
        let all = occupancy[0] | occupancy[1];
        if self.all_occ != all {
            return Err("all_occ is not the union of both occupancies".to_string());
        }
        if self.hash != hash {
            return Err(format!(
                "hash desynced: incremental {:#018x}, recomputed {hash:#018x}",
                self.hash
            ));
        }
        if self.pawn_hash != pawn_hash {
            return Err(format!(
                "pawn_hash desynced: incremental {:#018x}, recomputed {pawn_hash:#018x}",
                self.pawn_hash
            ));
        }
        if self.minor_hash != minor_hash {
            return Err(format!(
                "minor_hash desynced: incremental {:#018x}, recomputed {minor_hash:#018x}",
                self.minor_hash
            ));
        }
        for color in [Color::White, Color::Black] {
            let i = color as usize;
            if self.non_pawn_hash[i] != non_pawn_hash[i] {
                return Err(format!(
                    "non_pawn_hash[{color:?}] desynced: incremental {:#018x}, recomputed {:#018x}",
                    self.non_pawn_hash[i], non_pawn_hash[i]
                ));
            }
        }
        let checkers = self.calculate_checkers();
        if self.checkers != checkers {
            return Err("checkers desynced from a fresh calculation".to_string());
        }
        Ok(())
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

    #[inline(always)]
    pub fn attackers_to_color(&self, sq: Square, occ: Bitboard, color: Color) -> Bitboard {
        let atk = &ATTACKS;
        let diagonal = self.pieces(color, Piece::Bishop) | self.pieces(color, Piece::Queen);
        let orthogonal = self.pieces(color, Piece::Rook) | self.pieces(color, Piece::Queen);

        (atk.pawn(!color, sq) & self.pieces(color, Piece::Pawn)
            | atk.knight(sq) & self.pieces(color, Piece::Knight)
            | atk.king(sq) & self.pieces(color, Piece::King)
            | atk.bishop(sq, occ) & diagonal
            | atk.rook(sq, occ) & orthogonal)
            & occ
    }

    /// Whether `attacker` attacks `sq` through the occupancy `occ`, counting
    /// only its pieces on `present`: the one attacked-square test. It reads
    /// the leapers first and the two slider lookups last, so a pawn, knight
    /// or king answers without them. `present` is `occ` to drop the pieces a
    /// line of play has taken off, everything but a captured piece's square
    /// to test after a capture, and the full board otherwise.
    #[inline(always)]
    pub(crate) fn attacked_by(
        &self,
        sq: Square,
        attacker: Color,
        occ: Bitboard,
        present: Bitboard,
    ) -> bool {
        let atk = &ATTACKS;
        let pieces = |piece| self.pieces(attacker, piece) & present;
        if (atk.pawn(!attacker, sq) & pieces(Piece::Pawn)).any() {
            return true;
        }
        if (atk.knight(sq) & pieces(Piece::Knight)).any() {
            return true;
        }
        if (atk.king(sq) & pieces(Piece::King)).any() {
            return true;
        }
        let queens = pieces(Piece::Queen);
        if (atk.bishop(sq, occ) & (pieces(Piece::Bishop) | queens)).any() {
            return true;
        }
        (atk.rook(sq, occ) & (pieces(Piece::Rook) | queens)).any()
    }

    /// Select the least-valued LEGAL recapturer under the evolving exchange
    /// occupancy. Original piece sets remain valid off `target`: every piece
    /// that moved there has had its source removed from `occ`. Keep target
    /// occupied as a ray blocker, but exclude its original (captured) occupant
    /// from enemy attacks when checking the recapturer's king.
    fn see_recapturer(
        &self,
        target: Square,
        occ: Bitboard,
        side: Color,
    ) -> Option<(Square, Piece)> {
        let mut attackers = self.attackers_to_color(target, occ, side);
        while attackers.any() {
            let (from, piece) = self.least_valuable_attacker(attackers, side);
            let after = occ ^ Bitboard::from(from);
            let king = if piece == Piece::King {
                target
            } else {
                self.king_sq(side)
            };
            if (self.attackers_to_color(king, after, !side) & !Bitboard::from(target)).is_empty() {
                return Some((from, piece));
            }
            attackers ^= Bitboard::from(from);
        }
        None
    }

    /// Queen promotion is optimal in this material-only exchange: if the
    /// promoted piece is recaptured, the extra gain and loss cancel; otherwise
    /// its larger gain wins. Every recapture removes it, so the choice cannot
    /// change the legality of that recapture. This is not a tactical claim.
    fn see_recapture_piece(piece: Piece, target: Square) -> Piece {
        if piece == Piece::Pawn && matches!(target.rank(), Rank::R1 | Rank::R8) {
            Piece::Queen
        } else {
            piece
        }
    }

    fn see_occupancy(&self, mv: Move) -> Bitboard {
        let target = mv.to_sq();
        let mut occ = self.all_occ ^ Bitboard::from(mv.from_sq());
        if mv.is_en_passant() {
            let captured = if self.side_to_move == Color::White {
                Square(target.0 - 8)
            } else {
                Square(target.0 + 8)
            };
            occ ^= Bitboard::from(captured);
        }
        occ | Bitboard::from(target)
    }

    #[inline(always)]
    pub fn see(&self, mv: Move) -> i32 {
        crate::diag_count!(board_see_full_calls);
        self.see_with_values(mv, PRODUCTION_SEE_VALUES)
    }

    #[inline(always)]
    pub fn see_with_values(&self, mv: Move, values: SeeValues) -> i32 {
        let Some(victim) = self.captured_piece(mv) else {
            return if mv.is_promo() {
                values.value(mv.promo_piece()) - values.value(Piece::Pawn)
            } else {
                0
            };
        };

        let target = mv.to_sq();
        let mut occ = self.see_occupancy(mv);
        let mut side = self.side_to_move;
        let mut gains = [0i32; 32];
        let mut depth = 0usize;
        gains[0] = values.value(victim);
        let mut occupant = self.moving_piece(mv);
        if mv.is_promo() {
            occupant = mv.promo_piece();
            gains[0] += values.value(occupant) - values.value(Piece::Pawn);
        }

        // A legal king capture ends the exchange; kings are never victims.
        while occupant != Piece::King {
            side = !side;
            let Some((from, piece)) = self.see_recapturer(target, occ, side) else {
                break;
            };
            let promoted = Self::see_recapture_piece(piece, target);
            let gain = values.value(occupant) + values.value(promoted) - values.value(piece);
            depth += 1;
            gains[depth] = gain - gains[depth - 1];
            occupant = promoted;
            occ ^= Bitboard::from(from);
        }

        while depth > 0 {
            depth -= 1;
            gains[depth] = -gains[depth + 1].max(-gains[depth]);
        }
        gains[0]
    }

    #[inline(always)]
    pub fn see_ge(&self, mv: Move, threshold: i32) -> bool {
        crate::diag_count!(board_see_threshold_calls);
        self.see_ge_impl(mv, threshold, QuietSee::Gain, PRODUCTION_SEE_VALUES)
    }

    #[inline(always)]
    pub fn see_ge_with_values(&self, mv: Move, threshold: i32, values: SeeValues) -> bool {
        self.see_ge_impl(mv, threshold, QuietSee::Gain, values)
    }

    /// The compact kernel's exchange test, as its donor's: every move is
    /// played through the exchange, a promotion push with the promoted piece
    /// at risk. A quiet move asks whether it hangs the piece it moved: nothing
    /// was won, so the balance starts at 0 with the moved piece at risk, where
    /// [`Board::see_ge`] answers it by its gain alone. A queen pushed onto a
    /// defended square is taken, so the push loses its pawn.
    pub fn see_ge_exchange_with_values(&self, mv: Move, threshold: i32, values: SeeValues) -> bool {
        self.see_ge_impl(mv, threshold, QuietSee::Exchange, values)
    }

    fn see_ge_impl(&self, mv: Move, threshold: i32, quiet: QuietSee, values: SeeValues) -> bool {
        let answered_by_gain = !mv.is_capture()
            && match quiet {
                QuietSee::Gain => true,
                QuietSee::Exchange => false,
            };
        if answered_by_gain {
            let gain = if mv.is_promo() {
                values.value(mv.promo_piece()) - values.value(Piece::Pawn)
            } else {
                0
            };
            return gain >= threshold;
        }

        let mut gain = self
            .captured_piece(mv)
            .map_or(0, |piece| values.value(piece));
        let mut occupant = self.moving_piece(mv);
        if mv.is_promo() {
            occupant = mv.promo_piece();
            gain += values.value(occupant) - values.value(Piece::Pawn);
        }
        if gain < threshold {
            return false;
        }

        let target = mv.to_sq();
        let mut occ = self.see_occupancy(mv);
        let mut side = self.side_to_move;
        let mut result = true;
        // We pass iff the opponent cannot gain >= gain - threshold + 1.
        // At each optional recapture V = max(0, capture_gain - next_V).
        // For positive `limit`, V >= limit iff next_V < capture_gain-limit+1.
        // Toggling result expresses that negation; +1 preserves equality.
        let mut limit = gain - threshold + 1;
        while occupant != Piece::King {
            side = !side;
            let Some((from, piece)) = self.see_recapturer(target, occ, side) else {
                break;
            };
            let promoted = Self::see_recapture_piece(piece, target);
            let capture_gain =
                values.value(occupant) + values.value(promoted) - values.value(piece);
            if capture_gain < limit {
                break;
            }
            limit = capture_gain - limit + 1;
            result = !result;
            occupant = promoted;
            occ ^= Bitboard::from(from);
        }
        result
    }

    pub fn game_result(&self) -> Option<GameResult> {
        if self.can_declare_draw() {
            return Some(GameResult::Draw);
        }

        if !generate_legal_moves(self).is_empty() {
            return None;
        }

        if self.is_in_check() {
            match self.side_to_move {
                Color::White => Some(GameResult::BlackCheckmates),
                Color::Black => Some(GameResult::WhiteCheckmates),
            }
        } else {
            Some(GameResult::Stalemate)
        }
    }

    // -----------------------------------------------------------------------
    // Check / attack queries
    // -----------------------------------------------------------------------

    /// Is `sq` attacked by `attacker` on the board as it stands?
    #[inline(always)]
    pub(crate) fn is_attacked(&self, sq: Square, attacker: Color) -> bool {
        self.attacked_by(sq, attacker, self.all_occ, Bitboard::FULL)
    }

    /// Is the side-to-move's king currently in check?
    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        self.checkers.any()
    }

    #[inline(always)]
    pub fn checkers(&self) -> Bitboard {
        self.checkers
    }

    /// Bitboard of all pieces that attack the given square (any color).
    #[inline(always)]
    pub fn attackers_to(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let atk = &ATTACKS;
        atk.pawn(Color::Black, sq) & self.pieces(Color::White, Piece::Pawn)
            | atk.pawn(Color::White, sq) & self.pieces(Color::Black, Piece::Pawn)
            | atk.knight(sq)
                & (self.pieces(Color::White, Piece::Knight)
                    | self.pieces(Color::Black, Piece::Knight))
            | atk.king(sq)
                & (self.pieces(Color::White, Piece::King) | self.pieces(Color::Black, Piece::King))
            | atk.bishop(sq, occ)
                & (self.pieces(Color::White, Piece::Bishop)
                    | self.pieces(Color::Black, Piece::Bishop)
                    | self.pieces(Color::White, Piece::Queen)
                    | self.pieces(Color::Black, Piece::Queen))
            | atk.rook(sq, occ)
                & (self.pieces(Color::White, Piece::Rook)
                    | self.pieces(Color::Black, Piece::Rook)
                    | self.pieces(Color::White, Piece::Queen)
                    | self.pieces(Color::Black, Piece::Queen))
    }

    // -----------------------------------------------------------------------
    // Make / Unmake
    // -----------------------------------------------------------------------

    /// Apply a move in-place, computing the new checker set from scratch. The
    /// move must be legal.
    #[inline(always)]
    pub fn make_move(&mut self, mv: Move) {
        crate::diag_count!(board_make_plain_calls);
        self.make_move_inner(mv, None);
    }

    /// Play `mv` when the caller ALREADY knows whether it gives check.
    ///
    /// `make_move` otherwise runs [`Board::calculate_checkers`] — four attack
    /// lookups, two of them slider lookups — on every single move, and the
    /// overwhelming majority of moves produce `EMPTY`. A caller that already
    /// knows the answer turns the common case into a store of `EMPTY`.
    /// Checking moves still take the exact computation, because evasion
    /// generation needs the precise checker set.
    ///
    /// The hint is `debug_assert!`ed against the real computation, and
    /// `tests/board_differential.rs` independently rebuilds `checkers` after
    /// every make and unmake — a wrong hint fails loudly rather than
    /// producing illegal moves.
    pub fn make_move_with_check(&mut self, mv: Move, gives_check: bool) {
        crate::diag_count!(board_make_with_check_calls);
        self.make_move_inner(mv, Some(gives_check));
    }

    fn make_move_inner(&mut self, mv: Move, check_hint: Option<bool>) {
        let from = mv.from_sq();
        let to = mv.to_sq();
        let flags = mv.flags();
        let us = self.side_to_move;
        let them = !us;

        let zob = &ZOBRIST;

        let old_castling = self.castling;
        let old_ep_sq = self.ep_sq;
        let old_halfmove_clock = self.halfmove_clock;
        let old_fullmove = self.fullmove;
        let old_plies_from_null = self.plies_from_null;
        self.plies_from_null = self.plies_from_null.saturating_add(1);
        let old_hash = self.hash;
        let old_checkers = self.checkers;
        let mut captured = 255;

        // Halfmove clock: reset on pawn move or capture; increment otherwise.
        // We set it properly below after determining if it's a pawn move.

        // Remove old EP contribution from hash
        if self.ep_sq != 255 {
            self.hash ^= zob.ep(Square(self.ep_sq).file());
        }
        self.ep_sq = 255;

        debug_assert!(self.mailbox[from.index()] < 12);
        let moving_piece = self.occupant_type(from);

        if flags == QUIET {
            // Fused ordinary relocation. `to` is empty and the piece
            // keeps its identity, so one from/to mask and one paired key
            // reproduce remove_piece(from) + add_piece(to) exactly.
            self.move_piece(us, moving_piece, from, to);
            self.hash ^= zob.piece(us, moving_piece, from) ^ zob.piece(us, moving_piece, to);
            if moving_piece == Piece::Pawn {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock = self.halfmove_clock.saturating_add(1);
            }
        } else {
            // Remove moving piece from origin
            self.remove_piece(us, moving_piece, from);
            self.hash ^= zob.piece(us, moving_piece, from);

            // Handle en passant capture
            if flags == EN_PASSANT {
                let ep_cap_sq = if us == Color::White {
                    Square(to.0 - 8)
                } else {
                    Square(to.0 + 8)
                };
                captured = encode_piece(them, Piece::Pawn);
                self.remove_piece(them, Piece::Pawn, ep_cap_sq);
                self.hash ^= zob.piece(them, Piece::Pawn, ep_cap_sq);
                self.halfmove_clock = 0;
            } else if flags == CAPTURE || flags >= PROMO_CAPTURE_KNIGHT {
                // Regular capture (including promo-captures)
                debug_assert!(self.mailbox[to.index()] < 12);
                let captured_piece = self.occupant_type(to);
                captured = encode_piece(them, captured_piece);
                self.remove_piece(them, captured_piece, to);
                self.hash ^= zob.piece(them, captured_piece, to);
                self.halfmove_clock = 0;
            } else if moving_piece == Piece::Pawn {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock = self.halfmove_clock.saturating_add(1);
            }

            // Place moving piece on destination (or promotion piece)
            if flags >= PROMO_KNIGHT {
                let promo = mv.promo_piece();
                self.add_piece(us, promo, to);
                self.hash ^= zob.piece(us, promo, to);
            } else {
                self.add_piece(us, moving_piece, to);
                self.hash ^= zob.piece(us, moving_piece, to);
            }
        }

        // Castling: move the rook as well
        match flags {
            CASTLE_KINGSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::H1, Square::F1)
                } else {
                    (Square::H8, Square::F8)
                };
                self.remove_piece(us, Piece::Rook, rook_from);
                self.hash ^= zob.piece(us, Piece::Rook, rook_from);
                self.add_piece(us, Piece::Rook, rook_to);
                self.hash ^= zob.piece(us, Piece::Rook, rook_to);
            }
            CASTLE_QUEENSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::A1, Square::D1)
                } else {
                    (Square::A8, Square::D8)
                };
                self.remove_piece(us, Piece::Rook, rook_from);
                self.hash ^= zob.piece(us, Piece::Rook, rook_from);
                self.add_piece(us, Piece::Rook, rook_to);
                self.hash ^= zob.piece(us, Piece::Rook, rook_to);
            }
            DOUBLE_PUSH => {
                // Set en passant square (one step behind the destination)
                let ep = if us == Color::White {
                    Square(to.0 - 8)
                } else {
                    Square(to.0 + 8)
                };
                if self.legal_ep_capture_exists(them, ep) {
                    self.ep_sq = ep.0;
                    self.hash ^= zob.ep(ep.file());
                }
            }
            _ => {}
        }

        // Update castling rights
        let new_castling = self.castling.update(from, to);
        if new_castling != self.castling {
            self.hash ^= zob.castling(self.castling) ^ zob.castling(new_castling);
            self.castling = new_castling;
        }

        // Flip side to move
        self.hash ^= zob.side();
        self.side_to_move = them;

        // Fullmove counter
        if us == Color::Black {
            self.fullmove = self.fullmove.checked_add(1).unwrap_or(MAX_FULLMOVE);
        }
        #[cfg(feature = "diag")]
        {
            crate::diag_count!(board_history_pushes);
            if self.history.len() == self.history.capacity() {
                crate::diag_count!(board_history_growths);
            }
        }
        self.history.push(UnmakeInfo {
            captured,
            castling: old_castling,
            ep_sq: old_ep_sq,
            halfmove_clock: old_halfmove_clock,
            fullmove: old_fullmove,
            plies_from_null: old_plies_from_null,
            hash: old_hash,
            checkers: old_checkers,
        });
        self.checkers = match check_hint {
            Some(false) => {
                debug_assert!(
                    self.calculate_checkers().is_empty(),
                    "check hint claimed no check, but {mv} gives check"
                );
                Bitboard::EMPTY
            }
            hint => {
                let checkers = self.calculate_checkers();
                debug_assert!(
                    hint != Some(true) || !checkers.is_empty(),
                    "check hint claimed check, but {mv} does not give check"
                );
                checkers
            }
        };
    }

    pub fn make_null_move(&mut self) {
        crate::diag_count!(board_make_null_calls);
        debug_assert!(!self.is_in_check(), "null move while in check");
        let old_castling = self.castling;
        let old_ep_sq = self.ep_sq;
        let old_halfmove_clock = self.halfmove_clock;
        let old_fullmove = self.fullmove;
        let old_plies_from_null = self.plies_from_null;
        self.plies_from_null = 0;
        let old_hash = self.hash;
        let old_checkers = self.checkers;

        if self.ep_sq != 255 {
            self.hash ^= ZOBRIST.ep(Square(self.ep_sq).file());
            self.ep_sq = 255;
        }
        if self.side_to_move == Color::Black {
            self.fullmove = self.fullmove.checked_add(1).unwrap_or(MAX_FULLMOVE);
        }
        self.halfmove_clock = self.halfmove_clock.saturating_add(1);
        self.side_to_move = !self.side_to_move;
        self.hash ^= ZOBRIST.side();
        #[cfg(feature = "diag")]
        {
            crate::diag_count!(board_history_pushes);
            if self.history.len() == self.history.capacity() {
                crate::diag_count!(board_history_growths);
            }
        }
        self.history.push(UnmakeInfo {
            captured: NO_PIECE,
            castling: old_castling,
            ep_sq: old_ep_sq,
            halfmove_clock: old_halfmove_clock,
            fullmove: old_fullmove,
            plies_from_null: old_plies_from_null,
            hash: old_hash,
            checkers: old_checkers,
        });
        self.checkers = Bitboard::EMPTY;
    }

    pub fn unmake_null_move(&mut self) {
        crate::diag_count!(board_unmake_null_calls);
        let info = self
            .history
            .pop()
            .expect("unmake_null_move with empty history");
        debug_assert_eq!(info.captured, NO_PIECE);
        self.side_to_move = !self.side_to_move;
        self.castling = info.castling;
        self.ep_sq = info.ep_sq;
        self.halfmove_clock = info.halfmove_clock;
        self.fullmove = info.fullmove;
        self.plies_from_null = info.plies_from_null;
        self.hash = info.hash;
        self.checkers = info.checkers;
    }

    /// Undo the last move.
    #[inline(always)]
    pub fn unmake_move(&mut self, mv: Move) {
        crate::diag_count!(board_unmake_calls);
        let info = self.history.pop().expect("unmake_move with empty history");

        let from = mv.from_sq();
        let to = mv.to_sq();
        let flags = mv.flags();

        // Restore side to move (it was flipped by make_move)
        self.side_to_move = !self.side_to_move;
        let us = self.side_to_move;
        let _them = !us;

        // Restore state fields
        self.castling = info.castling;
        self.ep_sq = info.ep_sq;
        self.halfmove_clock = info.halfmove_clock;
        self.fullmove = info.fullmove;
        self.plies_from_null = info.plies_from_null;
        self.hash = info.hash;
        self.checkers = info.checkers;

        // Move the piece back from `to` to `from`
        if flags == QUIET {
            // Fused ordinary relocation; mirrors the make-side path.
            debug_assert!(self.mailbox[to.index()] < 12);
            let p = self.occupant_type(to);
            self.move_piece(us, p, to, from);
        } else {
            let moved_piece = if flags >= PROMO_KNIGHT {
                // Promotion: remove the promo piece, restore a pawn
                let promo = mv.promo_piece();
                self.remove_piece(us, promo, to);
                Piece::Pawn
            } else {
                debug_assert!(self.mailbox[to.index()] < 12);
                let p = self.occupant_type(to);
                self.remove_piece(us, p, to);
                p
            };

            self.add_piece(us, moved_piece, from);
        }

        // Restore captured piece
        if info.captured != 255 {
            let cap_color = if info.captured < 6 {
                Color::White
            } else {
                Color::Black
            };
            let cap_piece = PIECE_FROM_ENCODED[info.captured as usize];
            let cap_sq = if flags == EN_PASSANT {
                if us == Color::White {
                    Square(to.0 - 8)
                } else {
                    Square(to.0 + 8)
                }
            } else {
                to
            };
            self.add_piece(cap_color, cap_piece, cap_sq);
        }

        // Undo castling rook move
        match flags {
            CASTLE_KINGSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::H1, Square::F1)
                } else {
                    (Square::H8, Square::F8)
                };
                self.remove_piece(us, Piece::Rook, rook_to);
                self.add_piece(us, Piece::Rook, rook_from);
            }
            CASTLE_QUEENSIDE => {
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::A1, Square::D1)
                } else {
                    (Square::A8, Square::D8)
                };
                self.remove_piece(us, Piece::Rook, rook_to);
                self.add_piece(us, Piece::Rook, rook_from);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Toggle `key` into the pawn, minor-piece and non-pawn keys a piece of
    /// type `piece` belongs to. A mask per type, not a branch: the moving
    /// piece's type is as unpredictable as the move.
    #[inline(always)]
    fn toggle_derived_keys(&mut self, color: Color, piece: Piece, key: u64) {
        let [pawn, minor, non_pawn] = DERIVED_KEY_MASKS[piece as usize];
        self.pawn_hash ^= key & pawn;
        self.minor_hash ^= key & minor;
        self.non_pawn_hash[color as usize] ^= key & non_pawn;
    }

    #[inline(always)]
    fn add_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let bb = Bitboard::from(sq);
        self.mailbox[sq.index()] = encode_piece(color, piece);
        self.pieces[color as usize * 6 + piece as usize] |= bb;
        self.occupancy[color as usize] |= bb;
        self.all_occ |= bb;
        let piece_key = ZOBRIST.piece(color, piece, sq);
        self.toggle_derived_keys(color, piece, piece_key);
    }

    /// Fused ordinary relocation of `piece` from `from` to `to`.
    ///
    /// Only valid when `to` is empty and the piece keeps its identity, i.e. for
    /// `QUIET` moves. The combined from/to mask and paired key are exactly the
    /// XOR of [`Board::remove_piece`] and [`Board::add_piece`].
    #[inline(always)]
    fn move_piece(&mut self, color: Color, piece: Piece, from: Square, to: Square) {
        debug_assert!(self.mailbox[to.index()] == NO_PIECE);
        debug_assert!(from != to);
        let mask = Bitboard::from(from) | Bitboard::from(to);
        self.mailbox[from.index()] = NO_PIECE;
        self.mailbox[to.index()] = encode_piece(color, piece);
        self.pieces[color as usize * 6 + piece as usize] ^= mask;
        self.occupancy[color as usize] ^= mask;
        self.all_occ ^= mask;
        let key = ZOBRIST.piece(color, piece, from) ^ ZOBRIST.piece(color, piece, to);
        self.toggle_derived_keys(color, piece, key);
    }

    #[inline(always)]
    fn remove_piece(&mut self, color: Color, piece: Piece, sq: Square) {
        let bb = Bitboard::from(sq);
        self.mailbox[sq.index()] = NO_PIECE;
        self.pieces[color as usize * 6 + piece as usize] ^= bb;
        self.occupancy[color as usize] ^= bb;
        self.all_occ ^= bb;
        let piece_key = ZOBRIST.piece(color, piece, sq);
        self.toggle_derived_keys(color, piece, piece_key);
    }

    /// The type of the piece on `sq`, which must be occupied: debug builds
    /// check it, and a release read of an empty square yields some type.
    #[inline(always)]
    fn occupant_type(&self, sq: Square) -> Piece {
        debug_assert!(self.mailbox[sq.index()] < 12);
        // `& 15` into the padded 16-entry table — check elided, no unsafe.
        PIECE_FROM_ENCODED[self.mailbox[sq.index()] as usize & 15]
    }

    #[inline(always)]
    fn least_valuable_attacker(&self, attackers: Bitboard, color: Color) -> (Square, Piece) {
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let bb = attackers & self.pieces(color, piece);
            if bb.any() {
                return (bb.lsb(), piece);
            }
        }
        unreachable!("least_valuable_attacker called with no attackers")
    }

    #[inline(always)]
    fn calculate_checkers(&self) -> Bitboard {
        crate::diag_count!(board_calculate_checkers_calls);
        let attacker = !self.side_to_move;
        let king_sq = self.king_sq(self.side_to_move);
        let atk = &ATTACKS;
        let diagonal = self.pieces(attacker, Piece::Bishop) | self.pieces(attacker, Piece::Queen);
        let orthogonal = self.pieces(attacker, Piece::Rook) | self.pieces(attacker, Piece::Queen);

        atk.pawn(!attacker, king_sq) & self.pieces(attacker, Piece::Pawn)
            | atk.knight(king_sq) & self.pieces(attacker, Piece::Knight)
            | atk.bishop(king_sq, self.all_occ) & diagonal
            | atk.rook(king_sq, self.all_occ) & orthogonal
    }

    fn validate_position(&self) -> Result<(), String> {
        let white_king = self.pieces(Color::White, Piece::King);
        let black_king = self.pieces(Color::Black, Piece::King);
        if white_king.count() != 1 || black_king.count() != 1 {
            return Err("FEN must contain exactly one king for each side".to_string());
        }

        // Pawn count and promoted-piece consistency, which the other checks
        // (back-rank pawns, king counts, adjacency, side not to move in check)
        // do not cover: a FEN with 9 pawns must be rejected, not searched.
        for color in [Color::White, Color::Black] {
            let pawns = infra::to_i32(self.pieces(color, Piece::Pawn).count());
            if pawns > 8 {
                return Err(format!("{color:?} has more than 8 pawns"));
            }
            // A side can have at most (8 - pawns) promoted pieces: each promotion
            // consumes one of its pawns.
            let promoted = (infra::to_i32(self.pieces(color, Piece::Knight).count()) - 2).max(0)
                + (infra::to_i32(self.pieces(color, Piece::Bishop).count()) - 2).max(0)
                + (infra::to_i32(self.pieces(color, Piece::Rook).count()) - 2).max(0)
                + (infra::to_i32(self.pieces(color, Piece::Queen).count()) - 1).max(0);
            if promoted > 8 - pawns {
                return Err(format!(
                    "{color:?} has more promoted pieces than missing pawns allow"
                ));
            }
        }

        let white_king_sq = white_king.lsb();
        let black_king_sq = black_king.lsb();
        if KING_DISTANCE[white_king_sq.index()][black_king_sq.index()] <= 1 {
            return Err("kings may not be adjacent".to_string());
        }

        let just_moved = !self.side_to_move;
        if self.is_attacked(self.king_sq(just_moved), self.side_to_move) {
            return Err("side not to move may not be in check".to_string());
        }

        // No move gives three checks at once (a discovered double check is the
        // most), so a third checker makes the position unreachable.
        let to_move = self.side_to_move;
        if self
            .attackers_to_color(self.king_sq(to_move), self.occupied(), !to_move)
            .count()
            > 2
        {
            return Err("side to move may not be in check from more than two pieces".to_string());
        }

        Ok(())
    }

    fn set_legal_ep_square(&mut self, ep_sq: Square) -> Result<(), String> {
        let capturer = self.side_to_move;
        let captured = !capturer;
        let expected_rank = if capturer == Color::White { 5 } else { 2 };
        if ep_sq.rank() as u8 != expected_rank {
            return Err(format!("invalid en passant rank: {ep_sq}"));
        }
        if self.piece_at(ep_sq).is_some() {
            return Err(format!("en passant target square is occupied: {ep_sq}"));
        }

        let cap_sq = ep_capture_square(capturer, ep_sq)
            .ok_or_else(|| format!("invalid en passant square: {ep_sq}"))?;
        if self.piece_at(cap_sq) != Some((captured, Piece::Pawn)) {
            return Err(format!("missing en passant capturable pawn at {cap_sq}"));
        }

        let origin_sq = ep_origin_square(capturer, ep_sq)
            .ok_or_else(|| format!("invalid en passant square: {ep_sq}"))?;
        if self.piece_at(origin_sq).is_some() {
            return Err(format!("en passant origin square is occupied: {origin_sq}"));
        }

        if self.legal_ep_capture_exists(capturer, ep_sq) {
            self.ep_sq = ep_sq.0;
            self.hash ^= ZOBRIST.ep(ep_sq.file());
        }
        Ok(())
    }

    fn legal_ep_capture_exists(&self, capturer: Color, ep_sq: Square) -> bool {
        let Some(cap_sq) = ep_capture_square(capturer, ep_sq) else {
            return false;
        };
        let atk = &ATTACKS;
        let mut attackers = atk.pawn(!capturer, ep_sq) & self.pieces(capturer, Piece::Pawn);
        while attackers.any() {
            let from = attackers.pop_lsb();
            if self.ep_capture_is_legal(capturer, from, ep_sq, cap_sq) {
                return true;
            }
        }
        false
    }

    /// Whether `capturer`'s pawn on `from` may take en passant on `ep_sq`,
    /// removing the pawn on `cap_sq`, without uncovering a slider on its own
    /// king along the rank or a diagonal the two pawns leave.
    #[inline]
    pub(super) fn ep_capture_is_legal(
        &self,
        capturer: Color,
        from: Square,
        ep_sq: Square,
        cap_sq: Square,
    ) -> bool {
        let them = !capturer;
        let king_sq = self.king_sq(capturer);
        let occ_after =
            (self.all_occ ^ Bitboard::from(from) ^ Bitboard::from(cap_sq)) | Bitboard::from(ep_sq);
        let atk = &ATTACKS;
        let exposed_rook = (self.pieces(them, Piece::Rook) | self.pieces(them, Piece::Queen))
            & atk.rook(king_sq, occ_after);
        let exposed_diag = (self.pieces(them, Piece::Bishop) | self.pieces(them, Piece::Queen))
            & atk.bishop(king_sq, occ_after);

        exposed_rook.is_empty() && exposed_diag.is_empty()
    }

    fn has_insufficient_material(&self) -> bool {
        let pawns = self.pieces(Color::White, Piece::Pawn) | self.pieces(Color::Black, Piece::Pawn);
        if pawns.any() {
            return false;
        }

        let majors = self.pieces(Color::White, Piece::Rook)
            | self.pieces(Color::Black, Piece::Rook)
            | self.pieces(Color::White, Piece::Queen)
            | self.pieces(Color::Black, Piece::Queen);
        if majors.any() {
            return false;
        }

        let knights =
            self.pieces(Color::White, Piece::Knight) | self.pieces(Color::Black, Piece::Knight);
        let bishops =
            self.pieces(Color::White, Piece::Bishop) | self.pieces(Color::Black, Piece::Bishop);
        let minors = knights | bishops;
        if minors.count() <= 1 {
            return true;
        }
        if knights.any() {
            return false;
        }

        let mut bishop_squares = bishops;
        let mut color_complex: Option<u8> = None;
        while bishop_squares.any() {
            let sq = bishop_squares.pop_lsb();
            let complex = (sq.file() as u8 + sq.rank() as u8) & 1;
            if color_complex.is_some_and(|known| known != complex) {
                return false;
            }
            color_complex = Some(complex);
        }
        true
    }

    fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    fn is_repetition(&self, needed_count: usize) -> bool {
        let mut count = 1usize;
        let max_plies = self.halfmove_clock as usize;
        let mut plies_back = 2usize;

        while plies_back <= max_plies && plies_back <= self.history.len() {
            if self.history[self.history.len() - plies_back].hash == self.hash {
                count += 1;
                if count >= needed_count {
                    return true;
                }
            }
            plies_back += 2;
        }

        false
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod repetition_tests;
