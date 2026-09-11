use std::mem::size_of;
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU16, AtomicU64, Ordering},
};

use crate::board::Move;
use crate::eval::MATE_SCORE;
use crate::evidence::{OutcomeKind, debug_assert_outcome};
use crate::infra;

const MAX_PLY: i32 = 128;
const BOUND_MASK: u8 = 0x03;
const PV_BIT: u8 = 0x04;
const SPECULATIVE_BIT: u8 = 0x08;
const AGE_MASK: u8 = 0xF0;
const AGE_STRIDE: u8 = 0x10;
const AGE_QUALITY_DIVISOR: i32 = 4;

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

    #[inline(always)]
    fn is_occupied(self) -> bool {
        self.flag_age & BOUND_MASK != 0
    }

    #[inline(always)]
    pub fn is_pv_node(self) -> bool {
        self.flag_age & PV_BIT != 0
    }

    /// Whether this entry came from a window-speculative producer such as
    /// ProbCut. This is orthogonal to bound, depth and move authority.
    #[inline(always)]
    pub fn is_speculative(self) -> bool {
        self.flag_age & SPECULATIVE_BIT != 0
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
/// per position the single-threaded table has always used, i.e. going
/// multi-threaded no longer costs capacity at all.
const SHARED_CLUSTER_ENTRIES: usize = 6;

#[repr(align(32))]
#[derive(Copy, Clone, Default)]
struct LocalCluster {
    entries: [TtEntry; LOCAL_CLUSTER_ENTRIES],
    _padding: [u8; 2],
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

// The shared cluster must be exactly 64 bytes, and must store positions at the
// same density as the local one. Both were violated silently before — asserting
// them at compile time is free.
//
// ⚠ 4.8c — "64 bytes" IS "one cache line" on x86-64 and is NOT on Apple
// Silicon. Measured on an M4 (RAR-P11): `hw.cachelinesize` is **128**, so two
// independent `SharedCluster`s share one line there and two threads touching
// unrelated TT entries can contend. Neither cluster type can ever STRADDLE a
// 128 B line — 32 and 64 both divide 128 and both are aligned to their own size
// — so the 128 B block wrapper this project twice considered was aimed at a
// hazard that cannot occur; this is the real one.
//
// Both halves are now settled, so this is a CLOSED question, not an open TODO.
// RAR-P12 measured the Threads>1 exposure and found none (3.89x at 4T against a
// pre-registered >=3.8x bar). RAR-P16 then measured the wrapper itself on an M4:
// -0.12% median, 4/12 paired wins, inside the noise floor — and showed why, by
// probing the allocator, which already returns 128 B-aligned TT bases at every
// Hash size, so the wrapper cannot move a single address. Do not reintroduce it
// without a Threads>1 ARM result that contradicts RAR-P12; RAR-P16 carries the
// recipe if one is ever needed. Naive padding to 128 B would additionally halve
// the density this second assert exists to hold.
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
    /// What produced `score`. The full kind drives the debug contract/census;
    /// 4.3c also persists its speculative/non-speculative class in one TT bit.
    /// Required, not defaulted: invariant 1 is that every result is typed, and
    /// an optional field would let the next store site skip it.
    pub kind: OutcomeKind,
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
            clusters: Vec::new(),
            mask: 0,
            age,
        });
        self.storage = TtStorage::Shared(Arc::new(new_shared_table(mb, age)));
    }

    /// Bytes actually handed to the allocator for the table itself.
    /// The 9.4 regression tests assert this against the `Hash` budget.
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
                let clusters = table.clusters.as_mut_slice();
                let num_threads =
                    std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
                let chunk_size = (clusters.len() / num_threads).max(1);
                std::thread::scope(|s| {
                    for chunk in clusters.chunks_mut(chunk_size) {
                        s.spawn(|| chunk.fill(LocalCluster::default()));
                    }
                });
                table.age = 0;
            }
            TtStorage::Shared(table) => {
                let clusters = table.clusters.as_ref();
                let num_threads =
                    std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
                let chunk_size = (clusters.len() / num_threads).max(1);
                std::thread::scope(|s| {
                    for chunk in clusters.chunks(chunk_size) {
                        s.spawn(move || {
                            for cluster in chunk {
                                for index in 0..SHARED_CLUSTER_ENTRIES {
                                    cluster.clear_slot(index);
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
            TtStorage::Local(table) => {
                let ptr = table
                    .clusters
                    .as_ptr()
                    .wrapping_add(infra::index(key) & table.mask);
                prefetch_ptr(ptr);
            }
            TtStorage::Shared(table) => {
                let ptr = table
                    .clusters
                    .as_ptr()
                    .wrapping_add(infra::index(key) & table.mask);
                prefetch_ptr(ptr);
            }
        }
    }

    #[inline(always)]
    pub fn store(&mut self, e: TtStore) {
        // 4.2 producer contract and census. Both compile out of a production
        // build: `debug_assert_outcome` under `debug_assertions`, the counter
        // under `--features diag`. Placed here rather than at the seven call
        // sites so a new store path cannot bypass either.
        debug_assert_outcome(e.kind, e.depth, e.bound, e.mv);
        count_store_kind(e.kind);
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
                let sample = table.clusters.len().min(334);
                if sample == 0 {
                    return 0;
                }
                let age = table.age;
                let used = table
                    .clusters
                    .iter()
                    .take(sample)
                    .flat_map(|cluster| cluster.entries)
                    .filter(|entry| current_entry(*entry, age))
                    .count();
                used * 1000 / (sample * LOCAL_CLUSTER_ENTRIES)
            }
            TtStorage::Shared(table) => {
                let sample = table.clusters.len().min(334);
                if sample == 0 {
                    return 0;
                }
                let age = table.age.load(Ordering::Relaxed);
                let used = table
                    .clusters
                    .iter()
                    .take(sample)
                    .flat_map(|cluster| {
                        (0..SHARED_CLUSTER_ENTRIES).filter_map(|index| cluster.load_any(index))
                    })
                    .filter(|(_, entry)| current_entry(*entry, age))
                    .count();
                used * 1000 / (sample * SHARED_CLUSTER_ENTRIES)
            }
        }
    }
}

/// 4.3 hazard census: a moveless store INHERITS the resident move.
///
/// This is why a persisted producer class is not the only thing missing — for a
/// `StandPat` store the inheritance means a purely static estimate walks away
/// carrying a searched move, and the resulting entry (depth 0, `Lower`, with a
/// move) is byte-identical to a searched `QsearchMove`. Any attempt to infer
/// "stand pat" from "depth 0 + Lower + no move" is therefore only as sound as
/// this counter is small, which is exactly why it is measured before 4.3
/// designs around the inference.
#[inline(always)]
fn count_move_inheritance(kind: OutcomeKind, resident: u16) {
    #[cfg(feature = "diag")]
    if resident != 0 {
        crate::diag_count!(tt_move_inherited);
        if kind == OutcomeKind::StandPat {
            crate::diag_count!(tt_move_inherited_stand_pat);
        }
    }
    #[cfg(not(feature = "diag"))]
    {
        let _ = (kind, resident);
    }
}

/// 4.3 hazard census: a depth-0 horizon store landing on a deeper searched entry
/// for the SAME position. The depth-preservation rule above only protects an
/// entry more than 3 plies deeper, so depths 1..3 are overwritten by a horizon
/// estimate. Counted to size that evidence loss before 4.3 tightens anything.
///
/// Also records the COMMITTED-store denominators. Both call sites sit after the
/// depth-preservation `return`, so everything counted here actually landed —
/// which is what makes the published hazard rates exact rather than biased low.
#[inline(always)]
fn count_horizon_overwrite(kind: OutcomeKind, depth: i32, same_key: bool, resident_depth: i8) {
    #[cfg(feature = "diag")]
    {
        match kind {
            OutcomeKind::StandPat => crate::diag_count!(store_committed_stand_pat),
            OutcomeKind::QsearchMove => crate::diag_count!(store_committed_qsearch_move),
            _ => {}
        }
        if kind.is_horizon() {
            crate::diag_count!(store_committed_horizon);
            if same_key && i32::from(resident_depth) > depth {
                crate::diag_count!(tt_horizon_overwrote_searched);
            }
        }
    }
    #[cfg(not(feature = "diag"))]
    {
        let _ = (kind, depth, same_key, resident_depth);
    }
}

/// 4.3: a store the depth-preservation rule threw away. Counted at the `return`
/// so `attempted - skipped == committed` holds on both backends, which is the
/// arithmetic the tool asserts.
#[inline(always)]
fn count_skipped_store() {
    crate::diag_count!(store_skipped_depth_rule);
}

/// Exact per-producer store census. Expands to nothing without `--features
/// diag`; the `match` and the unused binding both disappear.
#[inline(always)]
fn count_store_kind(kind: OutcomeKind) {
    #[cfg(feature = "diag")]
    match kind {
        OutcomeKind::Full => crate::diag_count!(store_kind_full),
        OutcomeKind::VerifiedReduced => crate::diag_count!(store_kind_verified_reduced),
        OutcomeKind::QsearchMove => crate::diag_count!(store_kind_qsearch_move),
        OutcomeKind::QsearchTail => crate::diag_count!(store_kind_qsearch_tail),
        OutcomeKind::StandPat => crate::diag_count!(store_kind_stand_pat),
        OutcomeKind::ProbCut => crate::diag_count!(store_kind_probcut),
        OutcomeKind::Tablebase => crate::diag_count!(store_kind_tablebase),
        // `debug_assert_outcome` already rejects these; a release diag build
        // must still not miscount them into a neighbouring bucket.
        OutcomeKind::Null | OutcomeKind::Incomplete => {}
    }
    #[cfg(not(feature = "diag"))]
    {
        let _ = kind;
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

    // 4.8b — the ARM64 spelling of the SAME hint. Until now this function had
    // an x86 body and, for every other target, `let _ = ptr;` — so all three
    // shipped ARM64 assets did NO TT prefetching at all while the x86 assets
    // did, an ISA-specific search-speed difference nobody chose. Rarog issues
    // the hint after making the child move, so there is useful work between it
    // and the child's TT probe, which is what makes a prefetch worth issuing.
    //
    // `pldl1keep` is the ARM analogue of `_MM_HINT_T0`: prefetch for load, into
    // L1, temporal (keep). Written as inline `asm!` because
    // `core::arch::aarch64::_prefetch` is still unstable, and stable AArch64
    // inline assembly is exactly what PLAN 4.8 pins as available.
    //
    // ⚠ This does NOT raise the unsafe floor that principle #8 froze at 19. The
    // two arms are `cfg`-exclusive, so any single compiled target contains the
    // same number of unsafe blocks as before — on ARM64 this replaces a no-op
    // with the same intrinsic-class cache hint x86 already had, rather than
    // adding a new mechanism.
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
fn store_local(table: &mut LocalTable, e: TtStore) {
    let key16 = key16_of(e.key);
    let cluster = &mut table.clusters[crate::infra::index(e.key) & table.mask];

    let mut replace_index = 0usize;
    let mut replace_quality = i32::MAX;
    for index in 0..cluster.entries.len() {
        let entry = cluster.entries[index];
        if entry.key16 == key16 {
            replace_index = index;
            break;
        }
        let quality = entry_quality(entry, table.age);
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
        && (replace.flag_age & AGE_MASK) == table.age
    {
        count_skipped_store();
        return;
    }

    let stored_move = if e.mv.is_null() && replace.key16 == key16 {
        count_move_inheritance(e.kind, replace.mv);
        replace.mv
    } else {
        e.mv.0
    };
    count_horizon_overwrite(e.kind, e.depth, replace.key16 == key16, replace.depth);

    *replace = make_entry(key16, stored_move, table.age, e);
}

#[inline(always)]
fn store_shared(table: &SharedTable, e: TtStore) {
    let age = table.age.load(Ordering::Relaxed);
    let key16 = key16_of(e.key);
    let cluster = &table.clusters[crate::infra::index(e.key) & table.mask];

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
        && (replace_entry.flag_age & AGE_MASK) == age
    {
        count_skipped_store();
        return;
    }

    let stored_move = if e.mv.is_null() && replace_hits_same_key {
        count_move_inheritance(e.kind, replace_entry.mv);
        replace_entry.mv
    } else {
        e.mv.0
    };
    count_horizon_overwrite(e.kind, e.depth, replace_hits_same_key, replace_entry.depth);

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
        kind,
        ..
    } = e;
    TtEntry {
        key16,
        score: crate::infra::saturating_i16(score_to_tt(score, ply)),
        static_eval: crate::infra::saturating_i16(static_eval),
        mv,
        depth: crate::infra::saturating_i8(depth, -1),
        flag_age: age
            | bound as u8
            | if is_pv { PV_BIT } else { 0 }
            | if kind.is_speculative() {
                SPECULATIVE_BIT
            } else {
                0
            },
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
    use super::{
        AGE_MASK, AGE_QUALITY_DIVISOR, AGE_STRIDE, Bound, LocalTable, PV_BIT, SPECULATIVE_BIT,
        TranspositionTable, TtEntry, TtStorage, entry_quality,
    };

    #[test]
    fn four_bit_age_preserves_the_per_generation_replacement_penalty() {
        let entry = TtEntry {
            depth: 20,
            flag_age: Bound::Exact as u8 | PV_BIT | SPECULATIVE_BIT,
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
}
