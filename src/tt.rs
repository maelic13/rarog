use std::mem::{align_of, size_of};
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU16, AtomicU64, Ordering},
};

use crate::board::Move;
use crate::eval::MATE_SCORE;
use crate::infra;

const MAX_PLY: i32 = 128;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact = 1,
    Upper = 2,
    Lower = 3,
}

impl Bound {
    #[inline(always)]
    fn from_bits(bits: u8) -> Option<Self> {
        match bits & 3 {
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

    #[inline(always)]
    fn is_occupied(self) -> bool {
        self.flag_age & 3 != 0
    }

    #[inline(always)]
    pub fn is_pv_node(self) -> bool {
        (self.flag_age >> 2) & 1 != 0
    }

    #[inline(always)]
    pub fn best_move(self) -> Option<Move> {
        (self.mv != 0).then_some(Move(self.mv))
    }
}

/// Entries per logical cluster in the single-threaded table: 3 × 10 B + 2 B
/// padding fills one 32 B bucket.
const LOCAL_CLUSTER_ENTRIES: usize = 3;
/// Entries per cluster in the shared table. A slot costs 8 B of payload + 2 B
/// of verification tag, so six of them fill a 64 B bucket — the same 10 B
/// per position the single-threaded table has always used, i.e. going
/// multi-threaded no longer costs capacity at all.
const SHARED_CLUSTER_ENTRIES: usize = 6;

// Apple Silicon has 128-byte data-cache lines. Keep the logical clusters and
// their associativity unchanged, but group them in one cache-line-aligned
// allocation unit so the allocator cannot leave the TT base only 32/64-byte
// aligned. Other targets retain their existing storage layout.
const LOCAL_CLUSTERS_PER_BLOCK: usize = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
    4
} else {
    1
};
const SHARED_CLUSTERS_PER_BLOCK: usize = if cfg!(all(target_os = "macos", target_arch = "aarch64"))
{
    2
} else {
    1
};

#[repr(align(32))]
#[derive(Copy, Clone, Default)]
struct LocalCluster {
    entries: [TtEntry; LOCAL_CLUSTER_ENTRIES],
    _padding: [u8; 2],
}

#[cfg_attr(all(target_os = "macos", target_arch = "aarch64"), repr(align(128)))]
#[cfg_attr(
    not(all(target_os = "macos", target_arch = "aarch64")),
    repr(align(32))
)]
#[derive(Copy, Clone, Default)]
struct LocalBlock {
    clusters: [LocalCluster; LOCAL_CLUSTERS_PER_BLOCK],
}

#[derive(Clone)]
struct LocalTable {
    blocks: Vec<LocalBlock>,
    mask: usize,
    age: u8,
}

impl LocalTable {
    #[inline(always)]
    fn cluster_count(&self) -> usize {
        self.blocks.len() * LOCAL_CLUSTERS_PER_BLOCK
    }

    #[inline(always)]
    fn cluster(&self, index: usize) -> &LocalCluster {
        &self.blocks[index / LOCAL_CLUSTERS_PER_BLOCK].clusters[index % LOCAL_CLUSTERS_PER_BLOCK]
    }

    #[inline(always)]
    fn cluster_mut(&mut self, index: usize) -> &mut LocalCluster {
        &mut self.blocks[index / LOCAL_CLUSTERS_PER_BLOCK].clusters
            [index % LOCAL_CLUSTERS_PER_BLOCK]
    }
}

/// Bit-exact deserialization of the packed 64-bit entry word. Every cast here
/// is deliberate slicing/reinterpretation (score and static_eval are i16
/// stored through u16; depth is i8 stored through u8, so −1 travels as 255) —
/// a range-checking helper would be WRONG, not just noisy.
#[inline(always)]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn unpack_entry(key16: u16, data: u64) -> Option<TtEntry> {
    let flag_age = (data >> 56) as u8;
    Bound::from_bits(flag_age)?;

    Some(TtEntry {
        key16,
        score: (data as u16) as i16,
        static_eval: ((data >> 16) as u16) as i16,
        mv: (data >> 32) as u16,
        depth: ((data >> 48) as u8) as i8,
        flag_age,
    })
}

