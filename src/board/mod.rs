use core::fmt::Display;

pub mod bit_board;
pub mod game_state;
pub mod piece;
pub mod square;
pub mod zobrist;
pub mod fen;

use bit_board::BitBoard;
use piece::{Piece, ALL_PIECES, BLACK_PIECES, WHITE_PIECES};
use square::Square;

use crate::move_generator::move_data::{Flag, Move};

use self::{
    game_state::GameState,
    zobrist::Zobrist,
};

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
    pub fn make_move(&mut self, move_data: &Move) {
        let white_to_move = self.white_to_move;

        self.history.push(self.game_state);

        self.game_state.zobrist_key.flip_side_to_move();

        let piece = self.friendly_piece_at(move_data.from()).unwrap();

        self.game_state.zobrist_key
            .xor_piece(piece as usize, move_data.from().index() as usize);

        let flag = move_data.flag();

        self.game_state.zobrist_key
            .xor_castling_rights(&self.game_state.castling_rights);

        let is_castle = *flag == Flag::Castle;
        if is_castle
            || piece
                == (if white_to_move {
                    Piece::WhiteKing
                } else {
                    Piece::BlackKing
                })
        {
            if white_to_move {
                self.game_state.castling_rights.unset_white_king_side();
                self.game_state.castling_rights.unset_white_queen_side();
            } else {
                self.game_state.castling_rights.unset_black_king_side();
                self.game_state.castling_rights.unset_black_queen_side();
            }
        } else {
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
        }

        self.game_state.zobrist_key
            .xor_castling_rights(&self.game_state.castling_rights);

        let promotion_piece = flag.get_promotion_piece(white_to_move);

        let moving_bit_board = self.get_bit_board_mut(piece);

        if let Some(promotion_piece) = promotion_piece {
            moving_bit_board.unset(&move_data.from());
            self.get_bit_board_mut(promotion_piece).set(&move_data.to());
            self.game_state.zobrist_key
                .xor_piece(promotion_piece as usize, move_data.to().index() as usize);
        } else {
            moving_bit_board.toggle(&move_data.from(), &move_data.to());
            self.game_state.zobrist_key
                .xor_piece(piece as usize, move_data.to().index() as usize);
        }

        let en_passant_square = self.game_state.en_passant_square;

        if let Some(en_passant_square) = en_passant_square {
            self.game_state.zobrist_key.xor_en_passant(&en_passant_square);
        }
        self.game_state.en_passant_square = None;

        if *flag == Flag::PawnTwoUp {
            let en_passant_square = move_data.from().up(if self.white_to_move { 1 } else { -1 });
            self.game_state.en_passant_square = Some(en_passant_square);
            self.game_state.zobrist_key.xor_en_passant(&en_passant_square);
            self.game_state.captured = None;
        } else if is_castle {
            let is_king_side = move_data.to().file() == 6;
            let rook_to_offset = if is_king_side { -1 } else { 1 };
            let rook_from_offset = if is_king_side { 1 } else { -2 };
            let rook = if white_to_move {
                Piece::WhiteRook
            } else {
                Piece::BlackRook
            };
            let rook_bit_board = self.get_bit_board_mut(rook);
            let rook_from = &move_data.to().offset(rook_from_offset);
            let rook_to = &move_data.to().offset(rook_to_offset);
            rook_bit_board.toggle(&rook_from, &rook_to);
            self.game_state.zobrist_key
                .xor_piece(rook as usize, rook_from.index() as usize);
            self.game_state.zobrist_key
                .xor_piece(rook as usize, rook_to.index() as usize);
        } else if *flag == Flag::EnPassant {
            let capture_position =
                en_passant_square
                    .unwrap()
                    .down(if self.white_to_move { 1 } else { -1 });
            let captured = if white_to_move {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };
            self.game_state.captured = Some(captured);

            let capturing_bit_board = self.get_bit_board_mut(captured);
            capturing_bit_board.unset(&capture_position);
            self.game_state.zobrist_key
                .xor_piece(captured as usize, capture_position.index() as usize);
        } else {
            self.game_state.captured = self.enemy_piece_at(move_data.to());
            if let Some(captured) = self.game_state.captured {
                let capturing_bit_board = self.get_bit_board_mut(captured);
                capturing_bit_board.unset(&move_data.to());
                self.game_state.zobrist_key
                    .xor_piece(captured as usize, move_data.to().index() as usize);
            }
        }

        self.white_to_move = !white_to_move;
    }
    pub fn unmake_move(&mut self, move_data: &Move) {
        let capture = self.game_state.captured;
        self.game_state = self.history.pop().unwrap();

        let white_to_move = !self.white_to_move;
        self.white_to_move = white_to_move;

        let flag = move_data.flag();
        let promotion_piece = flag.get_promotion_piece(white_to_move);
        if let Some(promotion_piece) = promotion_piece {
            let moving_bit_board = if white_to_move {
                self.get_bit_board_mut(Piece::WhitePawn)
            } else {
                self.get_bit_board_mut(Piece::BlackPawn)
            };
            moving_bit_board.set(&move_data.from());
            self.get_bit_board_mut(promotion_piece)
                .unset(&move_data.to());
        } else {
            let moving_bit_board =
                self.get_bit_board_mut(self.friendly_piece_at(move_data.to()).unwrap());
            moving_bit_board.toggle(&move_data.from(), &move_data.to());
        }

        if *flag == Flag::Castle {
            let is_king_side = move_data.to().file() == 6;
            let rook_to_offset = if is_king_side { -1 } else { 1 };
            let rook_from_offset = if is_king_side { 1 } else { -2 };
            let rook_bit_board = if white_to_move {
                self.get_bit_board_mut(Piece::WhiteRook)
            } else {
                self.get_bit_board_mut(Piece::BlackRook)
            };
            rook_bit_board.toggle(
                &move_data.to().offset(rook_from_offset),
                &move_data.to().offset(rook_to_offset),
            );
        } else if let Some(captured) = capture {
            let capture_position = if *flag == Flag::EnPassant {
                self.game_state
                    .en_passant_square
                    .unwrap()
                    .down(if white_to_move { 1 } else { -1 })
            } else {
                move_data.to()
            };
            let capturing_bit_board = self.get_bit_board_mut(captured);
            capturing_bit_board.set(&capture_position)
        }
    }
    pub fn get_bit_board(&self, piece: Piece) -> &BitBoard {
        &self.bit_boards[piece as usize]
    }
    fn get_bit_board_mut(&mut self, piece: Piece) -> &mut BitBoard {
        &mut self.bit_boards[piece as usize]
    }
}
