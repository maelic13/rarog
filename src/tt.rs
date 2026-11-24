use std::mem::size_of;
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU16, AtomicU64, Ordering},
};

use crate::board::Move;
use crate::eval::{MATE_SCORE, VALUE_NONE};

/// The search's ply cap (`search::MAX_PLY`), which places the mate band.
const MAX_PLY: i32 = 240;
/// The decisive band's floor: a tablebase win or anything better. Below
/// the mate band (`MATE_SCORE - MAX_PLY` and up) lies the tablebase band,
/// `TB_VALUE - ply` for a win found `ply` plies from the root. Window
/// arithmetic near mate scores also lands in it (a fail-hard quiescence
/// stores ProbCut's `beta + margin` at a losing beta), so the table keeps
/// the band's values as stored: moving them by ply changes the search
/// without tablebases.
pub(crate) const TB_WIN_SCORE: i32 = MATE_SCORE - 2 * MAX_PLY;
/// A tablebase win at the root; one below the mate band.
pub(crate) const TB_VALUE: i32 = MATE_SCORE - MAX_PLY - 1;
const BOUND_MASK: u8 = 0x03;
const PV_BIT: u8 = 0x04;
// The age takes the five high bits: 32 generations. One generation costs an
// entry four plies of quality.
const AGE_MASK: u8 = 0xF8;
const AGE_STRIDE: u8 = 0x08;
const AGE_QUALITY_DIVISOR: i32 = 2;
/// Stored depth of a quiescence result, the donor's: one ply below every
/// main-search store, so it never satisfies a main-search depth test, and
/// level with an evaluation-only entry in the keep rule and as a victim.
pub(crate) const QUIESCENCE_DEPTH: i32 = -1;
/// Stored depth of an entry that holds only a raw static eval: no bound, no
/// score, no move. A quiescence result's depth, as in the donor's table; the
/// missing bound tells the two apart.
pub(crate) const EVAL_ONLY_DEPTH: i32 = QUIESCENCE_DEPTH;

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

    /// A slot holds a position when it has a bound, or when it holds only a
    /// static eval.
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

    /// The local slot keeps its tag in the entry, and the tag alone decides:
    /// an empty slot's tag is 0, so a position tagged 0 takes an empty slot
    /// as its own, which is where its store would land anyway.
    #[inline(always)]
    fn holds(slot_key16: u16, _entry: TtEntry, key16: u16) -> bool {
        slot_key16 == key16
    }
}

struct LocalTable {
    clusters: Vec<LocalCluster>,
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

// The shared cluster must be exactly 64 bytes, one x86-64 cache line, and
// must store positions at the same density as the local one.
//
// Apple Silicon's line is 128 bytes, so two shared clusters share one there;
// neither cluster type can straddle a line. A 128-byte wrapper measured
// −0.12% on an M4 and four threads scaled 3.89x, so the clusters stay 64
// bytes until a multi-thread ARM read shows contention; padding would also
// halve the density the second assert holds.
const _: () = assert!(size_of::<SharedCluster>() == 64);
const _: () = assert!(
    SHARED_CLUSTER_ENTRIES * size_of::<LocalCluster>()
        == LOCAL_CLUSTER_ENTRIES * size_of::<SharedCluster>()
);

struct SharedTable {
    clusters: Box<[SharedCluster]>,
    age: AtomicU8,
}

enum TtStorage {
    Local(LocalTable),
    Shared(Arc<SharedTable>),
}

pub struct TranspositionTable {
    storage: TtStorage,
}

/// A handle on the shared table, the one thing about the table that crosses
/// to another thread. Only the shared form produces one, so handing a table
/// to a helper can never copy it, and the table lives as long as its handles.
#[derive(Clone)]
pub struct SharedTt(Arc<SharedTable>);

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

