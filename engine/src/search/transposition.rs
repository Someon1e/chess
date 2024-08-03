use super::{encoded_move::EncodedMove, eval_data::EvalNumber, Ply};

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

pub const MEMORY_OF_ONE_ENTRY_IN_BYTES: usize = core::mem::size_of::<Option<NodeValue>>();

pub const fn megabytes_to_capacity(megabytes: usize) -> usize {
    (megabytes * 1_000_000) / MEMORY_OF_ONE_ENTRY_IN_BYTES
}
