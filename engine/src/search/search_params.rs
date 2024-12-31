pub struct Tunable {
    pub history_decay: i16,

    pub iir_min_depth: u8,
    pub iir_depth_reduction: u8,

    pub futility_margin: i32,

    pub static_null_margin: i32,
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
}

pub const DEFAULT_TUNABLES: Tunable = Tunable {
    history_decay: 9,

    iir_min_depth: 3,
    iir_depth_reduction: 1,

    futility_margin: 130,

    static_null_margin: 60,
    static_null_min_depth: 5,

    lmr_min_index: 3,
    lmr_min_depth: 3,
    lmr_ply_divisor: 10,
    lmr_index_divisor: 9,

    lmp_base: 3,

    nmp_min_depth: 2,
    nmp_base_reduction: 3,
    nmp_ply_divisor: 6,

    aspiration_window_start: 32,
    aspiration_window_growth: 47,
};