    /// [`Self::resize`] straight into the shared form, so a `Hash` change at
    /// `Threads > 1` fills one table rather than a local one that conversion
    /// drops. A refused size keeps the table in use, which is why the old
    /// table is still held while the new one fills.
    pub fn resize_shared(&mut self, mb: usize) -> bool {
        if let Some(table) = new_shared_table(mb, 0) {
            self.storage = TtStorage::Shared(Arc::new(table));
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
    /// is lost. Returns a handle on the shared table.
    pub fn make_shared(&mut self, mb: usize) -> SharedTt {
        let age = match &self.storage {
            TtStorage::Local(local) => local.age,
            TtStorage::Shared(table) => return SharedTt(Arc::clone(table)),
        };
        self.storage = TtStorage::Local(LocalTable {
            clusters: Vec::new(),
            age,
        });
        let table = Arc::new(
            new_shared_table(mb, age).expect("the shared transposition table must allocate"),
        );
        self.storage = TtStorage::Shared(Arc::clone(&table));
        SharedTt(table)
    }

    /// The table a helper searches for one job.
    pub(crate) fn attached(handle: SharedTt) -> Self {
        Self {
            storage: TtStorage::Shared(handle.0),
        }
    }

    /// Whether the table is in the atomic form a multi-threaded search uses.
    #[cfg(test)]
    pub(crate) fn is_shared(&self) -> bool {
        matches!(self.storage, TtStorage::Shared(_))
    }

    /// Live handles on the shared table, this one included; 0 in local form.
    #[cfg(test)]
    pub(crate) fn shared_handles(&self) -> usize {
        match &self.storage {
            TtStorage::Local(_) => 0,
            TtStorage::Shared(table) => Arc::strong_count(table),
        }
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
            TtStorage::Local(table) => prefetch_cluster(&table.clusters, key),
            TtStorage::Shared(table) => prefetch_cluster(&table.clusters, key),
        }
    }

    #[inline(always)]
    pub fn store(&mut self, e: TtStore) {
        self.store_with_bits(e, e.bound as u8, false);
    }

    /// [`Self::store`], overwriting a deeper result for the same position:
    /// a PV node's final result in the compact kernel.
    #[inline(always)]
    pub fn store_forced(&mut self, e: TtStore) {
        self.store_with_bits(e, e.bound as u8, true);
    }

    /// Store the raw static eval of a position the probe missed, so a later
    /// visit skips the evaluation. The entry has no bound, score or move and
    /// sits at [`EVAL_ONLY_DEPTH`], so any searched result replaces it.
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
            false,
        );
    }

    #[inline(always)]
    fn store_with_bits(&mut self, e: TtStore, bound_bits: u8, force: bool) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                store_local(table, e, bound_bits, force);
            }
            TtStorage::Shared(table) => {
                store_shared(table, e, bound_bits, force);
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
    // x86 ones do. Rarog issues the hint before making the child move, from
    // the key the move will produce, and again after, so the make runs while
    // the line arrives; with the hint after the make alone the child's probe
    // stalled on it.
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

/// Low 16 bits of the hash — the cluster-entry verification tag. The cluster
/// comes from the high bits ([`cluster_index`]), so the two stay independent.
/// The truncation IS the design (a 16-bit tag), hence the scoped allow.
#[inline(always)]
#[expect(clippy::cast_possible_truncation)]
fn key16_of(key: u64) -> u16 {
    key as u16
}

/// The cluster of `key` in a table of `clusters`: the high half of their
/// product, which maps the keys evenly onto any count, so `Hash` need not be a
/// power of two. The product's high half is below `clusters`, so it fits.
#[inline(always)]
#[expect(clippy::cast_possible_truncation)]
fn cluster_index(key: u64, clusters: usize) -> usize {
    ((u128::from(key) * clusters as u128) >> 64) as usize
}

#[inline(always)]
fn current_entry(entry: TtEntry, age: u8) -> bool {
    entry.is_occupied() && (entry.flag_age & AGE_MASK) == age
}

/// A mate score is stored as its distance from the node rather than from the
/// root. Tablebase-band values are stored as they are; see [`TB_WIN_SCORE`].
pub fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score >= MATE_SCORE - MAX_PLY {
        score + crate::infra::to_i32(ply)
    } else if score <= -MATE_SCORE + MAX_PLY {
        score - crate::infra::to_i32(ply)
    } else {
        score
    }
}

