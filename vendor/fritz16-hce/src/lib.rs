//! Fritz 16 static evaluation recovered from the supplied executable.
//!
//! The evaluator owns its state and cache. Construction allocates; inference
//! uses only existing storage. See docs/contracts/HCE.md for qualification scope.

mod recovered;

const BASE: u64 = 0x140000000;
const PAWNS: u64 = BASE + 0x600000;
const SCRATCH: u64 = BASE + 0x3c1520;
const STACK: u64 = BASE + 0x5ff000;
pub const ORIGINAL_SHA256: &str =
    "264f2eede4c737f938c726e3c91e5ce07460e998110cf197abfc463dfa4622ae";

#[derive(Clone)]
struct Memory {
    bytes: Box<[u8]>,
}

impl Memory {
    fn new() -> Self {
        let mut bytes = vec![0; 0x700000].into_boxed_slice();
        let rdata = include_bytes!("../assets/rdata.bin");
        let data = include_bytes!("../assets/data.bin");
        bytes[0xed000..0xed000 + rdata.len()].copy_from_slice(rdata);
        bytes[0x299000..0x299000 + data.len()].copy_from_slice(data);
        // Startup initializer 0x140001040: 48 iterations of 128-byte copies.
        bytes.copy_within(0x2923b0..0x293bb0, 0x4721b0);
        let mut this = Self { bytes };
        this.write::<64>(BASE + 0x355520, PAWNS);
        this
    }

    #[inline(always)]
    fn read<const W: usize>(&self, address: u64) -> u64 {
        let at = usize::try_from(address.wrapping_sub(BASE)).expect("address width");
        match W {
            8 => u64::from(self.bytes[at]),
            16 => u64::from(u16::from_le_bytes(
                self.bytes[at..at + 2].try_into().unwrap(),
            )),
            32 => u64::from(u32::from_le_bytes(
                self.bytes[at..at + 4].try_into().unwrap(),
            )),
            64 => u64::from_le_bytes(self.bytes[at..at + 8].try_into().unwrap()),
            _ => unreachable!("unsupported integer width"),
        }
    }

    #[inline(always)]
    fn write<const W: usize>(&mut self, address: u64, value: u64) {
        let at = usize::try_from(address.wrapping_sub(BASE)).expect("address width");
        self.bytes[at..at + W / 8].copy_from_slice(&value.to_le_bytes()[..W / 8]);
    }

    #[inline(always)]
    fn read128(&self, address: u64) -> u128 {
        let at = usize::try_from(address.wrapping_sub(BASE)).expect("address width");
        u128::from_le_bytes(self.bytes[at..at + 16].try_into().unwrap())
    }

    #[inline(always)]
    fn write128(&mut self, address: u64, value: u128) {
        let at = usize::try_from(address.wrapping_sub(BASE)).expect("address width");
        self.bytes[at..at + 16].copy_from_slice(&value.to_le_bytes());
    }
}

#[derive(Default)]
struct Flags {
    z: bool,
    s: bool,
    o: bool,
}

impl Flags {
    #[inline(always)]
    fn logic(&mut self, value: u64, width: u32) {
        self.z = value == 0;
        self.s = value & (1 << (width - 1)) != 0;
        self.o = false;
    }

    #[inline(always)]
    fn arithmetic(&mut self, a: u64, b: u64, width: u32, subtract: bool) -> u64 {
        let mask = u64::MAX >> (64 - width);
        let result = if subtract {
            a.wrapping_sub(b)
        } else {
            a.wrapping_add(b)
        } & mask;
        self.z = result == 0;
        self.s = result & (1 << (width - 1)) != 0;
        let sign = 1u64 << (width - 1);
        self.o = if subtract {
            (a ^ b) & (a ^ result) & sign != 0
        } else {
            !(a ^ b) & (a ^ result) & sign != 0
        };
        result
    }
}

/// Piece bitboards in white P,N,B,R,Q,K then black P,N,B,R,Q,K order.
/// Squares a1=0..h8=63; castling bits are white K/Q, black K/Q in bits 0..3.
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub pieces: [u64; 12],
    pub black_to_move: bool,
    pub castling: u8,
}

