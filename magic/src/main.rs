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
    index_bits: u8,
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

fn find_magics(relevant_blockers: &[BitBoard; 64], direction_offset: usize) {
    let mut random = rand_chacha::ChaCha20Rng::seed_from_u64(420);

    let mut best_magics = [0; 64];
    let mut best_index_bits = [0; 64];
    let mut keys = [Key {
        magic: 0,
        shift: 0,
        offset: 0,
    }; 64];
    let mut length = 0;

    for square_index in 0..64 {
        let square = Square::from_index(square_index as i8);

        let blockers = relevant_blockers[square_index];
        let index_bits = blockers.count() as u8;
        best_index_bits[square_index] = index_bits;

        loop {
            let magic = random.next_u64() & random.next_u64() & random.next_u64();
            let filled = fill_magic_table(square, blockers, magic, index_bits, direction_offset);
            if let Ok(filled) = filled {
                keys[square_index] = Key {
                    magic,
                    shift: 64 - index_bits,
                    offset: length,
                };
                best_magics[square_index] = magic;

                length += filled.len();
                break;
            }
        }
    }
    println!("{keys:?} {length}");

    loop {
        let mut did_improve = false;

        length = 0;

        for square_index in 0..64 {
            let square = Square::from_index(square_index as i8);

            let blockers = relevant_blockers[square_index];
            let previous_index_bits = best_index_bits[square_index];
            let index_bits = previous_index_bits - 1;

            let magic = random.next_u64() & random.next_u64() & random.next_u64();
            let filled = fill_magic_table(square, blockers, magic, index_bits, direction_offset);
            if let Ok(filled) = filled {
                did_improve = true;

                keys[square_index] = Key {
                    magic,
                    shift: 64 - index_bits,
                    offset: length,
                };
                best_magics[square_index] = magic;
                best_index_bits[square_index] = index_bits;

                length += filled.len();
                continue;
            }

            keys[square_index].offset = length;
            length += 1 << previous_index_bits;
        }
        if did_improve {
            println!("{keys:?} {length}");
        }
    }
}

fn main() {
    if true {
        find_magics(relevant_rook_blockers(), 0);
    } else {
        find_magics(relevant_bishop_blockers(), 4);
    }
}