/// The inverse of [`score_to_tt`]. A mate further away than the rule-50
/// counter allows may be false, through the counter or a graph-history path,
/// so it reads back as the highest score that is not decisive.
pub fn score_from_tt(score: i32, ply: usize, halfmove_clock: u8) -> i32 {
    let horizon = 100 - i32::from(halfmove_clock.min(100));
    if score >= MATE_SCORE - MAX_PLY {
        if MATE_SCORE - score > horizon {
            return TB_WIN_SCORE - 1;
        }
        score - crate::infra::to_i32(ply)
    } else if score <= -MATE_SCORE + MAX_PLY {
        if MATE_SCORE + score > horizon {
            return -TB_WIN_SCORE + 1;
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
                score: if entry.bound().is_some() {
                    score_from_tt(i32::from(entry.score), ply, halfmove_clock)
                } else {
                    VALUE_NONE
                },
                raw_static_eval: i32::from(entry.static_eval),
                mv: entry.best_move(),
                stored_pv: entry.is_pv_node(),
            },
        }
    }

    /// Does this node sit on a PV line, either currently or per the stored bit?
    #[inline(always)]
    pub(crate) fn pv_line(&self, is_pv: bool) -> bool {
        is_pv || self.stored_pv
    }
}

fn new_local_table(mb: usize) -> Option<LocalTable> {
    let count = cluster_count::<LocalCluster>(mb);
    let mut clusters = Vec::new();
    clusters.try_reserve_exact(count).ok()?;
    clusters.resize(count, LocalCluster::default());
    LocalTable { clusters, age: 0 }.into()
}

fn new_shared_table(mb: usize, age: u8) -> Option<SharedTable> {
    // Sized from the byte budget with SharedCluster's own size, not the local
    // cluster count (see `make_shared`).
    let count = cluster_count::<SharedCluster>(mb);
    let mut clusters = Vec::new();
    clusters.try_reserve_exact(count).ok()?;
    clusters.resize_with(count, SharedCluster::default);
    SharedTable {
        clusters: clusters.into_boxed_slice(),
        age: AtomicU8::new(age),
    }
    .into()
}

fn cluster_count<T>(mb: usize) -> usize {
    let bytes = mb.max(1).saturating_mul(1024).saturating_mul(1024);
    (bytes / size_of::<T>()).max(1)
}

#[inline(always)]
fn probe_local(table: &LocalTable, key: u64) -> Option<TtEntry> {
    let key16 = key16_of(key);
    let entries = &table.clusters[cluster_index(key, table.clusters.len())].entries;
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
    let cluster = &table.clusters[cluster_index(key, table.clusters.len())];
    (0..SHARED_CLUSTER_ENTRIES).find_map(|index| cluster.load(index, key16))
}

#[inline(always)]
fn store_local(table: &mut LocalTable, e: TtStore, bound_bits: u8, force: bool) {
    let key16 = key16_of(e.key);
    let count = table.clusters.len();
    let cluster = &mut table.clusters[cluster_index(e.key, count)];
    if let Some((index, entry)) = replacement(cluster, table.age, key16, e, bound_bits, force) {
        cluster.entries[index] = entry;
    }
}

#[inline(always)]
fn store_shared(table: &SharedTable, e: TtStore, bound_bits: u8, force: bool) {
    let age = table.age.load(Ordering::Relaxed);
    let key16 = key16_of(e.key);
    let cluster = &table.clusters[cluster_index(e.key, table.clusters.len())];
    if let Some((index, entry)) = replacement(cluster, age, key16, e, bound_bits, force) {
        cluster.store(index, key16, entry);
    }
}

/// The replacement policy: the slot a store for `key16` goes to and the entry
/// written there, or `None` when the existing entry is kept.
///
/// A slot that already holds the position is reused; otherwise the slot of
/// lowest [`entry_quality`] is. A result of any bound does not overwrite a
/// current-generation entry for the same position at least four plies deeper
/// (six for a PV store), though a new move still replaces its move; a store
/// without a move keeps the position's stored move. A forced store always
/// writes its result.
#[inline(always)]
fn replacement<C: ClusterSlots>(
    cluster: &C,
    age: u8,
    key16: u16,
    e: TtStore,
    bound_bits: u8,
    force: bool,
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

    let stored_move = if e.mv.is_null() && same_position {
        replace_entry.mv
    } else {
        e.mv.0
    };

    // A current-generation entry for the same position that is at least four
    // plies deeper (six on a PV line) keeps its result; a new move still
    // replaces its move.
    if !force
        && same_position
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
fn prefetch_cluster<T>(clusters: &[T], key: u64) {
    prefetch_ptr(
        clusters
            .as_ptr()
            .wrapping_add(cluster_index(key, clusters.len())),
    );
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
        // A searched result floors at -1; only a bound-less entry sits below.
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
mod tests;
