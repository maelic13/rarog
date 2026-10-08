//! The attack-map producer: every piece's attacks, computed once per full
//! evaluation and borrowed by mobility, the bishop long-diagonal term,
//! threats, hanging pieces and king safety.

use crate::board::attacks::AttackTables;
use crate::board::{Bitboard, Board, Color, Piece, Square};
use crate::infra;

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
    /// Each king's ring, by the king's colour: the king's square and attacks
    /// with the king moved to files B–G and ranks 2–7, less the squares two of
    /// its own pawns defend.
    pub(super) king_ring: [Bitboard; 2],
    /// Ring attackers of each king, by the king's colour: the enemy pawn
    /// attacks on the ring (counted on the ring before the doubly defended
    /// squares leave it), then each enemy knight, bishop, rook and queen whose
    /// attacks touch the ring.
    pub(super) ring_attackers: [RingAttackers; 2],
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

/// The pieces attacking one king's ring.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) struct RingAttackers {
    /// Ring squares the enemy pawns attack.
    pub(super) pawn_attacks: i32,
    /// Enemy knights, bishops, rooks and queens whose attacks touch the ring.
    pub(super) pieces: [i32; 4],
    /// Those pieces' attacks on the squares next to the king, summed.
    pub(super) king_attacks: i32,
}

impl AttackMaps {
    pub(super) fn new() -> Self {
        Self {
            attacked_by: [[Bitboard::EMPTY; 6]; 2],
            attacked: [Bitboard::EMPTY; 2],
            attacked2: [Bitboard::EMPTY; 2],
            mobility_area: [Bitboard::EMPTY; 2],
            king_ring: [Bitboard::EMPTY; 2],
            ring_attackers: [RingAttackers::default(); 2],
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
        // Squares two of a side's pawns both attack (the two diagonal
        // directions overlap) are attacked twice; the union `pawn_attacks`
        // loses that.
        let double_pawn = [Color::White, Color::Black].map(|color| {
            let pawns_bb = board.pieces(color, Piece::Pawn);
            if color == Color::White {
                pawns_bb.north_east() & pawns_bb.north_west()
            } else {
                pawns_bb.south_east() & pawns_bb.south_west()
            }
        });
        let king_atk = [Color::White, Color::Black].map(|color| atk.king(board.king_sq(color)));
        // Both rings exist before either side's pieces are scanned, since each
        // side's loop counts the attackers of the other king's ring.
        for color in [Color::White, Color::Black] {
            let ci = color as usize;
            let ring = king_ring_before_pawns(atk, board.king_sq(color));
            self.ring_attackers[ci] = RingAttackers {
                pawn_attacks: infra::to_i32((ring & pawn_attacks[(!color) as usize]).count()),
                ..RingAttackers::default()
            };
            self.king_ring[ci] = ring & !double_pawn[ci];
        }
        for color in [Color::White, Color::Black] {
            let ci = color as usize;
            let ti = (!color) as usize;
            let their_ring = self.king_ring[ti];
            let their_king_atk = king_atk[ti];
            let mut ring_attackers = self.ring_attackers[ti];
            let mut attacked_by = [Bitboard::EMPTY; 6];
            let mut attacked = Bitboard::EMPTY;
            let mut attacked2 = double_pawn[ci];
            attacked_by[Piece::Pawn as usize] = pawn_attacks[ci];
            attacked |= pawn_attacks[ci];

            attacked_by[Piece::King as usize] = king_atk[ci];
            attacked2 |= attacked & king_atk[ci];
            attacked |= king_atk[ci];

            // One loop per piece type, so the attack generator is a direct
            // call rather than `attacks_for`'s six-way `match piece`
            // re-evaluated for every piece on the board.
            macro_rules! attack_loop {
                ($piece:expr, $slot:expr, $gen:expr) => {{
                    let piece_index = $piece as usize;
                    let mut bb = board.pieces(color, $piece);
                    while bb.any() {
                        let sq = bb.pop_lsb();
                        let atks = $gen(sq);
                        self.attacks_from_sq[ci][sq.index()] = atks;
                        attacked_by[piece_index] |= atks;
                        attacked2 |= attacked & atks;
                        attacked |= atks;
                        if (atks & their_ring).any() {
                            ring_attackers.pieces[$slot] += 1;
                            ring_attackers.king_attacks +=
                                infra::to_i32((atks & their_king_atk).count());
                        }
                    }
                }};
            }
            attack_loop!(Piece::Knight, 0, |sq| atk.knight(sq));
            attack_loop!(Piece::Bishop, 1, |sq| atk.bishop(sq, occupied));
            attack_loop!(Piece::Rook, 2, |sq| atk.rook(sq, occupied));
            attack_loop!(Piece::Queen, 3, |sq| atk.queen(sq, occupied));

            self.ring_attackers[ti] = ring_attackers;
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

/// The king's square and attacks with the king moved to files B–G and ranks
/// 2–7, so an edge or corner king keeps a full ring.
#[inline(always)]
fn king_ring_before_pawns(atk: &AttackTables, king: Square) -> Bitboard {
    let file = (king.index() & 7).clamp(1, 6);
    let rank = (king.index() >> 3).clamp(1, 6);
    let centre = Square(infra::to_u8(rank * 8 + file));
    atk.king(centre) | Bitboard::from(centre)
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

/// Attack-map slices the king-danger model reads for one king, bundled to
/// keep `eval_king_safety`'s signature small.
pub(super) struct KsMaps<'a> {
    /// Squares attacked by each colour (pawns and king included).
    pub(super) attacked: &'a [Bitboard; 2],
    /// Squares attacked at least twice by each colour.
    pub(super) attacked2: &'a [Bitboard; 2],
    /// Squares attacked by the king's side, by piece type.
    pub(super) attacked_by_us: &'a [Bitboard; 6],
    /// Squares attacked by the other side, by piece type.
    pub(super) attacked_by_them: &'a [Bitboard; 6],
    pub(super) king_ring: Bitboard,
    pub(super) ring_attackers: RingAttackers,
    pub(super) occupied: Bitboard,
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
        let us = color as usize;
        let them = (!color) as usize;
        Self {
            attacked: &maps.attacked,
            attacked2: &maps.attacked2,
            attacked_by_us: &maps.attacked_by[us],
            attacked_by_them: &maps.attacked_by[them],
            king_ring: maps.king_ring[us],
            ring_attackers: maps.ring_attackers[us],
            occupied,
            their_occ: color_occ[them],
        }
    }
}
