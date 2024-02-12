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
    direction_start_index: usize,
    direction_end_index: usize,
) -> Vec<BitBoard> {
    let mut lookup = vec![BitBoard::EMPTY; size];
    for square_index in 0..64 {
        let square = Square::from_index(square_index);

        let key = keys[square_index as usize];
        let blockers = all_blockers(
            square,
            &DIRECTIONS[direction_start_index..direction_end_index],
            &SQUARES_FROM_EDGE[square.index() as usize][direction_start_index..direction_end_index],
        );
        for blocker_combination in iterate_combinations(blockers) {
            let moves = gen_slider_moves(
                square,
                &blocker_combination,
                direction_start_index,
                direction_end_index,
            );
            lookup[key.offset + magic_index(&blocker_combination, key.magic, key.shift)] = moves;
        }
    }
    lookup
}

fn rook_lookup() -> &'static Vec<BitBoard> {
    static COMPUTATION: OnceLock<Vec<BitBoard>> = OnceLock::new();
    COMPUTATION.get_or_init(|| init_lookup(ROOK_TABLE_SIZE, ROOK_KEYS, 0, 4))
}

pub fn get_rook_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = ROOK_KEYS[square.index() as usize];
    rook_lookup()[key.offset + magic_index(&relevant_blockers, key.magic, key.shift)]
}

fn bishop_lookup() -> &'static Vec<BitBoard> {
    static COMPUTATION: OnceLock<Vec<BitBoard>> = OnceLock::new();
    COMPUTATION.get_or_init(|| init_lookup(BISHOP_TABLE_SIZE, BISHOP_KEYS, 4, 8))
}

pub fn get_bishop_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = BISHOP_KEYS[square.index() as usize];
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

    use super::{all_blockers, gen_slider_moves, iterate_combinations, magic_index};

    use rand_chacha::rand_core::{RngCore, SeedableRng};

    #[derive(Debug)]
    struct TableFillError;
    fn fill_magic_table(
        square: Square,
        blockers: BitBoard,
        magic: u64,
        index_bits: u64,
        direction_start_index: usize,
        direction_end_index: usize,
    ) -> Result<Vec<BitBoard>, TableFillError> {
        let mut table = vec![BitBoard::EMPTY; 1 << index_bits];
        for blocker_combination in iterate_combinations(blockers) {
            let moves = gen_slider_moves(
                square,
                &blocker_combination,
                direction_start_index,
                direction_end_index,
            );
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

            loop {
                let magic = random.next_u64() & random.next_u64() & random.next_u64();
                let blockers = relevant_rook_blockers()[square_index as usize];
                let index_bits = blockers.count() as u64;
                let filled = fill_magic_table(square, blockers, magic, index_bits, 0, 4);
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

            loop {
                let magic = random.next_u64() & random.next_u64() & random.next_u64();
                let blockers = relevant_bishop_blockers()[square_index as usize];
                let index_bits = blockers.count() as u64;
                let filled = fill_magic_table(square, blockers, magic, index_bits, 4, 8);
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

        let rook_moves =
            get_rook_moves(d4, blockers & relevant_rook_blockers()[d4.index() as usize]);
        let bishop_moves = get_bishop_moves(
            d4,
            blockers & relevant_bishop_blockers()[d4.index() as usize],
        );

        let legal_moves = rook_moves | bishop_moves;
        println!("{}", legal_moves);
        assert_eq!(legal_moves.count(), 25)
    }
}
