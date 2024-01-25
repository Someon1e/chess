use core::fmt::Display;

pub mod bit_board;
pub mod game_state;
pub mod piece;
pub mod square;

use bit_board::BitBoard;
use piece::{Piece, ALL_PIECES, BLACK_PIECES, WHITE_PIECES};
use square::Square;

use crate::move_generator::move_data::Move;

use self::game_state::{CastlingRights, GameState};

pub struct Board {
    pub white_to_move: bool,

    bit_boards: [BitBoard; 12],

    pub game_state: GameState,
    history: Vec<GameState>,
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
                bit_boards[piece as usize].set(&Square::from_coords(rank, file));
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
            "b" => false,
            _ => panic!("No w/b to move"),
        };

        let castling_rights =
            CastlingRights::from_fen_section(split.next().expect("Missing castling rights"));

        let en_passant = split.next().expect("Missing en passant");
        let en_passant_square = if en_passant == "-" {
            None
        } else {
            Some(Square::from_notation(en_passant))
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
        };

        Self {
            bit_boards,

            white_to_move,

            game_state,

            history: vec![],
        }
    }
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        for piece in ALL_PIECES {
            let bit_board = self.get_bit_board(piece);
            if bit_board.get(&square) {
                return Some(piece);
            }
        }
        None
    }
    pub fn white_piece_at(&self, square: Square) -> Option<Piece> {
        for piece in WHITE_PIECES {
            let bit_board = self.get_bit_board(piece);
            if bit_board.get(&square) {
                return Some(piece);
            }
        }
        None
    }
    pub fn black_piece_at(&self, square: Square) -> Option<Piece> {
        for piece in BLACK_PIECES {
            let bit_board = self.get_bit_board(piece);
            if bit_board.get(&square) {
                return Some(piece);
            }
        }
        None
    }
    pub fn make_move(&mut self, move_data: &Move) {
        self.history.push(self.game_state);

        if move_data.from() == Square::from_notation("a1")
            || move_data.to() == Square::from_notation("a1")
        {
            self.game_state.castling_rights.unset_white_queen_side();
        }
        if move_data.from() == Square::from_notation("h1")
            || move_data.to() == Square::from_notation("h1")
        {
            self.game_state.castling_rights.unset_white_king_side();
        }
        if move_data.from() == Square::from_notation("a8")
            || move_data.to() == Square::from_notation("a8")
        {
            self.game_state.castling_rights.unset_black_queen_side();
        }
        if move_data.from() == Square::from_notation("h8")
            || move_data.to() == Square::from_notation("h8")
        {
            self.game_state.castling_rights.unset_black_king_side();
        }

        let moving_bit_board = self.get_bit_board_mut(move_data.piece());
        moving_bit_board.unset(&move_data.from());
        moving_bit_board.set(&move_data.to());

        let en_passant_square = self.game_state.en_passant_square;
        self.game_state.en_passant_square = None;

        if let Some(captured) = move_data.captured() {
            let capture_position = if move_data.is_en_passant() {
                en_passant_square
                    .unwrap()
                    .down(if self.white_to_move { 1 } else { -1 })
            } else {
                move_data.to()
            };
            let capturing_bit_board = self.get_bit_board_mut(captured);
            capturing_bit_board.unset(&capture_position);
        }
        
        if move_data.piece()
            == (if self.white_to_move {
                Piece::WhiteKing
            } else {
                Piece::BlackKing
            })
        {
            if self.white_to_move {
                self.game_state.castling_rights.unset_white_king_side();
                self.game_state.castling_rights.unset_white_queen_side();
            } else {
                self.game_state.castling_rights.unset_black_king_side();
                self.game_state.castling_rights.unset_black_queen_side();
            }
            if move_data.is_castle() {
                let (rook_from, rook_to) = if move_data.to() == Square::from_notation("g1") {
                    (Square::from_notation("h1"), Square::from_notation("f1"))
                } else {
                    (Square::from_notation("a1"), Square::from_notation("d1"))
                };
                let rook_bit_board = if self.white_to_move {
                    self.get_bit_board_mut(Piece::WhiteRook)
                } else {
                    self.get_bit_board_mut(Piece::BlackRook)
                };
                rook_bit_board.unset(&rook_from);
                rook_bit_board.set(&rook_to)
            }
        } else if move_data.is_pawn_two_up() {
            self.game_state.en_passant_square =
                Some(move_data.from().up(if self.white_to_move { 1 } else { -1 }))
        }

        self.white_to_move = !self.white_to_move;
    }
    pub fn unmake_move(&mut self, move_data: &Move) {
        self.game_state = self.history.pop().unwrap();

        let bit_board = self.get_bit_board_mut(move_data.piece());
        bit_board.unset(&move_data.to());
        bit_board.set(&move_data.from());

        if move_data.is_castle() {
            let (rook_from, rook_to) = if move_data.to() == Square::from_notation("g1") {
                (Square::from_notation("h1"), Square::from_notation("f1"))
            } else {
                (Square::from_notation("a1"), Square::from_notation("d1"))
            };
            let rook_bit_board = if self.white_to_move {
                self.get_bit_board_mut(Piece::WhiteRook)
            } else {
                self.get_bit_board_mut(Piece::BlackRook)
            };
            rook_bit_board.set(&rook_from);
            rook_bit_board.unset(&rook_to)
        } else if let Some(captured) = move_data.captured() {
            let capture_position = if move_data.is_en_passant() {
                self.game_state
                    .en_passant_square
                    .unwrap()
                    .up(if self.white_to_move { 1 } else { -1 })
            } else {
                move_data.to()
            };
            let capturing_bit_board = self.get_bit_board_mut(captured);
            capturing_bit_board.set(&capture_position)
        }

        self.white_to_move = !self.white_to_move
    }
    pub fn get_bit_board(&self, piece: Piece) -> &BitBoard {
        &self.bit_boards[piece as usize]
    }
    fn get_bit_board_mut(&mut self, piece: Piece) -> &mut BitBoard {
        &mut self.bit_boards[piece as usize]
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
