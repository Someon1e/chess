use super::{bit_board::BitBoard, game_state::{CastlingRights, GameState}, piece, square::Square, zobrist::Zobrist, Board};

impl Board {
    pub fn from_fen(fen: &str) -> Self {
        let mut bit_boards = [BitBoard::empty(); 12];
        let mut zobrist_key = Zobrist::empty();
    
        let (mut rank, mut file) = (7, 0);
    
        let mut characters = fen.chars().peekable();
    
        for character in characters.by_ref() {
            if character == '/' {
                continue;
            }
            if let Some(digit) = character.to_digit(10) {
                file += digit as i8;
            } else {
                let piece = piece::from_fen_char(&character).expect("{square} {character}");
                let square = &Square::from_coords(rank, file);
                bit_boards[piece as usize].set(&square);
                zobrist_key.xor_piece(piece as usize, square.index() as usize);
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
    
        let state = characters.collect::<String>();
        let mut split = state.split_whitespace();
    
        let white_to_move = match split.next().expect("Missing w/b to move") {
            "w" => true,
            "b" => {
                zobrist_key.flip_side_to_move();
                false
            }
            _ => panic!("No w/b to move"),
        };
    
        let castling_rights =
            CastlingRights::from_fen_section(split.next().expect("Missing castling rights"));
        zobrist_key.xor_castling_rights(&castling_rights);
    
        let en_passant = split.next().expect("Missing en passant");
        let en_passant_square = if en_passant == "-" {
            None
        } else {
            let en_passant_square = Square::from_notation(en_passant);
            zobrist_key.xor_en_passant(&en_passant_square);
            Some(en_passant_square)
        };
        let half_move_clock = split
            .next()
            .expect("No half move clock")
            .parse()
            .expect("No half move clock");
        let full_move_counter = split
            .next()
            .expect("No full move counter")
            .parse()
            .expect("No full move counter");
    
        let game_state = GameState {
            en_passant_square,
    
            castling_rights,
    
            half_move_clock,
            full_move_counter,
            captured: None,
    
            zobrist_key
        };
    
        Self {
            bit_boards,
    
            white_to_move,
    
            game_state,
    
            history: Vec::new(),
        }
    }
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
                    empty += 1
                }
            }
            if empty != 0 {
                fen.push(char::from_digit(empty, 10).unwrap());
                empty = 0;
            }
            if rank != 0 {
                fen.push('/')
            }
        }

        if self.white_to_move {
            fen.push_str(" w ")
        } else {
            fen.push_str(" b ")
        }

        if self.game_state.castling_rights.none() {
            fen.push('-')
        } else {
            if self.game_state.castling_rights.get_white_king_side() {
                fen.push('K')
            }
            if self.game_state.castling_rights.get_white_queen_side() {
                fen.push('Q')
            }
            if self.game_state.castling_rights.get_black_king_side() {
                fen.push('k')
            }
            if self.game_state.castling_rights.get_black_queen_side() {
                fen.push('q')
            }
        };
        fen.push(' ');

        if let Some(en_passant_square) = &self.game_state.en_passant_square {
            fen.push_str(&en_passant_square.to_notation())
        } else {
            fen.push('-')
        }

        fen.push(' ');
        fen.push_str(&self.game_state.half_move_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.game_state.full_move_counter.to_string());

        fen
    }
}