pub mod board;
pub mod engine;
pub mod move_generator;

#[cfg(test)]
mod tests {
    use std::io::{stdin, BufRead};
    use std::time::Instant;

    use crate::board::bit_board::BitBoard;

    use crate::board::square::Square;
    use crate::board::zobrist::Zobrist;

    use super::board::Board;
    use super::engine::Engine;
    use super::move_generator::MoveGenerator;

    const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const TEST_FENS: [(u16, usize, &str); 24] = [
        (6, 119060324, START_POSITION_FEN),
        (1, 8, "r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2"),
        (1, 8, "8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3"),
        (
            1,
            19,
            "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2",
        ),
        (
            1,
            5,
            "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2",
        ),
        (
            1,
            44,
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
        ),
        (
            1,
            39,
            "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9",
        ),
        (1, 9, "2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4"),
        (
            3,
            62379,
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        ),
        (
            3,
            89890,
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        ),
        (6, 1134888, "3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1"),
        (6, 1015133, "8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1"),
        (6, 1440467, "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1"),
        (6, 661072, "5k2/8/8/8/8/8/8/4K2R w K - 0 1"),
        (6, 803711, "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1"),
        (4, 1274206, "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1"),
        (4, 1720476, "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1"),
        (6, 3821001, "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1"),
        (5, 1004658, "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1"),
        (6, 217342, "4k3/1P6/8/8/8/8/K7/8 w - - 0 1"),
        (6, 92683, "8/P1k5/K7/8/8/8/8/8 w - - 0 1"),
        (6, 2217, "K1k5/8/P7/8/8/8/8/8 w - - 0 1"),
        (7, 567584, "8/k1P5/8/1K6/8/8/8/8 w - - 0 1"),
        (4, 23527, "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1"),
    ];

    fn perft_inner(board: &mut Board, depth: u16) -> usize {
        if depth == 0 {
            return 1;
        };

        let mut move_count = 0;
        MoveGenerator::new(board).gen(&mut |move_data| {
            board.make_move(&move_data);

            move_count += perft_inner(board, depth - 1);
            board.unmake_move(&move_data);
        });

        move_count
    }
    fn perft_run(fen: &str, depth: u16, expected_move_count: usize) {
        let start = Instant::now();

        let board = &mut Board::from_fen(fen);

        let mut move_count = 0;

        MoveGenerator::new(board).gen(&mut |move_data| {
            board.make_move(&move_data);
            assert!(Zobrist::compute(board) == board.zobrist_key());

            let inner = perft_inner(board, depth - 1);
            move_count += inner;
            println!("{move_data} {inner}");
            board.unmake_move(&move_data);
        });

        let seconds_elapsed = start.elapsed().as_secs_f32();
        println!(
            "Done in {} seconds, {} nodes per second",
            seconds_elapsed,
            (move_count as f32) / seconds_elapsed
        );
        if move_count != expected_move_count {
            panic!("Expected {expected_move_count} got {move_count}")
        } else {
            println!("Nodes searched: {move_count}");
        }
    }

    #[test]
    fn test_perft() {
        for (depth, expected_move_count, fen) in TEST_FENS {
            println!("{fen}");
            perft_run(fen, depth, expected_move_count);
            println!();
        }
    }

    #[test]
    fn get_best_move() {
        let mut stdin = stdin().lock();
        loop {
            let mut fen = String::new();
            stdin.read_line(&mut fen).unwrap();
            let board = &mut Board::from_fen(&fen);
            let engine = &mut Engine::new(board);
            let search_start = Instant::now();
            let (best_move, evaluation) = engine.iterative_deepening(
                &mut |depth, (best_move, evaluation)| {
                    println!("Best move at depth {depth} is {} {}", best_move, evaluation);
                },
                &mut || search_start.elapsed().as_millis() > 3 * 1000,
            );
            println!("Result: {best_move} {evaluation}")
        }
    }

    #[test]
    fn test_coordinates() {
        let a1 = Square::from_coords(0, 0);
        assert!(a1.to_notation() == "a1");
        let b1 = a1.right(1);
        assert!(b1.to_notation() == "b1");
        let b2 = b1.up(1);
        assert!(b2.to_notation() == "b2");
        let also_b2 = Square::from_index(b2.index());
        assert!(also_b2.to_notation() == "b2");
        assert!(also_b2.index() == 9);

        let bit_board = also_b2.bitboard();

        let mut same_bit_board = BitBoard::empty();
        same_bit_board.set(&also_b2);

        assert!(bit_board.to_string() == same_bit_board.to_string())
    }

    #[test]
    fn test_fen_encoding() {
        for (_, _, fen) in TEST_FENS {
            let board = Board::from_fen(fen);
            assert_eq!(fen, board.to_fen());
        }
    }
}
