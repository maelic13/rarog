//! Lazy piece deltas keyed by perspective and exact king square. Arbitrary
//! position jumps and skipped evaluations need no make/unmake notifications.
use crate::{Error, LANES, Network, feature};

#[cfg(target_arch = "x86_64")]
mod x86;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kernel {
    Portable,
    Avx2,
    AvxVnni,
}

impl Kernel {
    pub fn available(self) -> bool {
        match self {
            Self::Portable => true,
            #[cfg(target_arch = "x86_64")]
            Self::Avx2 => std::is_x86_feature_detected!("avx2"),
            #[cfg(target_arch = "x86_64")]
            Self::AvxVnni => {
                std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("avxvnni")
            }
            #[cfg(not(target_arch = "x86_64"))]
            _ => false,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Portable => "portable",
            Self::Avx2 => "AVX2",
            Self::AvxVnni => "AVX-VNNI",
        }
    }
}

#[repr(align(64))]
#[derive(Clone)]
struct Entry {
    accumulator: [i16; LANES],
    pieces: [u64; 12],
    auxiliary: [i32; 8],
    valid: bool,
    material: i32,
    pawns: i32,
}

#[derive(Clone)]
struct Scratch {
    input: [u8; LANES],
    removed: [usize; 32],
    added: [usize; 32],
}

/// Owned by one evaluator/worker. Allocated at configuration, never at a node.
#[derive(Clone)]
pub struct AccumulatorCache {
    entries: Vec<Entry>,
    kernel: Kernel,
    scratch: Box<Scratch>,
}

impl Default for AccumulatorCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AccumulatorCache {
    pub fn new() -> Self {
        let kernel = [Kernel::AvxVnni, Kernel::Avx2, Kernel::Portable]
            .into_iter()
            .find(|k| k.available())
            .unwrap();
        Self::with_kernel(kernel).unwrap()
    }

    /// Explicit kernel selection supports conformance and attribution checks.
    /// Unsupported instructions can never be selected by a safe caller.
    pub fn with_kernel(kernel: Kernel) -> Option<Self> {
        kernel.available().then(|| Self {
            entries: (0..128)
                .map(|_| Entry {
                    accumulator: [0; LANES],
                    pieces: [0; 12],
                    auxiliary: [0; 8],
                    valid: false,
                    material: 0,
                    pawns: 0,
                })
                .collect(),
            kernel,
            scratch: Box::new(Scratch {
                input: [0; LANES],
                removed: [0; 32],
                added: [0; 32],
            }),
        })
    }

