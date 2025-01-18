use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::{
    board::Board,
    move_generator::{move_data::Move, MoveGenerator},
};

pub fn get_move_list(move_list: &mut Vec<Move>, board: &mut Board) {
    MoveGenerator::new(board).gen(
        &mut |move_data| {
            move_list.push(move_data);
        },
        false,
    );
}

pub fn make_unmake_move_benchmark(c: &mut Criterion) {
    let mut board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut move_list = Vec::new();
    get_move_list(&mut move_list, &mut board);
    c.bench_function("kiwipete", |bencher| {
        bencher.iter(|| {
            for move_data in black_box(move_list.clone()) {
                let old_state = board.make_move(&move_data);
                board.unmake_move(&move_data, &old_state);
            }
        });
    });
}

criterion_group!(benches, make_unmake_move_benchmark);
criterion_main!(benches);
