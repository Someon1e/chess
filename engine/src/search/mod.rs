pub mod encoded_move;

pub mod pv;

mod move_ordering;
mod repetition_table;
pub mod search_params;
mod time_manager;
pub mod transposition;

use pv::Pv;
use search_params::{Tunable, DEFAULT_TUNABLES};
pub use time_manager::TimeManager;

use crate::{
    board::{game_state::GameState, zobrist::Zobrist, Board},
    evaluation::{
        eval_data::{self, EvalNumber},
        Eval,
    },
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

use self::{
    encoded_move::EncodedMove,
    move_ordering::MoveOrderer,
    repetition_table::RepetitionTable,
    transposition::{NodeType, NodeValue},
};

type Ply = u8;

/// Score of having checkmated the opponent.
pub const IMMEDIATE_CHECKMATE_SCORE: EvalNumber = -EvalNumber::MAX + 1;

const CHECKMATE_SCORE: EvalNumber = IMMEDIATE_CHECKMATE_SCORE.abs() - (Ply::MAX as EvalNumber);

const USE_STATIC_NULL_MOVE_PRUNING: bool = true;
const USE_NULL_MOVE_PRUNING: bool = true;
const USE_LATE_MOVE_REDUCTION: bool = true;
const USE_INTERNAL_ITERATIVE_REDUCTION: bool = true;
const USE_PVS: bool = true;
const USE_KILLER_MOVE: bool = true;
const USE_ASPIRATION_WINDOWS: bool = true;
const USE_FUTILITY_PRUNING: bool = true;

#[cfg(not(feature = "spsa"))]
macro_rules! param {
    ($self:expr) => {
        DEFAULT_TUNABLES
    };
}
#[cfg(feature = "spsa")]
macro_rules! param {
    ($self:expr) => {
        $self.tunable
    };
}

/// Search info at a depth.
pub struct DepthSearchInfo<'a> {
    /// Depth searched at.
    pub depth: Ply,

    /// Highest number of moves looked ahead.
    pub highest_depth: Ply,

    /// The best move and evaluation.
    pub best: (&'a Pv, EvalNumber),

    /// How many times `quiescence_search()` was called.
    pub quiescence_call_count: u32,
}

/// Looks for the best outcome in a position.
pub struct Search {
    board: Board,

    repetition_table: RepetitionTable,

    transposition_table: Vec<Option<NodeValue>>,

    killer_moves: [EncodedMove; 64],
    quiet_history: [[i16; 64 * 64]; 2],
    pawn_correction_history: [[i16; 8192]; 2],

    pub pv: Pv,
    pub highest_depth: Ply,

    quiescence_call_count: u32,

    #[cfg(feature = "spsa")]
    tunable: Tunable,
}

impl Search {
    /// Create a new search.
    #[must_use]
    pub fn new(
        board: Board,
        transposition_capacity: usize,
        #[cfg(feature = "spsa")] tunable: Tunable,
    ) -> Self {
        Self {
            board,

            repetition_table: RepetitionTable::new(),

            transposition_table: vec![None; transposition_capacity],

            killer_moves: [EncodedMove::NONE; 64],
            quiet_history: [[0; 64 * 64]; 2],

            pawn_correction_history: [[0; 8192]; 2],

            pv: Pv::new(),
            highest_depth: 0,

            quiescence_call_count: 0,

            #[cfg(feature = "spsa")]
            tunable,
        }
    }

    /// Sets an empty transposition table with the new capacity.
    pub fn resize_transposition_table(&mut self, transposition_capacity: usize) {
        self.transposition_table = vec![None; transposition_capacity];
    }

    /// Returns the current board.
    #[must_use]
    pub const fn board(&self) -> &Board {
        &self.board
    }

    /// A new position.
    pub fn new_board(&mut self, board: Board) {
        self.board = board;
        self.repetition_table.clear();
    }

    /// Another search.
    pub fn clear_for_new_search(&mut self) {
        self.quiescence_call_count = 0;
        self.highest_depth = 0;
        self.killer_moves.fill(EncodedMove::NONE);
        for side in &mut self.quiet_history {
            for value in side {
                *value /= param!(self).history_decay;
            }
        }
    }

    /// A new match.
    pub fn clear_cache_for_new_game(&mut self) {
        self.pawn_correction_history[0].fill(0);
        self.pawn_correction_history[1].fill(0);

        for side in &mut self.quiet_history {
            side.fill(0);
        }

        self.transposition_table.fill(None);
    }

    #[must_use]
    fn quiescence_search(&mut self, mut alpha: EvalNumber, beta: EvalNumber) -> EvalNumber {
        self.quiescence_call_count += 1;

        let mut best_score = Eval::evaluate(&self.board);
        if best_score > alpha {
            alpha = best_score;

            if best_score >= beta {
                return best_score;
            }
        }

        let move_generator = MoveGenerator::new(&self.board);
        let (mut move_guesses, move_count) =
            MoveOrderer::get_move_guesses_captures_only(self, &move_generator);
        let mut index = 0;
        while index != move_count {
            let move_data = unsafe {
                // SAFETY: `get_move_guesses_captures_only` guarantees that `move_guesses[0..move_count]` are initialised.
                // `index` can not be higher than `move_count`, due to the loop condition.

                MoveOrderer::put_highest_guessed_move(&mut move_guesses, index, move_count)
            }
            .move_data
            .decode();

            let old_state = self.board.make_move(&move_data);
            let score = -self.quiescence_search(-beta, -alpha);
            self.board.unmake_move(&move_data, &old_state);

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;

                    if score >= beta {
                        break;
                    }
                }
            }

            index += 1;
        }
        best_score
    }

    /// Adds the position into the repetition table and makes a move.
    pub fn make_move(&mut self, move_data: &Move) -> GameState {
        self.repetition_table
            .push(self.board.position_zobrist_key());

        self.board.make_move(move_data)
    }

    /// Unmakes a move, then removes the position from the repetition table.
    ///
    /// # Panics
    ///
    /// Will panic if the zobrist key after playing the move does not match the previous position's.
    pub fn unmake_move(&mut self, move_data: &Move, old_state: &GameState) {
        self.board.unmake_move(move_data, old_state);

        assert_eq!(
            self.repetition_table.pop(),
            self.board.position_zobrist_key()
        );
    }
    fn negamax(
        &mut self,

        time_manager: &TimeManager,

        mut ply_remaining: Ply,
        ply_from_root: Ply,

        allow_null_move: bool,

        mut alpha: EvalNumber,
        beta: EvalNumber,
    ) -> EvalNumber {
        if ply_from_root > self.highest_depth {
            self.highest_depth = ply_from_root;
        }

        self.pv.set_pv_length(ply_from_root, ply_from_root);

        // Get the zobrist key
        let zobrist_key = self.board.position_zobrist_key();

        // Check for repetition
        if ply_from_root != 0 && self.repetition_table.contains(zobrist_key) {
            return 0;
        }

        // Turn zobrist key into an index into the transposition table
        let zobrist_index = zobrist_key.distribute(self.transposition_table.len()) as usize;

        // This is the best move in this position according to previous searches
        let mut hash_move = EncodedMove::NONE;

        // Check if this is a pv node
        let is_not_pv_node = alpha + 1 == beta;

        // Get value from transposition table
        let saved = if let Some(saved) = self.transposition_table[zobrist_index] {
            // Check if it's actually the same position
            if saved.zobrist_key_32 == zobrist_key.lower_u32() {
                // Check if the saved depth is as high as the depth now
                if saved.ply_remaining >= ply_remaining {
                    let node_type = &saved.node_type;
                    if match node_type {
                        NodeType::Exact => is_not_pv_node,
                        NodeType::Beta => saved.value >= beta,
                        NodeType::Alpha => saved.value <= alpha,
                    } {
                        self.pv.update_move(ply_from_root, saved.transposition_move);

                        return saved.value;
                    }
                }

                hash_move = saved.transposition_move;

                Some(saved)
            } else {
                None
            }
        } else {
            None
        };
        if ply_from_root == 0 {
            // Use iterative deepening move as hash move
            hash_move = self.pv.root_best_move();
        }
        if USE_INTERNAL_ITERATIVE_REDUCTION
            && hash_move.is_none()
            && ply_remaining > param!(self).iir_min_depth
        {
            // Internal iterative reduction
            ply_remaining = ply_remaining.saturating_sub(param!(self).iir_depth_reduction);
        }

        if ply_remaining == 0 {
            // Enter quiescence search
            return self.quiescence_search(alpha, beta);
        };

        let move_generator = MoveGenerator::new(&self.board);

        let pawn_index = self
            .board
            .pawn_zobrist_key()
            .modulo(self.pawn_correction_history.len() as u64);
        let static_eval = {
            let mut static_eval = Eval::evaluate(&self.board);
            if let Some(saved) = saved {
                // Use saved value as better static evaluation
                if !Self::score_is_checkmate(saved.value)
                    && match saved.node_type {
                        NodeType::Exact => true,
                        NodeType::Beta => saved.value > static_eval,
                        NodeType::Alpha => saved.value < static_eval,
                    }
                {
                    static_eval = saved.value;
                }
            }
            let correction = self.pawn_correction_history
                [if self.board.white_to_move { 1 } else { 0 }][pawn_index as usize];
            static_eval += (correction / param!(self).pawn_correction_history_grain) as i32;
            static_eval
        };

        if is_not_pv_node && !move_generator.is_in_check() {
            // Static null move pruning (also known as reverse futility pruning)
            if USE_STATIC_NULL_MOVE_PRUNING
                && ply_remaining < param!(self).static_null_min_depth
                && static_eval - i32::from(ply_remaining) * param!(self).static_null_margin > beta
            {
                return static_eval;
            }

            // Null move pruning
            if USE_NULL_MOVE_PRUNING
            && allow_null_move
            && ply_remaining > param!(self).nmp_min_depth

            && static_eval >= beta

            // Do not do it if we only have pawns and a king
            && move_generator.friendly_pawns().count() + 1
                != move_generator.friendly_pieces().count()
            {
                let old_state = self.board.make_null_move();

                let score = -self.negamax(
                    time_manager,
                    ply_remaining.saturating_sub(
                        param!(self).nmp_base_reduction
                            + ply_remaining / param!(self).nmp_ply_divisor,
                    ),
                    ply_from_root + 1,
                    false,
                    -beta,
                    -beta + 1,
                );
                self.board.unmake_null_move(&old_state);

                if score >= beta {
                    return score;
                }
            }
        }

        // Get legal moves and their estimated value
        let (mut move_guesses, move_count) = MoveOrderer::get_move_guesses(
            self,
            &move_generator,
            hash_move,
            if USE_KILLER_MOVE && (ply_from_root as usize) < self.killer_moves.len() {
                self.killer_moves[ply_from_root as usize]
            } else {
                EncodedMove::NONE
            },
        );

        if move_count == 0 {
            // No moves
            let score = if move_generator.is_in_check() {
                // Checkmate
                IMMEDIATE_CHECKMATE_SCORE + EvalNumber::from(ply_from_root)
            } else {
                // Stalemate
                0
            };
            return score;
        }

        let mut node_type = NodeType::Alpha;
        let (mut best_move, mut best_score) = (EncodedMove::NONE, -EvalNumber::MAX);

        let mut quiets_evaluated: Vec<EncodedMove> = Vec::new();
        let mut index = 0;
        loop {
            let encoded_move_data = unsafe {
                // SAFETY: `put_highest_guessed_move` guarantees that `move_guesses[0..move_count]` are initialised.
                // `index` can not be higher than `move_count`, due to the loop condition.

                MoveOrderer::put_highest_guessed_move(&mut move_guesses, index, move_count)
            }
            .move_data;
            let move_data = encoded_move_data.decode();

            let is_capture = move_generator.enemy_piece_bit_board().get(&move_data.to);
            let old_state = self.make_move(&move_data);
            #[cfg(target_feature = "sse")]
            {
                use core::arch::x86_64::{_mm_prefetch, _MM_HINT_NTA};
                let index =
                    self.board
                        .zobrist_key()
                        .distribute(self.transposition_table.len()) as usize;
                unsafe {
                    _mm_prefetch::<{ _MM_HINT_NTA }>(
                        self.transposition_table.as_ptr().add(index).cast::<i8>(),
                    );
                }
            }
            #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
            {
                use core::arch::aarch64::{_prefetch, _PREFETCH_LOCALITY0, _PREFETCH_READ};
                let index =
                    self.board
                        .position_zobrist_key()
                        .distribute(self.transposition_table.len()) as usize;
                unsafe {
                    _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>(
                        self.transposition_table.as_ptr().add(index).cast::<i8>(),
                    );
                }
            }

            // Search deeper when in check
            let check_extension = MoveGenerator::calculate_is_in_check(&self.board);

            let mut normal_search = check_extension // Do not reduce if extending
                || is_capture // Do not reduce if it's a capture
                || index < param!(self).lmr_min_index // Do not reduce if it's not a late move
                || (ply_remaining) < param!(self).lmr_min_depth // Do not reduce if there is little depth remaining
                || !USE_LATE_MOVE_REDUCTION; // Do not reduce if turned off
            let mut score = 0;

            if !normal_search {
                // Late move reduction
                let r = 2
                    + ply_remaining / param!(self).lmr_ply_divisor
                    + index as Ply / param!(self).lmr_index_divisor;
                score = -self.negamax(
                    time_manager,
                    ply_remaining.saturating_sub(r),
                    ply_from_root + 1,
                    true,
                    -alpha - 1,
                    -alpha,
                );
                if score > alpha {
                    // Need to search again without reduction
                    normal_search = true;
                }
            }

            if USE_PVS && normal_search && index != 0 {
                score = -self.negamax(
                    time_manager,
                    ply_remaining - 1 + Ply::from(check_extension),
                    ply_from_root + 1,
                    true,
                    -alpha - 1,
                    -alpha,
                );
                normal_search = alpha < score && score < beta;
            }
            if normal_search {
                score = -self.negamax(
                    time_manager,
                    ply_remaining - 1 + Ply::from(check_extension),
                    ply_from_root + 1,
                    true,
                    -beta,
                    -alpha,
                );
            }

            self.unmake_move(&move_data, &old_state);

            if ply_remaining > 1 && time_manager.hard_stop_inner_search() {
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_move = encoded_move_data;

                if score > alpha {
                    alpha = score;

                    self.pv.update_move(ply_from_root, encoded_move_data);

                    node_type = NodeType::Exact;

                    if score >= beta {
                        if !is_capture {
                            // Not a capture but still caused beta cutoff, sort this higher later

                            if (ply_from_root as usize) < self.killer_moves.len() {
                                self.killer_moves[usize::from(ply_from_root)] = encoded_move_data;
                            }

                            const MAX_HISTORY: i32 = 16384;
                            fn history_gravity(current_value: i16, history_bonus: i32) -> i16 {
                                (history_bonus
                                    - i32::from(current_value) * history_bonus / MAX_HISTORY)
                                    as i16
                            }

                            let history_bonus = (i32::from(ply_remaining)
                                * i32::from(ply_remaining))
                            .min(MAX_HISTORY);

                            let history_side =
                                &mut self.quiet_history[usize::from(self.board.white_to_move)];

                            let history =
                                &mut history_side[encoded_move_data.without_flag() as usize];
                            *history += history_gravity(*history, history_bonus);

                            for previous_quiet in quiets_evaluated {
                                let history =
                                    &mut history_side[previous_quiet.without_flag() as usize];
                                *history += history_gravity(*history, -history_bonus);
                            }
                        }
                        node_type = NodeType::Beta;
                        break;
                    }
                }
            }
            if !is_capture {
                if is_not_pv_node && !move_generator.is_in_check() {
                    if USE_FUTILITY_PRUNING
                        && static_eval + param!(self).futility_margin * i32::from(ply_remaining)
                            < alpha
                    {
                        // Futility pruning
                        break;
                    }
                    if quiets_evaluated.len() as u32 + 1
                        > param!(self).lmp_base
                            + u32::from(ply_remaining) * u32::from(ply_remaining)
                    {
                        // Late move pruning
                        break;
                    }
                }
                quiets_evaluated.push(encoded_move_data);
            }

            index += 1;
            if index == move_count {
                break;
            }
        }

        if !move_generator.is_in_check() && !Self::score_is_checkmate(best_score) {
            let not_loud_move = {
                if best_move.is_none() {
                    true
                } else {
                    // Not promotion and not capture
                    !matches!(
                        best_move.flag(),
                        Flag::QueenPromotion
                            | Flag::RookPromotion
                            | Flag::BishopPromotion
                            | Flag::KnightPromotion
                            | Flag::EnPassant
                    ) && !move_generator.enemy_piece_bit_board().get(&best_move.to())
                }
            };

            if not_loud_move
                && match node_type {
                    NodeType::Beta => best_score > static_eval,
                    NodeType::Alpha => best_score < static_eval,
                    NodeType::Exact => true,
                }
            {
                const CORRECTION_HISTORY_WEIGHT_SCALE: i16 = 1024;
                const CORRECTION_HISTORY_MAX: i16 = 16384;

                let error = best_score - static_eval;

                let mut entry = self.pawn_correction_history
                    [if self.board.white_to_move { 1 } else { 0 }][pawn_index as usize]
                    as i32;
                let scaled_error = error * param!(self).pawn_correction_history_grain as i32;
                let new_weight = i32::min(
                    (ply_remaining as i32) * (ply_remaining as i32)
                        + 2 * (ply_remaining as i32)
                        + 1,
                    128,
                );
                assert!(new_weight <= CORRECTION_HISTORY_WEIGHT_SCALE as i32);

                entry = (entry * (CORRECTION_HISTORY_WEIGHT_SCALE as i32 - new_weight)
                    + scaled_error * new_weight)
                    / CORRECTION_HISTORY_WEIGHT_SCALE as i32;
                entry = i32::clamp(
                    entry,
                    -CORRECTION_HISTORY_MAX as i32,
                    CORRECTION_HISTORY_MAX as i32,
                );

                self.pawn_correction_history[if self.board.white_to_move { 1 } else { 0 }]
                    [pawn_index as usize] = entry as i16;
            }
        }

        // Save to transposition table
        self.transposition_table[zobrist_index] = Some(NodeValue {
            zobrist_key_32: zobrist_key.lower_u32(),
            ply_remaining,
            node_type,
            value: best_score,
            transposition_move: best_move,
        });

        best_score
    }

    /// Returns whether a score means forced checkmate.
    #[must_use]
    pub const fn score_is_checkmate(score: EvalNumber) -> bool {
        score.abs() >= CHECKMATE_SCORE
    }

    #[must_use]
    fn aspiration_search(
        &mut self,
        time_manager: &TimeManager,
        mut best_score: EvalNumber,
        depth: Ply,
    ) -> EvalNumber {
        if USE_ASPIRATION_WINDOWS && depth > 2 {
            let mut alpha = best_score
                .saturating_sub(param!(self).aspiration_window_start)
                .max(-EvalNumber::MAX);
            let mut beta = best_score.saturating_add(param!(self).aspiration_window_start);
            for _ in 0..4 {
                best_score = self.negamax(time_manager, depth, 0, false, alpha, beta);
                if best_score <= alpha {
                    alpha = alpha
                        .saturating_sub(param!(self).aspiration_window_growth)
                        .max(-EvalNumber::MAX);
                    // -EvalNumber::MAX = -2147483647
                    // EvalNumber::MIN = -2147483648

                    beta = (alpha + beta) / 2;
                } else if best_score >= beta {
                    beta = beta.saturating_add(param!(self).aspiration_window_growth);
                } else {
                    return best_score;
                }
            }
        }
        self.negamax(
            time_manager,
            depth,
            0,
            false,
            -EvalNumber::MAX,
            EvalNumber::MAX,
        )
    }

    /// Repeatedly searches the board, increasing depth by one each time. Stops when `time_manager` returns `true`.
    #[must_use]
    pub fn iterative_deepening(
        &mut self,

        time_manager: &TimeManager,

        depth_completed: &mut dyn FnMut(DepthSearchInfo),
    ) -> (Ply, EvalNumber) {
        let mut depth = 0;
        let mut previous_best_score = -EvalNumber::MAX;
        loop {
            depth += 1;
            let best_score = self.aspiration_search(time_manager, previous_best_score, depth);

            if time_manager.hard_stop_iterative_deepening(depth) {
                // Must stop now.
                break;
            }
            previous_best_score = best_score;

            if self.pv.root_best_move().is_none() || Self::score_is_checkmate(best_score) {
                // No point searching more.
                break;
            }

            // Depth was completed
            // Report results of search iteration
            depth_completed(DepthSearchInfo {
                depth,
                best: (&self.pv, best_score),
                highest_depth: self.highest_depth,
                quiescence_call_count: self.quiescence_call_count,
            });

            if depth == Ply::MAX {
                // Maximum depth, can not continue
                break;
            }

            if time_manager.soft_stop() {
                // It would probably be a waste of time to start another iteration
                break;
            }
        }

        (depth, previous_best_score)
    }

    /// Returns how many times quiescence search was called.
    #[must_use]
    pub const fn quiescence_call_count(&self) -> u32 {
        self.quiescence_call_count
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        evaluation::{eval_data::EvalNumber, Eval},
        search::{search_params::DEFAULT_TUNABLES, transposition::megabytes_to_capacity, Search},
    };

    #[test]
    fn quiescence_search_works() {
        let board =
            Board::from_fen("rnbqkb1r/pppp1ppp/5n2/4p2Q/4P3/8/PPPPBPPP/RNB1K1NR b KQkq - 3 3");
        let quiet =
            Board::from_fen("rnbqkb1r/pppp1ppp/8/4p2B/4P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4");
        assert_eq!(
            Search::new(
                board,
                megabytes_to_capacity(8),
                #[cfg(feature = "spsa")]
                DEFAULT_TUNABLES,
            )
            .quiescence_search(-EvalNumber::MAX, EvalNumber::MAX),
            Eval::evaluate(&quiet)
        );
    }
}
