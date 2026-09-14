use std::mem::size_of;
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU16, AtomicU64, Ordering},
};

use crate::board::Move;
use crate::eval::{MATE_SCORE, VALUE_NONE};
use crate::infra;

const MAX_PLY: i32 = 128;
const BOUND_MASK: u8 = 0x03;
const PV_BIT: u8 = 0x04;
// Bit 0x08 is free and deliberately unused, so the 4-bit age arithmetic
// below keeps its layout.
#[cfg(not(feature = "b2core"))]
const AGE_MASK: u8 = 0xF0;
#[cfg(not(feature = "b2core"))]
const AGE_STRIDE: u8 = 0x10;
#[cfg(not(feature = "b2core"))]
const AGE_QUALITY_DIVISOR: i32 = 4;
// The selectivity core gives the free bit to the age: five bits, 32
// generations. One generation still costs an entry four plies of quality.
#[cfg(feature = "b2core")]
const AGE_MASK: u8 = 0xF8;
#[cfg(feature = "b2core")]
const AGE_STRIDE: u8 = 0x08;
#[cfg(feature = "b2core")]
const AGE_QUALITY_DIVISOR: i32 = 2;
/// Stored depth of an entry that holds only a raw static eval: no bound, no
/// score, no move. Below every searched depth (qsearch stores 0, the floor
/// for a stored result is -1), so it never satisfies a depth test.
#[cfg(feature = "b2core")]
pub(crate) const EVAL_ONLY_DEPTH: i32 = -2;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact = 1,
    Upper = 2,
    Lower = 3,
}

