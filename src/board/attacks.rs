/// Attack tables for all piece types.
///
/// Non-sliding pieces (pawn, knight, king) are stored as simple lookup arrays.
/// Sliding pieces (bishop, rook, queen) use either magic bitboards or a
/// compile-time PEXT table layout, initialized at startup via `LazyLock`.
use crate::infra;
use std::sync::LazyLock;

use super::bitboard::Bitboard;
use super::piece::Color;
use super::square::Square;

// -----------------------------------------------------------------------
// Public attack accessors
// -----------------------------------------------------------------------

/// All attack tables, initialized once.
pub struct AttackTables {
    pub(crate) pawn_attacks: [[Bitboard; 64]; 2],
    knight_attacks: [Bitboard; 64],
    pub(crate) king_attacks: [Bitboard; 64],
    bishop: [SliderEntry; 64],
    rook: [SliderEntry; 64],
    bishop_table: Vec<Bitboard>,
    rook_table: Vec<Bitboard>,
}

pub static ATTACKS: LazyLock<AttackTables> = LazyLock::new(AttackTables::init);

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
            // and init fills exactly that many entries from `e.offset`, so
            // `idx` is always in bounds.
            // KEEP-UNSAFE: safe indexing measured −0.5% NPS here (pext) and
            // −1.5% on the magic path — the index is occupancy-derived, so the
            // bounds check cannot elide, in the hottest load in the engine.
            return unsafe { *self.bishop_table.get_unchecked(idx) };
        }

        #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
        {
            let idx = e.offset + infra::index(((occ.0 & e.mask).wrapping_mul(e.magic)) >> e.shift);
            // SAFETY: `idx` is in bounds — the magic index is masked to the
            // per-square table width by `e.shift`, and init fills exactly
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
            // SAFETY / KEEP-UNSAFE: see `bishop()` — same construction and the
            // same measurement.
            return unsafe { *self.rook_table.get_unchecked(idx) };
        }

        #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
        {
            let idx = e.offset + infra::index(((occ.0 & e.mask).wrapping_mul(e.magic)) >> e.shift);
            // SAFETY: `idx` is in bounds — the magic index is masked to the
            // per-square table width by `e.shift`, and init fills exactly
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
    // Initialization
    // -----------------------------------------------------------------------

    fn init() -> Self {
        let pawn_attacks = Self::init_pawn_attacks();
        let knight_attacks = Self::init_knight_attacks();
        let king_attacks = Self::init_king_attacks();

        // Bishop slider tables
        let mut bishop_entries: [SliderEntry; 64] = std::array::from_fn(|_| SliderEntry::empty());
        let mut bishop_table: Vec<Bitboard> = Vec::new();
        #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
        let mut rng = Rng::new(0x1234_5678_9ABC_DEF0);
        for sq in 0..64 {
            let sq = Square(infra::to_u8(sq));
            let mask = bishop_mask(sq);
            let n = mask.count_ones();
            #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
            let shift = 64 - n;
            let size = 1usize << n;
            let offset = bishop_table.len();
            bishop_table.resize(offset + size, Bitboard::EMPTY);
            #[cfg(all(rarog_pext, target_arch = "x86_64"))]
            {
                init_pext_table(mask, false, sq, &mut bishop_table[offset..]);
                bishop_entries[sq.index()] = SliderEntry::new(mask, offset);
            }
            #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
            {
                let magic = find_magic(
                    mask,
                    shift,
                    false,
                    sq,
                    &mut rng,
                    &mut bishop_table[offset..],
                );
                bishop_entries[sq.index()] = SliderEntry::new(mask, magic, shift, offset);
            }
        }

        // Rook slider tables
        let mut rook_entries: [SliderEntry; 64] = std::array::from_fn(|_| SliderEntry::empty());
        let mut rook_table: Vec<Bitboard> = Vec::new();
        for sq in 0..64 {
            let sq = Square(infra::to_u8(sq));
            let mask = rook_mask(sq);
            let n = mask.count_ones();
            #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
            let shift = 64 - n;
            let size = 1usize << n;
            let offset = rook_table.len();
            rook_table.resize(offset + size, Bitboard::EMPTY);
            #[cfg(all(rarog_pext, target_arch = "x86_64"))]
            {
                init_pext_table(mask, true, sq, &mut rook_table[offset..]);
                rook_entries[sq.index()] = SliderEntry::new(mask, offset);
            }
            #[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
            {
                let magic = find_magic(mask, shift, true, sq, &mut rng, &mut rook_table[offset..]);
                rook_entries[sq.index()] = SliderEntry::new(mask, magic, shift, offset);
            }
        }

        Self {
            pawn_attacks,
            knight_attacks,
            king_attacks,
            bishop: bishop_entries,
            rook: rook_entries,
            bishop_table,
            rook_table,
        }
    }

    fn init_pawn_attacks() -> [[Bitboard; 64]; 2] {
        let mut table = [[Bitboard::EMPTY; 64]; 2];
        for s in 0..64u8 {
            let sq = Square(s);
            let bb = Bitboard::from(sq);
            // White pawns attack north-east and north-west
            table[Color::White as usize][sq.index()] = bb.north_east() | bb.north_west();
            // Black pawns attack south-east and south-west
            table[Color::Black as usize][sq.index()] = bb.south_east() | bb.south_west();
        }
        table
    }

    fn init_knight_attacks() -> [Bitboard; 64] {
        let mut table = [Bitboard::EMPTY; 64];
        for s in 0..64u8 {
            let sq = Square(s);
            let bb = Bitboard::from(sq);
            // Two-square jumps: (±1, ±2) and (±2, ±1)
            let h1 = bb.east() | bb.west();
            let h2 = bb.east().east() | bb.west().west();
            table[sq.index()] = h1.north().north() | h1.south().south() | h2.north() | h2.south();
        }
        table
    }

    fn init_king_attacks() -> [Bitboard; 64] {
        let mut table = [Bitboard::EMPTY; 64];
        for s in 0..64u8 {
            let sq = Square(s);
            let bb = Bitboard::from(sq);
            table[sq.index()] = bb.north()
                | bb.south()
                | bb.east()
                | bb.west()
                | bb.north_east()
                | bb.north_west()
                | bb.south_east()
                | bb.south_west();
        }
        table
    }
}

