use crate::board::piece::{self, Piece};
use crate::board::square::{Square, DIRECTIONS};
use crate::board::Board;

pub mod move_data;
mod precomputed;

use move_data::Move;

use self::precomputed::PrecomputedData;

pub struct PsuedoLegalMoveGenerator<'a> {
    board: &'a mut Board,
    precomputed: PrecomputedData,
}

impl<'a> PsuedoLegalMoveGenerator<'a> {
    pub fn friendly_piece_at(&self, square: Square) -> Option<Piece> {
        if self.board.white_to_move {
            self.board.white_piece_at(square)
        } else {
            self.board.black_piece_at(square)
        }
    }
    pub fn enemy_piece_at(&self, square: Square) -> Option<Piece> {
        if self.board.white_to_move {
            self.board.black_piece_at(square)
        } else {
            self.board.white_piece_at(square)
        }
    }
    fn gen_pawn(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        let attacks = if let Piece::WhitePawn = piece {
            &self.precomputed.white_pawn_attacks_at_square[square.index() as usize]
        } else {
            &self.precomputed.black_pawn_attacks_at_square[square.index() as usize]
        };
        let pawn_up = if let Piece::WhitePawn = piece { 1 } else { -1 };

        for attack in attacks {
            if let Some(enemy) = self.enemy_piece_at(*attack) {
                moves.push(Move::new(
                    piece,
                    square,
                    *attack,
                    Some(enemy),
                    false,
                    false,
                    false,
                ))
            } else if let Some(en_passant_square) = self.board.game_state.en_passant_square {
                if en_passant_square == *attack {
                    let enemy = self.enemy_piece_at(en_passant_square.down(pawn_up)); // TODO: make this only check for pawns
                    moves.push(Move::new(piece, square, *attack, enemy, true, false, false));
                }
            }
        }

        if self.board.piece_at(square.up(pawn_up)).is_none() {
            moves.push(Move::new(
                piece,
                square,
                square.up(pawn_up),
                None,
                false,
                false,
                false,
            ));

            if ((self.board.white_to_move && square.rank() == 1) || square.rank() == 6)
                && self.board.piece_at(square.up(pawn_up * 2)).is_none()
            {
                moves.push(Move::new(
                    piece,
                    square,
                    square.up(pawn_up * 2),
                    None,
                    false,
                    true,
                    false,
                ))
            }
        }
    }
    fn gen_directional(
        &self,
        moves: &mut Vec<Move>,
        piece: Piece,
        square: Square,
        directions: &[i8],
        squares_from_edge: &[i8],
    ) {
        // TODO: test if this works
        for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
            for count in 1..=*distance_from_edge {
                let move_to = square.offset(direction * count);
                if self.friendly_piece_at(move_to).is_some() {
                    break;
                }
                let enemy = self.enemy_piece_at(move_to);
                moves.push(Move::new(
                    piece, square, move_to, enemy, false, false, false,
                ));
                if enemy.is_some() {
                    break;
                }
            }
        }
    }
    pub fn gen_bishop(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        self.gen_directional(
            moves,
            piece,
            square,
            &DIRECTIONS[4..8],
            &self.precomputed.squares_from_edge[square.index() as usize][4..8],
        )
    }
    pub fn gen_rook(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        self.gen_directional(
            moves,
            piece,
            square,
            &DIRECTIONS[0..4],
            &self.precomputed.squares_from_edge[square.index() as usize][0..4],
        )
    }
    pub fn gen_queen(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        self.gen_directional(
            moves,
            piece,
            square,
            &DIRECTIONS,
            &self.precomputed.squares_from_edge[square.index() as usize],
        )
    }
    pub fn gen_knight(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        for move_to in &self.precomputed.knight_moves_at_square[square.index() as usize] {
            if self.friendly_piece_at(*move_to).is_none() {
                let enemy = self.enemy_piece_at(*move_to);
                moves.push(Move::new(
                    piece, square, *move_to, enemy, false, false, false,
                ))
            }
        }
    }
    pub fn gen_king(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        // TODO: test if this works
        for move_to in &self.precomputed.king_moves_at_square[square.index() as usize] {
            if self.friendly_piece_at(*move_to).is_none() {
                let enemy = self.enemy_piece_at(*move_to);
                moves.push(Move::new(
                    piece, square, *move_to, enemy, false, false, false,
                ))
            }
        }

        let castling_rights = self.board.game_state.castling_rights;
        let (king_side, queen_side) = if self.board.white_to_move {
            (
                castling_rights.get_white_king_side(),
                castling_rights.get_white_queen_side(),
            )
        } else {
            (
                castling_rights.get_black_king_side(),
                castling_rights.get_black_queen_side(),
            )
        };
        if king_side {
            let move_to = square.right(2);
            if self.board.piece_at(square.right(1)).is_none()
                && self.board.piece_at(move_to).is_none()
            {
                moves.push(Move::new(piece, square, move_to, None, false, false, true))
            }
        }
        if queen_side {
            let move_to = square.left(2);
            if self.board.piece_at(square.left(1)).is_none()
                && self.board.piece_at(move_to).is_none()
            {
                moves.push(Move::new(piece, square, move_to, None, false, false, true))
            }
        }
    }
    pub fn new(board: &'a mut Board) -> Self {
        let precomputed = PrecomputedData::compute();
        Self { board, precomputed }
    }
    pub fn board(&mut self) -> &mut Board {
        self.board
    }

    pub fn gen(&self, moves: &mut Vec<Move>) {
        let pieces = if self.board.white_to_move {
            piece::WHITE_PIECES
        } else {
            piece::BLACK_PIECES
        };
        for piece in pieces {
            let mut bit_board = *self.board.get_bit_board(piece);
            while !bit_board.is_empty() {
                let square = bit_board.pop_square();
                match piece {
                    Piece::WhitePawn | Piece::BlackPawn => self.gen_pawn(moves, piece, square),
                    Piece::WhiteKnight | Piece::BlackKnight => {
                        self.gen_knight(moves, piece, square)
                    }
                    Piece::WhiteBishop | Piece::BlackBishop => {
                        self.gen_bishop(moves, piece, square)
                    }
                    Piece::WhiteRook | Piece::BlackRook => self.gen_rook(moves, piece, square),
                    Piece::WhiteQueen | Piece::BlackQueen => self.gen_queen(moves, piece, square),
                    Piece::WhiteKing | Piece::BlackKing => self.gen_king(moves, piece, square),
                }
            }
        }
    }
}
