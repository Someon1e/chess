pub mod board;
pub mod move_generator;

#[cfg(test)]
mod tests {
    use super::board::Board;
    use super::move_generator::PsuedoLegalMoveGenerator;

    const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const TEST_FENS: [&str; 3] = [
        START_POSITION_FEN,
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ];

    #[test]
    fn test_start_position() {
        let board = &mut Board::from_fen(START_POSITION_FEN);
        let move_generator = &mut PsuedoLegalMoveGenerator::new(board);
        let mut move_count = 0;
        for move_data in move_generator.gen() {
            println!("{} {move_data}", move_data.piece().to_fen_char());
            move_count += 1;
        }
        assert_eq!(move_count, 20)
    }

    #[test]
    fn test_fen_encoding() {
        for fen in TEST_FENS {
            let board = Board::from_fen(fen);
            assert_eq!(fen, board.to_fen());
        }
    }
}
