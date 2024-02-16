use std::sync::OnceLock;

use crate::{
    board::{
        bit_board::BitBoard,
        square::{Direction, Square, DIRECTIONS},
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
                ray.set(&move_to)
            }
        }
    }
    rays
}
pub fn all_rays() -> &'static [[BitBoard; 8]; 65] {
    static COMPUTATION: OnceLock<[[BitBoard; 8]; 65]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_all_rays())
}

fn iterate_combinations(squares: BitBoard) -> impl core::iter::Iterator<Item = BitBoard> {
    let mut next = Some(BitBoard::EMPTY);
    core::iter::from_fn(move || {
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

fn gen_rook_or_bishop(from: Square, blockers: &BitBoard, direction_offset: usize) -> BitBoard {
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

pub fn relevant_rook_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(0, 4))
}
pub fn relevant_bishop_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(4, 8))
}

fn magic_index(blockers: &BitBoard, magic: u64, shift: u64) -> usize {
    let hash = blockers.wrapping_mul(BitBoard::new(magic));
    (hash >> shift).as_usize()
}

fn init_lookup(
    size: usize,
    keys: [Key; 64],
    blockers: [BitBoard; 64],
    direction_offset: usize,
) -> Vec<BitBoard> {
    let mut lookup = vec![BitBoard::EMPTY; size];
    for square_index in 0..64 {
        let square = Square::from_index(square_index);

        let key = keys[square_index as usize];
        let blockers = blockers[square_index as usize];
        for blocker_combination in iterate_combinations(blockers) {
            let moves = gen_rook_or_bishop(square, &blocker_combination, direction_offset);
            lookup[key.offset + magic_index(&blocker_combination, key.magic, key.shift)] = moves;
        }
    }
    lookup
}

fn rook_lookup() -> &'static Vec<BitBoard> {
    static COMPUTATION: OnceLock<Vec<BitBoard>> = OnceLock::new();
    COMPUTATION
        .get_or_init(|| init_lookup(ROOK_TABLE_SIZE, ROOK_KEYS, *relevant_rook_blockers(), 0))
}

pub fn get_rook_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = ROOK_KEYS[square.usize()];
    rook_lookup()[key.offset + magic_index(&relevant_blockers, key.magic, key.shift)]
}

fn bishop_lookup() -> &'static Vec<BitBoard> {
    static COMPUTATION: OnceLock<Vec<BitBoard>> = OnceLock::new();
    COMPUTATION.get_or_init(|| {
        init_lookup(
            BISHOP_TABLE_SIZE,
            BISHOP_KEYS,
            *relevant_bishop_blockers(),
            4,
        )
    })
}

pub fn get_bishop_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = BISHOP_KEYS[square.usize()];
    bishop_lookup()[key.offset + magic_index(&relevant_blockers, key.magic, key.shift)]
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
            slider_lookup::{
                get_bishop_moves, get_rook_moves, relevant_bishop_blockers, relevant_rook_blockers,
                Key,
            },
        },
    };

    use super::{all_blockers, gen_rook_or_bishop, iterate_combinations, magic_index};

    use rand_chacha::rand_core::{RngCore, SeedableRng};

    #[derive(Debug)]
    struct TableFillError;
    fn fill_magic_table(
        square: Square,
        blockers: BitBoard,
        magic: u64,
        index_bits: u64,
        direction_offset: usize,
    ) -> Result<Vec<BitBoard>, TableFillError> {
        let mut table = vec![BitBoard::EMPTY; 1 << index_bits];
        for blocker_combination in iterate_combinations(blockers) {
            let moves = gen_rook_or_bishop(square, &blocker_combination, direction_offset);
            let table_entry = &mut table[magic_index(&blocker_combination, magic, 64 - index_bits)];
            if table_entry.is_empty() {
                *table_entry = moves;
            } else if *table_entry != moves {
                return Err(TableFillError);
            }
        }
        Ok(table)
    }

    #[test]
    fn find_rook_magics() {
        let mut random = rand_chacha::ChaCha20Rng::seed_from_u64(1);

        let mut length = 0;

        let mut keys = [Key {
            magic: 0,
            shift: 0,
            offset: 0,
        }; 64];

        for square_index in 0..64 {
            let square = Square::from_index(square_index);

            let blockers = relevant_rook_blockers()[square_index as usize];
            let index_bits = blockers.count() as u64;
            loop {
                let magic = random.next_u64() & random.next_u64() & random.next_u64();
                let filled = fill_magic_table(square, blockers, magic, index_bits, 0);
                if let Ok(filled) = filled {
                    keys[square_index as usize] = Key {
                        magic: magic,
                        shift: 64 - index_bits,
                        offset: length,
                    };

                    length += filled.len();
                    break;
                }
            }
        }
        println!("{keys:?} {length}");
    }
    #[test]
    fn find_bishop_magics() {
        let mut random = rand_chacha::ChaCha20Rng::seed_from_u64(1);

        let mut length = 0;

        let mut keys = [Key {
            magic: 0,
            shift: 0,
            offset: 0,
        }; 64];

        for square_index in 0..64 {
            let square = Square::from_index(square_index);
            let blockers = relevant_bishop_blockers()[square_index as usize];
            let index_bits = blockers.count() as u64;
            loop {
                let magic = random.next_u64() & random.next_u64() & random.next_u64();
                let filled = fill_magic_table(square, blockers, magic, index_bits, 4);
                if let Ok(filled) = filled {
                    keys[square_index as usize] = Key {
                        magic: magic,
                        shift: 64 - index_bits,
                        offset: length,
                    };

                    length += filled.len();
                    break;
                }
            }
        }
        println!("{keys:?} {length}");
    }

    #[test]
    fn blocker_combinations() {
        let d4 = Square::from_notation("d4");
        let blockers = all_blockers(d4, &DIRECTIONS, &SQUARES_FROM_EDGE[d4.usize()]);
        let expected_number_of_combinations = 1 << blockers.count();
        println!("{blockers}");
        let mut number_of_combinations = 0;
        for _ in iterate_combinations(blockers) {
            number_of_combinations += 1;
        }
        assert_eq!(number_of_combinations, expected_number_of_combinations)
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
        assert_eq!(legal_moves.count(), 25)
    }
}
