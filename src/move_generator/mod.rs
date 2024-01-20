use crate::Board;
use crate::board::square::Square;

pub mod move_data;

use move_data::Move;

pub fn gen_moves(board: &Board) -> Vec<Move> {
    let mut moves = vec![];
    moves.push(Move::new(Square::from_index(0), Square::from_index(1)));
    moves
}
