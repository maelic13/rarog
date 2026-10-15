//! Scalar inference recovered from the bundled Dragon executable.
//! See docs/contracts/NNUE.md for original addresses and arithmetic contracts.
//! The full scalar API remains the reference; cached.rs owns host optimizations.

mod cached;
mod sha256;
pub use cached::{AccumulatorCache, Kernel};

use std::{fmt, fs, path::Path};

pub const MODEL_SHA256: &str = "c6e25a7fde5971dc47e78e19764c82b8d18a438bbd5ee4da065d061115064417";
pub const MODEL_SIZE: usize = 70_136_671;
const FEATURES: usize = 22_528;
const LANES: usize = 1536;
const HALF: usize = LANES / 2;

#[derive(Debug)]
pub struct Error(String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for Error {}

/// Squares a1=0 .. h8=63. Pieces: empty=0, white P/N/B/R/Q/K=1..6,
/// black=9..14. Castling, en-passant and clocks are not network inputs.
#[derive(Clone, Debug)]
pub struct Position {
    squares: [u8; 64],
    side: usize,
    kings: [usize; 2],
    pawns: i32,
    head: usize,
}

impl Position {
    pub fn new(squares: [u8; 64], black_to_move: bool) -> Result<Self, Error> {
        let mut kings = [64; 2];
        let mut count = 0;
        let mut pawns = 0;
        let mut material = 0;
        for (sq, &pc) in squares.iter().enumerate() {
            if pc == 0 {
                continue;
            }
            let kind = pc & 7;
            if !(1..=6).contains(&kind) || pc > 14 {
                return Err(Error(format!("invalid piece {pc} at square {sq}")));
            }
            count += 1;
            pawns += i32::from(kind == 1);
            material += [0, 0, 3, 4, 8, 18, 0][usize::from(kind)];
            if kind == 6 {
                let king = &mut kings[usize::from(pc >> 3)];
                if *king != 64 {
                    return Err(Error("multiple kings of one color".into()));
                }
                *king = sq;
            }
        }
        if kings.contains(&64) || count > 32 {
            return Err(Error(
                "position needs one king per color and at most 32 pieces".into(),
            ));
        }
        Ok(Self {
            squares,
            side: usize::from(black_to_move),
            kings,
            pawns,
            head: (material.min(95) / 12) as usize,
        })
    }

    /// Parses the piece placement and active color of a standard FEN. Remaining
    /// FEN fields may be supplied; they cannot affect network inference.
    pub fn from_fen(fen: &str) -> Result<Self, Error> {
        let mut fields = fen.split_whitespace();
        let board = fields
            .next()
            .ok_or_else(|| Error("missing FEN board".into()))?;
        let black = match fields.next() {
            Some("w") => false,
            Some("b") => true,
            _ => return Err(Error("FEN active color must be w or b".into())),
        };
        let mut squares = [0; 64];
        let ranks: Vec<_> = board.split('/').collect();
        if ranks.len() != 8 {
            return Err(Error("FEN needs eight ranks".into()));
        }
        for (rank, row) in ranks.iter().enumerate() {
            let mut file = 0;
            for c in row.chars() {
                if ('1'..='8').contains(&c) {
                    file += c.to_digit(10).unwrap() as usize;
                } else {
                    let kind = match c.to_ascii_lowercase() {
                        'p' => 1,
                        'n' => 2,
                        'b' => 3,
                        'r' => 4,
                        'q' => 5,
                        'k' => 6,
                        _ => return Err(Error(format!("invalid FEN piece {c}"))),
                    };
                    if file >= 8 {
                        return Err(Error("FEN rank overflow".into()));
                    }
                    squares[(7 - rank) * 8 + file] =
                        kind + if c.is_ascii_lowercase() { 8 } else { 0 };
                    file += 1;
                }
                if file > 8 {
                    return Err(Error("FEN rank overflow".into()));
                }
            }
            if file != 8 {
                return Err(Error("FEN rank must contain eight squares".into()));
            }
        }
        Self::new(squares, black)
    }

    pub fn head(&self) -> usize {
        self.head
    }
}

/// base:140023e00; tables at 14008c300 and 14008c400.
fn feature(king: usize, perspective: usize, piece: u8, square: usize) -> usize {
    let flip = if perspective == 1 { 56 } else { 0 } ^ if king & 4 == 0 { 7 } else { 0 };
    let oriented_king = king ^ flip;
    let bucket = (7 - oriented_king / 8) * 4 + (7 - oriented_king % 8);
    let kind = usize::from(piece & 7);
    let offset = if kind == 6 {
        640
    } else {
        (kind - 1) * 128 + usize::from(usize::from(piece >> 3) != perspective) * 64
    };
    bucket * 704 + offset + (square ^ flip)
}

// Vec keeps ownership; initialized padding permits a cache-line-aligned view
// without an unsafe allocator or a per-evaluation copy.
struct DenseWeights {
    storage: Vec<i8>,
    offset: usize,
}

impl DenseWeights {
    fn from_fn(len: usize, mut value: impl FnMut(usize) -> i8) -> Self {
        let mut storage = vec![0; len + 63];
        let offset = storage.as_ptr().align_offset(64);
        for (i, slot) in storage[offset..offset + len].iter_mut().enumerate() {
            *slot = value(i);
        }
        Self { storage, offset }
    }
}

impl std::ops::Deref for DenseWeights {
    type Target = [i8];
    fn deref(&self) -> &Self::Target {
        let len = self.storage.len() - 63;
        &self.storage[self.offset..self.offset + len]
    }
}

#[cfg(target_arch = "x86_64")]
#[repr(align(64))]
struct AlignedSecond([[f32; 32]; 30]);

#[cfg(target_arch = "x86_64")]
impl std::ops::Deref for AlignedSecond {
    type Target = [[f32; 32]; 30];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct Head {
    bias1: [i32; 16],
    weights1: DenseWeights,
    bias2: [f32; 32],
    weights2: [[f32; 30]; 32],
    #[cfg(target_arch = "x86_64")]
    weights2_transposed: AlignedSecond,
    bias_out: f32,
    weights_out: [f32; 32],
}

pub struct Network {
    bias: [i16; LANES],
    weights: Vec<i16>,
    auxiliary: Vec<i32>,
    heads: Vec<Head>,
}

impl Network {
    /// Load exactly the frozen extracted network. The full hash is checked
    /// before allocating decoded tensors or accepting any inference request.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let bytes = fs::read(path).map_err(|e| Error(e.to_string()))?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != MODEL_SIZE {
            return Err(Error(format!(
                "model size {}, expected {MODEL_SIZE}",
                bytes.len()
            )));
        }
        if sha256::hex(bytes) != MODEL_SHA256 {
            return Err(Error("model SHA-256 mismatch".into()));
        }
        let mut r = Reader { bytes, offset: 0 };
        assert_eq!(r.i32() as u32, 0x7af32f21);
        assert_eq!(r.i32() as u32, 0x1c102b72);
        assert_eq!(r.i32(), 15);
        assert_eq!(r.take(15), b"Dragon network.");
        assert_eq!(r.i32(), 2_133_016_760);
        let bias = std::array::from_fn(|_| r.i16());
        let weights = (0..FEATURES * LANES).map(|_| r.i16()).collect();
        let auxiliary = (0..FEATURES * 8).map(|_| r.i32()).collect();
        let mut heads = Vec::with_capacity(8);
        for _ in 0..8 {
            r.i32(); // Identical block tags are covered by the full model hash.
            let bias1 = std::array::from_fn(|_| r.i32());
            let bytes = r.take(16 * LANES);
            let weights1 = DenseWeights::from_fn(bytes.len(), |i| bytes[i] as i8);
            let bias2 = std::array::from_fn(|_| r.i32() as f32);
            let weights2 = std::array::from_fn(|_| {
                let row = std::array::from_fn(|_| r.take(1)[0] as i8 as f32);
                r.take(2);
                row
            });
            let bias_out = r.i32() as f32;
            let weights_out = std::array::from_fn(|_| r.take(1)[0] as i8 as f32);
            heads.push(Head {
                bias1,
                weights1,
                bias2,
                #[cfg(target_arch = "x86_64")]
                weights2_transposed: AlignedSecond(std::array::from_fn(|i| {
                    std::array::from_fn(|j| weights2[j][i])
                })),
                weights2,
                bias_out,
                weights_out,
            });
        }
        assert_eq!(r.offset, MODEL_SIZE);
        Ok(Self {
            bias,
            weights,
            auxiliary,
            heads,
        })
    }

    /// Full refresh scalar reference, with no allocation in the evaluation path.
    /// Returns the unscaled score from the side to move's perspective.
    pub fn evaluate(&self, position: &Position) -> i32 {
        let mut input = [0_u8; LANES];
        let mut psqt = [0_i32; 2];
        for (half, aux) in psqt.iter_mut().enumerate() {
            let perspective = position.side ^ half;
            let mut accumulator = self.bias;
            for (square, &pc) in position.squares.iter().enumerate() {
                if pc == 0 {
                    continue;
                }
                let index = feature(position.kings[perspective], perspective, pc, square);
                let row = &self.weights[index * LANES..(index + 1) * LANES];
                for (sum, &weight) in accumulator.iter_mut().zip(row) {
                    *sum = sum.wrapping_add(weight);
                }
                *aux = aux.wrapping_add(self.auxiliary[index * 8 + position.head]);
            }
            for i in 0..HALF {
                let a = i32::from(accumulator[i].clamp(0, 127));
                let b = i32::from(accumulator[i + HALF].clamp(0, 127));
                input[half * HALF + i] = ((a * b) >> 7).min(127) as u8;
            }
        }
        let head_score = self.heads[position.head].forward(&input);
        head_score.wrapping_add(psqt[0].wrapping_sub(psqt[1]) / 2) / 16
    }

    /// Pawn-count / NNUE_Scale transformation used by Dragon's diagnostic.
    /// Still side-to-move relative, suitable for a host engine adapter. No HCE,
    /// tempo, rule-50, draw scaling or Dragon search corrections are added.
    pub fn evaluate_scaled(&self, position: &Position, scale: i32) -> Result<i32, Error> {
        if !(10..=500).contains(&scale) {
            return Err(Error("NNUE_Scale must be in 10..=500".into()));
        }
        let factor = (position.pawns * 20 + 1024) * scale;
        Ok((self.evaluate(position).wrapping_mul(factor) / 102_400).clamp(-25_000, 25_000))
    }

    /// Matches the native `nnue` command: a White-perspective diagnostic.
    pub fn diagnostic(&self, position: &Position, scale: i32) -> Result<i32, Error> {
        let score = self.evaluate_scaled(position, scale)?;
        Ok(if position.side == 1 { -score } else { score })
    }
}