    pub fn kernel(&self) -> Kernel {
        self.kernel
    }

    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            entry.valid = false;
        }
    }

    /// Bitboards are White P/N/B/R/Q/K, then Black P/N/B/R/Q/K, a1=0.
    /// They must describe a legal engine position. All state is self-contained;
    /// null moves, captures, promotions and returning to ancestors use the same
    /// set difference. Auxiliary sums retain every head across material changes.
    pub fn evaluate_scaled(
        &mut self,
        network: &Network,
        pieces: &[u64; 12],
        black_to_move: bool,
        scale: i32,
    ) -> Result<i32, Error> {
        if !(10..=500).contains(&scale) {
            return Err(Error("NNUE_Scale must be in 10..=500".into()));
        }
        if !pieces[5].is_power_of_two() || !pieces[11].is_power_of_two() {
            return Err(Error("position needs one king per color".into()));
        }
        let side = usize::from(black_to_move);
        let (mut head, mut pawns) = (0, 0);
        let Scratch {
            input,
            removed,
            added,
        } = &mut *self.scratch;
        let mut auxiliary = [0_i32; 2];
        for (half, aux) in auxiliary.iter_mut().enumerate() {
            let perspective = side ^ half;
            let king = pieces[perspective * 6 + 5].trailing_zeros() as usize;
            let entry = &mut self.entries[perspective * 64 + king];
            if !entry.valid {
                entry.accumulator.copy_from_slice(&network.bias);
                entry.pieces.fill(0);
                entry.auxiliary.fill(0);
                entry.valid = true;
                entry.material = 0;
                entry.pawns = 0;
            }
            let (mut nr, mut na) = (0, 0);
            for (kind, (&old, &new)) in entry.pieces.iter().zip(pieces).enumerate() {
                if old == new {
                    continue;
                }
                let (old_nr, old_na) = (nr, na);
                let code = (kind % 6 + 1 + (kind / 6) * 8) as u8;
                let mut gone = old & !new;
                while gone != 0 {
                    removed[nr] = feature(king, perspective, code, gone.trailing_zeros() as usize);
                    nr += 1;
                    gone &= gone - 1;
                }
                let mut arrived = new & !old;
                while arrived != 0 {
                    added[na] = feature(king, perspective, code, arrived.trailing_zeros() as usize);
                    na += 1;
                    arrived &= arrived - 1;
                }
                let change = (na - old_na) as i32 - (nr - old_nr) as i32;
                entry.material += change * [0, 3, 4, 8, 18, 0, 0, 3, 4, 8, 18, 0][kind];
                entry.pawns += change * i32::from(kind == 0 || kind == 6);
            }
            if half == 0 {
                head = (entry.material.min(95) / 12) as usize;
                pawns = entry.pawns;
            } else {
                debug_assert_eq!(head, (entry.material.min(95) / 12) as usize);
                debug_assert_eq!(pawns, entry.pawns);
            }
            let removed = &removed[..nr];
            let added = &added[..na];
            for (&index, add) in removed
                .iter()
                .map(|i| (i, false))
                .chain(added.iter().map(|i| (i, true)))
            {
                for (j, sum) in entry.auxiliary.iter_mut().enumerate() {
                    let weight = network.auxiliary[index * 8 + j];
                    *sum = if add {
                        sum.wrapping_add(weight)
                    } else {
                        sum.wrapping_sub(weight)
                    };
                }
            }
            match self.kernel {
                Kernel::Portable => {
                    for &index in removed {
                        let row = &network.weights[index * LANES..(index + 1) * LANES];
                        for (sum, &w) in entry.accumulator.iter_mut().zip(row) {
                            *sum = sum.wrapping_sub(w);
                        }
                    }
                    for &index in added {
                        let row = &network.weights[index * LANES..(index + 1) * LANES];
                        for (sum, &w) in entry.accumulator.iter_mut().zip(row) {
                            *sum = sum.wrapping_add(w);
                        }
                    }
                    for (i, output) in input[half * 768..(half + 1) * 768].iter_mut().enumerate() {
                        let a = i32::from(entry.accumulator[i].clamp(0, 127));
                        let b = i32::from(entry.accumulator[i + 768].clamp(0, 127));
                        *output = ((a * b) >> 7) as u8;
                    }
                }
                #[cfg(target_arch = "x86_64")]
                _ => {
                    // SAFETY: Construction checks AVX2 support for both vector kernels.
                    unsafe {
                        x86::update_activate(
                            &mut entry.accumulator,
                            &network.weights,
                            removed,
                            added,
                            &mut input[half * 768..(half + 1) * 768],
                        );
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                _ => unreachable!("only portable kernels can be constructed on this target"),
            }
            entry.pieces.copy_from_slice(pieces);
            *aux = entry.auxiliary[head];
        }
        let selected = &network.heads[head];
        let score = match self.kernel {
            Kernel::Portable => selected.forward_portable(input),
            #[cfg(target_arch = "x86_64")]
            Kernel::Avx2 => {
                // SAFETY: AVX2 was checked at construction.
                unsafe { x86::forward_avx2(selected, input) }
            }
            #[cfg(target_arch = "x86_64")]
            Kernel::AvxVnni => {
                // SAFETY: Both AVX2 and AVX-VNNI were checked at construction.
                unsafe { x86::forward_vnni(selected, input) }
            }
            #[cfg(not(target_arch = "x86_64"))]
            _ => unreachable!(),
        };
        let raw = score.wrapping_add(auxiliary[0].wrapping_sub(auxiliary[1]) / 2) / 16;
        let factor = (pawns * 20 + 1024) * scale;
        Ok((raw.wrapping_mul(factor) / 102_400).clamp(-25_000, 25_000))
    }
}
