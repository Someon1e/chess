use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::{
    board::Board,
    search::{eval::Eval, eval_data},
};

pub fn evaluation_benchmark(c: &mut Criterion) {
    c.bench_function("evaluate start position", |bencher| {
        bencher.iter(|| {
            let board = Board::from_fen(black_box(Board::START_POSITION_FEN));
            Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLES,
                &eval_data::END_GAME_PIECE_SQUARE_TABLES,
                &eval_data::PHASES,
                &board,
            )
        });
    });
    c.bench_function("evaluate kiwipete", |bencher| {
        bencher.iter(|| {
            let board = Board::from_fen(black_box(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            ));
            Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLES,
                &eval_data::END_GAME_PIECE_SQUARE_TABLES,
                &eval_data::PHASES,
                &board,
            )
        });
    });
}

criterion_group!(benches, evaluation_benchmark);
criterion_main!(benches);
