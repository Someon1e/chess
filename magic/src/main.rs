use engine::{
    board::{bit_board::BitBoard, square::Square},
    move_generator::slider_lookup::{
        gen_rook_or_bishop, iterate_combinations, relevant_bishop_blockers, relevant_rook_blockers,
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

    for square_index in 0..64 {
        let square = Square::from_index(square_index as i8);

        let blockers = relevant_blockers[square_index];
        let index_bits = blockers.count() as u8;
        best_index_bits[square_index] = index_bits;

        loop {
            let magic = random.next_u64() & random.next_u64() & random.next_u64();
            let filled = fill_magic_table(square, blockers, magic, index_bits, direction_offset);
            if filled.is_ok() {
                best_index_bits[square_index] = index_bits;
                best_magics[square_index] = magic;

                break;
            }
        }
    }
    let mut length = 0;
    for (magic, index_bits) in best_magics.iter().zip(best_index_bits) {
        let shift = 64 - index_bits;
        println!("Key {{shift: {shift}, magic: {magic:#04X}, offset: {length}}},");
        length += 1 << index_bits;
    }
    println!("{length}");

    loop {
        let mut did_improve = false;

        for square_index in 0..64 {
            let square = Square::from_index(square_index as i8);

            let blockers = relevant_blockers[square_index];
            let previous_index_bits = best_index_bits[square_index];
            let index_bits = previous_index_bits - 1;

            let magic = match square_index {
                // Cheat, just use magics found online

                /* Rook
                48 => 0x48FFFE99FECFAA00,
                49 => 0x48FFFE99FECFAA00,
                50 => 0x497FFFADFF9C2E00,
                51 => 0x613FFFDDFFCE9200,
                52 => 0xffffffe9ffe7ce00,
                53 => 0xfffffff5fff3e600,
                54 => 0x0003ff95e5e6a4c0,
                55 => 0x510FFFF5F63C96A0,
                56 => 0xEBFFFFB9FF9FC526,
                57 => 0x61FFFEDDFEEDAEAE,
                58 => 0x53BFFFEDFFDEB1A2,
                59 => 0x127FFFB9FFDFB5F6,
                60 => 0x411FFFDDFFDBF4D6,
                62 => 0x0003ffef27eebe74,
                63 => 0x7645FFFECBFEA79E,
                */

                /* Bishop
                0 => 0xffedf9fd7cfcffff,
                1 => 0xfc0962854a77f576,
                6 => 0xfc0a66c64a7ef576,
                7 => 0x7ffdfdfcbd79ffff,
                8 => 0xfc0846a64a34fff6,
                9 => 0xfc087a874a3cf7f6,
                14 => 0xfc0864ae59b4ff76,
                15 => 0x3c0860af4b35ff76,
                16 => 0x73C01AF56CF4CFFB,
                17 => 0x41A01CFAD64AAFFC,
                22 => 0x7c0c028f5b34ff76,
                23 => 0xfc0a028e5ab4df76,
                40 => 0xDCEFD9B54BFCC09F,
                41 => 0xF95FFA765AFD602B,
                46 => 0x43ff9a5cf4ca0c01,
                47 => 0x4BFFCD8E7C587601,
                48 => 0xfc0ff2865334f576,
                49 => 0xfc0bf6ce5924f576,
                54 => 0xc3ffb7dc36ca8c89,
                55 => 0xc3ff8a54f4ca2c89,
                56 => 0xfffffcfcfd79edff,
                57 => 0xfc0863fccb147576,
                62 => 0xfc087e8e4bb2f736,
                63 => 0x43ff9e4ef4ca2c89,
                */
                _ => random.next_u64() & random.next_u64() & random.next_u64(),
            };
            let filled = fill_magic_table(square, blockers, magic, index_bits, direction_offset);
            if filled.is_ok() {
                did_improve = true;

                best_magics[square_index] = magic;
                best_index_bits[square_index] = index_bits;
                continue;
            }
        }
        if did_improve {
            let mut length = 0;
            for (magic, index_bits) in best_magics.iter().zip(best_index_bits) {
                let shift = 64 - index_bits;
                println!("Key {{shift: {shift}, magic: {magic:#04X}, offset: {length}}},");
                length += 1 << index_bits;
            }
            println!("{length}");
        }
    }
}

fn main() {
    if false {
        find_magics(relevant_rook_blockers(), 0);
    } else {
        find_magics(relevant_bishop_blockers(), 4);
    }
}
