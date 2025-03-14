use criterion::{Criterion, black_box, criterion_group, criterion_main};
use engine::{board::Board, evaluation::Eval};

pub fn evaluation_benchmark(c: &mut Criterion) {
    let start_position =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    c.bench_function("evaluate start position", |bencher| {
        bencher.iter(|| Eval::evaluate(black_box(&start_position)));
    });

    let kiwipete =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    c.bench_function("evaluate kiwipete", |bencher| {
        bencher.iter(|| Eval::evaluate(black_box(&kiwipete)));
    });
}

criterion_group!(benches, evaluation_benchmark);
criterion_main!(benches);
