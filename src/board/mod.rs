use core::fmt::Display;

use crate::move_generator::move_data::Move;
use crate::move_generator::gen_moves;

mod bit_board;
pub mod piece;
pub mod square;

use bit_board::BitBoard;
use piece::Piece;
use square::Square;

pub struct Board {
    bit_boards: [BitBoard; 12],

    white_to_move: bool,

    white_can_castle_king_side: bool,
    black_can_castle_queen_side: bool,

    black_can_castle_king_side: bool,
    white_can_castle_queen_side: bool,

    en_passant_square: Option<Square>,

    half_move_clock: u64,
    full_move_counter: u64,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Self {
        let mut bit_boards = [BitBoard::empty(); 12];

        // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR
        let mut square: Square = Square::from_index(64);

        let mut characters = fen.chars().peekable();

        for character in characters.by_ref() {
            if character == '/' {
                continue;
            }
            if let Some(digit) = character.to_digit(10) {
                square = square.sub(digit as u8);
            } else {
                let piece = piece::from_fen_char(&character)
                    .unwrap_or_else(|_| panic!("{square} {character}"));
                square = square.sub(1);
                bit_boards[piece as usize].set(&square);
            }
            if square.index() == 0 {
                break;
            }
        }

        let state = characters.collect::<String>();
        let mut split = state.split_whitespace();

        let white_to_move = match split.next().expect("Missing w/b to move") {
            "w" => true,
            "b" => false,
            _ => panic!("No w/b to move"),
        };

        let castling_rights = split.next().expect("Missing castling rights");
        let (
            white_can_castle_king_side,
            white_can_castle_queen_side,
            black_can_castle_king_side,
            black_can_castle_queen_side,
        ) = if castling_rights == "-" {
            (false, false, false, false)
        } else {
            (
                castling_rights.contains('K'),
                castling_rights.contains('Q'),
                castling_rights.contains('k'),
                castling_rights.contains('q'),
            )
        };

        let mut en_passant = split.next().expect("Missing en passant").chars().peekable();
        let en_passant_square = if *en_passant.peek().expect("Missing en passant") == '-' {
            None
        } else {
            let file = en_passant.next().expect("Missing en passant") as u8 - b'a';
            let rank = en_passant
                .next()
                .expect("Missing en passant")
                .to_digit(10)
                .unwrap() as u8
                - 1;
            Some(Square::from_coords(rank, file))
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

        Self {
            bit_boards,

            white_to_move,

            white_can_castle_king_side,
            black_can_castle_queen_side,

            black_can_castle_king_side,
            white_can_castle_queen_side,

            en_passant_square,

            half_move_clock,
            full_move_counter,
        }
    }
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        for piece in Piece::LIST {
            let bit_board = self.bit_boards[piece as usize];
            if bit_board.get(&square) {
                return Some(piece);
            }
        }
        None
    }
    pub fn gen_moves(&self) -> Vec<Move> {
        gen_moves(self)
    }
    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(87);

        let mut empty: u32 = 0;
        for rank in (0..8).rev() {
            for file in (0..8).rev() {
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

        if self.white_can_castle_king_side
            || self.white_can_castle_queen_side
            || self.black_can_castle_king_side
            || self.black_can_castle_queen_side
        {
            if self.white_can_castle_king_side {
                fen.push('K')
            }
            if self.white_can_castle_queen_side {
                fen.push('Q')
            }
            if self.black_can_castle_king_side {
                fen.push('k')
            }
            if self.black_can_castle_queen_side {
                fen.push('q')
            }
        } else {
            fen.push('-')
        }
        fen.push(' ');

        if let Some(en_passant_square) = &self.en_passant_square {
            fen.push_str(&en_passant_square.to_notation())
        } else {
            fen.push('-')
        }

        fen.push(' ');
        fen.push_str(&self.half_move_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.full_move_counter.to_string());

        fen
    }
}