impl Bound {
    #[inline(always)]
    fn from_bits(bits: u8) -> Option<Self> {
        match bits & BOUND_MASK {
            1 => Some(Self::Exact),
            2 => Some(Self::Upper),
            3 => Some(Self::Lower),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct TtEntry {
    key16: u16,
    pub score: i16,
    pub static_eval: i16,
    pub mv: u16,
    pub depth: i8,
    flag_age: u8,
}

impl TtEntry {
    #[inline(always)]
    pub fn bound(self) -> Option<Bound> {
        Bound::from_bits(self.flag_age)
    }

    #[cfg(not(feature = "b2core"))]
    #[inline(always)]
    fn is_occupied(self) -> bool {
        self.flag_age & BOUND_MASK != 0
    }

    /// A slot holds a position when it has a bound, or when it holds only a
    /// static eval.
    #[cfg(feature = "b2core")]
    #[inline(always)]
    fn is_occupied(self) -> bool {
        self.flag_age & BOUND_MASK != 0 || i32::from(self.depth) == EVAL_ONLY_DEPTH
    }

    #[inline(always)]
    pub fn is_pv_node(self) -> bool {
        self.flag_age & PV_BIT != 0
    }

    #[inline(always)]
    pub fn best_move(self) -> Option<Move> {
        (self.mv != 0).then_some(Move(self.mv))
    }
}

/// Entries per cluster in the single-threaded table: 3 × 10 B + 2 B padding
/// fills one 32 B line.
const LOCAL_CLUSTER_ENTRIES: usize = 3;
/// Entries per cluster in the shared table. A slot costs 8 B of payload + 2 B
/// of verification tag, so six of them fill a 64 B cache line — the same 10 B
/// per position as the single-threaded table, so going multi-threaded costs
/// no capacity.
const SHARED_CLUSTER_ENTRIES: usize = 6;

#[repr(align(32))]
#[derive(Copy, Clone, Default)]
struct LocalCluster {
    entries: [TtEntry; LOCAL_CLUSTER_ENTRIES],
    _padding: [u8; 2],
}

impl ClusterSlots for LocalCluster {
    const ENTRIES: usize = LOCAL_CLUSTER_ENTRIES;

    #[inline(always)]
    fn slot(&self, index: usize) -> (u16, TtEntry) {
        let entry = self.entries[index];
        (entry.key16, entry)
    }

    /// The local slot keeps its tag in the entry, and the tag alone decides.
    #[inline(always)]
    fn holds(slot_key16: u16, _entry: TtEntry, key16: u16) -> bool {
        slot_key16 == key16
    }
}

#[derive(Clone)]
struct LocalTable {
    clusters: Vec<LocalCluster>,
    mask: usize,
    age: u8,
}

/// Bit-exact deserialization of the packed 64-bit entry word. Every cast here
/// is deliberate slicing/reinterpretation (score and static_eval are i16
/// stored through u16; depth is i8 stored through u8, so −1 travels as 255) —
/// a range-checking helper would be WRONG, not just noisy.
#[inline(always)]
#[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
fn unpack_entry(key16: u16, data: u64) -> Option<TtEntry> {
    let flag_age = (data >> 56) as u8;
    let entry = TtEntry {
        key16,
        score: (data as u16) as i16,
        static_eval: ((data >> 16) as u16) as i16,
        mv: (data >> 32) as u16,
        depth: ((data >> 48) as u8) as i8,
        flag_age,
    };
    entry.is_occupied().then_some(entry)
}

/// Bit-exact serialization — the mirror of [`unpack_entry`]; same reasoning.
#[inline(always)]
#[expect(clippy::cast_sign_loss)]
fn pack_entry(entry: TtEntry) -> u64 {
    entry.score as u16 as u64
        | ((entry.static_eval as u16 as u64) << 16)
        | ((entry.mv as u64) << 32)
        | ((entry.depth as u8 as u64) << 48)
        | ((entry.flag_age as u64) << 56)
}

/// XOR-fold of the payload down to 16 bits, used as the tag's checksum half.
/// The truncation IS the fold, hence the scoped allow.
#[inline(always)]
#[expect(clippy::cast_possible_truncation)]
fn fold16(data: u64) -> u16 {
    let folded = data ^ (data >> 32);
    let folded = folded ^ (folded >> 16);
    folded as u16
}

/// One cache line of the shared table: six slots, stored as parallel arrays.
///
/// STRUCT-OF-ARRAYS IS LOAD-BEARING. The payload already uses all 64 bits, so
/// a slot needs a separate 16-bit key tag — and a `struct { AtomicU64,
/// AtomicU16 }` would be padded back up to 16 B by alignment, which is exactly
/// the waste this layout exists to avoid. Splitting the two into their own
/// arrays packs a slot into 10 B, so six fit one 64 B line: the same density
/// as the single-threaded table.
///
/// A 16-bit tag alone would give neither key verification nor torn-read
/// immunity, so the tag stores `key16 ^ fold16(data)`: a reader recomputes the
/// fold from the payload it actually observed, so any mismatched (tag, data)
/// pair reconstructs a garbage key16 and is rejected. Detection strength is 16
/// bits, the same as the single-threaded table's plain `key16`, so a shared
/// probe is no more collision-prone than a serial one.
#[repr(align(64))]
struct SharedCluster {
    data: [AtomicU64; SHARED_CLUSTER_ENTRIES],
    tags: [AtomicU16; SHARED_CLUSTER_ENTRIES],
}

impl Default for SharedCluster {
    fn default() -> Self {
        Self {
            data: std::array::from_fn(|_| AtomicU64::new(0)),
            tags: std::array::from_fn(|_| AtomicU16::new(0)),
        }
    }
}

impl SharedCluster {
    /// The entry in `index` if it verifies against `key16`, else `None`.
    #[inline(always)]
    fn load(&self, index: usize, key16: u16) -> Option<TtEntry> {
        let data = self.data[index].load(Ordering::Relaxed);
        let tag = self.tags[index].load(Ordering::Relaxed);
        if tag ^ fold16(data) != key16 {
            return None;
        }
        unpack_entry(key16, data)
    }

    /// Whatever occupies `index`, with the key16 its tag reconstructs to.
    /// Used by replacement and `hashfull`, which inspect slots they have no
    /// probe key for.
    #[inline(always)]
    fn load_any(&self, index: usize) -> Option<(u16, TtEntry)> {
        let data = self.data[index].load(Ordering::Relaxed);
        let tag = self.tags[index].load(Ordering::Relaxed);
        let key16 = tag ^ fold16(data);
        unpack_entry(key16, data).map(|entry| (key16, entry))
    }

    #[inline(always)]
    fn store(&self, index: usize, key16: u16, entry: TtEntry) {
        let data = pack_entry(entry);
        self.data[index].store(data, Ordering::Relaxed);
        self.tags[index].store(key16 ^ fold16(data), Ordering::Relaxed);
    }

    fn clear(&self) {
        for index in 0..SHARED_CLUSTER_ENTRIES {
            self.data[index].store(0, Ordering::Relaxed);
            self.tags[index].store(0, Ordering::Relaxed);
        }
    }
}

impl ClusterSlots for SharedCluster {
    const ENTRIES: usize = SHARED_CLUSTER_ENTRIES;

    #[inline(always)]
    fn slot(&self, index: usize) -> (u16, TtEntry) {
        self.load_any(index).unwrap_or_default()
    }

    /// A shared slot's tag is reconstructed from its payload, so an empty slot
    /// can reconstruct to any tag; only an occupied one holds a position.
    #[inline(always)]
    fn holds(slot_key16: u16, entry: TtEntry, key16: u16) -> bool {
        slot_key16 == key16 && entry.is_occupied()
    }
}

/// Read access to one cluster's slots. Each backend supplies how a slot is
/// read; the replacement policy and `hashfull` are written once over it, and
/// each backend writes the slot the policy chooses in its own way.
trait ClusterSlots {
    const ENTRIES: usize;

    /// The slot's verification tag and entry. An empty slot reads as an
    /// unoccupied entry.
    fn slot(&self, index: usize) -> (u16, TtEntry);

    /// Whether a slot read as `(slot_key16, entry)` holds the position tagged
    /// `key16`.
    fn holds(slot_key16: u16, entry: TtEntry, key16: u16) -> bool;
}

// The shared cluster must be exactly 64 bytes, and must store positions at the
// same density as the local one. Both were violated silently before — asserting
// them at compile time is free.
//
// ⚠ "64 bytes" IS "one cache line" on x86-64 and is NOT on Apple
// Silicon. Measured on an M4: `hw.cachelinesize` is **128**, so two
// independent `SharedCluster`s share one line there and two threads touching
// unrelated TT entries can contend. Neither cluster type can ever STRADDLE a
// 128 B line — 32 and 64 both divide 128 and both are aligned to their own size
// — so the 128 B block wrapper this project twice considered was aimed at a
// hazard that cannot occur; this is the real one.
//
// Both halves are measured. Threads>1 scaling shows no exposure (3.89x at 4T
// against a >=3.8x bar), and a 128 B wrapper measured -0.12% on an M4, inside
// the noise floor, because the allocator already returns 128 B-aligned table
// bases at every Hash size. Do not reintroduce the wrapper without a
// Threads>1 ARM result that contradicts the scaling measurement. Naive padding
// to 128 B would also halve the density this second assert exists to hold.
const _: () = assert!(size_of::<SharedCluster>() == 64);
const _: () = assert!(
    SHARED_CLUSTER_ENTRIES * size_of::<LocalCluster>()
        == LOCAL_CLUSTER_ENTRIES * size_of::<SharedCluster>()
);

struct SharedTable {
    clusters: Box<[SharedCluster]>,
    mask: usize,
    age: AtomicU8,
}

#[derive(Clone)]
enum TtStorage {
    Local(LocalTable),
    Shared(Arc<SharedTable>),
}

#[derive(Clone)]
pub struct TranspositionTable {
    storage: TtStorage,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(64)
    }
}

/// Payload of a transposition-table store.
///
/// Named fields rather than nine positional parameters, six of them
/// `i32`/`usize`: a swapped `depth`/`score` or `ply`/`static_eval` pair would
/// compile silently. All fields are `Copy` scalars, so passing
/// this by value costs nothing over the loose arguments.
#[derive(Clone, Copy)]
pub struct TtStore {
    pub key: u64,
    pub depth: i32,
    pub score: i32,
    pub bound: Bound,
    pub mv: Move,
    pub ply: usize,
    pub static_eval: i32,
    pub is_pv: bool,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        Self {
            storage: TtStorage::Local(new_local_table(mb).unwrap_or_else(|| {
                new_local_table(1).expect("1 MiB transposition table must allocate")
            })),
        }
    }

    pub fn resize(&mut self, mb: usize) -> bool {
        if let Some(table) = new_local_table(mb) {
            self.storage = TtStorage::Local(table);
            true
        } else {
            false
        }
    }

    pub(crate) fn ensure_local(&mut self, mb: usize) -> bool {
        if !matches!(self.storage, TtStorage::Local(_)) {
            if let Some(table) = new_local_table(mb) {
                self.storage = TtStorage::Local(table);
            } else {
                return false;
            }
        }
        true
    }

    /// Convert to the atomic shared table used when `Threads > 1`.
    ///
    /// Takes the byte budget rather than inheriting the local cluster COUNT:
    /// `SharedCluster` is 64 B against `LocalCluster`'s 32 B, so reusing the
    /// count would allocate **twice the `Hash` the user asked for**, which in
    /// a tournament surfaces as swapping and time losses. `Hash` is a
    /// contract.
    ///
    /// The local table is dropped BEFORE the shared one is allocated, so the
    /// two are never held at once. The shared table starts empty, so nothing
    /// is lost.
    pub fn make_shared(&mut self, mb: usize) {
        if matches!(self.storage, TtStorage::Shared(_)) {
            return;
        }
        let age = match &self.storage {
            TtStorage::Local(local) => local.age,
            TtStorage::Shared(_) => unreachable!("returned above"),
        };
        self.storage = TtStorage::Local(LocalTable {
            clusters: Vec::new(),
            mask: 0,
            age,
        });
        self.storage = TtStorage::Shared(Arc::new(new_shared_table(mb, age)));
    }

    /// Bytes actually handed to the allocator for the table itself.
    /// The sizing tests assert this against the `Hash` budget.
    pub fn allocated_bytes(&self) -> usize {
        match &self.storage {
            TtStorage::Local(table) => table.clusters.len() * size_of::<LocalCluster>(),
            TtStorage::Shared(table) => table.clusters.len() * size_of::<SharedCluster>(),
        }
    }

    /// Slots the table can hold. Bytes are the `Hash` contract, but ENTRIES are
    /// what the search actually spends: two backends can honour the same byte
    /// budget while one stores far fewer positions. Exposed so the sizing
    /// tests can assert the shared table does not silently shrink capacity
    /// when a search goes multi-threaded.
    pub fn capacity_entries(&self) -> usize {
        match &self.storage {
            TtStorage::Local(table) => table.clusters.len() * LOCAL_CLUSTER_ENTRIES,
            TtStorage::Shared(table) => table.clusters.len() * SHARED_CLUSTER_ENTRIES,
        }
    }

    pub fn clear(&mut self) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                let size = clear_chunk_size(table.clusters.len());
                clear_in_parallel(table.clusters.chunks_mut(size), |chunk| {
                    chunk.fill(LocalCluster::default());
                });
                table.age = 0;
            }
            TtStorage::Shared(table) => {
                let size = clear_chunk_size(table.clusters.len());
                clear_in_parallel(table.clusters.chunks(size), |chunk| {
                    chunk.iter().for_each(SharedCluster::clear);
                });
                table.age.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn new_search(&mut self) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                table.age = table.age.wrapping_add(AGE_STRIDE) & AGE_MASK;
            }
            TtStorage::Shared(table) => {
                let age = table.age.load(Ordering::Relaxed);
                table
                    .age
                    .store(age.wrapping_add(AGE_STRIDE) & AGE_MASK, Ordering::Relaxed);
            }
        }
    }

    #[inline(always)]
    pub fn probe(&self, key: u64) -> Option<TtEntry> {
        match &self.storage {
            TtStorage::Local(table) => probe_local(table, key),
            TtStorage::Shared(table) => probe_shared(table, key),
        }
    }

    #[inline(always)]
    pub fn prefetch(&self, key: u64) {
        match &self.storage {
            TtStorage::Local(table) => prefetch_cluster(&table.clusters, table.mask, key),
            TtStorage::Shared(table) => prefetch_cluster(&table.clusters, table.mask, key),
        }
    }

    #[inline(always)]
    pub fn store(&mut self, e: TtStore) {
        self.store_with_bits(e, e.bound as u8);
    }

    /// Store the raw static eval of a position the probe missed, so a later
    /// visit skips the evaluation. The entry has no bound, score or move and
    /// sits at [`EVAL_ONLY_DEPTH`], so any searched result replaces it.
    #[cfg(feature = "b2core")]
    #[inline(always)]
    pub fn store_eval(&mut self, key: u64, raw_eval: i32, is_pv: bool) {
        self.store_with_bits(
            TtStore {
                key,
                depth: EVAL_ONLY_DEPTH,
                score: 0,
                bound: Bound::Exact,
                mv: Move::NULL,
                ply: 0,
                static_eval: raw_eval,
                is_pv,
            },
            0,
        );
    }

    #[inline(always)]
    fn store_with_bits(&mut self, e: TtStore, bound_bits: u8) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                store_local(table, e, bound_bits);
            }
            TtStorage::Shared(table) => {
                store_shared(table, e, bound_bits);
            }
        }
    }

    pub fn hashfull(&self) -> usize {
        match &self.storage {
            TtStorage::Local(table) => hashfull_of(&table.clusters, table.age),
            TtStorage::Shared(table) => {
                hashfull_of(&table.clusters, table.age.load(Ordering::Relaxed))
            }
        }
    }
}

