use super::game_state::CastlingRights;
use super::square::Square;
use super::{Board, Piece};
use lazy_static::lazy_static;
use rand_chacha;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use std::cell::RefCell;

thread_local! {
    static RNG: RefCell<ChaCha20Rng> = RefCell::new(ChaCha20Rng::seed_from_u64(69));
}
fn random_u64() -> u64 {
    RNG.with(|rng| rng.borrow_mut().next_u64())
}

pub struct ZobristRandoms {
    pub piece_arrays: [[u64; 64]; 12],
    pub side_to_move: u64,
    pub en_passant_square_file: [u64; 8],
    pub castling_rights: [u64; 16],
}

lazy_static! {
    pub static ref ZOBRIST_RANDOMS: ZobristRandoms = {
        let mut piece_arrays: [[u64; 64]; 12] = [[0; 64]; 12];
        for piece in Piece::ALL_PIECES {
            let square_array = &mut piece_arrays[piece as usize];
            square_array.fill_with(random_u64);
        }
        let side_to_move = random_u64();

        let mut en_passant_square_file = [0; 8];
        en_passant_square_file.fill_with(random_u64);

        let mut castling_rights = [0; 16];
        castling_rights.fill_with(random_u64);

        ZobristRandoms {
            piece_arrays,
            side_to_move,
            en_passant_square_file,
            castling_rights,
        }
    };
}

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub struct Zobrist(u64);

impl Zobrist {
    pub fn xor_en_passant(&mut self, en_passant_square: &Square) {
        self.0 ^= ZOBRIST_RANDOMS.en_passant_square_file[en_passant_square.file() as usize]
    }

    pub fn xor_castling_rights(&mut self, castling_rights: &CastlingRights) {
        self.0 ^= ZOBRIST_RANDOMS.castling_rights[castling_rights.internal_value() as usize];
    }

    pub fn xor_piece(&mut self, piece_index: usize, square_index: usize) {
        self.0 ^= ZOBRIST_RANDOMS.piece_arrays[piece_index][square_index];
    }

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn compute(board: &Board) -> Self {
        // For debugging only.

        let mut key = Self::empty();

        for (piece, bit_board) in board.bit_boards.iter().enumerate() {
            let mut bit_board = *bit_board;
            while !bit_board.is_empty() {
                let square = bit_board.pop_square();
                key.xor_piece(piece, square.index() as usize)
            }
        }

        if !board.white_to_move {
            key.flip_side_to_move()
        }

        if let Some(en_passant_square) = board.game_state.en_passant_square {
            key.xor_en_passant(&en_passant_square)
        }

        key.xor_castling_rights(&board.game_state.castling_rights);

        key
    }
    pub fn flip_side_to_move(&mut self) {
        self.0 ^= ZOBRIST_RANDOMS.side_to_move;
    }

    pub fn index(&self, size: usize) -> usize {
        self.0 as usize % size
    }
}