/// Bit-exact serialization — the mirror of [`unpack_entry`]; same reasoning.
#[inline(always)]
#[allow(clippy::cast_sign_loss)]
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
#[allow(clippy::cast_possible_truncation)]
fn fold16(data: u64) -> u16 {
    let folded = data ^ (data >> 32);
    let folded = folded ^ (folded >> 16);
    folded as u16
}

/// One logical bucket of the shared table: six slots, stored as parallel arrays.
///
/// STRUCT-OF-ARRAYS IS LOAD-BEARING. The payload already uses all 64 bits, so
/// a slot needs a separate 16-bit key tag — and a `struct { AtomicU64,
/// AtomicU16 }` would be padded back up to 16 B by alignment, which is exactly
/// the waste this layout exists to avoid. Splitting the two into their own
/// arrays packs a slot into 10 B, so six fit one 64 B line: the same density
/// the single-threaded table has always had.
///
/// The old layout stored `(key ^ data, data)` — 16 B — which bought 64-bit key
/// verification AND torn-read immunity. A 16-bit tag alone would keep neither,
/// so the tag stores `key16 ^ fold16(data)`: a reader recomputes the fold from
/// the payload it actually observed, so any mismatched (tag, data) pair
/// reconstructs a garbage key16 and is rejected. Detection strength is 16 bits
/// — precisely the strength the single-threaded table has always had from its
/// plain `key16`, so a shared probe is no more collision-prone than a serial
/// one, and it is no longer paying 60% more memory for a guarantee the serial
/// engine never had.
#[repr(align(64))]
struct SharedCluster {
    data: [AtomicU64; SHARED_CLUSTER_ENTRIES],
    tags: [AtomicU16; SHARED_CLUSTER_ENTRIES],
}

#[cfg_attr(all(target_os = "macos", target_arch = "aarch64"), repr(align(128)))]
#[cfg_attr(
    not(all(target_os = "macos", target_arch = "aarch64")),
    repr(align(64))
)]
struct SharedBlock {
    clusters: [SharedCluster; SHARED_CLUSTERS_PER_BLOCK],
}

impl Default for SharedBlock {
    fn default() -> Self {
        Self {
            clusters: std::array::from_fn(|_| SharedCluster::default()),
        }
    }
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

    #[inline(always)]
    fn clear_slot(&self, index: usize) {
        self.data[index].store(0, Ordering::Relaxed);
        self.tags[index].store(0, Ordering::Relaxed);
    }
}

// Pin both the logical-cluster density and the physical allocation units.
const _: () = assert!(size_of::<SharedCluster>() == 64);
const _: () = assert!(
    SHARED_CLUSTER_ENTRIES * size_of::<LocalCluster>()
        == LOCAL_CLUSTER_ENTRIES * size_of::<SharedCluster>()
);
const _: () =
    assert!(size_of::<LocalBlock>() == LOCAL_CLUSTERS_PER_BLOCK * size_of::<LocalCluster>());
const _: () =
    assert!(size_of::<SharedBlock>() == SHARED_CLUSTERS_PER_BLOCK * size_of::<SharedCluster>());
const _: () = assert!(align_of::<LocalBlock>() == size_of::<LocalBlock>());
const _: () = assert!(align_of::<SharedBlock>() == size_of::<SharedBlock>());

struct SharedTable {
    blocks: Box<[SharedBlock]>,
    mask: usize,
    age: AtomicU8,
}

impl SharedTable {
    #[inline(always)]
    fn cluster_count(&self) -> usize {
        self.blocks.len() * SHARED_CLUSTERS_PER_BLOCK
    }

