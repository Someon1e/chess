use crate::{
    board::bit_board::BitBoard,
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

use super::{encoded_move::EncodedMove, Search};

#[derive(Clone, Copy)]
pub struct MoveGuess {
    guess: i32,
    pub move_data: EncodedMove,
}

const MAX_LEGAL_MOVES: usize = 218;
const MAX_CAPTURES: usize = 74;

/*
    PV-move of the principal variation from the previous iteration of an iterative deepening framework for the leftmost path, often implicitly done by 2.
    Hash move from hash tables
    Winning captures/promotions
    Equal captures/promotions
    Killer moves (non capture), often with mate killers first
    Non-captures sorted by history heuristic and that like
    Losing captures (* but see below
*/
const HASH_MOVE_BONUS: i32 = 100000;
const WINNING_CAPTURE_BONUS: i32 = 2000;
const PROMOTION_BONUS: i32 = 2000;
const EQUAL_CAPTURE_BONUS: i32 = 1000;
const KILLER_MOVE_BONUS: i32 = 500;
const LOSING_CAPTURE_BONUS: i32 = 50;

const PIECE_VALUES: [i32; 12] = [100, 300, 320, 500, 900, 0, 100, 300, 320, 500, 900, 0];

pub struct MoveOrderer {}
impl MoveOrderer {
    fn guess_move_value(search: &Search, enemy_pawn_attacks: BitBoard, move_data: Move) -> i32 {
        let moving_from = move_data.from;
        let moving_to = move_data.to;

        let mut score = 0;
        match move_data.flag {
            Flag::EnPassant => {
                return i32::from(
                    search.history_heuristic[usize::from(search.board.white_to_move)]
                        [moving_from.usize()][moving_to.usize()],
                )
            }
            Flag::PawnTwoUp => {}
            Flag::Castle => {
                return i32::from(
                    search.history_heuristic[usize::from(search.board.white_to_move)]
                        [moving_from.usize()][moving_to.usize()],
                )
            }

            Flag::BishopPromotion => return PROMOTION_BONUS + 300,
            Flag::KnightPromotion => return PROMOTION_BONUS + 400,
            Flag::RookPromotion => return PROMOTION_BONUS + 300,
            Flag::QueenPromotion => return PROMOTION_BONUS + 800,

            Flag::None => {}
        }

        let moving_piece = search.board.friendly_piece_at(moving_from).unwrap();

        // This won't take into account en passant
        if let Some(capturing) = search.board.enemy_piece_at(moving_to) {
            let mut potential_value_loss = PIECE_VALUES[moving_piece as usize];
            if !enemy_pawn_attacks.get(&moving_to) {
                potential_value_loss /= 2;
            }
            let score_difference = PIECE_VALUES[capturing as usize] - potential_value_loss;
            score += score_difference;
            if score_difference.is_positive() {
                score += WINNING_CAPTURE_BONUS;
            } else if score_difference.is_negative() {
                score += LOSING_CAPTURE_BONUS;
            } else {
                score += EQUAL_CAPTURE_BONUS;
            }
        } else {
            score += i32::from(
                search.history_heuristic[usize::from(search.board.white_to_move)]
                    [moving_from.usize()][moving_to.usize()],
            );

            if enemy_pawn_attacks.get(&moving_to) {
                score -= 50;
            }
        }
        score
    }

    pub fn put_highest_guessed_move(
        move_guesses: &mut [MoveGuess],
        unsorted_index: usize,
        move_count: usize
    ) -> MoveGuess {
        let (mut index_of_highest_move, mut highest_guess) =
            (unsorted_index, move_guesses[unsorted_index].guess);
        for index in unsorted_index..move_count {
            let guess = move_guesses[index].guess;
            if guess > highest_guess {
                highest_guess = guess;
                index_of_highest_move = index;
            }
        }
        if index_of_highest_move != unsorted_index {
            move_guesses.swap(index_of_highest_move, unsorted_index);
        }
        move_guesses[unsorted_index]
    }

    fn guess_capture_value(search: &Search, move_data: Move) -> i32 {
        let mut score = match move_data.flag {
            Flag::EnPassant => return 0,

            Flag::BishopPromotion => 1400,
            Flag::KnightPromotion => 1500,
            Flag::RookPromotion => 1400,
            Flag::QueenPromotion => 1900,

            Flag::None => 0,

            _ => unreachable!(),
        };

        let moving_from = move_data.from;
        let moving_to = move_data.to;

        let moving_piece = search.board.friendly_piece_at(moving_from).unwrap();
        let capturing = search.board.enemy_piece_at(moving_to).unwrap();

        score += PIECE_VALUES[capturing as usize] - PIECE_VALUES[moving_piece as usize];

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
                        )
                    } + if encoded == killer_move {
                        KILLER_MOVE_BONUS
                    } else {
                        0
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
        let mut board = Board::from_fen("8/P6p/6r1/1q1n4/2P3R1/8/2K2k2/8 w - - 0 1");
        let move_generator = MoveGenerator::new(&board);
        let (mut move_guesses, move_count) = MoveOrderer::get_move_guesses(
            &Search::new(&mut board),
            &move_generator,
            EncodedMove::NONE,
            EncodedMove::NONE,
        );
        let mut index = 0;
        let mut next_move = || {
            let move_guess = MoveOrderer::put_highest_guessed_move(&mut move_guesses, index, move_count);
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
        assert!(
            next_move().0.decode()
                == Move {
                    from: Square::from_notation("a7"),
                    to: Square::from_notation("a8"),
                    flag: Flag::QueenPromotion
                }
        );
        while next_move().1 {}
    }
}
