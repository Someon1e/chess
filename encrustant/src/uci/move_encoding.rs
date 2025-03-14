use crate::{
    board::{piece::Piece, square::Square, Board},
    move_generator::move_data::{Flag, Move},
    search::encoded_move::EncodedMove,
};

/// Encodes a move in uci notation.
#[must_use]
pub fn encode_move(move_data: Move) -> String {
    const NULL_MOVE: Move = EncodedMove::NONE.decode();
    if move_data == NULL_MOVE {
        return "0000".to_owned();
    }

    let mut encoded = String::with_capacity(4);
    encoded.push_str(&move_data.from.to_notation());
    encoded.push_str(&move_data.to.to_notation());

    match move_data.flag {
        Flag::QueenPromotion => encoded.push('q'),
        Flag::RookPromotion => encoded.push('r'),
        Flag::KnightPromotion => encoded.push('n'),
        Flag::BishopPromotion => encoded.push('b'),
        _ => {}
    }
    encoded
}

/// # Panics
///
/// Will panic if there is no friendly piece at `from`.
#[must_use]
pub fn decode_move(board: &Board, from: Square, to: Square, promotion: Flag) -> Move {
    let piece = board
        .friendly_piece_at(from)
        .expect("Tried to play illegal move");

    let mut flag = promotion;
    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
        if from.rank().abs_diff(to.rank()) == 2 {
            flag = Flag::PawnTwoUp;
        } else if board.game_state.en_passant_square == Some(to) {
            flag = Flag::EnPassant;
        }
    } else if (piece == Piece::BlackKing || piece == Piece::WhiteKing)
        && from.file().abs_diff(to.file()) > 1
    {
        flag = Flag::Castle;
    }

    Move { from, to, flag }
}
