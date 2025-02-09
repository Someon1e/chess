use super::{encoded_move::EncodedMove, Ply};

pub type PvTable = [[EncodedMove; Ply::MAX as usize]; Ply::MAX as usize];
pub type PvLength = [Ply; Ply::MAX as usize];

pub struct Pv {
    pv_table: PvTable,
    pv_length: PvLength,
}

impl Pv {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pv_table: [[EncodedMove::NONE; Ply::MAX as usize]; Ply::MAX as usize],
            pv_length: [0; Ply::MAX as usize],
        }
    }

    pub fn best_line(&self) -> core::iter::Take<core::slice::Iter<'_, EncodedMove>> {
        self.pv_table[0].iter().take(self.pv_length[0] as usize)
    }

    /// Returns the best move at the first ply.
    #[must_use]
    pub const fn root_best_move(&self) -> EncodedMove {
        self.pv_table[0][0]
    }

    pub const fn set_pv_length(&mut self, ply_from_root: Ply, length: Ply) {
        self.pv_length[ply_from_root as usize] = length;
    }

    pub fn update_move(&mut self, ply_from_root: Ply, encoded_move_data: EncodedMove) {
        self.pv_table[ply_from_root as usize][ply_from_root as usize] = encoded_move_data;
        for next_ply in (ply_from_root + 1)..self.pv_length[ply_from_root as usize + 1] {
            self.pv_table[ply_from_root as usize][next_ply as usize] =
                self.pv_table[ply_from_root as usize + 1][next_ply as usize];
        }
        self.pv_length[ply_from_root as usize] = self.pv_length[ply_from_root as usize + 1];
    }
}

impl Default for Pv {
    fn default() -> Self {
        Self::new()
    }
}