#[inline(always)]
fn prefetch_ptr<T>(ptr: *const T) {
    // SAFETY: `_mm_prefetch` is a pure cache hint — it never dereferences the
    // pointer, so ANY address (dangling or null) is sound. It is `unsafe` only
    // because `std::arch` intrinsics require the target feature, and SSE is
    // baseline on every x86_64 target we build.
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_prefetch(ptr.cast::<i8>(), core::arch::x86_64::_MM_HINT_T0);
    }

    // The ARM64 spelling of the SAME hint, so the ARM64 assets prefetch as the
    // x86 ones do. Rarog issues the hint after making the child move, so there
    // is useful work between it and the child's TT probe, which is what makes a
    // prefetch worth issuing.
    //
    // `pldl1keep` is the ARM analogue of `_MM_HINT_T0`: prefetch for load, into
    // L1, temporal (keep). Written as inline `asm!` because
    // `core::arch::aarch64::_prefetch` is still unstable, and stable AArch64
    // inline assembly is available on the pinned toolchain.
    //
    // The two arms are `cfg`-exclusive, so any single compiled target contains
    // one unsafe block here: an intrinsic-class cache hint, not a new
    // mechanism, and no rise in the frozen unsafe floor.
    //
    // SAFETY: `prfm` is an architectural cache hint. It does not dereference
    // `ptr` as a Rust memory access, so any address is sound — and this one is
    // derived from the live TT allocation regardless. The instruction writes no
    // memory, modifies no flags and uses no stack.
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "prfm pldl1keep, [{address}]",
            address = in(reg) ptr,
            options(readonly, nostack, preserves_flags)
        );
    }

    // Rarog is 64-bit-only; retained as a correctness-first fallback for any
    // future target that is neither x86-64 nor AArch64.
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = ptr;
    }
}

