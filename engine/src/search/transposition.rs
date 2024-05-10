use super::{encoded_move::EncodedMove, eval_data::EvalNumber, Ply};

#[derive(Clone, Copy)]
pub struct NodeValue {
    pub zobrist_key_32: u32,
    pub ply_remaining: Ply,
    pub node_type: NodeType,
    pub value: EvalNumber,

    /// The best move found.
    pub transposition_move: EncodedMove,
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Exact,

    /// Lower bound.
    Beta,

    /// Upper bound.
    Alpha,
}

pub const TRANSPOSITION_CAPACITY: usize = {
    const MEGABYTES: usize = 32;

    const MEMORY_OF_ONE_ENTRY_IN_BYTES: usize = core::mem::size_of::<Option<NodeValue>>();
    (MEGABYTES * 1_000_000) / MEMORY_OF_ONE_ENTRY_IN_BYTES
};
