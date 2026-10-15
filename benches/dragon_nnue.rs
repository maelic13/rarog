//! Component attribution with exact checksums; loading is outside timing.
use dragon_nnue::{AccumulatorCache, Kernel, Network, Position};
use rarog::board::{Board, Color, Piece};
use std::hint::black_box;
use std::time::Instant;

struct Sample {
    position: Position,
    pieces: [u64; 12],
    black: bool,
}
fn main() {
    let network = Network::load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/nnue/dragon-network.nnue"
    ))
    .unwrap();
    let mut samples = Vec::new();
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    for fen in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "8/P7/7k/8/8/8/7p/4K3 w - - 0 1",
    ] {
        let mut board = Board::from_fen(fen).unwrap();
        for _ in 0..64 {
            samples.push(Sample {
                position: Position::from_fen(&board.to_fen()).unwrap(),
                pieces: std::array::from_fn(|i| {
                    board
                        .pieces(
                            if i < 6 { Color::White } else { Color::Black },
                            [
                                Piece::Pawn,
                                Piece::Knight,
                                Piece::Bishop,
                                Piece::Rook,
                                Piece::Queen,
                                Piece::King,
                            ][i % 6],
                        )
                        .0
                }),
                black: board.side_to_move() == Color::Black,
            });
            let legal = board.generate_legal_moves();
            if legal.is_empty() {
                break;
            }
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let i = usize::try_from(seed % u64::try_from(legal.len()).unwrap()).unwrap();
            board.make_move(legal[i]);
        }
    }
    let mut variants = vec![("full-scalar", None)];
    for kernel in [Kernel::Portable, Kernel::Avx2, Kernel::AvxVnni] {
        if let Some(cache) = AccumulatorCache::with_kernel(kernel) {
            variants.push((kernel.name(), Some(cache)));
        }
    }
    for sample in &samples {
        let expected = network.evaluate_scaled(&sample.position, 72).unwrap();
        for (_, cache) in &mut variants {
            if let Some(cache) = cache {
                assert_eq!(
                    cache
                        .evaluate_scaled(&network, &sample.pieces, sample.black, 72)
                        .unwrap(),
                    expected
                );
            }
        }
    }
    let calls = 65_536;
    println!(
        "samples={} calls={} automatic={}",
        samples.len(),
        calls,
        AccumulatorCache::new().kernel().name()
    );
    let mut expected_checksum = None;
    for round in 0..3 {
        let count = variants.len();
        for offset in 0..count {
            let (name, cache) = &mut variants[(round + offset) % count];
            let start = Instant::now();
            let mut checksum = 0_i64;
            for i in 0..calls {
                let sample = black_box(&samples[i % samples.len()]);
                let score = match cache {
                    Some(cache) => cache
                        .evaluate_scaled(black_box(&network), &sample.pieces, sample.black, 72)
                        .unwrap(),
                    None => network.evaluate_scaled(&sample.position, 72).unwrap(),
                };
                checksum += i64::from(black_box(score));
            }
            let ns = start.elapsed().as_nanos();
            if let Some(expected) = expected_checksum {
                assert_eq!(checksum, expected);
            } else {
                expected_checksum = Some(checksum);
            }
            println!("{name}\t{round}\t{calls}\t{ns}\t{checksum}");
        }
    }
}
