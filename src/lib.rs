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
    const TEST_FENS: [(&str, u16, usize); 4] = [
        (START_POSITION_FEN, 5, 4865609),
        ("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8),
        ("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9),
        ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3, 62379),
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
    fn perft_start() {
        for (fen, depth, expected_move_count) in TEST_FENS {
            let board = &mut Board::from_fen(fen);
            let move_generator = &mut PsuedoLegalMoveGenerator::new(board);
            let engine = &mut Engine::new(move_generator);
            let move_count = perft(engine, depth);
    
            assert_eq!(move_count, expected_move_count);
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
        for (fen, _, _) in TEST_FENS {
            let board = Board::from_fen(fen);
            assert_eq!(fen, board.to_fen());
        }
    }
}
