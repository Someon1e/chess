use crate::board::zobrist::Zobrist;

use super::{encoded_move::EncodedMove, eval_data::EvalNumber};

#[derive(Clone, Copy)]
pub struct NodeValue {
    pub zobrist_key: Zobrist,
    pub ply_remaining: u16,
    pub node_type: NodeType,
    pub value: EvalNumber,
    pub transposition_move: EncodedMove,
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Exact,
    Beta,
    Alpha,
}

pub const TRANSPOSITION_CAPACITY: usize = {
    const MEGABYTES: usize = 32;

    const MEMORY_OF_ONE_ENTRY_IN_BYTES: usize = core::mem::size_of::<Option<NodeValue>>();
    (MEGABYTES * 1000000) / MEMORY_OF_ONE_ENTRY_IN_BYTES
};
