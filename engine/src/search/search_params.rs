#[derive(Clone, Copy)]
pub(crate) struct Tunable {
    pub history_decay: i16,

    pub iir_min_depth: u8,
    pub iir_depth_reduction: u8,

    pub futility_margin: i32,

    pub static_null_margin: i32,
    pub improving_static_null_margin: i32,
    pub static_null_min_depth: u8,

    pub lmr_min_index: usize,
    pub lmr_min_depth: u8,
    pub lmr_ply_divisor: u8,
    pub lmr_index_divisor: u8,

    pub lmp_base: u32,

    pub nmp_min_depth: u8,
    pub nmp_base_reduction: u8,
    pub nmp_ply_divisor: u8,

    pub aspiration_window_start: i32,
    pub aspiration_window_growth: i32,
    pub pawn_correction_history_grain: i16,

    pub lmr_not_improving: u8,
}

pub(crate) const DEFAULT_TUNABLES: Tunable = Tunable {
    history_decay: 9,
    iir_min_depth: 5,
    iir_depth_reduction: 2,
    futility_margin: 116,
    static_null_margin: 65,
    improving_static_null_margin: 45,
    static_null_min_depth: 7,
    lmr_min_index: 6,
    lmr_min_depth: 3,
    lmr_ply_divisor: 10,
    lmr_index_divisor: 10,
    lmp_base: 2,
    nmp_min_depth: 2,
    nmp_base_reduction: 3,
    nmp_ply_divisor: 4,
    aspiration_window_start: 20,
    aspiration_window_growth: 43,
    pawn_correction_history_grain: 256,
    lmr_not_improving: 1,
};
