//! Making and unmaking moves, the null move, and the piece and key
//! bookkeeping they share.

use super::*;

impl Board {
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

    pub(super) fn make_move_inner(&mut self, mv: Move, check_hint: Option<bool>) {
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

    /// Toggle `key` into the pawn, minor-piece and non-pawn keys a piece of
    /// type `piece` belongs to. A mask per type, not a branch: the moving
    /// piece's type is as unpredictable as the move.
    #[inline(always)]
    pub(super) fn toggle_derived_keys(&mut self, color: Color, piece: Piece, key: u64) {
        let [pawn, minor, non_pawn] = DERIVED_KEY_MASKS[piece as usize];
        self.pawn_hash ^= key & pawn;
        self.minor_hash ^= key & minor;
        self.non_pawn_hash[color as usize] ^= key & non_pawn;
    }

    #[inline(always)]
    pub(super) fn add_piece(&mut self, color: Color, piece: Piece, sq: Square) {
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
    pub(super) fn move_piece(&mut self, color: Color, piece: Piece, from: Square, to: Square) {
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
    pub(super) fn remove_piece(&mut self, color: Color, piece: Piece, sq: Square) {
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
    pub(super) fn occupant_type(&self, sq: Square) -> Piece {
        debug_assert!(self.mailbox[sq.index()] < 12);
        // `& 15` into the padded 16-entry table — check elided, no unsafe.
        PIECE_FROM_ENCODED[self.mailbox[sq.index()] as usize & 15]
    }
}
