//! Building a board: the start position, a placement, FEN in and out, and
//! the checks a set-up position must pass.

use super::*;

impl Board {
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

    pub(super) fn validate_position(&self) -> Result<(), String> {
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

    pub(super) fn set_legal_ep_square(&mut self, ep_sq: Square) -> Result<(), String> {
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
}