impl Head {
    fn forward(&self, input: &[u8; LANES]) -> i32 {
        let mut first = [0_f32; 16];
        for (row, value) in first.iter_mut().enumerate() {
            let mut sum = self.bias1[row];
            for (a, w) in input.as_chunks::<2>().0.iter().zip(
                self.weights1[row * LANES..(row + 1) * LANES]
                    .as_chunks::<2>()
                    .0,
            ) {
                let pair = i32::from(a[0]) * i32::from(w[0]) + i32::from(a[1]) * i32::from(w[1]);
                sum = sum.wrapping_add(pair.clamp(i32::from(i16::MIN), i32::from(i16::MAX)));
            }
            *value = sum as f32;
        }
        self.finish_scalar(&first)
    }

    fn forward_portable(&self, input: &[u8; LANES]) -> i32 {
        let mut first = [0_f32; 16];
        for (row, value) in first.iter_mut().enumerate() {
            let weights = &self.weights1[row * LANES..(row + 1) * LANES];
            // SCReLU is at most 126, so pair saturation is inactive. A flat
            // wrapping dot lets LLVM vectorize without changing the reference.
            let sum = input
                .iter()
                .zip(weights)
                .fold(self.bias1[row], |sum, (&a, &w)| {
                    sum.wrapping_add(i32::from(a) * i32::from(w))
                });
            *value = sum as f32;
        }
        self.finish_scalar(&first)
    }

