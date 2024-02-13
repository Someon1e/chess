use super::square::Square;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BitBoard(u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                if !self.get(&Square::from_coords(rank, file)) {
                    write!(f, "0")?
                } else {
                    write!(f, "1")?
                }
                if file != 7 {
                    write!(f, " ")?
                }
            }
            writeln!(f)?
        }
        Ok(())
    }
}

impl BitBoard {
    pub const RANK_1: BitBoard = Self::new(0b11111111);
    pub const RANK_2: BitBoard = Self::new(0b11111111 << 8);
    pub const RANK_3: BitBoard = Self::new(0b11111111 << 16);
    pub const RANK_4: BitBoard = Self::new(0b11111111 << 24);
    pub const RANK_5: BitBoard = Self::new(0b11111111 << 32);
    pub const RANK_6: BitBoard = Self::new(0b11111111 << 40);
    pub const RANK_7: BitBoard = Self::new(0b11111111 << 48);
    pub const RANK_8: BitBoard = Self::new(0b11111111 << 56);

    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(!0);

    pub const fn new(bits: u64) -> Self {
        BitBoard(bits)
    }
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
    pub fn is_not_empty(&self) -> bool {
        self.0 != 0
    }
    pub fn from_square(square: &Square) -> Self {
        Self(1 << square.index())
    }
    pub fn set(&mut self, square: &Square) {
        self.0 |= square.bitboard().0
    }
    pub fn unset(&mut self, square: &Square) {
        self.0 &= !(square.bitboard().0);
    }
    pub fn toggle(&mut self, a: &Square, b: &Square) {
        self.0 ^= (a.bitboard() | b.bitboard()).0;
    }
    pub fn overlaps(&self, bitboard: &BitBoard) -> bool {
        (*self & *bitboard).is_not_empty()
    }
    pub fn get(&self, square: &Square) -> bool {
        self.overlaps(&square.bitboard())
    }
    pub fn first_square(&self) -> Square {
        Square::from_index(self.0.trailing_zeros() as i8)
    }
    pub fn pop_square(&mut self) -> Square {
        let index = self.first_square();
        self.0 &= self.0 - 1;
        index
    }
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }
    pub fn wrapping_sub(&self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }
    pub fn wrapping_mul(&self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

macro_rules! implement_op {
    ($op:ident, $name:ident, $operator:tt) => {
        impl $op<BitBoard> for BitBoard {
            type Output = BitBoard;

            fn $name(self, rhs: BitBoard) -> Self::Output {
               Self(self.0 $operator rhs.0)
            }
        }
    };
}
macro_rules! implement_assign_op {
    ($op:ident, $name:ident, $operator:tt) => {
        impl $op<BitBoard> for BitBoard {
            fn $name(&mut self, rhs: Self) {
                *self = Self(self.0 $operator rhs.0)
            }
        }
    };
}
implement_op!(BitOr, bitor, |);
implement_assign_op!(BitOrAssign, bitor_assign, |);

implement_op!(BitAnd, bitand, &);
implement_assign_op!(BitAndAssign, bitand_assign, &);

macro_rules! shift {
    ($op:ident, $name:ident, $operator:tt) => {
        impl $op<u64> for BitBoard {
            type Output = BitBoard;

            fn $name(self, rhs: u64) -> Self::Output {
               Self(self.0 $operator rhs)
            }
        }
    };
}

shift!(Shl, shl, <<);
shift!(Shr, shr, >>);

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
