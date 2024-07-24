use engine::{
    board::{bit_board::BitBoard, square::Square},
    move_generator::slider_lookup::{
        gen_rook_or_bishop, iterate_combinations, RELEVANT_BISHOP_BLOCKERS, RELEVANT_ROOK_BLOCKERS,
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

fn out(best_magics: &[u64; 64], best_index_bits: &[u8; 64]) {
    let mut length = 0;
    for (magic, index_bits) in best_magics.iter().zip(best_index_bits) {
        let shift = 64 - index_bits;

        let magic_hex = format!("{magic:#018X}");
        let mut magic_hex_underscores: String =
            String::with_capacity(magic_hex.len() + magic_hex.len() / 2);
        for (i, character) in magic_hex.chars().enumerate() {
            if i != 2 && (i + 2) % 4 == 0 {
                magic_hex_underscores.push('_');
            }
            magic_hex_underscores.push(character);
        }

        println!("Key {{shift: {shift}, magic: {magic_hex_underscores}, offset: {length}}},");
        length += 1 << index_bits;
    }
    println!("{length}");
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
    out(&best_magics, &best_index_bits);

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
                48 => 0x48FF_FE99_FECF_AA00,
                49 => 0x48FF_FE99_FECF_AA00,
                50 => 0x497F_FFAD_FF9C_2E00,
                51 => 0x613F_FFDD_FFCE_9200,
                52 => 0xffff_ffe9_ffe7_ce00,
                53 => 0xffff_fff5_fff3_e600,
                54 => 0x0003_ff95_e5e6_a4c0,
                55 => 0x510F_FFF5_F63C_96A0,
                56 => 0xEBFF_FFB9_FF9F_C526,
                57 => 0x61FF_FEDD_FEED_AEAE,
                58 => 0x53BF_FFED_FFDE_B1A2,
                59 => 0x127F_FFB9_FFDF_B5F6,
                60 => 0x411F_FFDD_FFDB_F4D6,
                62 => 0x0003_ffef_27ee_be74,
                63 => 0x7645_FFFE_CBFE_A79E,
                */

                /* Bishop
                0 => 0xffed_f9fd_7cfc_ffff,
                1 => 0xfc09_6285_4a77_f576,
                6 => 0xfc0a_66c6_4a7e_f576,
                7 => 0x7ffd_fdfc_bd79_ffff,
                8 => 0xfc08_46a6_4a34_fff6,
                9 => 0xfc08_7a87_4a3c_f7f6,
                14 => 0xfc08_64ae_59b4_ff76,
                15 => 0x3c08_60af_4b35_ff76,
                16 => 0x73C0_1AF5_6CF4_CFFB,
                17 => 0x41A0_1CFA_D64A_AFFC,
                22 => 0x7c0c_028f_5b34_ff76,
                23 => 0xfc0a_028e_5ab4_df76,
                40 => 0xDCEF_D9B5_4BFC_C09F,
                41 => 0xF95F_FA76_5AFD_602B,
                46 => 0x43ff_9a5c_f4ca_0c01,
                47 => 0x4BFF_CD8E_7C58_7601,
                48 => 0xfc0f_f286_5334_f576,
                49 => 0xfc0b_f6ce_5924_f576,
                54 => 0xc3ff_b7dc_36ca_8c89,
                55 => 0xc3ff_8a54_f4ca_2c89,
                56 => 0xffff_fcfc_fd79_edff,
                57 => 0xfc08_63fc_cb14_7576,
                62 => 0xfc08_7e8e_4bb2_f736,
                63 => 0x43ff_9e4e_f4ca_2c89,
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
            out(&best_magics, &best_index_bits);
        }
    }
}

fn main() {
    if true {
        find_magics(&RELEVANT_ROOK_BLOCKERS, 0);
    } else {
        find_magics(&RELEVANT_BISHOP_BLOCKERS, 4);
    }
}
