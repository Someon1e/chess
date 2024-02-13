use core::fmt::Display;

pub mod bit_board;
pub mod fen;
pub mod game_state;
pub mod piece;
pub mod square;
pub mod zobrist;

use bit_board::BitBoard;
use piece::Piece;
use square::Square;

use self::{game_state::GameState, zobrist::Zobrist};

pub struct Board {
    pub white_to_move: bool,

    pub bit_boards: [BitBoard; 12],

    pub game_state: GameState,
    pub history: Vec<GameState>,
}

impl Display for Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl Board {
    pub const START_POSITION_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let square_bit_board = square.bit_board();
        for piece in Piece::ALL_PIECES {
            let piece_bit_board = self.get_bit_board(piece);
            if piece_bit_board.overlaps(&square_bit_board) {
                return Some(piece);
            }
        }
        None
    }
    pub fn white_piece_at(&self, square: Square) -> Option<Piece> {
        let square_bit_board = square.bit_board();
        for piece in Piece::WHITE_PIECES {
            let piece_bit_board = self.get_bit_board(piece);
            if piece_bit_board.overlaps(&square_bit_board) {
                return Some(piece);
            }
        }
        None
    }
    pub fn black_piece_at(&self, square: Square) -> Option<Piece> {
        let square_bit_board = square.bit_board();
        for piece in Piece::BLACK_PIECES {
            let piece_bit_board = self.get_bit_board(piece);
            if piece_bit_board.overlaps(&square_bit_board) {
                return Some(piece);
            }
        }
        None
    }
    pub fn friendly_piece_at(&self, square: Square) -> Option<Piece> {
        if self.white_to_move {
            self.white_piece_at(square)
        } else {
            self.black_piece_at(square)
        }
    }
    pub fn enemy_piece_at(&self, square: Square) -> Option<Piece> {
        if self.white_to_move {
            self.black_piece_at(square)
        } else {
            self.white_piece_at(square)
        }
    }
    pub fn zobrist_key(&self) -> Zobrist {
        self.game_state.zobrist_key
    }
    pub fn get_bit_board(&self, piece: Piece) -> &BitBoard {
        &self.bit_boards[piece as usize]
    }
    pub fn get_bit_board_mut(&mut self, piece: Piece) -> &mut BitBoard {
        &mut self.bit_boards[piece as usize]
    }
}
