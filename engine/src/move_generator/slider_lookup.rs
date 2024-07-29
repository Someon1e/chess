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

const fn all_rays(square_index: usize) -> [u64; 8] {
    let from = Square::from_index(square_index as i8);

    let mut rays = [0; 8];
    let mut direction_index = 0;
    loop {
        let direction = DIRECTIONS[direction_index];
        let distance_from_edge = SQUARES_FROM_EDGE[square_index][direction_index];

        let mut count = 0;
        let mut bit_board = 0;
        loop {
            if count == distance_from_edge {
                break;
            }

            count += 1;

            let move_to = from.offset(direction * count);
            bit_board |= 1 << move_to.index();
        }
        rays[direction_index] = bit_board;

        direction_index += 1;
        if direction_index == DIRECTIONS.len() {
            break;
        }
    }

    rays
}

const ALL_RAYS: [[u64; 8]; 65] = {
    let mut rays = [[0; 8]; 65];
    let mut index = 0;
    loop {
        rays[index] = all_rays(index);

        index += 1;
        if index == 64 {
            break;
        }
    }
    rays
};

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
#[rustfmt::skip]
pub const fn rook_or_bishop_blockers(from: Square, direction_offset: usize) -> BitBoard {
    let square_index = from.usize();
    let squares_from_edge = SQUARES_FROM_EDGE[square_index];

    let rays = ALL_RAYS[square_index];

    BitBoard::new(
        rays[direction_offset] & !(1 << from.offset(DIRECTIONS[direction_offset] * squares_from_edge[direction_offset]).index())
            | rays[direction_offset + 1] & !(1 << from.offset(DIRECTIONS[direction_offset + 1] * squares_from_edge[direction_offset + 1]).index())
            | rays[direction_offset + 2] & !(1 << from.offset(DIRECTIONS[direction_offset + 2] * squares_from_edge[direction_offset + 2]).index())
            | rays[direction_offset + 3] & !(1 << from.offset(DIRECTIONS[direction_offset + 3] * squares_from_edge[direction_offset + 3]).index()),
    )
}

/// Computes rook (`direction_offset` = 0) or bishop (`direction_offset` = 4) moves.
#[must_use]
pub fn gen_rook_or_bishop(from: Square, blockers: &BitBoard, direction_offset: usize) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    let rays = &ALL_RAYS[from.usize()];

    let mut ray = BitBoard::new(rays[direction_offset]);
    let blocker = ray & *blockers;
    ray ^= BitBoard::new(ALL_RAYS[blocker.first_square().usize()][direction_offset]);
    moves |= ray;

    let mut ray = BitBoard::new(rays[direction_offset + 1]);
    let blocker = ray & *blockers;
    ray ^= BitBoard::new(ALL_RAYS[blocker.first_square().usize()][direction_offset + 1]);
    moves |= ray;

    let mut ray = BitBoard::new(rays[direction_offset + 2]);
    let blocker = ray & *blockers;
    ray ^= BitBoard::new(
        ALL_RAYS[(blocker | BitBoard::new(1)).last_square().usize()][direction_offset + 2],
    );
    moves |= ray;

    let mut ray = BitBoard::new(rays[direction_offset + 3]);
    let blocker = ray & *blockers;
    ray ^= BitBoard::new(
        ALL_RAYS[(blocker | BitBoard::new(1)).last_square().usize()][direction_offset + 3],
    );
    moves |= ray;

    moves
}

const fn calculate_blockers_for_each_square(direction_offset: usize) -> [BitBoard; 64] {
    let mut blockers = [BitBoard::EMPTY; 64];
    let mut index = 0;
    loop {
        blockers[index as usize] =
            rook_or_bishop_blockers(Square::from_index(index), direction_offset);
        index += 1;
        if index == 64 {
            break;
        }
    }
    blockers
}

/// Lookup table for relevant rook blockers.
pub const RELEVANT_ROOK_BLOCKERS: [BitBoard; 64] = calculate_blockers_for_each_square(0);

/// Lookup table for relevant bishop blockers.
pub const RELEVANT_BISHOP_BLOCKERS: [BitBoard; 64] = calculate_blockers_for_each_square(4);

fn init_lookup(
    size: usize,
    keys: &[Key; 64],
    blockers: &[BitBoard; 64],
    direction_offset: usize,
) -> Box<[BitBoard]> {
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
    lookup.into_boxed_slice()
}

#[must_use]
fn rook_lookup() -> &'static [BitBoard; ROOK_TABLE_SIZE] {
    static COMPUTATION: OnceLock<Box<[BitBoard; ROOK_TABLE_SIZE]>> = OnceLock::new();
    COMPUTATION.get_or_init(|| {
        init_lookup(ROOK_TABLE_SIZE, &ROOK_KEYS, &RELEVANT_ROOK_BLOCKERS, 0)
            .try_into()
            .unwrap()
    })
}

/// Returns possible rook moves at a square given blockers which are relevant.
#[must_use]
pub fn get_rook_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = ROOK_KEYS[square.usize()];
    rook_lookup()[key.offset as usize + relevant_blockers.magic_index(key.magic, key.shift)]
}

#[must_use]
fn bishop_lookup() -> &'static [BitBoard; BISHOP_TABLE_SIZE] {
    static COMPUTATION: OnceLock<Box<[BitBoard; BISHOP_TABLE_SIZE]>> = OnceLock::new();
    COMPUTATION.get_or_init(|| {
        init_lookup(
            BISHOP_TABLE_SIZE,
            &BISHOP_KEYS,
            &RELEVANT_BISHOP_BLOCKERS,
            4,
        )
        .try_into()
        .unwrap()
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
            rook_or_bishop_blockers, RELEVANT_BISHOP_BLOCKERS, RELEVANT_ROOK_BLOCKERS,
        },
    };

    #[test]
    fn blocker_combinations() {
        let d4 = Square::from_notation("d4");
        let rook_blockers = rook_or_bishop_blockers(d4, 0);
        let bishop_blockers = rook_or_bishop_blockers(d4, 4);

        let blockers = rook_blockers | bishop_blockers;
        println!("{blockers}");

        let expected_number_of_combinations = 1 << blockers.count();

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

        let rook_moves = get_rook_moves(d4, blockers & RELEVANT_ROOK_BLOCKERS[d4.usize()]);
        let bishop_moves = get_bishop_moves(d4, blockers & RELEVANT_BISHOP_BLOCKERS[d4.usize()]);

        let legal_moves = rook_moves | bishop_moves;
        println!("{}", legal_moves);
        assert_eq!(legal_moves.count(), 25);
    }
}
