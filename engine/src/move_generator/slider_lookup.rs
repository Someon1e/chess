use std::sync::OnceLock;

use crate::{
    board::{
        bit_board::BitBoard,
        square::{Square, DIRECTIONS},
    },
    move_generator::slider_keys::{BISHOP_TABLE_SIZE, ROOK_KEYS, ROOK_TABLE_SIZE},
};

use super::{
    precomputed::SQUARES_FROM_EDGE,
    slider_keys::{Key, BISHOP_KEYS},
};

fn calculate_all_rays() -> [[BitBoard; 8]; 65] {
    let mut rays = [[BitBoard::EMPTY; 8]; 65];
    for square_index in 0..64 {
        let from = Square::from_index(square_index as i8);
        for (direction_index, (direction, distance_from_edge)) in DIRECTIONS
            .iter()
            .zip(SQUARES_FROM_EDGE[square_index])
            .enumerate()
        {
            let ray = &mut rays[square_index][direction_index];
            for count in 1..=distance_from_edge {
                let move_to = from.offset(direction * count);
                ray.set(&move_to);
            }
        }
    }
    rays
}

/// Lookup table for all eight rays at a square.
pub fn all_rays() -> &'static [[BitBoard; 8]; 65] {
    static COMPUTATION: OnceLock<[[BitBoard; 8]; 65]> = OnceLock::new();
    COMPUTATION.get_or_init(calculate_all_rays)
}

/// # Panics
///
/// Should not panic.
pub fn iterate_combinations(squares: BitBoard) -> impl core::iter::Iterator<Item = BitBoard> {
    let mut next = Some(BitBoard::EMPTY);
    core::iter::from_fn(move || {
        let result = next;
        next = Some(next?.carry_rippler(squares));
        if next.unwrap() == BitBoard::EMPTY {
            next = None;
        }
        result
    })
}

/// Finds relevant rook (`direction_offset` = 0) or bishop (`direction_offset` = 4) blockers.
#[must_use]
pub fn rook_or_bishop_blockers(from: Square, direction_offset: usize) -> BitBoard {
    let square_index = from.usize();
    let squares_from_edge = SQUARES_FROM_EDGE[square_index];

    let rays = &all_rays()[square_index];

    rays[direction_offset]
        & !from
            .offset(DIRECTIONS[direction_offset] * squares_from_edge[direction_offset])
            .bit_board()
        | rays[direction_offset + 1]
            & !from
                .offset(DIRECTIONS[direction_offset + 1] * squares_from_edge[direction_offset + 1])
                .bit_board()
        | rays[direction_offset + 2]
            & !from
                .offset(DIRECTIONS[direction_offset + 2] * squares_from_edge[direction_offset + 2])
                .bit_board()
        | rays[direction_offset + 3]
            & !from
                .offset(DIRECTIONS[direction_offset + 3] * squares_from_edge[direction_offset + 3])
                .bit_board()
}

/// Computes rook (`direction_offset` = 0) or bishop (`direction_offset` = 4) moves.
#[must_use]
pub fn gen_rook_or_bishop(from: Square, blockers: &BitBoard, direction_offset: usize) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    let rays = &all_rays()[from.usize()];

    let mut ray = rays[direction_offset];
    let blocker = ray & *blockers;
    ray ^= all_rays()[blocker.first_square().usize()][direction_offset];
    moves |= ray;

    let mut ray = rays[direction_offset + 1];
    let blocker = ray & *blockers;
    ray ^= all_rays()[blocker.first_square().usize()][direction_offset + 1];
    moves |= ray;

    let mut ray = rays[direction_offset + 2];
    let blocker = ray & *blockers;
    ray ^= all_rays()[(blocker | BitBoard::new(1)).last_square().usize()][direction_offset + 2];
    moves |= ray;

    let mut ray = rays[direction_offset + 3];
    let blocker = ray & *blockers;
    ray ^= all_rays()[(blocker | BitBoard::new(1)).last_square().usize()][direction_offset + 3];
    moves |= ray;

    moves
}

