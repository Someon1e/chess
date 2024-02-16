use engine::{
    board::{bit_board::BitBoard, square::Square},
    move_generator::{
        slider_keys::Key,
        slider_lookup::{
            gen_rook_or_bishop, iterate_combinations, relevant_bishop_blockers,
            relevant_rook_blockers,
        },
    },
};

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
        let table_entry = &mut table[blocker_combination.magic_index(magic, 64 - index_bits)];
        if table_entry.is_empty() {
            *table_entry = moves;
        } else if *table_entry != moves {
            return Err(TableFillError);
        }
    }
    Ok(table)
}

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

fn main() {
    find_rook_magics();
    find_bishop_magics();
}