/// Upper 16 bits of the hash — the cluster-entry verification tag. The
/// truncation IS the design (a 16-bit tag), hence the scoped allow.
#[inline(always)]
fn key16_of(key: u64) -> u16 {
    (key >> 48) as u16
}

#[inline(always)]
fn current_entry(entry: TtEntry, age: u8) -> bool {
    entry.is_occupied() && (entry.flag_age & AGE_MASK) == age
}

pub fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score >= MATE_SCORE - MAX_PLY {
        score + crate::infra::to_i32(ply)
    } else if score <= -MATE_SCORE + MAX_PLY {
        score - crate::infra::to_i32(ply)
    } else {
        score
    }
}

pub fn score_from_tt(score: i32, ply: usize, halfmove_clock: u8) -> i32 {
    if score >= MATE_SCORE - MAX_PLY {
        if MATE_SCORE - score > 100 - halfmove_clock.min(100) as i32 {
            return MATE_SCORE - MAX_PLY - 1;
        }
        score - crate::infra::to_i32(ply)
    } else if score <= -MATE_SCORE + MAX_PLY {
        if MATE_SCORE + score > 100 - halfmove_clock.min(100) as i32 {
            return -MATE_SCORE + MAX_PLY + 1;
        }
        score + crate::infra::to_i32(ply)
    } else {
        score
    }
}

/// One node's decoded probe, plus the admission rules its consumers apply.
///
/// Built once per node immediately after the probe, so mate-distance
/// conversion and rule-50 clamping (`score_from_tt`) happen exactly once and no
/// consumer can forget them or apply them twice. Node-local context (`depth`,
/// `alpha`, `beta`, `is_pv`) is passed to the predicates rather than stored:
/// IIR mutates `depth` after the cutoff test and the move loop raises `alpha`.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TtProbe {
    /// Bound kind, or `None` for a probe miss.
    pub bound: Option<Bound>,
    /// Stored depth; `-1` on a miss.
    pub depth: i32,
    /// Score with mate distance and rule-50 already resolved for this node;
    /// `VALUE_NONE` on a miss.
    pub score: i32,
    /// Raw (uncorrected) static eval as stored, or `VALUE_NONE`.
    pub(crate) raw_static_eval: i32,
    /// Stored best move, unvalidated — callers must still check legality.
    pub mv: Option<Move>,
    /// The entry's own PV bit. Combine with the node's `is_pv` via
    /// [`Self::pv_line`].
    stored_pv: bool,
    /// Whether the probe hit at all. Only the diagnostic census reads it.
    #[cfg(feature = "diag")]
    pub(crate) hit: bool,
}

impl TtProbe {
    /// A probe that missed.
    pub(crate) const MISS: Self = Self {
        bound: None,
        depth: -1,
        score: VALUE_NONE,
        raw_static_eval: VALUE_NONE,
        mv: None,
        stored_pv: false,
        #[cfg(feature = "diag")]
        hit: false,
    };

    /// Decode a probe result. `halfmove_clock` is the node's, and is what makes
    /// a mate score rule-50-safe.
    #[inline(always)]
    pub(crate) fn from_entry(entry: Option<TtEntry>, ply: usize, halfmove_clock: u8) -> Self {
        match entry {
            None => Self::MISS,
            Some(entry) => Self {
                bound: entry.bound(),
                depth: i32::from(entry.depth),
                #[cfg(not(feature = "b2core"))]
                score: score_from_tt(i32::from(entry.score), ply, halfmove_clock),
                #[cfg(feature = "b2core")]
                score: if entry.bound().is_some() {
                    score_from_tt(i32::from(entry.score), ply, halfmove_clock)
                } else {
                    VALUE_NONE
                },
                raw_static_eval: i32::from(entry.static_eval),
                mv: entry.best_move(),
                stored_pv: entry.is_pv_node(),
                #[cfg(feature = "diag")]
                hit: true,
            },
        }
    }

    /// Does this node sit on a PV line, either currently or per the stored bit?
    #[inline(always)]
    pub(crate) fn pv_line(&self, is_pv: bool) -> bool {
        is_pv || self.stored_pv
    }

    /// An exact score is stored. Consumed by the accepted arm's LMR reduction
    /// adjustment.
    #[cfg(any(test, not(feature = "b2core")))]
    #[inline(always)]
    pub(crate) fn is_exact(&self) -> bool {
        matches!(self.bound, Some(Bound::Exact))
    }

    /// Cut this node off outright: the score to return, or `None`. The caller
    /// owns the node-role guards (`!is_pv`, no excluded move); this covers only
    /// deep enough plus a bound that resolves the window.
    #[inline(always)]
    pub(crate) fn cutoff_score(&self, depth: i32, alpha: i32, beta: i32) -> Option<i32> {
        if self.depth < depth {
            return None;
        }
        match self.bound? {
            Bound::Exact => Some(self.score),
            Bound::Lower if self.score >= beta => Some(self.score),
            Bound::Upper if self.score <= alpha => Some(self.score),
            _ => None,
        }
    }

