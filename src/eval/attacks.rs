//! The attack-map producer: every piece's attacks, computed once per full
//! evaluation and borrowed by mobility, the bishop long-diagonal term,
//! threats, hanging pieces and king safety.

use crate::board::attacks::AttackTables;
use crate::board::{Bitboard, Board, Color, Piece, Square};

/// What a debug build writes into every `attacks_from_sq` slot before a fill,
/// so a read of a square the fill did not write is caught.
const UNWRITTEN: Bitboard = Bitboard(u64::MAX);

/// Attack maps for both colours, filled by `fill` at the start of piece
/// activity.
///
/// `attacked[color]` is the union over all of `color`'s pieces (pawns and king
/// included) of the squares they attack with the current occupancy, which is
/// equivalent to `attackers_to_color(sq, occupied, color)` being non-empty for
/// any `sq`, by the same symmetric attack-table argument that function relies
/// on.
#[derive(Clone)]
pub(super) struct AttackMaps {
    /// Squares attacked by each colour's pieces of each type.
    pub(super) attacked_by: [[Bitboard; 6]; 2],
    /// Squares attacked by each colour.
    pub(super) attacked: [Bitboard; 2],
    /// Squares attacked at least twice by each colour.
    pub(super) attacked2: [Bitboard; 2],
    /// Squares a colour's pieces count as mobility: not attacked by an enemy
    /// pawn and not occupied by an own piece.
    pub(super) mobility_area: [Bitboard; 2],
    /// Each knight, bishop, rook and queen's attacks, by its square.
    ///
    /// Written for exactly the squares holding a knight, bishop, rook or queen
    /// of each colour, and every read iterates that same piece set, so no read
    /// can observe a slot this fill did not write. The array is therefore kept
    /// across evaluations rather than re-zeroed: zeroing it was 1 KB per
    /// evaluation, about 1.4 GB per bench, to no purpose. The invariant is
    /// enforced, not assumed: debug builds poison every slot before the fill
    /// and every read asserts it is not the poison.
    attacks_from_sq: [[Bitboard; 64]; 2],
}

impl AttackMaps {
    pub(super) fn new() -> Self {
        Self {
            attacked_by: [[Bitboard::EMPTY; 6]; 2],
            attacked: [Bitboard::EMPTY; 2],
            attacked2: [Bitboard::EMPTY; 2],
            mobility_area: [Bitboard::EMPTY; 2],
            attacks_from_sq: [[Bitboard::EMPTY; 64]; 2],
        }
    }

    /// Recompute every map for `board`. `pawn_attacks` is the pawn cache's
    /// per-colour pawn-attack union.
    pub(super) fn fill(&mut self, board: &Board, atk: &AttackTables, pawn_attacks: &[Bitboard; 2]) {
        #[cfg(debug_assertions)]
        for side in &mut self.attacks_from_sq {
            side.fill(UNWRITTEN);
        }
        let occupied = board.occupied();
        for color in [Color::White, Color::Black] {
            let ci = color as usize;
            let mut attacked_by = [Bitboard::EMPTY; 6];
            let mut attacked = Bitboard::EMPTY;
            let mut attacked2 = Bitboard::EMPTY;
            attacked_by[Piece::Pawn as usize] = pawn_attacks[ci];
            // Squares two of this side's pawns both attack (the two diagonal
            // directions overlap) are attacked twice; the union
            // `pawn_attacks[ci]` loses that, so seed them into `attacked2`.
            let pawns_bb = board.pieces(color, Piece::Pawn);
            let double_pawn = if color == Color::White {
                pawns_bb.north_east() & pawns_bb.north_west()
            } else {
                pawns_bb.south_east() & pawns_bb.south_west()
            };
            attacked2 |= double_pawn;
            attacked |= pawn_attacks[ci];

            let king_atk = atk.king(board.king_sq(color));
            attacked_by[Piece::King as usize] = king_atk;
            attacked2 |= attacked & king_atk;
            attacked |= king_atk;

            // One loop per piece type, so the attack generator is a direct
            // call rather than `attacks_for`'s six-way `match piece`
            // re-evaluated for every piece on the board.
            macro_rules! attack_loop {
                ($piece:expr, $gen:expr) => {{
                    let piece_index = $piece as usize;
                    let mut bb = board.pieces(color, $piece);
                    while bb.any() {
                        let sq = bb.pop_lsb();
                        let atks = $gen(sq);
                        self.attacks_from_sq[ci][sq.index()] = atks;
                        attacked_by[piece_index] |= atks;
                        attacked2 |= attacked & atks;
                        attacked |= atks;
                    }
                }};
            }
            attack_loop!(Piece::Knight, |sq| atk.knight(sq));
            attack_loop!(Piece::Bishop, |sq| atk.bishop(sq, occupied));
            attack_loop!(Piece::Rook, |sq| atk.rook(sq, occupied));
            attack_loop!(Piece::Queen, |sq| atk.queen(sq, occupied));

            self.attacked_by[ci] = attacked_by;
            self.attacked[ci] = attacked;
            self.attacked2[ci] = attacked2;
            self.mobility_area[ci] = !pawn_attacks[(!color) as usize] & !board.color_occ(color);
        }
    }

    /// The attacks of `color`'s knight, bishop, rook or queen on `sq`.
    #[inline(always)]
    pub(super) fn attacks_from(&self, color: Color, sq: Square) -> Bitboard {
        read_written(&self.attacks_from_sq[color as usize], sq)
    }
}

#[inline(always)]
fn read_written(side: &[Bitboard; 64], sq: Square) -> Bitboard {
    debug_assert_ne!(
        side[sq.index()],
        UNWRITTEN,
        "attacks_from_sq read for a square this evaluation never wrote"
    );
    side[sq.index()]
}

/// Attack-map slices the king-danger model reads, bundled to keep
/// `eval_king_safety`'s signature small.
pub(super) struct KsMaps<'a> {
    /// `attacks_from_sq` for the *attacking* side (the enemy of the king).
    their_from_sq: &'a [Bitboard; 64],
    /// Union of squares attacked by each colour (incl. pawns + king).
    pub(super) attacked: &'a [Bitboard; 2],
    /// Squares attacked ≥2 times by each colour.
    pub(super) attacked2: &'a [Bitboard; 2],
    /// Per-piece-type attack union for the attacking side.
    pub(super) attacked_by_them: &'a [Bitboard; 6],
    pub(super) occupied: Bitboard,
    pub(super) own_occ: Bitboard,
    pub(super) their_occ: Bitboard,
}

impl<'a> KsMaps<'a> {
    /// The maps for the king of `color`.
    pub(super) fn new(
        maps: &'a AttackMaps,
        color: Color,
        occupied: Bitboard,
        color_occ: &[Bitboard; 2],
    ) -> Self {
        let them = !color;
        Self {
            their_from_sq: &maps.attacks_from_sq[them as usize],
            attacked: &maps.attacked,
            attacked2: &maps.attacked2,
            attacked_by_them: &maps.attacked_by[them as usize],
            occupied,
            own_occ: color_occ[color as usize],
            their_occ: color_occ[them as usize],
        }
    }

    /// The attacks of the attacking side's knight, bishop, rook or queen on
    /// `sq`.
    #[inline(always)]
    pub(super) fn their_from_sq(&self, sq: Square) -> Bitboard {
        read_written(self.their_from_sq, sq)
    }
}
