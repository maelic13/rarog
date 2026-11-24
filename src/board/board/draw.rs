//! The draw rules: fifty moves, repetition in the game and in the search's
//! line, the cycle detector, dead and insufficient material, the result.

use super::*;

impl Board {
    /// Rule-50 draw with mate precedence (the FIDE Laws' mate-first rule):
    /// at clock >= 100 the game is drawn UNLESS the position is checkmate —
    /// a mate delivered by the 100th-clock move wins. Stalemate at the
    /// boundary is a draw either way, so only the mated case needs the
    /// (rare) legal-move probe.
    #[inline(always)]
    pub(super) fn is_rule50_draw(&self) -> bool {
        self.halfmove_clock >= 100 && (!self.is_in_check() || self.has_legal_move())
    }

    /// The search asks this per node at the rule-50 boundary, so the moves go
    /// to a stack list, and the list is kept out of line and off the search's
    /// frame.
    #[cold]
    #[inline(never)]
    pub(super) fn has_legal_move(&self) -> bool {
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
    pub(super) fn repetition_window(&self) -> usize {
        usize::from(self.halfmove_clock)
            .min(usize::from(self.plies_from_null))
            .min(self.history.len())
    }

    /// The key of the position `plies` plies back, inside the history.
    #[inline(always)]
    pub(super) fn key_back(&self, plies: usize) -> u64 {
        self.history[self.history.len() - plies].hash
    }

    /// Whether the position `plies` back occurred again earlier inside
    /// `window`.
    pub(super) fn repeats_back(&self, plies: usize, window: usize) -> bool {
        let key = self.key_back(plies);
        (plies + 4..=window)
            .step_by(2)
            .any(|older| self.key_back(older) == key)
    }

    /// A repetition draws when the nearest earlier occurrence lies inside the
    /// search, fewer than `ply` plies back, or had itself occurred before.
    pub(super) fn repetition_draw_at(&self, ply: usize) -> bool {
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

    pub(super) fn has_insufficient_material(&self) -> bool {
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

    pub(super) fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    pub(super) fn is_repetition(&self, needed_count: usize) -> bool {
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