// -----------------------------------------------------------------------
// Slow (reference) attack generators — used only during magic init
// -----------------------------------------------------------------------

/// Rook relevant occupancy mask (excludes edges).
fn rook_mask(sq: Square) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut mask = 0u64;
    for i in (r + 1)..7 {
        mask |= 1u64 << (i * 8 + f);
    }
    for i in 1..r {
        mask |= 1u64 << (i * 8 + f);
    }
    for i in (f + 1)..7 {
        mask |= 1u64 << (r * 8 + i);
    }
    for i in 1..f {
        mask |= 1u64 << (r * 8 + i);
    }
    mask
}

/// Bishop relevant occupancy mask (excludes edges).
fn bishop_mask(sq: Square) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut mask = 0u64;
    for i in 1..8i32 {
        let nr = r as i32 + i;
        let nf = f as i32 + i;
        if nr >= 7 || nf >= 7 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    for i in 1..8i32 {
        let nr = r as i32 + i;
        let nf = f as i32 - i;
        if nr >= 7 || nf <= 0 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    for i in 1..8i32 {
        let nr = r as i32 - i;
        let nf = f as i32 + i;
        if nr <= 0 || nf >= 7 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    for i in 1..8i32 {
        let nr = r as i32 - i;
        let nf = f as i32 - i;
        if nr <= 0 || nf <= 0 {
            break;
        }
        mask |= 1u64 << (nr * 8 + nf);
    }
    mask
}

fn rook_attacks_slow(sq: Square, occ: u64) -> u64 {
    let r = sq.0 / 8;
    let f = sq.0 % 8;
    let mut att = 0u64;
    for i in (r + 1)..8 {
        att |= 1u64 << (i * 8 + f);
        if (occ >> (i * 8 + f)) & 1 != 0 {
            break;
        }
    }
    for i in (0..r).rev() {
        att |= 1u64 << (i * 8 + f);
        if (occ >> (i * 8 + f)) & 1 != 0 {
            break;
        }
    }
    for i in (f + 1)..8 {
        att |= 1u64 << (r * 8 + i);
        if (occ >> (r * 8 + i)) & 1 != 0 {
            break;
        }
    }
    for i in (0..f).rev() {
        att |= 1u64 << (r * 8 + i);
        if (occ >> (r * 8 + i)) & 1 != 0 {
            break;
        }
    }
    att
}

fn bishop_attacks_slow(sq: Square, occ: u64) -> u64 {
    let r = sq.0 as i32 / 8;
    let f = sq.0 as i32 % 8;
    let mut att = 0u64;
    for i in 1..8 {
        let (nr, nf) = (r + i, f + i);
        if nr >= 8 || nf >= 8 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    for i in 1..8 {
        let (nr, nf) = (r + i, f - i);
        if nr >= 8 || nf < 0 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    for i in 1..8 {
        let (nr, nf) = (r - i, f + i);
        if nr < 0 || nf >= 8 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    for i in 1..8 {
        let (nr, nf) = (r - i, f - i);
        if nr < 0 || nf < 0 {
            break;
        }
        att |= 1u64 << (nr * 8 + nf);
        if (occ >> (nr * 8 + nf)) & 1 != 0 {
            break;
        }
    }
    att
}

#[cfg(all(rarog_pext, target_arch = "x86_64"))]
fn init_pext_table(mask: u64, is_rook: bool, sq: Square, table: &mut [Bitboard]) {
    let size = 1usize << mask.count_ones();
    debug_assert_eq!(table.len(), size);

    let mut occ = 0u64;
    loop {
        let idx = pext_index(occ, mask);
        table[idx] = Bitboard(if is_rook {
            rook_attacks_slow(sq, occ)
        } else {
            bishop_attacks_slow(sq, occ)
        });
        occ = occ.wrapping_sub(mask) & mask;
        if occ == 0 {
            break;
        }
    }
}

#[cfg(all(rarog_pext, target_arch = "x86_64"))]
#[inline(always)]
fn pext_index(occ: u64, mask: u64) -> usize {
    unsafe { std::arch::x86_64::_pext_u64(occ, mask) as usize }
}

// -----------------------------------------------------------------------
// Magic finding
// -----------------------------------------------------------------------

/// Find a magic number for `sq` with the given `mask` / `shift`.
/// Fills `table[0..size]` with the correct attack bitboards on success.
#[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
fn find_magic(
    mask: u64,
    shift: u32,
    is_rook: bool,
    sq: Square,
    rng: &mut Rng,
    table: &mut [Bitboard],
) -> u64 {
    let n = mask.count_ones() as usize;
    let size = 1usize << n;
    debug_assert_eq!(table.len(), size);

    // Enumerate all subsets of `mask` via carry-rippler and precompute attacks.
    let mut occs = vec![0u64; size];
    let mut atts = vec![0u64; size];
    let mut occ = 0u64;
    let mut i = 0;
    loop {
        occs[i] = occ;
        atts[i] = if is_rook {
            rook_attacks_slow(sq, occ)
        } else {
            bishop_attacks_slow(sq, occ)
        };
        i += 1;
        occ = occ.wrapping_sub(mask) & mask;
        if occ == 0 {
            break;
        }
    }

    // Try the baked magic for this square first. It is exactly what
    // this search produces — the RNG is seeded to a constant and is
    // deterministic — so the fast path is the overwhelming case and the search
    // below is a fallback that never runs in a stock build. Keeping the
    // fallback (rather than trusting the constant) means a wrong or stale
    // baked value costs startup time, never correctness.
    let baked = if is_rook {
        ROOK_MAGICS[sq.index()]
    } else {
        BISHOP_MAGICS[sq.index()]
    };
    if try_magic(baked, shift, &occs, &atts, table) {
        return baked;
    }

    // Try random sparse candidates until we find a valid magic.
    loop {
        let magic = rng.sparse();
        // Quick reject: upper byte of (mask * magic) should have enough bits set.
        if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 {
            continue;
        }
        if try_magic(magic, shift, &occs, &atts, table) {
            return magic;
        }
    }
}

/// Fill `table` using `magic`, reporting whether the mapping is collision-free.
/// A `false` return leaves the table dirty; the caller retries or overwrites.
#[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
fn try_magic(magic: u64, shift: u32, occs: &[u64], atts: &[u64], table: &mut [Bitboard]) -> bool {
    for t in table.iter_mut() {
        *t = Bitboard::EMPTY;
    }
    for j in 0..occs.len() {
        let idx = crate::infra::index((occs[j].wrapping_mul(magic)) >> shift);
        if table[idx].is_empty() {
            table[idx] = Bitboard(atts[j]);
        } else if table[idx].0 != atts[j] {
            return false;
        }
    }
    true
}

// -----------------------------------------------------------------------
// splitmix64 PRNG (sparse variant) — same as basilisk
// -----------------------------------------------------------------------

#[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
struct Rng(u64);

#[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    #[inline]
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Sparse 64-bit value (AND of three randoms — biased toward few set bits,
    /// which gives good magic candidates).
    #[inline]
    fn sparse(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}

// Magics baked at build time. The runtime search that produced them
// is deterministic (fixed RNG seed), so these ARE what `find_magic` computes --
// baking them removes the ~170 ms magic search from startup on the generic /
// AVX2 build without changing a single table entry. `find_magic` verifies each
// baked value before use and falls back to searching if one ever fails, so a
// stale constant can only cost startup time, never correctness.
// `baked_magics_cover_every_square` asserts the fallback stays unused.
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

#[cfg(test)]
#[cfg(not(all(rarog_pext, target_arch = "x86_64")))]
mod magic_tests {
    use super::*;

    /// Every square must be served by its BAKED magic, i.e. the
    /// `find_magic` fallback never ran. If this fails the engine is still
    /// correct (the fallback searched a fresh magic) but startup silently
    /// regressed by ~170 ms, which is exactly what baking was meant to remove.
    #[test]
    fn baked_magics_cover_every_square() {
        let attacks = &*ATTACKS;
        for sq in 0..64 {
            assert_eq!(
                attacks.bishop[sq].magic, BISHOP_MAGICS[sq],
                "bishop magic for square {sq} fell back to a runtime search"
            );
            assert_eq!(
                attacks.rook[sq].magic, ROOK_MAGICS[sq],
                "rook magic for square {sq} fell back to a runtime search"
            );
        }
    }
}
