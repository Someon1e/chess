use super::{
    bit_board::BitBoard,
    game_state::{CastlingRights, GameState},
    piece::Piece,
    square::Square,
    zobrist::Zobrist,
    Board,
};

impl Board {
    /// Creates a Board from Forsyth-Edwards Notation
    /// # Panics
    ///
    /// Will panic if the FEN is invalid
    #[must_use]
    pub fn from_fen(fen: &str) -> Self {
        let mut components = fen.split_whitespace();

        let mut bit_boards = [BitBoard::EMPTY; 12];
        let mut zobrist_key = Zobrist::EMPTY;

        let (mut rank, mut file) = (7, 0);

        let position = components.next().expect("Missing position").chars();

        for character in position {
            if character == '/' {
                continue;
            }
            if let Some(digit) = character.to_digit(10) {
                file += digit as i8;
            } else {
                let piece =
                    Piece::from_fen_char(&character).expect("Failed to parse FEN character");
                let square = &Square::from_coords(rank, file);
                bit_boards[piece as usize].set(square);
                zobrist_key.xor_piece(piece as usize, square.usize());
                file += 1;
            }
            if file == 8 {
                if rank == 0 {
                    break;
                }
                rank -= 1;
                file = 0;
            }
        }

        let white_to_move = match components.next().expect("Missing w/b to move") {
            "w" => true,
            "b" => {
                zobrist_key.flip_side_to_move();
                false
            }
            _ => panic!("No w/b to move"),
        };

        let castling_rights =
            CastlingRights::from_fen_section(components.next().expect("Missing castling rights"));
        zobrist_key.xor_castling_rights(&castling_rights);

        let en_passant = components.next().expect("Missing en passant");
        let en_passant_square = if en_passant == "-" {
            None
        } else {
            let en_passant_square = Square::from_notation(en_passant);
            zobrist_key.xor_en_passant(&en_passant_square);
            Some(en_passant_square)
        };
        let half_move_clock = components
            .next()
            .expect("No half move clock")
            .parse()
            .expect("No half move clock");
        let full_move_counter = components
            .next()
            .expect("No full move counter")
            .parse()
            .expect("No full move counter");

        let game_state = GameState {
            en_passant_square,

            castling_rights,

            half_move_clock,
            captured: None,

            zobrist_key,
        };

        Self {
            white_to_move,

            bit_boards,

            full_move_counter,

            game_state,
        }
    }

    /// Gets the Forsyth-Edwards Notation of the Board
    /// # Panics
    ///
    /// Should not panic
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
        };
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

#[test]
fn test_fen_encoding() {
    for (_, _, fen) in crate::tests::TEST_FENS {
        let board = Board::from_fen(fen);
        assert_eq!(fen, board.to_fen());
    }
}
