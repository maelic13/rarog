/// Attack tables for all piece types.
///
/// Non-sliding pieces (pawn, knight, king) are stored as simple lookup arrays.
/// Sliding pieces (bishop, rook, queen) use either magic bitboards or a
/// PEXT table layout. The compiler builds every table, so a lookup reads a
/// fixed address, with no check that the tables exist and no table pointer to
/// load first.
use crate::infra;

use super::bitboard::Bitboard;
use super::piece::Color;
use super::square::Square;

/// Slider table entries: one per relevant blocker subset of every square, the
/// same count in the PEXT and magic layouts.
const BISHOP_TABLE_SIZE: usize = 5_248;
const ROOK_TABLE_SIZE: usize = 102_400;

// -----------------------------------------------------------------------
// Public attack accessors
// -----------------------------------------------------------------------

/// All attack tables.
pub struct AttackTables {
    pub(crate) pawn_attacks: [[Bitboard; 64]; 2],
    knight_attacks: [Bitboard; 64],
    pub(crate) king_attacks: [Bitboard; 64],
    bishop: [SliderEntry; 64],
    rook: [SliderEntry; 64],
    bishop_table: [Bitboard; BISHOP_TABLE_SIZE],
    rook_table: [Bitboard; ROOK_TABLE_SIZE],
}

pub static ATTACKS: AttackTables = AttackTables::build();

#[derive(Copy, Clone)]
struct SliderEntry {
    mask: u64,
    offset: usize,
    #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
    magic: u64,
    #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
    shift: u32,
}

impl SliderEntry {
    #[cfg(all(rarog_pext, target_arch = "x86_64"))]
    const fn new(mask: u64, offset: usize) -> Self {
        Self { mask, offset }
    }

    #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
    const fn new(mask: u64, magic: u64, shift: u32, offset: usize) -> Self {
        Self {
            mask,
            offset,
            magic,
            shift,
        }
    }

    const fn empty() -> Self {
        #[cfg(all(rarog_pext, target_arch = "x86_64"))]
        {
            Self { mask: 0, offset: 0 }
        }

        #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
        {
            Self {
                mask: 0,
                offset: 0,
                magic: 0,
                shift: 0,
            }
        }
    }
}

impl AttackTables {
    // -----------------------------------------------------------------------
    // Public accessors
    // -----------------------------------------------------------------------

    #[inline(always)]
    pub fn pawn(&self, color: Color, sq: Square) -> Bitboard {
        self.pawn_attacks[color as usize][sq.index()]
    }

    #[inline(always)]
    pub fn knight(&self, sq: Square) -> Bitboard {
        self.knight_attacks[sq.index()]
    }

    #[inline(always)]
    pub fn king(&self, sq: Square) -> Bitboard {
        self.king_attacks[sq.index()]
    }

    #[inline(always)]
    pub fn bishop(&self, sq: Square, occ: Bitboard) -> Bitboard {
        // Safe — `Square::index()` is masked to 0..=63, so the bounds
        // check on this `[_; 64]` elides.
        let e = &self.bishop[sq.index()];
        #[cfg(all(rarog_pext, target_arch = "x86_64"))]
        {
            let idx = e.offset + pext_index(occ.0, e.mask);
            // SAFETY: `pext_index` yields < 2^popcount(e.mask) by construction
            // and `build` fills exactly that many entries from `e.offset`, so
            // `idx` is always in bounds.
            // KEEP-UNSAFE: safe indexing measured −0.5% NPS here (pext) and
            // −1.5% on the magic path — the index is occupancy-derived, so the
            // bounds check cannot elide, in the hottest load in the engine.
            unsafe { *self.bishop_table.get_unchecked(idx) }
        }

        #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
        {
            let idx = e.offset + infra::index(((occ.0 & e.mask).wrapping_mul(e.magic)) >> e.shift);
            // SAFETY: `idx` is in bounds — the magic index is masked to the
            // per-square table width by `e.shift`, and `build` fills exactly
            // `e.offset .. e.offset + 2^popcount(e.mask)` for that square.
            // KEEP-UNSAFE: safe indexing measured −1.5% NPS on this path
            // (the index is occupancy-derived, so the check cannot elide).
            unsafe { *self.bishop_table.get_unchecked(idx) }
        }
    }