    #[inline(always)]
    fn cluster(&self, index: usize) -> &SharedCluster {
        &self.blocks[index / SHARED_CLUSTERS_PER_BLOCK].clusters[index % SHARED_CLUSTERS_PER_BLOCK]
    }
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
/// Grouped into a struct because the four store paths (`store`, the
/// local/shared backends, and `make_entry`) previously took the same nine
/// positional parameters — six of them `i32`/`usize`. A swapped `depth`/
/// `score` or `ply`/`static_eval` pair compiled silently; named fields make
/// that class of bug impossible. All fields are `Copy` scalars, so passing
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

    pub fn ensure_local(&mut self, mb: usize) -> bool {
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
    /// 9.4: takes the byte budget rather than inheriting the local cluster
    /// COUNT. `SharedCluster` is 64 B against `LocalCluster`'s 32 B, so
    /// reusing the count silently allocated **twice the `Hash` the user
    /// asked for** the moment a search went multi-threaded — invisible, and
    /// in a tournament it surfaces as swapping and time losses rather than
    /// as a memory bug. `Hash` is a contract; this keeps it.
    ///
    /// The local table is dropped BEFORE the shared one is allocated. It
    /// carries no entries across (the shared table has always started empty),
    /// so nothing is lost, and the old order peaked at local + shared held
    /// simultaneously — at `go` time, mid-game.
    pub fn make_shared(&mut self, mb: usize) {
        if matches!(self.storage, TtStorage::Shared(_)) {
            return;
        }
        let age = match &self.storage {
            TtStorage::Local(local) => local.age,
            TtStorage::Shared(_) => unreachable!("returned above"),
        };
        self.storage = TtStorage::Local(LocalTable {
            blocks: Vec::new(),
            mask: 0,
            age,
        });
        self.storage = TtStorage::Shared(Arc::new(new_shared_table(mb, age)));
    }

    /// Bytes actually handed to the allocator for the table itself.
    /// The 9.4 regression tests assert this against the `Hash` budget.
    pub fn allocated_bytes(&self) -> usize {
        match &self.storage {
            TtStorage::Local(table) => table.blocks.len() * size_of::<LocalBlock>(),
            TtStorage::Shared(table) => table.blocks.len() * size_of::<SharedBlock>(),
        }
    }

    /// Slots the table can hold. Bytes are the `Hash` contract, but ENTRIES are
    /// what the search actually spends: two backends can honour the same byte
    /// budget while one stores far fewer positions. Exposed so the sizing
    /// tests can assert the shared table does not silently shrink capacity
    /// when a search goes multi-threaded.
    pub fn capacity_entries(&self) -> usize {
        match &self.storage {
            TtStorage::Local(table) => table.cluster_count() * LOCAL_CLUSTER_ENTRIES,
            TtStorage::Shared(table) => table.cluster_count() * SHARED_CLUSTER_ENTRIES,
        }
    }

    pub fn clear(&mut self) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                let blocks = table.blocks.as_mut_slice();
                let num_threads =
                    std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
                let chunk_size = (blocks.len() / num_threads).max(1);
                std::thread::scope(|s| {
                    for chunk in blocks.chunks_mut(chunk_size) {
                        s.spawn(|| chunk.fill(LocalBlock::default()));
                    }
                });
                table.age = 0;
            }
            TtStorage::Shared(table) => {
                let blocks = table.blocks.as_ref();
                let num_threads =
                    std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
                let chunk_size = (blocks.len() / num_threads).max(1);
                std::thread::scope(|s| {
                    for chunk in blocks.chunks(chunk_size) {
                        s.spawn(move || {
                            for block in chunk {
                                for cluster in &block.clusters {
                                    for index in 0..SHARED_CLUSTER_ENTRIES {
                                        cluster.clear_slot(index);
                                    }
                                }
                            }
                        });
                    }
                });
                table.age.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn new_search(&mut self) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                table.age = table.age.wrapping_add(8) & 0xF8;
            }
            TtStorage::Shared(table) => {
                let age = table.age.load(Ordering::Relaxed);
                table
                    .age
                    .store(age.wrapping_add(8) & 0xF8, Ordering::Relaxed);
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
            TtStorage::Local(table) => {
                let ptr = table.cluster(infra::index(key) & table.mask);
                prefetch_ptr(ptr);
            }
            TtStorage::Shared(table) => {
                let ptr = table.cluster(infra::index(key) & table.mask);
                prefetch_ptr(ptr);
            }
        }
    }

    #[inline(always)]
    pub fn store(&mut self, e: TtStore) {
        match &mut self.storage {
            TtStorage::Local(table) => {
                store_local(table, e);
            }
            TtStorage::Shared(table) => {
                store_shared(table, e);
            }
        }
    }

    pub fn hashfull(&self) -> usize {
        match &self.storage {
            TtStorage::Local(table) => {
                let sample = table.cluster_count().min(334);
                if sample == 0 {
                    return 0;
                }
                let age = table.age;
                let used = (0..sample)
                    .flat_map(|index| table.cluster(index).entries)
                    .filter(|entry| current_entry(*entry, age))
                    .count();
                used * 1000 / (sample * LOCAL_CLUSTER_ENTRIES)
            }
            TtStorage::Shared(table) => {
                let sample = table.cluster_count().min(334);
                if sample == 0 {
                    return 0;
                }
                let age = table.age.load(Ordering::Relaxed);
                let used = (0..sample)
                    .flat_map(|index| {
                        let cluster = table.cluster(index);
                        (0..SHARED_CLUSTER_ENTRIES).filter_map(|index| cluster.load_any(index))
                    })
                    .filter(|(_, entry)| current_entry(*entry, age))
                    .count();
                used * 1000 / (sample * SHARED_CLUSTER_ENTRIES)
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

    // Match the x86 T0 hint on ARM64. Rarog deliberately issues this after
    // making a child move, leaving useful work between the hint and the child
    // TT probe. This used to compile to a no-op on every ARM64 release while
    // the x86 builds emitted `_mm_prefetch`, creating an avoidable ISA-specific
    // search-speed difference.
    //
    // SAFETY: `prfm` is an architectural cache hint and does not dereference
    // `ptr` as a Rust memory access. The pointer is nevertheless derived from
    // the live TT allocation above. The instruction does not modify memory or
    // flags and uses no stack storage.
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "prfm pldl1keep, [{address}]",
            address = in(reg) ptr,
            options(readonly, nostack, preserves_flags)
        );
    }

    // Rarog is 64-bit-only; this is retained as a correctness-first fallback
    // for any future non-x86_64/non-aarch64 target.
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = ptr;
    }
}