    fn finish_scalar(&self, first: &[f32; 16]) -> i32 {
        let skip = (first[15] as i32).wrapping_mul(9600) / 8128;
        let mut activated = [0_f32; 30];
        for i in 0..15 {
            activated[i] = (first[i] * first[i] * (1.0 / 524_288.0)).min(127.0);
            activated[i + 15] = (first[i].max(0.0) * (1.0 / 64.0)).min(127.0);
        }
        let mut second = [0_f32; 32];
        for (row, value) in second.iter_mut().enumerate() {
            let w = &self.weights2[row];
            let mut lanes = [0_f32; 4];
            for i in (0..28).step_by(4) {
                for lane in 0..4 {
                    lanes[lane] += w[i + lane] * activated[i + lane];
                }
            }
            let reduced = (lanes[0] + lanes[2]) + (lanes[1] + lanes[3]);
            let tail = w[28] * activated[28] + w[29] * activated[29];
            let dot = tail + reduced;
            *value = ((dot + self.bias2[row]).max(0.0) * (1.0 / 64.0)).min(127.0);
        }
        self.output(skip, &second)
    }

    fn output(&self, skip: i32, second: &[f32; 32]) -> i32 {
        let mut lanes = [0_f32; 4];
        for (lane, value) in lanes.iter_mut().enumerate() {
            let p = |group: usize| self.weights_out[group * 4 + lane] * second[group * 4 + lane];
            *value = (((p(2) + p(3)) + p(1)) + ((p(4) + p(5)) + (p(6) + p(7)))) + p(0);
        }
        let dot = (lanes[1] + lanes[3]) + (lanes[0] + lanes[2]);
        (skip as f32 + (dot + self.bias_out)) as i32
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let s = &self.bytes[self.offset..self.offset + n];
        self.offset += n;
        s
    }
    fn i16(&mut self) -> i16 {
        i16::from_le_bytes(self.take(2).try_into().unwrap())
    }
    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }
}

#[cfg(test)]
mod storage_tests {
    use super::*;

    #[test]
    fn dense_storage_preserves_every_byte_and_cache_line_alignment() {
        for len in [0, 1, 31, 32, 63, 64, 1536, 16 * LANES] {
            let expected: Vec<i8> = (0..len).map(|i| (i * 137) as i8).collect();
            let stored = DenseWeights::from_fn(len, |i| expected[i]);
            assert_eq!(&*stored, &expected);
            assert_eq!(stored.as_ptr() as usize % 64, 0);
        }
        #[cfg(target_arch = "x86_64")]
        {
            let matrices: Vec<_> = (0..8).map(|_| AlignedSecond([[0.0; 32]; 30])).collect();
            for matrix in &matrices {
                for row in &matrix.0 {
                    assert_eq!(row.as_ptr() as usize % 64, 0);
                }
            }
        }
    }
}