    #[inline(always)]
    pub fn rook(&self, sq: Square, occ: Bitboard) -> Bitboard {
        // Safe — see `bishop()`.
        let e = &self.rook[sq.index()];
        #[cfg(all(rarog_pext, target_arch = "x86_64"))]
        {
            let idx = e.offset + pext_index(occ.0, e.mask);
            // SAFETY: as in `bishop()`, `pext_index` yields < 2^popcount(e.mask)
            // and `build` fills exactly that many entries from `e.offset`.
            // KEEP-UNSAFE: see `bishop()`; the same measurement.
            unsafe { *self.rook_table.get_unchecked(idx) }
        }

        #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
        {
            let idx = e.offset + infra::index(((occ.0 & e.mask).wrapping_mul(e.magic)) >> e.shift);
            // SAFETY: `idx` is in bounds — the magic index is masked to the
            // per-square table width by `e.shift`, and `build` fills exactly
            // `e.offset .. e.offset + 2^popcount(e.mask)` for that square.
            // KEEP-UNSAFE: safe indexing measured −1.5% NPS on this path
            // (the index is occupancy-derived, so the check cannot elide).
            unsafe { *self.rook_table.get_unchecked(idx) }
        }
    }

    #[inline(always)]
    pub fn queen(&self, sq: Square, occ: Bitboard) -> Bitboard {
        self.bishop(sq, occ) | self.rook(sq, occ)
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Every table, evaluated by the compiler for [`ATTACKS`].
    const fn build() -> Self {
        let mut tables = Self {
            pawn_attacks: [[Bitboard::EMPTY; 64]; 2],
            knight_attacks: [Bitboard::EMPTY; 64],
            king_attacks: [Bitboard::EMPTY; 64],
            bishop: [SliderEntry::empty(); 64],
            rook: [SliderEntry::empty(); 64],
            bishop_table: [Bitboard::EMPTY; BISHOP_TABLE_SIZE],
            rook_table: [Bitboard::EMPTY; ROOK_TABLE_SIZE],
        };
        let mut bishop_offset = 0;
        let mut rook_offset = 0;
        let mut sq = Square(0);
        while sq.0 < 64 {
            let index = sq.0 as usize;
            let bb = Bitboard(1 << index);
            tables.pawn_attacks[Color::White as usize][index] = pawn_attacks_of(Color::White, bb);
            tables.pawn_attacks[Color::Black as usize][index] = pawn_attacks_of(Color::Black, bb);
            tables.knight_attacks[index] = knight_attacks_of(bb);
            tables.king_attacks[index] = king_attacks_of(bb);
            tables.bishop[index] = fill_slider(&mut tables.bishop_table, bishop_offset, sq, false);
            bishop_offset += 1 << bishop_mask(sq).count_ones();
            tables.rook[index] = fill_slider(&mut tables.rook_table, rook_offset, sq, true);
            rook_offset += 1 << rook_mask(sq).count_ones();
            sq.0 += 1;
        }
        assert!(bishop_offset == BISHOP_TABLE_SIZE && rook_offset == ROOK_TABLE_SIZE);
        tables
    }
}

/// The squares attacked by pawns of `color` standing on `pawns`. The pawn
/// table is built from these shifts square by square, and each shift
/// distributes over a union, so this is the union of the table's entries for
/// `pawns` without a lookup per pawn.
#[inline(always)]
pub(crate) const fn pawn_attacks_of(color: Color, pawns: Bitboard) -> Bitboard {
    match color {
        // White pawns attack north-east and north-west, Black south-east and
        // south-west.
        Color::White => Bitboard(pawns.north_east().0 | pawns.north_west().0),
        Color::Black => Bitboard(pawns.south_east().0 | pawns.south_west().0),
    }
}

/// The squares attacked by knights standing on `knights`; see
/// [`pawn_attacks_of`].
#[inline(always)]
pub(crate) const fn knight_attacks_of(knights: Bitboard) -> Bitboard {
    // Two-square jumps: (±1, ±2) and (±2, ±1)
    let h1 = Bitboard(knights.east().0 | knights.west().0);
    let h2 = Bitboard(knights.east().east().0 | knights.west().west().0);
    Bitboard(h1.north().north().0 | h1.south().south().0 | h2.north().0 | h2.south().0)
}

/// The squares a king on `king` attacks.
const fn king_attacks_of(king: Bitboard) -> Bitboard {
    Bitboard(
        king.north().0
            | king.south().0
            | king.east().0
            | king.west().0
            | king.north_east().0
            | king.north_west().0
            | king.south_east().0
            | king.south_west().0,
    )
}

/// Fill one square's run of a slider table, starting at `offset`, and return
/// the square's entry. Under PEXT a blocker subset's slot is its rank among
/// the mask's subsets, which is the order the carry-rippler visits them in;
/// under magics it is the top bits of the magic product, and a baked magic
/// that sends two different attack sets to one slot stops the build.
const fn fill_slider(
    table: &mut [Bitboard],
    offset: usize,
    sq: Square,
    is_rook: bool,
) -> SliderEntry {
    let mask = if is_rook {
        rook_mask(sq)
    } else {
        bishop_mask(sq)
    };
    #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
    let magic = if is_rook {
        ROOK_MAGICS[sq.0 as usize]
    } else {
        BISHOP_MAGICS[sq.0 as usize]
    };
    #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
    let shift = 64 - mask.count_ones();
    let mut occ = 0u64;
    let mut rank = 0;
    loop {
        let attacks = if is_rook {
            rook_attacks_slow(sq, occ)
        } else {
            bishop_attacks_slow(sq, occ)
        };
        #[cfg(all(rarog_pext, target_arch = "x86_64"))]
        let slot = offset + rank;
        #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
        let slot = offset + infra::index(occ.wrapping_mul(magic) >> shift);
        assert!(
            table[slot].is_empty() || table[slot].0 == attacks,
            "a baked magic sends two attack sets to one slot"
        );
        table[slot] = Bitboard(attacks);
        rank += 1;
        occ = occ.wrapping_sub(mask) & mask;
        if occ == 0 {
            break;
        }
    }
    assert!(rank == 1 << mask.count_ones());
    #[cfg(all(rarog_pext, target_arch = "x86_64"))]
    {
        SliderEntry::new(mask, offset)
    }
    #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
    {
        SliderEntry::new(mask, magic, shift, offset)
    }
}

// -----------------------------------------------------------------------
// Slow (reference) attack generators — used only to build the tables
// -----------------------------------------------------------------------

/// Rook relevant occupancy mask (excludes edges).
const fn rook_mask(sq: Square) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut mask = 0u64;
    let mut i = r + 1;
    while i < 7 {
        mask |= 1u64 << (i * 8 + f);
        i += 1;
    }
    let mut i = 1;
    while i < r {
        mask |= 1u64 << (i * 8 + f);
        i += 1;
    }
    let mut i = f + 1;
    while i < 7 {
        mask |= 1u64 << (r * 8 + i);
        i += 1;
    }
    let mut i = 1;
    while i < f {
        mask |= 1u64 << (r * 8 + i);
        i += 1;
    }
    mask
}

