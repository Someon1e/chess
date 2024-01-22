pub mod board;
pub mod engine;
pub mod move_generator;

#[cfg(test)]
mod tests {
    use crate::board::bit_board::BitBoard;
    use crate::board::piece::Piece;
    use crate::board::square::Square;

    use super::board::Board;
    use super::engine::Engine;
    use super::move_generator::PsuedoLegalMoveGenerator;

    const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const TEST_FENS: [&str; 3] = [
        START_POSITION_FEN,
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ];

    fn perft(engine: &mut Engine, depth: u16) -> usize {
        if depth == 0 {
            return 1;
        };

        let mut moves = Vec::new();
        engine.move_generator().gen(&mut moves);

        let mut move_count = 0;
        for move_data in moves {
            engine.board().make_move(&move_data);
            if !engine.can_capture_king() {
                move_count += perft(engine, depth - 1);
            }
            engine.board().unmake_move(&move_data);
        }
        move_count
    }

    #[test]
    fn test_start_position() {
        let board = &mut Board::from_fen(START_POSITION_FEN);
        let move_generator = &mut PsuedoLegalMoveGenerator::new(board);
        let engine = &mut Engine::new(move_generator);
        let move_count = perft(engine, 3);

        assert_eq!(move_count, 0);
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
        for fen in TEST_FENS {
            let board = Board::from_fen(fen);
            assert_eq!(fen, board.to_fen());
        }
    }
}
