//! The static exchange evaluator: the cheapest attacker, the recapture
//! sequence and its thresholded forms.

use super::*;

impl Board {
    /// Select the least-valued LEGAL recapturer under the evolving exchange
    /// occupancy. Original piece sets remain valid off `target`: every piece
    /// that moved there has had its source removed from `occ`. Keep target
    /// occupied as a ray blocker, but exclude its original (captured) occupant
    /// from enemy attacks when checking the recapturer's king.
    pub(super) fn see_recapturer(
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
    pub(super) fn see_recapture_piece(piece: Piece, target: Square) -> Piece {
        if piece == Piece::Pawn && matches!(target.rank(), Rank::R1 | Rank::R8) {
            Piece::Queen
        } else {
            piece
        }
    }

    pub(super) fn see_occupancy(&self, mv: Move) -> Bitboard {
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

    pub(super) fn see_ge_impl(
        &self,
        mv: Move,
        threshold: i32,
        quiet: QuietSee,
        values: SeeValues,
    ) -> bool {
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

    #[inline(always)]
    pub(super) fn least_valuable_attacker(
        &self,
        attackers: Bitboard,
        color: Color,
    ) -> (Square, Piece) {
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
}