impl Position {
    /// Parse placement, side and castling. En-passant and clocks are not HCE inputs.
    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let mut fields = fen.split_whitespace();
        let placement = fields.next().ok_or("missing placement")?;
        let black_to_move = match fields.next() {
            Some("w") => false,
            Some("b") => true,
            _ => return Err("invalid side"),
        };
        let rights = fields.next().ok_or("missing castling rights")?;
        let mut castling = 0;
        for c in rights.chars() {
            castling |= match c {
                'K' => 1,
                'Q' => 2,
                'k' => 4,
                'q' => 8,
                '-' => 0,
                _ => return Err("invalid castling rights"),
            };
        }
        let mut pieces = [0u64; 12];
        let mut rank = 7u32;
        let mut file = 0u32;
        for c in placement.chars() {
            if c == '/' {
                if file != 8 || rank == 0 {
                    return Err("invalid rank");
                }
                rank -= 1;
                file = 0;
            } else if let Some(n) = c.to_digit(10) {
                if !(1..=8).contains(&n) {
                    return Err("invalid empty-square count");
                }
                file += n;
            } else {
                if file >= 8 {
                    return Err("rank overflow");
                }
                let index = "PNBRQKpnbrqk".find(c).ok_or("invalid piece")?;
                pieces[index] |= 1u64 << (rank * 8 + file);
                file += 1;
            }
            if file > 8 {
                return Err("rank overflow");
            }
        }
        if rank != 0 || file != 8 {
            return Err("incomplete board");
        }
        let position = Self {
            pieces,
            black_to_move,
            castling,
        };
        position.validate()?;
        Ok(position)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        let mut occupied = 0;
        for bb in self.pieces {
            if occupied & bb != 0 {
                return Err("overlapping pieces");
            }
            occupied |= bb;
        }
        if self.pieces[5].count_ones() != 1 || self.pieces[11].count_ones() != 1 {
            return Err("one king per side required");
        }
        if (self.pieces[0] | self.pieces[6]) & 0xff000000000000ff != 0 {
            return Err("pawns on promotion ranks");
        }
        if self.castling > 15 {
            return Err("invalid castling bits");
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Evaluator {
    memory: Memory,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            memory: Memory::new(),
        }
    }
}

impl Evaluator {
    pub fn clear(&mut self) {
        self.memory.bytes[0x600000..0x700000].fill(0);
    }

    /// Full evaluator result in its original units, from White's perspective.
    /// Requires disjoint bitboards, one king per side and no last-rank pawns.
    pub fn evaluate_white(&mut self, position: &Position) -> i32 {
        self.prepare(position);
        let mut registers = [0; 16];
        registers[1] = SCRATCH;
        registers[4] = STACK;
        recovered::eval_14009e270(&mut self.memory, &mut registers, &mut [0; 2]) as i32
    }

    pub fn evaluate(&mut self, position: &Position) -> i32 {
        let white = self.evaluate_white(position);
        if position.black_to_move {
            white.wrapping_neg()
        } else {
            white
        }
    }

