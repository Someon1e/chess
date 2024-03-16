use crate::{board::Board, move_generator::MoveGenerator, uci};

#[cfg(test)]
use crate::board::zobrist::Zobrist;

fn perft(board: &mut Board, depth: u16) -> usize {
    if depth == 0 {
        return 1;
    };

    let mut move_count = 0;
    MoveGenerator::new(board).gen(
        &mut |move_data| {
            if !cfg!(test) && depth == 1 {
                move_count += 1;
                return;
            }

            let old_state = board.make_move(&move_data);

            #[cfg(test)]
            assert!(Zobrist::compute(board) == board.zobrist_key());

            move_count += perft(board, depth - 1);
            board.unmake_move(&move_data, &old_state);
        },
        false,
    );

    move_count
}

pub fn perft_root(board: &mut Board, depth: u16, log: fn(&str)) -> usize {
    let mut move_count = 0;
    MoveGenerator::new(board).gen(
        &mut |move_data| {
            if !cfg!(test) && depth == 1 {
                log(&format!("{}: 1", uci::encode_move(move_data)));
                move_count += 1;
                return;
            }

            let old_state = board.make_move(&move_data);

            #[cfg(test)]
            assert!(Zobrist::compute(board) == board.zobrist_key());

            let inner = perft(board, depth - 1);
            move_count += inner;
            log(&format!("{}: {}", uci::encode_move(move_data), inner));

            board.unmake_move(&move_data, &old_state);
        },
        false,
    );
    move_count
}
