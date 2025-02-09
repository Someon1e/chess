use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::{board::Board, perft::perft_root};

pub fn move_generation_benchmark(c: &mut Criterion) {
    c.bench_function("perft start position", |bencher| {
        bencher.iter(|| {
            let mut board = Board::from_fen(black_box(Board::START_POSITION_FEN)).unwrap();
            perft_root(&mut board, 5, |_| {})
        });
    });
    c.bench_function("perft kiwipete", |bencher| {
        bencher.iter(|| {
            let mut board = Board::from_fen(black_box(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            ))
            .unwrap();
            perft_root(&mut board, 4, |_| {})
        });
    });
}

criterion_group!(benches, move_generation_benchmark);
criterion_main!(benches);