/// Upper 16 bits of the hash — the cluster-entry verification tag. The
/// truncation IS the design (a 16-bit tag), hence the scoped allow.
#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
fn key16_of(key: u64) -> u16 {
    (key >> 48) as u16
}

#[inline(always)]
fn current_entry(entry: TtEntry, age: u8) -> bool {
    entry.is_occupied() && (entry.flag_age & 0xF8) == age
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

fn new_local_table(mb: usize) -> Option<LocalTable> {
    let power = cluster_count::<LocalCluster>(mb);
    debug_assert_eq!(power % LOCAL_CLUSTERS_PER_BLOCK, 0);
    let block_count = power / LOCAL_CLUSTERS_PER_BLOCK;
    let mut blocks = Vec::new();
    blocks.try_reserve_exact(block_count).ok()?;
    blocks.resize(block_count, LocalBlock::default());
    LocalTable {
        blocks,
        mask: power - 1,
        age: 0,
    }
    .into()
}

fn new_shared_table(mb: usize, age: u8) -> SharedTable {
    // Sized from the byte budget with SharedCluster's own size — see
    // `make_shared` for why inheriting the local cluster count was wrong.
    let power = cluster_count::<SharedCluster>(mb);
    debug_assert_eq!(power % SHARED_CLUSTERS_PER_BLOCK, 0);
    let blocks = (0..power / SHARED_CLUSTERS_PER_BLOCK)
        .map(|_| SharedBlock::default())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    SharedTable {
        blocks,
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
    let entries = &table.cluster(crate::infra::index(key) & table.mask).entries;
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
    let cluster = table.cluster(crate::infra::index(key) & table.mask);
    (0..SHARED_CLUSTER_ENTRIES).find_map(|index| cluster.load(index, key16))
}

#[inline(always)]
fn store_local(table: &mut LocalTable, e: TtStore) {
    let key16 = key16_of(e.key);
    let age = table.age;
    let cluster = table.cluster_mut(crate::infra::index(e.key) & table.mask);

    let mut replace_index = 0usize;
    let mut replace_quality = i32::MAX;
    for index in 0..cluster.entries.len() {
        let entry = cluster.entries[index];
        if entry.key16 == key16 {
            replace_index = index;
            break;
        }
        let quality = entry_quality(entry, age);
        if quality < replace_quality {
            replace_quality = quality;
            replace_index = index;
        }
    }

    let replace = &mut cluster.entries[replace_index];
    // 9.7.5(b): does this store land on a slot that already held THIS position?
    // Instrumented on both backends so the 1T (local) and NT (shared) duplication
    // shares are comparable — a same_key share that climbs with thread count
    // means threads are re-deriving each other's work.
    if replace.key16 == key16 {
        crate::diag_count!(tt_store_same_key);
    } else {
        crate::diag_count!(tt_store_fresh);
    }
    if replace.key16 == key16
        && e.bound != Bound::Exact
        && e.depth < replace.depth as i32 - 3
        && (replace.flag_age & 0xF8) == age
    {
        return;
    }

    let stored_move = if e.mv.is_null() && replace.key16 == key16 {
        replace.mv
    } else {
        e.mv.0
    };

    *replace = make_entry(key16, stored_move, age, e);
}

#[inline(always)]
fn store_shared(table: &SharedTable, e: TtStore) {
    let age = table.age.load(Ordering::Relaxed);
    let key16 = key16_of(e.key);
    let cluster = table.cluster(crate::infra::index(e.key) & table.mask);

    let mut replace_index = 0usize;
    let mut replace_quality = i32::MAX;
    let mut replace_entry = TtEntry::default();
    // Whether the chosen slot already holds THIS position — verification is
    // now 16-bit, matching the local backend, so the same-position test is a
    // key16 comparison rather than a full-key one.
    let mut replace_hits_same_key = false;
    for index in 0..SHARED_CLUSTER_ENTRIES {
        let (entry_key16, entry) = cluster.load_any(index).unwrap_or_default();
        if entry_key16 == key16 && entry.bound().is_some() {
            replace_index = index;
            replace_entry = entry;
            replace_hits_same_key = true;
            break;
        }
        let quality = entry_quality(entry, age);
        if quality < replace_quality {
            replace_quality = quality;
            replace_index = index;
            replace_entry = entry;
        }
    }

    // 9.7.5(b): see the local backend for what this measures.
    if replace_hits_same_key {
        crate::diag_count!(tt_store_same_key);
    } else {
        crate::diag_count!(tt_store_fresh);
    }
    if replace_hits_same_key
        && e.bound != Bound::Exact
        && e.depth < replace_entry.depth as i32 - 3
        && (replace_entry.flag_age & 0xF8) == age
    {
        return;
    }

    let stored_move = if e.mv.is_null() && replace_hits_same_key {
        replace_entry.mv
    } else {
        e.mv.0
    };

    cluster.store(replace_index, key16, make_entry(key16, stored_move, age, e));
}

#[inline(always)]
fn make_entry(key16: u16, mv: u16, age: u8, e: TtStore) -> TtEntry {
    let TtStore {
        depth,
        score,
        bound,
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
        depth: crate::infra::saturating_i8(depth, -1),
        flag_age: age | bound as u8 | ((is_pv as u8) << 2),
    }
}

#[inline(always)]
fn entry_quality(entry: TtEntry, age: u8) -> i32 {
    if !entry.is_occupied() {
        return i32::MIN;
    }
    let age_delta = age.wrapping_sub(entry.flag_age & 0xF8) & 0xF8;
    entry.depth as i32 - age_delta as i32 / 2
}