fn calculate_blockers_for_each_square(direction_offset: usize) -> [BitBoard; 64] {
    let mut blockers = [BitBoard::EMPTY; 64];
    for square_index in 0..64 {
        blockers[square_index as usize] =
            rook_or_bishop_blockers(Square::from_index(square_index), direction_offset);
    }
    blockers
}

/// Lookup table for relevant rook blockers.
pub fn relevant_rook_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(0))
}

/// Lookup table for relevant bishop blockers.
pub fn relevant_bishop_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(4))
}

fn init_lookup(
    size: usize,
    keys: &[Key; 64],
    blockers: &[BitBoard; 64],
    direction_offset: usize,
) -> Vec<BitBoard> {
    let mut lookup = vec![BitBoard::EMPTY; size];
    for square_index in 0..64 {
        let square = Square::from_index(square_index);

        let key = keys[square_index as usize];
        let blockers = blockers[square_index as usize];
        for blocker_combination in iterate_combinations(blockers) {
            let moves = gen_rook_or_bishop(square, &blocker_combination, direction_offset);
            lookup[key.offset as usize + blocker_combination.magic_index(key.magic, key.shift)] =
                moves;
        }
    }
    lookup
}

#[must_use]
fn rook_lookup() -> &'static Vec<BitBoard> {
    static COMPUTATION: OnceLock<Vec<BitBoard>> = OnceLock::new();
    COMPUTATION
        .get_or_init(|| init_lookup(ROOK_TABLE_SIZE, &ROOK_KEYS, relevant_rook_blockers(), 0))
}

/// Returns possible rook moves at a square given blockers which are relevant.
#[must_use]
pub fn get_rook_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = ROOK_KEYS[square.usize()];
    rook_lookup()[key.offset as usize + relevant_blockers.magic_index(key.magic, key.shift)]
}

#[must_use]
fn bishop_lookup() -> &'static Vec<BitBoard> {
    static COMPUTATION: OnceLock<Vec<BitBoard>> = OnceLock::new();
    COMPUTATION.get_or_init(|| {
        init_lookup(
            BISHOP_TABLE_SIZE,
            &BISHOP_KEYS,
            relevant_bishop_blockers(),
            4,
        )
    })
}

/// Returns possible bishop moves at a square given blockers which are relevant.
#[must_use]
pub fn get_bishop_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = BISHOP_KEYS[square.usize()];
    bishop_lookup()[key.offset as usize + relevant_blockers.magic_index(key.magic, key.shift)]
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{bit_board::BitBoard, square::Square},
        move_generator::slider_lookup::{
            gen_rook_or_bishop, get_bishop_moves, get_rook_moves, iterate_combinations,
            relevant_bishop_blockers, relevant_rook_blockers, rook_or_bishop_blockers,
        },
    };

    #[test]
    fn blocker_combinations() {
        let d4 = Square::from_notation("d4");
        let rook_blockers = rook_or_bishop_blockers(d4, 0);
        let bishop_blockers = rook_or_bishop_blockers(d4, 4);
        let blockers = rook_blockers | bishop_blockers;
        let expected_number_of_combinations = 1 << blockers.count();
        println!("{blockers}");
        let mut number_of_combinations = 0;
        for _ in iterate_combinations(blockers) {
            number_of_combinations += 1;
        }
        assert_eq!(number_of_combinations, expected_number_of_combinations);
    }
    #[test]
    fn move_lookup_slow() {
        let d4 = Square::from_notation("d4");
        let mut blockers = BitBoard::EMPTY;
        blockers.set(&Square::from_notation("h8"));
        println!("{}", blockers.last_square());
        println!("{}", blockers.first_square());

        let rook_moves = gen_rook_or_bishop(d4, &blockers, 4);

        println!("{}", rook_moves);
    }
    #[test]
    fn move_lookup() {
        let d4 = Square::from_notation("d4");
        let mut blockers = BitBoard::EMPTY;
        blockers.set(&Square::from_notation("f4"));

        let rook_moves = get_rook_moves(d4, blockers & relevant_rook_blockers()[d4.usize()]);
        let bishop_moves = get_bishop_moves(d4, blockers & relevant_bishop_blockers()[d4.usize()]);

        let legal_moves = rook_moves | bishop_moves;
        println!("{}", legal_moves);
        assert_eq!(legal_moves.count(), 25);
    }
}
