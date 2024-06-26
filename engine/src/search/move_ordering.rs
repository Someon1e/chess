use crate::{
    board::bit_board::BitBoard,
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

use super::{encoded_move::EncodedMove, Search};

pub type MoveGuessNum = i32;

#[derive(Clone, Copy)]
pub struct MoveGuess {
    guess: MoveGuessNum,
    pub move_data: EncodedMove,
}

const MAX_LEGAL_MOVES: usize = 218;
const MAX_CAPTURES: usize = 74;

const HASH_MOVE_BONUS: MoveGuessNum = MoveGuessNum::MAX;
const QUEEN_PROMOTION_BONUS: MoveGuessNum = 50_000_000;
const CAPTURE_BONUS: MoveGuessNum = 50_000_000;
const KILLER_MOVE_BONUS: MoveGuessNum = 30_000_000;
const KNIGHT_PROMOTION_BONUS: MoveGuessNum = 20_000_000;
const ROOK_PROMOTION_BONUS: MoveGuessNum = 0;
const BISHOP_PROMOTION_BONUS: MoveGuessNum = 0;

const PIECE_VALUES: [MoveGuessNum; 12] = [
    100, // Pawn
    300, // Knight
    320, // Bishop
    500, // Rook
    900, // Queen
    0,   // King
    100, // Pawn
    300, // Knight
    320, // Bishop
    500, // Rook
    900, // Queen
    0,   // King
];

pub struct MoveOrderer {}
impl MoveOrderer {
    fn guess_move_value(
        search: &Search,
        enemy_pawn_attacks: BitBoard,
        move_data: Move,
    ) -> MoveGuessNum {
        let moving_from = move_data.from;
        let moving_to = move_data.to;

        let mut score = 0;
        match move_data.flag {
            Flag::EnPassant | Flag::Castle => {
                return search.quiet_history[usize::from(search.board.white_to_move)]
                    [moving_from.usize() + moving_to.usize() * 64]
            }

            Flag::BishopPromotion => return BISHOP_PROMOTION_BONUS,
            Flag::KnightPromotion => return KNIGHT_PROMOTION_BONUS,
            Flag::RookPromotion => return ROOK_PROMOTION_BONUS,
            Flag::QueenPromotion => return QUEEN_PROMOTION_BONUS,

            Flag::PawnTwoUp | Flag::None => {}
        }

        let moving_piece = search.board.friendly_piece_at(moving_from).unwrap();

        // This won't consider en passant
        if let Some(capturing) = search.board.enemy_piece_at(moving_to) {
            let mut potential_value_loss = PIECE_VALUES[moving_piece as usize];
            if !enemy_pawn_attacks.get(&moving_to) {
                potential_value_loss /= 2;
            }
            score += CAPTURE_BONUS;
            score += PIECE_VALUES[capturing as usize];
            score -= potential_value_loss;
        } else {
            score += MoveGuessNum::from(
                search.quiet_history[usize::from(search.board.white_to_move)]
                    [moving_from.usize() + moving_to.usize() * 64],
            );
        }
        score
    }

    pub fn put_highest_guessed_move(
        move_guesses: &mut [MoveGuess],
        unsorted_index: usize,
        move_count: usize,
    ) -> MoveGuess {
        let (mut index_of_highest_move, mut highest_guess) =
            (unsorted_index, move_guesses[unsorted_index].guess);

        // Find highest guessed unsorted move
        for (index, item) in move_guesses
            .iter()
            .enumerate()
            .take(move_count)
            .skip(unsorted_index)
        {
            // Iterate part of the array that is unsorted
            let guess = item.guess;
            if guess > highest_guess {
                // New highest guess
                highest_guess = guess;
                index_of_highest_move = index;
            }
        }

        if index_of_highest_move != unsorted_index {
            // Swap highest with first unsorted
            move_guesses.swap(index_of_highest_move, unsorted_index);
        }

        move_guesses[unsorted_index]
    }

    fn guess_capture_value(search: &Search, move_data: Move) -> MoveGuessNum {
        let mut score = match move_data.flag {
            Flag::EnPassant => return 0,
            Flag::BishopPromotion => return -1,
            Flag::RookPromotion => return -1,

            Flag::KnightPromotion => 1300,
            Flag::QueenPromotion => 1900,

            Flag::None => 0,

            _ => unreachable!(),
        };

        let capturing = search.board.enemy_piece_at(move_data.to).unwrap();
        let capturing_value = PIECE_VALUES[capturing as usize];

        score += capturing_value;

        let moving_piece = search.board.friendly_piece_at(move_data.from).unwrap();
        score -= PIECE_VALUES[moving_piece as usize];

        score
    }

    pub fn get_move_guesses_captures_only(
        search: &Search,
        move_generator: &MoveGenerator,
    ) -> ([MoveGuess; MAX_CAPTURES], usize) {
        let mut move_guesses = [MoveGuess {
            move_data: EncodedMove::NONE,
            guess: 0,
        }; MAX_CAPTURES];

        let mut index = 0;
        move_generator.gen(
            &mut |move_data| {
                let encoded = EncodedMove::new(move_data);
                move_guesses[index] = MoveGuess {
                    move_data: encoded,
                    guess: Self::guess_capture_value(search, move_data),
                };
                index += 1;
            },
            true,
        );

        (move_guesses, index)
    }

    pub fn get_move_guesses(
        search: &Search,
        move_generator: &MoveGenerator,
        hash_move: EncodedMove,
        killer_move: EncodedMove,
    ) -> ([MoveGuess; MAX_LEGAL_MOVES], usize) {
        let mut move_guesses = [MoveGuess {
            move_data: EncodedMove::NONE,
            guess: 0,
        }; MAX_LEGAL_MOVES];

        let mut index = 0;
        move_generator.gen(
            &mut |move_data| {
                let encoded = EncodedMove::new(move_data);
                move_guesses[index] = MoveGuess {
                    move_data: encoded,
                    guess: if encoded == hash_move {
                        HASH_MOVE_BONUS
                    } else {
                        Self::guess_move_value(
                            search,
                            move_generator.enemy_pawn_attacks(),
                            move_data,
                        ) + if encoded == killer_move {
                            KILLER_MOVE_BONUS
                        } else {
                            0
                        }
                    },
                };
                index += 1;
            },
            false,
        );

        (move_guesses, index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{square::Square, Board},
        move_generator::{
            move_data::{Flag, Move},
            MoveGenerator,
        },
        search::{encoded_move::EncodedMove, move_ordering::MoveOrderer, Search},
    };

    #[test]
    fn move_ordering_works() {
        let board = Board::from_fen("8/P6p/6r1/1q1n4/2P3R1/8/2K2k2/8 w - - 0 1");
        let move_generator = MoveGenerator::new(&board);

        let (mut move_guesses, move_count) = MoveOrderer::get_move_guesses(
            &Search::new(board),
            &move_generator,
            EncodedMove::NONE,
            EncodedMove::NONE,
        );

        let mut index = 0;
        let mut next_move = || {
            let move_guess =
                MoveOrderer::put_highest_guessed_move(&mut move_guesses, index, move_count);
            println!("{index} {} {}", move_guess.move_data, move_guess.guess);
            index += 1;
            (move_guess.move_data, index != move_count)
        };

        assert!(
            next_move().0.decode()
                == Move {
                    from: Square::from_notation("c4"),
                    to: Square::from_notation("b5"),
                    flag: Flag::None
                }
        );
        while next_move().1 {}
    }
}