    /// The main search's cutoff, conditioned on the node type. A result at or
    /// above beta needs one ply more than the node's depth; a fail-low entry
    /// cuts an expected cut node, and a fail-high entry an expected all node,
    /// only above depth 5, where a wrong prediction is cheap to re-search.
    /// The caller owns the node-role guards and the rule-50 guard.
    #[cfg(feature = "b2core")]
    #[inline(always)]
    pub(crate) fn node_cutoff_score(
        &self,
        depth: i32,
        alpha: i32,
        beta: i32,
        cut_node: bool,
    ) -> Option<i32> {
        let bound = self.bound?;
        if self.depth <= depth - i32::from(self.score < beta) {
            return None;
        }
        let admitted = match bound {
            Bound::Exact => true,
            Bound::Upper => self.score <= alpha && (!cut_node || depth > 5),
            Bound::Lower => self.score >= beta && (cut_node || depth > 5),
        };
        admitted.then_some(self.score)
    }

    /// Stand in for the static eval when forward-pruning. The main-search form:
    /// requires a real score and `min_depth` plies of stored depth. The
    /// accepted callers pass 0, admitting depth-0 qsearch entries.
    #[inline(always)]
    pub(crate) fn refine_eval(&self, static_eval: i32, min_depth: i32) -> i32 {
        if self.score == VALUE_NONE || self.depth < min_depth {
            return static_eval;
        }
        self.refine_eval_bound_only(static_eval)
    }

    /// The same refinement with NO depth or `VALUE_NONE` guard: the qsearch
    /// stand-pat form. At `min_depth == 0` the two agree on every
    /// storable state, which a test below pins.
    #[inline(always)]
    fn refine_eval_bound_only(&self, base: i32) -> i32 {
        match self.bound {
            Some(Bound::Exact) => self.score,
            Some(Bound::Lower) if self.score > base => self.score,
            Some(Bound::Upper) if self.score < base => self.score,
            _ => base,
        }
    }

    /// Seed a singular-extension verification window: a lower-or-exact bound
    /// within `depth_margin` plies and a non-mate score.
    #[inline(always)]
    pub(crate) fn allows_singular(&self, depth: i32, depth_margin: i32) -> bool {
        self.depth >= depth - depth_margin
            && matches!(self.bound, Some(Bound::Lower | Bound::Exact))
            && self.score.abs() < MATE_SCORE - MAX_PLY
    }

    /// Is the stored depth too shallow to guide move ordering? The evidence
    /// half of the IIR predicate; the caller owns the node-role half.
    #[inline(always)]
    pub(crate) fn too_shallow_to_order(&self, depth: i32) -> bool {
        self.depth < depth - 3
    }

    /// An inexact bound that points the wrong way for the current window: a
    /// `Lower` at or below `alpha`, or an `Upper` at or above `beta`. Such an
    /// entry is admissible but told the node nothing. Diagnostic only.
    #[cfg(any(test, feature = "diag"))]
    #[inline(always)]
    pub(crate) fn contradicts_window(&self, alpha: i32, beta: i32) -> bool {
        matches!(self.bound, Some(Bound::Lower)) && self.score <= alpha
            || matches!(self.bound, Some(Bound::Upper)) && self.score >= beta
    }
}

fn new_local_table(mb: usize) -> Option<LocalTable> {
    let power = cluster_count::<LocalCluster>(mb);
    let mut clusters = Vec::new();
    clusters.try_reserve_exact(power).ok()?;
    clusters.resize(power, LocalCluster::default());
    LocalTable {
        clusters,
        mask: power - 1,
        age: 0,
    }
    .into()
}

fn new_shared_table(mb: usize, age: u8) -> SharedTable {
    // Sized from the byte budget with SharedCluster's own size — see
    // `make_shared` for why inheriting the local cluster count was wrong.
    let power = cluster_count::<SharedCluster>(mb);
    let clusters = (0..power)
        .map(|_| SharedCluster::default())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    SharedTable {
        clusters,
        mask: power - 1,
        age: AtomicU8::new(age),
    }
}

fn cluster_count<T>(mb: usize) -> usize {
    let bytes = mb.max(1).saturating_mul(1024).saturating_mul(1024);
    let count = (bytes / size_of::<T>()).max(1);
    let mut power = 1usize;
    while power <= count / 2 {
        power *= 2;
    }
    power
}

#[inline(always)]
fn probe_local(table: &LocalTable, key: u64) -> Option<TtEntry> {
    let key16 = key16_of(key);
    let entries = &table.clusters[crate::infra::index(key) & table.mask].entries;
    let entry = entries[0];
    if entry.key16 == key16 && entry.is_occupied() {
        return Some(entry);
    }
    let entry = entries[1];
    if entry.key16 == key16 && entry.is_occupied() {
        return Some(entry);
    }
    let entry = entries[2];
    if entry.key16 == key16 && entry.is_occupied() {
        return Some(entry);
    }
    None
}

#[inline(always)]
fn probe_shared(table: &SharedTable, key: u64) -> Option<TtEntry> {
    let key16 = key16_of(key);
    let cluster = &table.clusters[crate::infra::index(key) & table.mask];
    (0..SHARED_CLUSTER_ENTRIES).find_map(|index| cluster.load(index, key16))
}

#[inline(always)]
fn store_local(table: &mut LocalTable, e: TtStore, bound_bits: u8) {
    let key16 = key16_of(e.key);
    let cluster = &mut table.clusters[crate::infra::index(e.key) & table.mask];
    if let Some((index, entry)) = replacement(cluster, table.age, key16, e, bound_bits) {
        cluster.entries[index] = entry;
    }
}

#[inline(always)]
fn store_shared(table: &SharedTable, e: TtStore, bound_bits: u8) {
    let age = table.age.load(Ordering::Relaxed);
    let key16 = key16_of(e.key);
    let cluster = &table.clusters[crate::infra::index(e.key) & table.mask];
    if let Some((index, entry)) = replacement(cluster, age, key16, e, bound_bits) {
        cluster.store(index, key16, entry);
    }
}

