use super::{
    Board,
    bit_board::BitBoard,
    game_state::{CastlingRights, GameState},
    piece::Piece,
    square::Square,
};

/// Errors that can occur when parsing a FEN string.
#[derive(Debug)]
pub enum FenParseErr {
    /// The position section of the FEN string is missing.
    MissingPosition,

    /// An invalid piece character was encountered in the position section of the FEN string.
    InvalidPiece,

    /// An invalid digit was encountered in the position section (e.g., a number greater than 8 or incorrect rank structure).
    InvalidDigit,

    /// The side to move ("w" or "b") is missing from the FEN string.
    MissingSideToMove,

    /// The side to move is present but contains an invalid value (not "w" or "b").
    InvalidSideToMove,

    /// The half-move clock (used for the fifty-move rule) is missing from the FEN string.
    MissingHalfMoveClock,

    /// The half-move clock is present but contains an invalid value (not a valid integer).
    InvalidHalfMoveClock,

    /// The full-move counter (which counts the number of full moves in the game) is missing from the FEN string.
    MissingFullMoveCounter,

    /// The full-move counter is present but contains an invalid value (not a valid integer).
    InvalidFullMoveCounter,

    /// The en passant target square is missing from the FEN string.
    MissingEnPassant,

    /// The en passant target square is present but contains an invalid value (not a valid square notation or "-").
    InvalidEnPassant,

    /// The castling rights section is missing from the FEN string.
    MissingCastling,
}

impl Board {
    /// The starting position FEN in standard chess.
    pub const START_POSITION_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// Creates a Board from Forsyth-Edwards Notation.
    #[must_use]
    pub fn from_fen(fen: &str) -> Result<Self, FenParseErr> {
        let mut components = fen.split_whitespace();

        let mut bit_boards = [BitBoard::EMPTY; 12];

        let (mut rank, mut file) = (7, 0);

        let position = {
            let component = components.next();
            if let Some(component) = component {
                component.chars()
            } else {
                return Err(FenParseErr::MissingPosition);
            }
        };

        for character in position {
            if character == '/' {
                continue;
            }

            if let Some(digit) = character.to_digit(10) {
                if digit > 8 {
                    return Err(FenParseErr::InvalidDigit);
                }
                file += digit as i8;
                if file > 8 {
                    return Err(FenParseErr::InvalidDigit);
                }
            } else {
                if let Some(piece) = Piece::from_fen_char(&character) {
                    let square = &Square::from_coords(rank, file);
                    bit_boards[piece as usize].set(square);

                    file += 1;
                } else {
                    return Err(FenParseErr::InvalidPiece);
                }
            }

            if file == 8 {
                if rank == 0 {
                    break;
                }
                rank -= 1;
                file = 0;
            }
        }

        let white_to_move = match components.next() {
            Some("w") => true,
            Some("b") => false,
            None => return Err(FenParseErr::MissingSideToMove),
            _ => return Err(FenParseErr::InvalidSideToMove),
        };

        let castling_rights = CastlingRights::from_fen_section({
            if let Some(component) = components.next() {
                component
            } else {
                return Err(FenParseErr::MissingCastling);
            }
        });

        let en_passant = {
            if let Some(en_passant) = components.next() {
                en_passant
            } else {
                return Err(FenParseErr::MissingEnPassant);
            }
        };
        let en_passant_square = if en_passant == "-" {
            None
        } else {
            let en_passant_square = Square::from_notation(en_passant);
            if en_passant_square.is_err() {
                return Err(FenParseErr::InvalidEnPassant);
            }
            Some(en_passant_square.unwrap())
        };
        let half_move_clock = {
            let component = components.next();
            if component.is_none() {
                return Err(FenParseErr::MissingHalfMoveClock);
            }
            let parsed = component.unwrap().parse();
            if parsed.is_err() {
                return Err(FenParseErr::InvalidHalfMoveClock);
            }
            parsed.unwrap()
        };

        let full_move_counter = {
            let component = components.next();
            if component.is_none() {
                return Err(FenParseErr::MissingFullMoveCounter);
            }
            let parsed = component.unwrap().parse();
            if parsed.is_err() {
                return Err(FenParseErr::InvalidFullMoveCounter);
            }
            parsed.unwrap()
        };

        let game_state = GameState {
            en_passant_square,

            castling_rights,

            half_move_clock,
            captured: None,
        };

        let board = Self {
            white_to_move,

            bit_boards,

            full_move_counter,

            game_state,
        };

        Ok(board)
    }

    /// Gets the Forsyth-Edwards Notation of the Board.
    ///
    /// # Panics
    ///
    /// Should not panic.
    #[must_use]
    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(87);

        let mut empty: u32 = 0;
        for rank in (0..8).rev() {
            for file in 0..8 {
                if let Some(piece) = self.piece_at(Square::from_coords(rank, file)) {
                    if empty != 0 {
                        fen.push(char::from_digit(empty, 10).unwrap());
                        empty = 0;
                    }
                    fen.push(piece.to_fen_char());
                } else {
                    empty += 1;
                }
            }
            if empty != 0 {
                fen.push(char::from_digit(empty, 10).unwrap());
                empty = 0;
            }
            if rank != 0 {
                fen.push('/');
            }
        }

        if self.white_to_move {
            fen.push_str(" w ");
        } else {
            fen.push_str(" b ");
        }

        if self.game_state.castling_rights.is_none() {
            fen.push('-');
        } else {
            if self.game_state.castling_rights.get_white_king_side() {
                fen.push('K');
            }
            if self.game_state.castling_rights.get_white_queen_side() {
                fen.push('Q');
            }
            if self.game_state.castling_rights.get_black_king_side() {
                fen.push('k');
            }
            if self.game_state.castling_rights.get_black_queen_side() {
                fen.push('q');
            }
        }
        fen.push(' ');

        if let Some(en_passant_square) = &self.game_state.en_passant_square {
            fen.push_str(&en_passant_square.to_notation());
        } else {
            fen.push('-');
        }

        fen.push(' ');
        fen.push_str(&self.game_state.half_move_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.full_move_counter.to_string());

        fen
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    #[test]
    fn test_fen_encoding() {
        for (_, _, fen) in crate::tests::TEST_FENS {
            let board = Board::from_fen(fen).unwrap();
            assert_eq!(fen, board.to_fen());
        }
    }
}
