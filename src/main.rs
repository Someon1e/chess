mod board;
mod piece;
mod bitboard;
mod square;

use crate::board::Board;

const TEST_FENS: [&str; 3] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
];

fn main() {
    for fen in TEST_FENS {
        let board = Board::from_fen(fen);
        assert_eq!(fen, board.to_fen());
    }
}