    fn prepare(&mut self, position: &Position) {
        let m = &mut self.memory;
        m.bytes[0x3c0e00..0x3c1000].fill(0);
        let mut occupied = [0; 2];
        let mut signature = 0x0200000001000000u64;
        let signature_weights = [
            0x01000000000205b2,
            0x00fffffffffe3342,
            0x0100010008060002,
            0x0100040007fa0006,
            0,
            0,
            0x01001000300a00a2,
            0x010040002ff601e6,
            0x0100004070140001,
            0x010000806fec0000,
            0,
            0,
        ];
        let mut pst = 0x80008000u64;
        let mut adjustment = 0i32;
        let mut key =
            m.read::<64>(BASE + 0x1ef090 + u64::from(position.castling) * 8) ^ 0x27c06528b11258a8;
        for (color, color_occupied) in occupied.iter_mut().enumerate() {
            for kind in 0..6 {
                let bb = position.pieces[color * 6 + kind];
                let internal = kind * 2 + color;
                m.write::<64>(BASE + 0x3c1110 + internal as u64 * 8, bb);
                *color_occupied |= bb;
                signature = signature.wrapping_add(
                    u64::from(bb.count_ones()).wrapping_mul(signature_weights[internal]),
                );
                let value = [30, 90, 90, 150, 300, 0][kind];
                adjustment += (if color == 0 { -value } else { value }) * bb.count_ones() as i32;
                let mut remaining = bb;
                while remaining != 0 {
                    let sq = u64::from(remaining.trailing_zeros());
                    remaining &= remaining - 1;
                    let encoded = m.read::<64>(BASE + 0x2fec50 + (internal as u64 + 2) * 8);
                    m.write::<64>(BASE + 0x3c0e00 + sq * 8, encoded);
                    pst = pst.wrapping_add(
                        m.read::<64>(BASE + 0x4721b0 + (internal as u64 * 64 + sq) * 8),
                    );
                    if kind == 0 || kind == 5 {
                        key ^= m.read::<64>(BASE + 0x1f1110 + (internal as u64 * 64 + sq) * 8);
                    }
                    if kind == 5 {
                        m.write::<32>(BASE + 0x3c1170 + color as u64 * 4, sq);
                    }
                    if kind == 2 {
                        let on_a1_color = (1u64 << sq) & 0xaa55aa55aa55aa55 != 0;
                        signature = signature.wrapping_add(if color == 0 {
                            if on_a1_color {
                                0x0101000810060012
                            } else {
                                0x0101000410060012
                            }
                        } else if on_a1_color {
                            0x010400200ffa0036
                        } else {
                            0x010400100ffa0036
                        });
                        key ^= match (color, on_a1_color) {
                            (0, true) => 0x1cdd2c1990d18b82,
                            (0, false) => 0x5b5be68669f3efb1,
                            (1, true) => 0x88971596b48623e9,
                            (1, false) => 0x6ee19f4ae7391a20,
                            _ => unreachable!(),
                        };
                    }
                }
            }
        }
        for (bit, value) in [
            (1, 0x640000i64),
            (2, 0x140000),
            (4, -0x640000),
            (8, -0x140000),
        ] {
            if position.castling & bit != 0 {
                pst = pst.wrapping_add(value as u64);
            }
        }
        m.write::<64>(BASE + 0x3c1100, occupied[0]);
        m.write::<64>(BASE + 0x3c1108, occupied[1]);
        m.write::<64>(BASE + 0x3c1178, occupied[0] | occupied[1]);
        m.write::<32>(BASE + 0x3c1180, u64::from(position.castling));
        m.write::<64>(BASE + 0x3c1198, pst);
        m.write::<64>(BASE + 0x3c11b0, key);
        m.write::<64>(BASE + 0x3c11b8, signature);
        m.write::<32>(BASE + 0x3c11c0, adjustment as u64);
        m.write::<32>(SCRATCH + 0x70, u64::from(position.black_to_move));
        m.write::<32>(
            SCRATCH + 0x74,
            if position.black_to_move { u64::MAX } else { 1 },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn direct_native_snapshots_and_independent_preparation() {
        let mut evaluator = Evaluator::default();
        let mut differences = Vec::new();
        for line in include_str!("../tests/fixtures/hce-native.tsv")
            .lines()
            .chain(include_str!("../tests/fixtures/hce-native-expanded.tsv").lines())
            .filter(|l| !l.starts_with('#'))
        {
            let columns: Vec<_> = line.split('\t').collect();
            let fen = columns[0];
            let expected = columns[1].parse::<i32>().unwrap();
            let board = unhex(columns[2]);
            let scratch = unhex(columns[3]);
            let position = Position::from_fen(fen).unwrap();
            evaluator.prepare(&position);
            if let Some(root) = columns.get(4) {
                let root = unhex(root);
                assert_eq!(
                    &evaluator.memory.bytes[0x3c11c0..0x3c11c4],
                    &root[0x4c0..0x4c4],
                    "default search-root adjustment: {fen}"
                );
            }
            for (start, end) in [
                (0x100, 0x200),
                (0x400, 0x484),
                (0x498, 0x4a0),
                (0x4b0, 0x4c0),
            ] {
                if evaluator.memory.bytes[0x3c0d00 + start..0x3c0d00 + end] != board[start..end] {
                    differences.push(format!("preparation {start:x}..{end:x}: {fen}"));
                    for i in (start..end).step_by(8) {
                        if evaluator.memory.bytes[0x3c0d00 + i..0x3c0d00 + i + 8] != board[i..i + 8]
                        {
                            println!(
                                "offset={i:x} rust={:x?} native={:x?}",
                                &evaluator.memory.bytes[0x3c0d00 + i..0x3c0d00 + i + 8],
                                &board[i..i + 8]
                            );
                        }
                    }
                }
            }
            evaluator.memory.bytes[0x3c0d00..0x3c11c8].copy_from_slice(&board);
            evaluator.memory.bytes[0x3c1520..0x3c15a0].copy_from_slice(&scratch);
            let mut r = [0; 16];
            r[1] = SCRATCH;
            r[4] = STACK;
            let actual =
                recovered::eval_14009e270(&mut evaluator.memory, &mut r, &mut [0; 2]) as i32;
            if actual != expected {
                differences.push(format!("score actual={actual}, expected={expected}: {fen}"));
            }
            for reset in [false, true] {
                if reset {
                    evaluator.clear();
                }
                let mut r = [0; 16];
                r[1] = SCRATCH;
                r[4] = STACK;
                let repeated =
                    recovered::eval_14009e270(&mut evaluator.memory, &mut r, &mut [0; 2]) as i32;
                assert_eq!(repeated, expected, "warm/reset={reset}: {fen}");
            }
        }
        assert!(differences.is_empty(), "{}", differences.join("\n"));
    }

    #[test]
    fn machine_width_arithmetic_and_signed_overflow() {
        let mut flags = Flags::default();
        assert_eq!(flags.arithmetic(0xffff_ffff, 1, 32, false), 0);
        assert!(flags.z);
        assert!(!flags.o);
        assert_eq!(flags.arithmetic(0x7fff_ffff, 1, 32, false), 0x8000_0000);
        assert!(flags.s && flags.o);
        assert_eq!(flags.arithmetic(0x8000_0000, 1, 32, true), 0x7fff_ffff);
        assert!(!flags.s && flags.o);
        assert_eq!(-9i32 / 8, -1);
        assert_eq!(-9i32 >> 3, -2);
    }

    #[test]
    fn parameter_mutation_is_rejected_by_the_native_fixture() {
        let line = include_str!("../tests/fixtures/hce-native.tsv")
            .lines()
            .find(|l| !l.starts_with('#'))
            .unwrap();
        let columns: Vec<_> = line.split('\t').collect();
        let expected = columns[1].parse::<i32>().unwrap();
        let mut evaluator = Evaluator::default();
        evaluator.memory.bytes[0x3c0d00..0x3c11c8].copy_from_slice(&unhex(columns[2]));
        evaluator.memory.bytes[0x3c1520..0x3c15a0].copy_from_slice(&unhex(columns[3]));
        let signature = evaluator.memory.read::<64>(BASE + 0x3c11b8);
        let address = BASE + 0x202910 + (signature & 0x1ffff) * 2;
        let original = evaluator.memory.read::<16>(address);
        evaluator
            .memory
            .write::<16>(address, original.wrapping_add(1000));
        let mut r = [0; 16];
        r[1] = SCRATCH;
        r[4] = STACK;
        let mutated = recovered::eval_14009e270(&mut evaluator.memory, &mut r, &mut [0; 2]) as i32;
        assert_ne!(
            mutated, expected,
            "the fixture must detect a wrong material table"
        );
    }

    #[test]
    fn starting_position_smoke_and_warm_cache() {
        let p = Position {
            pieces: [
                0xff00,
                0x42,
                0x24,
                0x81,
                8,
                16,
                0xff000000000000,
                0x4200000000000000,
                0x2400000000000000,
                0x8100000000000000,
                0x0800000000000000,
                0x1000000000000000,
            ],
            black_to_move: false,
            castling: 15,
        };
        let mut evaluator = Evaluator::default();
        let first = evaluator.evaluate(&p);
        assert_eq!(first, evaluator.evaluate(&p));
        evaluator.clear();
        assert_eq!(first, evaluator.evaluate(&p));
        println!("startpos={first}");
    }
}