/// Bishop relevant occupancy mask (excludes edges).
const fn bishop_mask(sq: Square) -> u64 {
    let r = (sq.0 / 8) as i32;
    let f = (sq.0 % 8) as i32;
    let mut mask = 0u64;
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r + i, f + i);
        if nr >= 7 || nf >= 7 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
        i += 1;
    }
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r + i, f - i);
        if nr >= 7 || nf <= 0 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
        i += 1;
    }
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r - i, f + i);
        if nr <= 0 || nf >= 7 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
        i += 1;
    }
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r - i, f - i);
        if nr <= 0 || nf <= 0 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
        i += 1;
    }
    mask
}

const fn rook_attacks_slow(sq: Square, occ: u64) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut att = 0u64;
    let mut i = r + 1;
    while i < 8 {
        att |= 1u64 << (i * 8 + f);
        if (occ >> (i * 8 + f)) & 1 != 0 {
            break;
        }
        i += 1;
    }
    let mut i = r;
    while i > 0 {
        i -= 1;
        att |= 1u64 << (i * 8 + f);
        if (occ >> (i * 8 + f)) & 1 != 0 {
            break;
        }
    }
    let mut i = f + 1;
    while i < 8 {
        att |= 1u64 << (r * 8 + i);
        if (occ >> (r * 8 + i)) & 1 != 0 {
            break;
        }
        i += 1;
    }
    let mut i = f;
    while i > 0 {
        i -= 1;
        att |= 1u64 << (r * 8 + i);
        if (occ >> (r * 8 + i)) & 1 != 0 {
            break;
        }
    }
    att
}

const fn bishop_attacks_slow(sq: Square, occ: u64) -> u64 {
    let r = (sq.0 / 8) as i32;
    let f = (sq.0 % 8) as i32;
    let mut att = 0u64;
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r + i, f + i);
        if nr >= 8 || nf >= 8 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
        i += 1;
    }
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r + i, f - i);
        if nr >= 8 || nf < 0 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
        i += 1;
    }
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r - i, f + i);
        if nr < 0 || nf >= 8 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
        i += 1;
    }
    let mut i = 1;
    while i < 8 {
        let (nr, nf) = (r - i, f - i);
        if nr < 0 || nf < 0 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
        i += 1;
    }
    att
}

