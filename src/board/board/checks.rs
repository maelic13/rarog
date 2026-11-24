//! Check, attack and threat queries, and the legality tests around the king:
//! castling through attacked squares, king safety after a move, en passant.

use super::*;

impl Board {
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
    pub(crate) fn threats(&self) -> Threats {
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

    pub(super) fn castling_move(
        &self,
        from: Square,
        to: Square,
        us: Color,
        them: Color,
    ) -> Option<Move> {
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

    pub(super) fn king_safe_after(&self, mv: Move) -> bool {
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

    #[inline(always)]
    pub(super) fn calculate_checkers(&self) -> Bitboard {
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

    pub(super) fn legal_ep_capture_exists(&self, capturer: Color, ep_sq: Square) -> bool {
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
    pub(in crate::board) fn ep_capture_is_legal(
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
}
