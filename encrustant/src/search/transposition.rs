//! Transposition table utilities.

use super::{Ply, encoded_move::EncodedMove, eval_data::EvalNumber, zobrist::Zobrist};

#[derive(Clone, Copy)]
pub(super) struct NodeValue {
    pub zobrist_key_32: u32,
    pub ply_remaining: Ply,
    pub node_type: NodeType,
    pub value: EvalNumber,

    /// The best move found.
    pub transposition_move: EncodedMove,
}

#[derive(Clone, Copy)]
pub(super) enum NodeType {
    Exact,

    /// Lower bound.
    Beta,

    /// Upper bound.
    Alpha,
}

/// How many bytes one transposition table entry takes.
pub const MEMORY_OF_ONE_ENTRY_IN_BYTES: usize = core::mem::size_of::<Option<NodeValue>>();

/// Returns how many transposition table entries could fit into `megabytes` megabytes.
#[must_use]
pub const fn megabytes_to_capacity(megabytes: usize) -> usize {
    (megabytes * 1_000_000) / MEMORY_OF_ONE_ENTRY_IN_BYTES
}

pub struct TranspositionTable {
    vec: Vec<Option<NodeValue>>,
}
impl TranspositionTable {
    pub fn new(transposition_capacity: usize) -> Self {
        Self {
            vec: vec![None; transposition_capacity],
        }
    }
    pub fn clear(&mut self) {
        self.vec.fill(None);
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }
    pub fn get(&self, zobrist_index: usize) -> Option<NodeValue> {
        self.vec[zobrist_index]
    }
    pub fn set(&mut self, zobrist_index: usize, node_value: NodeValue) {
        self.vec[zobrist_index] = Some(node_value);
    }
    pub fn get_index(&self, zobrist_key: Zobrist) -> usize {
        zobrist_key.distribute(self.len()) as usize
    }
    pub fn prefetch(&mut self, zobrist_key: Zobrist) {
        #[cfg(target_feature = "sse")]
        {
            use core::arch::x86_64::{_MM_HINT_NTA, _mm_prefetch};
            let index = self.get_index(zobrist_key);
            unsafe {
                _mm_prefetch::<{ _MM_HINT_NTA }>(self.vec.as_ptr().add(index).cast::<i8>());
            }
        }
        #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
        {
            use core::arch::aarch64::{_PREFETCH_LOCALITY0, _PREFETCH_READ, _prefetch};
            let index = self.get_index(zobrist_key);
            unsafe {
                _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>(
                    self.vec.as_ptr().add(index).cast::<i8>(),
                );
            }
        }
    }
}
