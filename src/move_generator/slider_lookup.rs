use fnv::FnvHashMap;
use lazy_static::lazy_static;

use crate::board::{
    bit_board::BitBoard,
    square::{Direction, Square, DIRECTIONS},
};

use super::precomputed::PRECOMPUTED;

fn iterate_combinations(squares: BitBoard) -> impl std::iter::Iterator<Item = BitBoard> {
    let mut next = Some(BitBoard::EMPTY);
    std::iter::from_fn(move || {
        let result = next;
        next = Some((next?.wrapping_sub(squares)) & squares);
        if next.unwrap() == BitBoard::EMPTY {
            next = None
        }
        result
    })
}

fn all_blockers(
    from: Square,
    directions: &[Direction],
    squares_from_edge: &[Direction],
) -> BitBoard {
    let mut bit_board = BitBoard::EMPTY;
    for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
        for count in 1..=*distance_from_edge - 1 {
            let move_to = from.offset(direction * count);
            bit_board.set(&move_to)
        }
    }
    bit_board
}

fn gen_slider_moves(
    from: Square,
    blockers: &BitBoard,
    direction_start_index: usize,
    direction_end_index: usize,
) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    let squares_from_edge = &PRECOMPUTED.squares_from_edge[from.index() as usize];
    for index in direction_start_index..direction_end_index {
        let direction = DIRECTIONS[index];

        for count in 1..=squares_from_edge[index] {
            let move_to = from.offset(direction * count);

            moves.set(&move_to);
            if blockers.get(&move_to) {
                break;
            }
        }
    }
    moves
}

fn make_legal_move_map(
    square_blockers: [BitBoard; 64],
    direction_start_index: usize,
    direction_end_index: usize,
) -> Vec<FnvHashMap<BitBoard, BitBoard>> {
    let mut legal_moves_map: Vec<FnvHashMap<BitBoard, BitBoard>> = vec![FnvHashMap::default(); 64];
    for square_index in 0..64 {
        let from = Square::from_index(square_index);
        let blockers = square_blockers[from.index() as usize];
        for blocker_combination in iterate_combinations(blockers) {
            legal_moves_map[square_index as usize].insert(
                blocker_combination,
                gen_slider_moves(
                    from,
                    &blocker_combination,
                    direction_start_index,
                    direction_end_index,
                ),
            );
        }
    }
    legal_moves_map
}

fn get_blockers_for_each_square(
    direction_start_index: usize,
    direction_end_index: usize,
) -> [BitBoard; 64] {
    let mut blockers = [BitBoard::EMPTY; 64];
    for square_index in 0..64 {
        blockers[square_index as usize] = all_blockers(
            Square::from_index(square_index),
            &DIRECTIONS[direction_start_index..direction_end_index],
            &PRECOMPUTED.squares_from_edge[square_index as usize]
                [direction_start_index..direction_end_index],
        )
    }
    blockers
}

lazy_static! {
    pub static ref ROOK_BLOCKERS: [BitBoard; 64] = get_blockers_for_each_square(0, 4);
    pub static ref ROOK_MOVE_MAP: Vec<FnvHashMap<BitBoard, BitBoard>> =
        make_legal_move_map(*ROOK_BLOCKERS, 0, 4);
    pub static ref BISHOP_BLOCKERS: [BitBoard; 64] = get_blockers_for_each_square(4, 8);
    pub static ref BISHOP_MOVE_MAP: Vec<FnvHashMap<BitBoard, BitBoard>> =
        make_legal_move_map(*BISHOP_BLOCKERS, 4, 8);
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{
            bit_board::BitBoard,
            square::{Square, DIRECTIONS},
        },
        move_generator::{
            precomputed::PRECOMPUTED,
            slider_lookup::{BISHOP_BLOCKERS, BISHOP_MOVE_MAP, ROOK_BLOCKERS, ROOK_MOVE_MAP},
        },
    };

    use super::{all_blockers, iterate_combinations};

    #[test]
    fn blocker_combinations() {
        let d4 = Square::from_notation("d4");
        let blockers = all_blockers(
            d4,
            &DIRECTIONS,
            &PRECOMPUTED.squares_from_edge[d4.index() as usize],
        );
        let expected_number_of_combinations = 1 << blockers.count();
        println!("{blockers}");
        let mut number_of_combinations = 0;
        for _ in iterate_combinations(blockers) {
            number_of_combinations += 1;
        }
        assert_eq!(number_of_combinations, expected_number_of_combinations)
    }

    #[test]
    fn move_lookup() {
        let d4 = Square::from_notation("d4");
        let mut blockers = BitBoard::EMPTY;
        blockers.set(&Square::from_notation("f4"));

        let rook_moves =
            ROOK_MOVE_MAP[d4.index() as usize][&(blockers & ROOK_BLOCKERS[d4.index() as usize])];
        let bishop_moves = BISHOP_MOVE_MAP[d4.index() as usize]
            [&(blockers & BISHOP_BLOCKERS[d4.index() as usize])];

        let legal_moves = rook_moves | bishop_moves;
        println!("{}", legal_moves);
        assert_eq!(legal_moves.count(), 25)
    }
}
