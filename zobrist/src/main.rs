#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use engine::board::piece::Piece;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

use engine::search::zobrist::ZobristRandoms;

fn main() {
    let mut rng = ChaCha20Rng::seed_from_u64(69);

    let mut piece_arrays: [[u64; 64]; 12] = [[0; 64]; 12];
    for piece in Piece::ALL_PIECES {
        let square_array = &mut piece_arrays[piece as usize];
        square_array.fill_with(|| rng.next_u64());
    }
    let side_to_move = rng.next_u64();

    let mut en_passant_square_file = [0; 8];
    en_passant_square_file.fill_with(|| rng.next_u64());

    let mut castling_rights = [0; 16];
    castling_rights.fill_with(|| rng.next_u64());

    println!(
        "{:?}",
        ZobristRandoms {
            piece_arrays,
            side_to_move,
            en_passant_square_file,
            castling_rights,
        }
    );
}