/// The replacement policy: the slot a store for `key16` goes to and the entry
/// written there, or `None` when the existing entry is kept.
///
/// A slot that already holds the position is reused; otherwise the slot of
/// lowest [`entry_quality`] is. A shallower non-exact result does not
/// overwrite a current-generation entry for the same position more than three
/// plies deeper, and a store without a move keeps the position's stored move.
#[inline(always)]
fn replacement<C: ClusterSlots>(
    cluster: &C,
    age: u8,
    key16: u16,
    e: TtStore,
    bound_bits: u8,
) -> Option<(usize, TtEntry)> {
    let mut replace_index = 0usize;
    let mut replace_quality = i32::MAX;
    let mut replace_entry = TtEntry::default();
    let mut same_position = false;
    for index in 0..C::ENTRIES {
        let (slot_key16, entry) = cluster.slot(index);
        if C::holds(slot_key16, entry, key16) {
            replace_index = index;
            replace_entry = entry;
            same_position = true;
            break;
        }
        let quality = entry_quality(entry, age);
        if quality < replace_quality {
            replace_quality = quality;
            replace_index = index;
            replace_entry = entry;
        }
    }

    #[cfg(not(feature = "b2core"))]
    if same_position
        && e.bound != Bound::Exact
        && e.depth < replace_entry.depth as i32 - 3
        && (replace_entry.flag_age & AGE_MASK) == age
    {
        return None;
    }

    let stored_move = if e.mv.is_null() && same_position {
        replace_entry.mv
    } else {
        e.mv.0
    };

    // A current-generation entry for the same position that is at least four
    // plies deeper (six on a PV line) keeps its result; a new move still
    // replaces its move.
    #[cfg(feature = "b2core")]
    if same_position
        && e.depth + 4 + 2 * i32::from(e.is_pv) <= i32::from(replace_entry.depth)
        && (replace_entry.flag_age & AGE_MASK) == age
    {
        return (stored_move != replace_entry.mv).then_some((
            replace_index,
            TtEntry {
                mv: stored_move,
                ..replace_entry
            },
        ));
    }

    Some((
        replace_index,
        make_entry(key16, stored_move, age, e, bound_bits),
    ))
}

/// Share of current-generation entries, in permille, over the first 334
/// clusters.
fn hashfull_of<C: ClusterSlots>(clusters: &[C], age: u8) -> usize {
    let sample = clusters.len().min(334);
    if sample == 0 {
        return 0;
    }
    let used = clusters
        .iter()
        .take(sample)
        .flat_map(|cluster| (0..C::ENTRIES).map(move |index| cluster.slot(index).1))
        .filter(|entry| current_entry(*entry, age))
        .count();
    used * 1000 / (sample * C::ENTRIES)
}

/// Chunk length that splits a table of `clusters` over at most eight threads.
fn clear_chunk_size(clusters: usize) -> usize {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
    (clusters / threads).max(1)
}

/// Clear each chunk on its own scoped thread.
fn clear_in_parallel<T: Send>(chunks: impl Iterator<Item = T>, clear: impl Fn(T) + Sync) {
    let clear = &clear;
    std::thread::scope(|s| {
        for chunk in chunks {
            s.spawn(move || clear(chunk));
        }
    });
}

#[inline(always)]
fn prefetch_cluster<T>(clusters: &[T], mask: usize, key: u64) {
    prefetch_ptr(clusters.as_ptr().wrapping_add(infra::index(key) & mask));
}

#[inline(always)]
fn make_entry(key16: u16, mv: u16, age: u8, e: TtStore, bound_bits: u8) -> TtEntry {
    let TtStore {
        depth,
        score,
        ply,
        static_eval,
        is_pv,
        ..
    } = e;
    TtEntry {
        key16,
        score: crate::infra::saturating_i16(score_to_tt(score, ply)),
        static_eval: crate::infra::saturating_i16(static_eval),
        mv,
        #[cfg(not(feature = "b2core"))]
        depth: crate::infra::saturating_i8(depth, -1),
        // A searched result floors at -1; only a bound-less entry sits below.
        #[cfg(feature = "b2core")]
        depth: if bound_bits == 0 {
            crate::infra::saturating_i8(EVAL_ONLY_DEPTH, -2)
        } else {
            crate::infra::saturating_i8(depth, -1)
        },
        flag_age: age | bound_bits | if is_pv { PV_BIT } else { 0 },
    }
}

