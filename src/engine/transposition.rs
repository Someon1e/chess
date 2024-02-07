use crate::board::zobrist::Zobrist;

use super::encoded_move::EncodedMove;

#[derive(Clone, Copy)]
pub struct NodeValue {
    pub key: Zobrist,
    pub ply_remaining: u16,
    pub node_type: NodeType,
    pub value: i32,
    pub best_move: EncodedMove,
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Exact,
    Beta,
    Alpha,
}

pub const TRANSPOSITION_CAPACITY: usize = {
    const MEGABYTES: usize = 128;

    const MEMORY_OF_ONE_ENTRY_IN_BYTES: usize = std::mem::size_of::<NodeValue>();
    (MEGABYTES * 1000000) / MEMORY_OF_ONE_ENTRY_IN_BYTES
};