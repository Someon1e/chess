use crate::{
    board::{zobrist::Zobrist, Board},
    move_generator::MoveGenerator,
    uci,
};

fn perft(board: &mut Board, check_zobrist: bool, bulk_count: bool, depth: u16) -> usize {
    if depth == 0 {
        return 1;
    };

    let mut move_count = 0;
    MoveGenerator::new(board).gen(
        &mut |move_data| {
            if !bulk_count || depth != 1 {
                board.make_move(&move_data);
            }

            if check_zobrist {
                assert!(Zobrist::compute(board) == board.zobrist_key());
            }

            move_count += perft(board, check_zobrist, bulk_count, depth - 1);

            if !bulk_count || depth != 1 {
                board.unmake_move(&move_data);
            }
        },
        false,
    );

    move_count
}

pub fn perft_root(
    board: &mut Board,
    check_zobrist: bool,
    bulk_count: bool,
    depth: u16,
    log: fn(&str),
) -> usize {
    let mut move_count = 0;
    MoveGenerator::new(board).gen(
        &mut |move_data| {
            if !bulk_count || depth != 1 {
                board.make_move(&move_data);
            }

            if check_zobrist {
                assert!(Zobrist::compute(board) == board.zobrist_key());
            }

            let inner = perft(board, check_zobrist, bulk_count, depth - 1);
            log(&format!("{}: {}", uci::encode_move(move_data), inner));
            move_count += inner;

            if !bulk_count || depth != 1 {
                board.unmake_move(&move_data);
            }
        },
        false,
    );
    move_count
}