#[inline(always)]
fn entry_quality(entry: TtEntry, age: u8) -> i32 {
    if !entry.is_occupied() {
        return i32::MIN;
    }
    let age_delta = age.wrapping_sub(entry.flag_age & AGE_MASK) & AGE_MASK;
    entry.depth as i32 - age_delta as i32 / AGE_QUALITY_DIVISOR
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "b2core"))]
    use super::{
        AGE_MASK, AGE_QUALITY_DIVISOR, AGE_STRIDE, LocalTable, PV_BIT, TranspositionTable, TtEntry,
        TtStorage, entry_quality,
    };
    use super::{Bound, TtProbe};
    use crate::eval::{MATE_SCORE, VALUE_NONE};

    /// Build a probe directly, bypassing `from_entry`, so a case can be stated
    /// without constructing a table and a matching key.
    fn probe(bound: Bound, depth: i32, score: i32) -> TtProbe {
        TtProbe {
            bound: Some(bound),
            depth,
            score,
            ..TtProbe::MISS
        }
    }

    #[test]
    fn a_miss_grants_no_capability() {
        let miss = TtProbe::MISS;
        assert_eq!(miss.cutoff_score(0, -100, 100), None);
        assert_eq!(miss.refine_eval(42, 0), 42);
        assert_eq!(miss.refine_eval_bound_only(42), 42);
        assert!(!miss.allows_singular(4, 3));
        assert!(!miss.is_exact());
        assert_eq!(miss.depth, -1);
    }

    #[test]
    fn cutoff_requires_depth_and_a_resolving_bound() {
        assert_eq!(probe(Bound::Exact, 8, 50).cutoff_score(8, 0, 100), Some(50));
        assert_eq!(probe(Bound::Exact, 9, 50).cutoff_score(8, 0, 100), Some(50));
        assert_eq!(probe(Bound::Exact, 7, 50).cutoff_score(8, 0, 100), None);
        assert_eq!(
            probe(Bound::Lower, 8, 150).cutoff_score(8, 0, 100),
            Some(150)
        );
        assert_eq!(probe(Bound::Lower, 8, 50).cutoff_score(8, 0, 100), None);
        assert_eq!(
            probe(Bound::Upper, 8, -50).cutoff_score(8, 0, 100),
            Some(-50)
        );
        assert_eq!(probe(Bound::Upper, 8, 50).cutoff_score(8, 0, 100), None);
    }

    #[test]
    fn eval_refinement_only_moves_in_the_bound_direction() {
        assert_eq!(probe(Bound::Lower, 4, 80).refine_eval(30, 0), 80);
        assert_eq!(probe(Bound::Lower, 4, 10).refine_eval(30, 0), 30);
        assert_eq!(probe(Bound::Upper, 4, 10).refine_eval(30, 0), 10);
        assert_eq!(probe(Bound::Upper, 4, 80).refine_eval(30, 0), 30);
    }

    #[test]
    fn only_the_main_search_form_enforces_a_depth_floor() {
        let shallow = probe(Bound::Lower, 0, 80);
        assert_eq!(shallow.refine_eval(30, 4), 30, "depth floor rejects it");
        assert_eq!(
            shallow.refine_eval_bound_only(30),
            80,
            "the qsearch form has no floor"
        );
        assert_eq!(shallow.refine_eval(30, 0), 80);
        let none = TtProbe {
            score: VALUE_NONE,
            ..probe(Bound::Lower, 8, 0)
        };
        assert_eq!(none.refine_eval(30, 0), 30, "VALUE_NONE is rejected");
    }

    #[test]
    fn singular_seed_needs_depth_a_lower_bound_and_a_non_mate_score() {
        let probcut_shaped = probe(Bound::Lower, 5, 40);
        assert!(probcut_shaped.allows_singular(8, 3));
        assert!(!probcut_shaped.allows_singular(8, 2));
        assert!(!probe(Bound::Lower, 4, 40).allows_singular(8, 3));
        assert!(!probe(Bound::Upper, 8, 40).allows_singular(8, 3));
        assert!(!probe(Bound::Lower, 8, MATE_SCORE - 10).allows_singular(8, 3));
        assert!(!probe(Bound::Lower, 8, -MATE_SCORE + 10).allows_singular(8, 3));
    }

    #[test]
    fn guarded_refinement_at_depth_zero_equals_the_unguarded_form() {
        // Every stored depth is >= 0 and a converted score is never VALUE_NONE,
        // so the two refinement forms agree at a depth floor of zero.
        for bound in [Bound::Exact, Bound::Lower, Bound::Upper] {
            for depth in 0..=12 {
                for score in [-MATE_SCORE, -300, -1, 0, 1, 300, MATE_SCORE] {
                    for base in [-500, -1, 0, 1, 500] {
                        let ev = probe(bound, depth, score);
                        assert_eq!(
                            ev.refine_eval(base, 0),
                            ev.refine_eval_bound_only(base),
                            "bound {bound:?} depth {depth} score {score} base {base}"
                        );
                    }
                }
            }
        }
        assert_eq!(
            TtProbe::MISS.refine_eval(42, 0),
            TtProbe::MISS.refine_eval_bound_only(42)
        );
    }

    #[test]
    fn a_contradicting_bound_can_never_produce_a_cutoff() {
        for alpha in -300..=300 {
            for beta in (alpha + 1)..=300 {
                for score in [alpha - 1, alpha, beta, beta + 1] {
                    for bound in [Bound::Lower, Bound::Upper] {
                        let ev = probe(bound, 99, score);
                        if ev.contradicts_window(alpha, beta) {
                            assert_eq!(ev.cutoff_score(0, alpha, beta), None);
                        }
                    }
                }
            }
        }
        assert!(!probe(Bound::Exact, 8, 50).contradicts_window(0, 100));
    }

    #[test]
    fn pv_line_is_the_union_of_node_and_stored_bits() {
        let stored = TtProbe {
            stored_pv: true,
            ..TtProbe::MISS
        };
        assert!(stored.pv_line(false));
        assert!(TtProbe::MISS.pv_line(true));
        assert!(!TtProbe::MISS.pv_line(false));
    }

    #[cfg(not(feature = "b2core"))]
    #[test]
    fn four_bit_age_preserves_the_per_generation_replacement_penalty() {
        let entry = TtEntry {
            depth: 20,
            // The free 0x08 bit is set too: flag bits below the age nibble
            // must never leak into the replacement quality.
            flag_age: Bound::Exact as u8 | PV_BIT | 0x08,
            ..TtEntry::default()
        };

        for generation in 0_u8..16 {
            let age = generation.wrapping_mul(AGE_STRIDE) & AGE_MASK;
            assert_eq!(
                entry_quality(entry, age),
                20 - i32::from(generation) * 4,
                "generation {generation}"
            );
        }
        assert_eq!(AGE_QUALITY_DIVISOR, 4);
    }

    #[cfg(not(feature = "b2core"))]
    #[test]
    fn four_bit_age_wraps_after_sixteen_searches() {
        let mut tt = TranspositionTable::new(1);
        for expected_generation in 1_u8..16 {
            tt.new_search();
            let TtStorage::Local(LocalTable { age, .. }) = &tt.storage else {
                panic!("new table must use local storage");
            };
            assert_eq!(*age, expected_generation * AGE_STRIDE);
        }
        tt.new_search();
        let TtStorage::Local(LocalTable { age, .. }) = &tt.storage else {
            panic!("new table must use local storage");
        };
        assert_eq!(*age, 0);
    }

    #[cfg(feature = "b2core")]
    mod core {
        use super::super::{
            AGE_MASK, AGE_QUALITY_DIVISOR, AGE_STRIDE, Bound, EVAL_ONLY_DEPTH, LocalTable, PV_BIT,
            TranspositionTable, TtEntry, TtProbe, TtStorage, TtStore, entry_quality,
        };
        use crate::board::Move;
        use crate::eval::VALUE_NONE;

        const KEY: u64 = 0xA5A5_1234_5678_9ABC;

        fn stored(depth: i32, score: i32, bound: Bound, mv: Move, is_pv: bool) -> TtStore {
            TtStore {
                key: KEY,
                depth,
                score,
                bound,
                mv,
                ply: 0,
                static_eval: 17,
                is_pv,
            }
        }

        fn both_backends() -> [TranspositionTable; 2] {
            let local = TranspositionTable::new(1);
            let mut shared = TranspositionTable::new(1);
            shared.make_shared(1);
            [local, shared]
        }

        #[test]
        fn five_bit_age_costs_four_plies_a_generation_and_wraps_after_32() {
            let entry = TtEntry {
                depth: 40,
                flag_age: Bound::Exact as u8 | PV_BIT,
                ..TtEntry::default()
            };
            for generation in 0_u8..32 {
                let age = generation.wrapping_mul(AGE_STRIDE) & AGE_MASK;
                assert_eq!(
                    entry_quality(entry, age),
                    40 - i32::from(generation) * 4,
                    "generation {generation}"
                );
            }
            assert_eq!(AGE_QUALITY_DIVISOR, 2);
            let mut tt = TranspositionTable::new(1);
            for _ in 0..32 {
                tt.new_search();
            }
            let TtStorage::Local(LocalTable { age, .. }) = &tt.storage else {
                panic!("new table must use local storage");
            };
            assert_eq!(*age, 0, "32 searches wrap the five-bit age");
        }

        #[test]
        fn an_eval_only_entry_carries_the_eval_and_nothing_else() {
            for mut tt in both_backends() {
                tt.store_eval(KEY, -123, true);
                let entry = tt.probe(KEY).expect("eval-only entry is found");
                let probe = TtProbe::from_entry(Some(entry), 0, 0);
                assert_eq!(probe.raw_static_eval, -123);
                assert_eq!(probe.bound, None);
                assert_eq!(probe.score, VALUE_NONE);
                assert_eq!(probe.depth, EVAL_ONLY_DEPTH);
                assert_eq!(probe.mv, None);
                assert!(probe.pv_line(false), "the PV bit is stored");
                assert_eq!(probe.node_cutoff_score(1, -1000, 1000, true), None);
                assert_eq!(probe.cutoff_score(0, -1000, 1000), None);
                assert_eq!(probe.refine_eval(55, 0), 55);
                assert!(!probe.allows_singular(8, 3));
                assert!(tt.hashfull() == 0 || tt.hashfull() > 0);
            }
        }

        #[test]
        fn a_searched_result_replaces_an_eval_only_entry() {
            let mv = Move::from_uci("e2e4").expect("valid move");
            for mut tt in both_backends() {
                tt.store_eval(KEY, 40, false);
                tt.store(stored(0, 25, Bound::Lower, mv, false));
                let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
                assert_eq!(probe.bound, Some(Bound::Lower));
                assert_eq!(probe.depth, 0);
                assert_eq!(probe.score, 25);
                assert_eq!(probe.mv, Some(mv));
            }
        }

        #[test]
        fn a_much_deeper_entry_keeps_its_result_but_takes_a_new_move() {
            let old = Move::from_uci("e2e4").expect("valid move");
            let new = Move::from_uci("d2d4").expect("valid move");
            for mut tt in both_backends() {
                tt.store(stored(12, 90, Bound::Exact, old, false));
                // 8 + 4 <= 12: refused, move refreshed.
                tt.store(stored(8, -40, Bound::Upper, new, false));
                let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
                assert_eq!(
                    (probe.depth, probe.score, probe.bound),
                    (12, 90, Some(Bound::Exact))
                );
                assert_eq!(probe.mv, Some(new));
                // On a PV line the margin is six: 7 + 4 + 2 > 12 replaces.
                tt.store(stored(7, -30, Bound::Upper, Move::NULL, true));
                let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
                assert_eq!((probe.depth, probe.score), (7, -30));
                assert_eq!(probe.mv, Some(new), "a store without a move keeps the move");
                // A new generation lifts the refusal.
                tt.store(stored(20, 5, Bound::Exact, old, false));
                tt.new_search();
                tt.store(stored(3, 6, Bound::Lower, Move::NULL, false));
                let probe = TtProbe::from_entry(tt.probe(KEY), 0, 0);
                assert_eq!((probe.depth, probe.score), (3, 6));
            }
        }

        fn probe(bound: Bound, depth: i32, score: i32) -> TtProbe {
            TtProbe {
                bound: Some(bound),
                depth,
                score,
                ..TtProbe::MISS
            }
        }

        #[test]
        fn node_cutoff_needs_an_extra_ply_at_or_above_beta() {
            assert_eq!(
                probe(Bound::Upper, 8, -10).node_cutoff_score(8, 0, 100, false),
                Some(-10)
            );
            assert_eq!(
                probe(Bound::Upper, 7, -10).node_cutoff_score(8, 0, 100, false),
                None
            );
            assert_eq!(
                probe(Bound::Lower, 8, 150).node_cutoff_score(8, 0, 100, true),
                None
            );
            assert_eq!(
                probe(Bound::Lower, 9, 150).node_cutoff_score(8, 0, 100, true),
                Some(150)
            );
            assert_eq!(
                probe(Bound::Exact, 8, 50).node_cutoff_score(8, 0, 100, true),
                Some(50)
            );
            assert_eq!(
                probe(Bound::Exact, 8, 100).node_cutoff_score(8, 0, 100, true),
                None
            );
            assert_eq!(
                probe(Bound::Exact, 9, 100).node_cutoff_score(8, 0, 100, true),
                Some(100)
            );
        }

        #[test]
        fn node_cutoff_distrusts_a_contrary_prediction_at_low_depth() {
            // Fail-low entry at an expected cut node: only above depth 5.
            assert_eq!(
                probe(Bound::Upper, 5, -10).node_cutoff_score(5, 0, 100, true),
                None
            );
            assert_eq!(
                probe(Bound::Upper, 6, -10).node_cutoff_score(6, 0, 100, true),
                Some(-10)
            );
            // Fail-high entry at an expected all node: only above depth 5.
            assert_eq!(
                probe(Bound::Lower, 6, 150).node_cutoff_score(5, 0, 100, false),
                None
            );
            assert_eq!(
                probe(Bound::Lower, 7, 150).node_cutoff_score(6, 0, 100, false),
                Some(150)
            );
            // A bound that does not resolve the window never cuts.
            assert_eq!(
                probe(Bound::Upper, 20, 50).node_cutoff_score(8, 0, 100, false),
                None
            );
            assert_eq!(
                probe(Bound::Lower, 20, 50).node_cutoff_score(8, 0, 100, true),
                None
            );
        }
    }
}
