use core::fmt::Display;

pub mod bit_board;
pub mod fen;
pub mod game_state;

/// Handles pieces.
pub mod piece;

/// Handles squares.
pub mod square;

pub mod zobrist;

use bit_board::BitBoard;
use piece::Piece;
use square::Square;

use self::{game_state::GameState, zobrist::Zobrist};

/// Represents a chess position.
pub struct Board {
    /// Whether it is white's turn to move.
    pub white_to_move: bool,

    /// Bit boards, one for every piece type.
    pub bit_boards: [BitBoard; 12],

    /// The number of full moves.
    pub full_move_counter: u64,

    /// State of the game.
    pub game_state: GameState,
}

impl Display for Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl Board {
    /// The starting position FEN in standard chess.
    pub const START_POSITION_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// Returns a piece at a square.
    #[must_use]
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

    /// Returns a white piece at a square.
    #[must_use]
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

    /// Returns a black piece at a square.
    #[must_use]
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

    /// Returns a piece on the side-to-move at a square.
    #[must_use]
    pub fn friendly_piece_at(&self, square: Square) -> Option<Piece> {
        if self.white_to_move {
            self.white_piece_at(square)
        } else {
            self.black_piece_at(square)
        }
    }

    /// Returns an opponent's piece at a square.
    #[must_use]
    pub fn enemy_piece_at(&self, square: Square) -> Option<Piece> {
        if self.white_to_move {
            self.black_piece_at(square)
        } else {
            self.white_piece_at(square)
        }
    }

    /// Returns the current zobrist key
    #[must_use]
    pub const fn zobrist_key(&self) -> Zobrist {
        self.game_state.zobrist_key
    }

    /// Returns a reference to a piece type's bit board.
    #[must_use]
    pub const fn get_bit_board(&self, piece: Piece) -> &BitBoard {
        &self.bit_boards[piece as usize]
    }

    /// Returns a mutable reference to a piece type's bit board.
    #[must_use]
    pub fn get_bit_board_mut(&mut self, piece: Piece) -> &mut BitBoard {
        &mut self.bit_boards[piece as usize]
    }
}
