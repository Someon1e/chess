use std::sync::OnceLock;

use fnv::FnvHashMap;

use crate::board::{
    bit_board::BitBoard,
    square::{Direction, Square, DIRECTIONS},
};

use super::precomputed::SQUARES_FROM_EDGE;

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

    let squares_from_edge = &SQUARES_FROM_EDGE[from.index() as usize];
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

fn calculate_blockers_for_each_square(
    direction_start_index: usize,
    direction_end_index: usize,
) -> [BitBoard; 64] {
    let mut blockers = [BitBoard::EMPTY; 64];
    for square_index in 0..64 {
        blockers[square_index as usize] = all_blockers(
            Square::from_index(square_index),
            &DIRECTIONS[direction_start_index..direction_end_index],
            &SQUARES_FROM_EDGE[square_index as usize][direction_start_index..direction_end_index],
        )
    }
    blockers
}

pub fn rook_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(0, 4))
}
pub fn bishop_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(4, 8))
}
pub fn rook_move_map() -> &'static Vec<FnvHashMap<BitBoard, BitBoard>> {
    static COMPUTATION: OnceLock<Vec<FnvHashMap<BitBoard, BitBoard>>> = OnceLock::new();
    COMPUTATION.get_or_init(|| make_legal_move_map(*rook_blockers(), 0, 4))
}
pub fn bishop_move_map() -> &'static Vec<FnvHashMap<BitBoard, BitBoard>> {
    static COMPUTATION: OnceLock<Vec<FnvHashMap<BitBoard, BitBoard>>> = OnceLock::new();
    COMPUTATION.get_or_init(|| make_legal_move_map(*bishop_blockers(), 4, 8))
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{
            bit_board::BitBoard,
            square::{Square, DIRECTIONS},
        },
        move_generator::{
            precomputed::SQUARES_FROM_EDGE,
            slider_lookup::{bishop_blockers, bishop_move_map, rook_blockers, rook_move_map},
        },
    };

    use super::{all_blockers, iterate_combinations};

    #[test]
    fn blocker_combinations() {
        let d4 = Square::from_notation("d4");
        let blockers = all_blockers(d4, &DIRECTIONS, &SQUARES_FROM_EDGE[d4.index() as usize]);
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

        let rook_moves = rook_move_map()[d4.index() as usize]
            [&(blockers & rook_blockers()[d4.index() as usize])];
        let bishop_moves = bishop_move_map()[d4.index() as usize]
            [&(blockers & bishop_blockers()[d4.index() as usize])];

        let legal_moves = rook_moves | bishop_moves;
        println!("{}", legal_moves);
        assert_eq!(legal_moves.count(), 25)
    }
}