#[cfg(all(rarog_pext, target_arch = "x86_64"))]
#[inline(always)]
fn pext_index(occ: u64, mask: u64) -> usize {
    // SAFETY: `_pext_u64` needs BMI2. This path is compiled only under
    // `--cfg rarog_pext`, which `xtask --arch pext` sets together with
    // `-C target-feature=+bmi2`, so its binaries target BMI2 CPUs only.
    infra::index(unsafe { std::arch::x86_64::_pext_u64(occ, mask) })
}

// Magics found once by a seeded search and baked in. The compiler builds the
// tables from them and stops the build if one ever sends two different attack
// sets to one slot, so a wrong constant cannot reach a binary.
#[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
const BISHOP_MAGICS: [u64; 64] = [
    0x00500202205C0100,
    0x2002921404008000,
    0x2C08008434820000,
    0x0008208B20000000,
    0x4502021080800000,
    0x0008280808020180,
    0x0491009010081400,
    0x42C2010401010810,
    0x2000206091020085,
    0x4000200101020886,
    0x8A00040408920000,
    0x0000382080200512,
    0x2006045040008800,
    0x00002088044020D0,
    0x0800008088201041,
    0x2A0801A40904100C,
    0xC025404204080600,
    0x8850C120890A0081,
    0x340A0010022A0820,
    0x9008000424210013,
    0x800410220202029E,
    0x4006000108210411,
    0x0050501208044421,
    0x2008801A00940100,
    0x8602315120743000,
    0x00192001C4480210,
    0x0082120120408400,
    0x1020080023004028,
    0x0201010000104001,
    0x1040410008900800,
    0x0201012004042900,
    0x020880220D040202,
    0x0010843108045000,
    0xA00841040E082800,
    0x000C020120020400,
    0x8A92008020020200,
    0x4020008400050410,
    0x04101001A004C400,
    0xC044040400008090,
    0x110C206200008482,
    0x5001415010004084,
    0x240238090C012800,
    0x0000220030022200,
    0x800000A018020100,
    0x0003C0010A080300,
    0x2010121002480100,
    0x0102088904101100,
    0x8002520202011C20,
    0x0014420821080000,
    0x0010904802100011,
    0x0030004200D00040,
    0x500006002A080048,
    0x0104001202020280,
    0x8200202082808101,
    0x1910108200C40010,
    0x0004812401020018,
    0x0223040105080300,
    0x1010012901103008,
    0x8242000021280800,
    0x1000000000208800,
    0x8006000012020200,
    0x1010081004082820,
    0x1001C02441020600,
    0x0C08101888830202,
];

#[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
const ROOK_MAGICS: [u64; 64] = [
    0x0080001820804000,
    0x0040004020001004,
    0x0200208042001008,
    0x0100081000050021,
    0x0200042010020008,
    0x120002002C181045,
    0x0400280120921004,
    0x00800D0003402080,
    0x0020801080204002,
    0x1001002040011081,
    0x8213004020001304,
    0x0082000A00421022,
    0xC145000500080010,
    0x0E96001084020008,
    0x0001010001040200,
    0x2060802840800900,
    0x0C2081800020400C,
    0x3000404000201000,
    0x8002020020804012,
    0x0410018008001080,
    0x0288808008000401,
    0x0001010004000802,
    0x0086040010018802,
    0x0A00020000A40AC5,
    0x0080004440002000,
    0x8800200040005008,
    0x21A0080040401000,
    0x0000090100100020,
    0x0000080100110004,
    0x0001000300080400,
    0x2080500400186B22,
    0x1020304200008401,
    0x0200400024800089,
    0x4020402002401009,
    0x0000802004801004,
    0x0030008008080100,
    0x1002002012000904,
    0x0090800400800200,
    0x0014021004000108,
    0x8000244882000104,
    0x0002824000228000,
    0x0100201000404001,
    0x0181004020010010,
    0x0060100061030008,
    0x0000080011010004,
    0x1080020004008080,
    0x1000010208040010,
    0x2040041142860021,
    0x0880004000200040,
    0x140300E8820C4600,
    0x1081007E40200100,
    0x0100801000080080,
    0x1408008008040080,
    0x2240040080020080,
    0x2800880142100400,
    0x160210508D040200,
    0x8000120484204102,
    0xD422030082281042,
    0x0005108009204202,
    0x2008081000042101,
    0x0002000850218402,
    0x0011000802040001,
    0x2108009058020104,
    0x0001010406482282,
];
