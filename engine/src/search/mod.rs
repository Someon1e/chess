pub mod encoded_move;

pub mod pv;

mod move_ordering;
mod repetition_table;
mod search_params;
pub mod transposition;

use pv::Pv;
use search_params::DEFAULT_TUNABLES;

use crate::{
    board::{game_state::GameState, Board},
    evaluation::{
        eval_data::{self, EvalNumber},
        Eval,
    },
    move_generator::{move_data::Move, MoveGenerator},
    timer::Time,
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

macro_rules! param {
    () => {
        DEFAULT_TUNABLES
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

    pub pv: Pv,
    pub highest_depth: Ply,

    quiescence_call_count: u32,
}

impl Search {
    /// Create a new search.
    #[must_use]
    pub fn new(board: Board, transposition_capacity: usize) -> Self {
        Self {
            board,

            repetition_table: RepetitionTable::new(),

            transposition_table: vec![None; transposition_capacity],

            killer_moves: [EncodedMove::NONE; 64],
            quiet_history: [[0; 64 * 64]; 2],

            pv: Pv::new(),
            highest_depth: 0,

            quiescence_call_count: 0,
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
                *value /= param!().history_decay;
            }
        }
    }

    /// A new match.
    pub fn clear_cache_for_new_game(&mut self) {
        self.transposition_table.fill(None);
        for side in &mut self.quiet_history {
            side.fill(0);
        }
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
        self.repetition_table.push(self.board.zobrist_key());

        self.board.make_move(move_data)
    }

    /// Unmakes a move, then removes the position from the repetition table.
    ///
    /// # Panics
    ///
    /// Will panic if the zobrist key after playing the move does not match the previous position's.
    pub fn unmake_move(&mut self, move_data: &Move, old_state: &GameState) {
        self.board.unmake_move(move_data, old_state);

        assert_eq!(self.repetition_table.pop(), self.board.zobrist_key());
    }
    fn negamax(
        &mut self,

        timer: &Time,
        hard_time_limit: u64,

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
        let zobrist_key = self.board.zobrist_key();

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
            && ply_remaining > param!().iir_min_depth
        {
            // Internal iterative reduction
            ply_remaining -= param!().iir_depth_reduction;
        }

        if ply_remaining == 0 {
            // Enter quiescence search
            return self.quiescence_search(alpha, beta);
        };

        let move_generator = MoveGenerator::new(&self.board);

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
            static_eval
        };

        if is_not_pv_node && !move_generator.is_in_check() {
            // Static null move pruning (also known as reverse futility pruning)
            if USE_STATIC_NULL_MOVE_PRUNING
                && ply_remaining < param!().static_null_min_depth
                && static_eval - i32::from(ply_remaining) * param!().static_null_margin > beta
            {
                return static_eval;
            }

            // Null move pruning
            if USE_NULL_MOVE_PRUNING
            && allow_null_move
            && ply_remaining > param!().nmp_min_depth

            && static_eval >= beta

            // Do not do it if we only have pawns and a king
            && move_generator.friendly_pawns().count() + 1
                != move_generator.friendly_pieces().count()
            {
                let old_state = self.board.make_null_move();

                let score = -self.negamax(
                    timer,
                    hard_time_limit,
                    ply_remaining.saturating_sub(
                        param!().nmp_base_reduction + ply_remaining / param!().nmp_ply_divisor,
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
        let (mut transposition_move, mut best_score) = (EncodedMove::NONE, -EvalNumber::MAX);

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

            // Search deeper when in check
            let check_extension = MoveGenerator::calculate_is_in_check(&self.board);

            let mut normal_search = check_extension // Do not reduce if extending
                || is_capture // Do not reduce if it's a capture
                || index < param!().lmr_min_index // Do not reduce if it's not a late move
                || (ply_remaining) < param!().lmr_min_depth // Do not reduce if there is little depth remaining
                || !USE_LATE_MOVE_REDUCTION; // Do not reduce if turned off
            let mut score = 0;

            if !normal_search {
                // Late move reduction
                let r = 2
                    + ply_remaining / param!().lmr_ply_divisor
                    + index as Ply / param!().lmr_index_divisor;
                score = -self.negamax(
                    timer,
                    hard_time_limit,
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
                    timer,
                    hard_time_limit,
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
                    timer,
                    hard_time_limit,
                    ply_remaining - 1 + Ply::from(check_extension),
                    ply_from_root + 1,
                    true,
                    -beta,
                    -alpha,
                );
            }

            self.unmake_move(&move_data, &old_state);

            if ply_remaining > 1 && timer.milliseconds() > hard_time_limit {
                return 0;
            }

            if score > best_score {
                best_score = score;
                transposition_move = encoded_move_data;

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
                        && static_eval + param!().futility_margin * i32::from(ply_remaining) < alpha
                    {
                        // Futility pruning
                        break;
                    }
                    if quiets_evaluated.len() as u32 + 1
                        > param!().lmp_base + u32::from(ply_remaining) * u32::from(ply_remaining)
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

        // Save to transposition table
        self.transposition_table[zobrist_index] = Some(NodeValue {
            zobrist_key_32: zobrist_key.lower_u32(),
            ply_remaining,
            node_type,
            value: best_score,
            transposition_move,
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
        timer: &Time,
        hard_time_limit: u64,
        mut best_score: EvalNumber,
        depth: Ply,
    ) -> EvalNumber {
        if USE_ASPIRATION_WINDOWS && depth != 1 {
            for window in [
                param!().aspiration_window,
                param!().aspiration_window * 2,
                param!().aspiration_window * 3,
                param!().aspiration_window * 5,
            ] {
                let alpha = best_score - window;
                let beta = best_score + window;
                best_score = self.negamax(timer, hard_time_limit, depth, 0, false, alpha, beta);
                if alpha < best_score && best_score < beta {
                    return best_score;
                }
            }
        }
        self.negamax(
            timer,
            hard_time_limit,
            depth,
            0,
            false,
            -EvalNumber::MAX,
            EvalNumber::MAX,
        )
    }

    /// Repeatedly searches the board, increasing depth by one each time. Stops when `should_cancel` returns `true`.
    #[must_use]
    pub fn iterative_deepening(
        &mut self,

        timer: &Time,
        hard_time_limit: u64,
        soft_time_limit: u64,

        depth_completed: &mut dyn FnMut(DepthSearchInfo),
    ) -> (Ply, EvalNumber) {
        assert!(hard_time_limit >= soft_time_limit);

        let mut depth = 0;
        let mut previous_best_score = -EvalNumber::MAX;
        loop {
            depth += 1;
            let best_score =
                self.aspiration_search(timer, hard_time_limit, previous_best_score, depth);

            if timer.milliseconds() > hard_time_limit {
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

            if timer.milliseconds() > soft_time_limit {
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
    use std::{sync::mpsc, thread};

    use crate::{
        board::Board,
        evaluation::{eval_data::EvalNumber, Eval},
        search::{transposition::megabytes_to_capacity, Search},
        timer::Time,
        uci,
    };

    #[test]
    fn quiescence_search_works() {
        let board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        let quiet = Board::from_fen("rnb1kbnr/ppp1pppp/8/3q4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            Search::new(board, megabytes_to_capacity(8))
                .quiescence_search(-EvalNumber::MAX, EvalNumber::MAX),
            Eval::evaluate(&quiet)
        );
    }

    const OBVIOUS_POSITIONS_RANDOMISED: [(&str, &str); 201] = {
        let mut obvious_positions: [(&str, &str); 201] = [
            (
                "5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - - 0 1",
                "e3g3",
            ), // "WAC.003"
            (
                "r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w - - 0 1",
                "h6h7",
            ), // "WAC.004"
            ("5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1", "c6c4"), // "WAC.005"
            ("7k/p7/1R5K/6r1/6p1/6P1/8/8 w - - 0 1", "b6b7"),               // "WAC.006"
            (
                "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1",
                "g4e3",
            ), // "WAC.007"
            (
                "r4q1k/p2bR1rp/2p2Q1N/5p2/5p2/2P5/PP3PPP/R5K1 w - - 0 1",
                "e7f7",
            ), // "WAC.008"
            (
                "2br2k1/2q3rn/p2NppQ1/2p1P3/Pp5R/4P3/1P3PPP/3R2K1 w - - 0 1",
                "h4h7",
            ), // "WAC.010"
            (
                "r1b1kb1r/3q1ppp/pBp1pn2/8/Np3P2/5B2/PPP3PP/R2Q1RK1 w kq - 0 1",
                "f3c6",
            ), // "WAC.011"
            (
                "4k1r1/2p3r1/1pR1p3/3pP2p/3P2qP/P4N2/1PQ4P/5R1K b - - 0 1",
                "g4f3",
            ), // "WAC.012"
            (
                "5rk1/pp4p1/2n1p2p/2Npq3/2p5/6P1/P3P1BP/R4Q1K w - - 0 1",
                "f1f8",
            ), // "WAC.013"
            (
                "r2rb1k1/pp1q1p1p/2n1p1p1/2bp4/5P2/PP1BPR1Q/1BPN2PP/R5K1 w - - 0 1",
                "h3h7",
            ), // "WAC.014"
            (
                "r4rk1/ppp2ppp/2n5/2bqp3/8/P2PB3/1PP1NPPP/R2Q1RK1 w - - 0 1",
                "e2c3",
            ), // "WAC.016"
            (
                "1k5r/pppbn1pp/4q1r1/1P3p2/2NPp3/1QP5/P4PPP/R1B1R1K1 w - - 0 1",
                "c4e5",
            ), // "WAC.017"
            ("R7/P4k2/8/8/8/8/r7/6K1 w - - 0 1", "a8h8"),                   // "WAC.018"
            (
                "r1b2rk1/ppbn1ppp/4p3/1QP4q/3P4/N4N2/5PPP/R1B2RK1 w - - 0 1",
                "c5c6",
            ), // "WAC.019"
            (
                "r2qkb1r/1ppb1ppp/p7/4p3/P1Q1P3/2P5/5PPP/R1B2KNR b kq - 0 1",
                "d7b5",
            ), // "WAC.020"
            (
                "5rk1/1b3p1p/pp3p2/3n1N2/1P6/P1qB1PP1/3Q3P/4R1K1 w - - 0 1",
                "d2h6",
            ), // "WAC.021"
            (
                "r1bqk2r/ppp1nppp/4p3/n5N1/2BPp3/P1P5/2P2PPP/R1BQK2R w KQkq - 0 1",
                "c4a2 g5f7",
            ), // "WAC.022"
            (
                "r3nrk1/2p2p1p/p1p1b1p1/2NpPq2/3R4/P1N1Q3/1PP2PPP/4R1K1 w - - 0 1",
                "g2g4",
            ), // "WAC.023"
            (
                "6k1/1b1nqpbp/pp4p1/5P2/1PN5/4Q3/P5PP/1B2B1K1 b - - 0 1",
                "g7d4",
            ), // "WAC.024"
            ("3R1rk1/8/5Qpp/2p5/2P1p1q1/P3P3/1P2PK2/8 b - - 0 1", "g4h4"),  // "WAC.025"
            (
                "3r2k1/1p1b1pp1/pq5p/8/3NR3/2PQ3P/PP3PP1/6K1 b - - 0 1",
                "d7f5",
            ), // "WAC.026"
            (
                "7k/pp4np/2p3p1/3pN1q1/3P4/Q7/1r3rPP/2R2RK1 w - - 0 1",
                "a3f8",
            ), // "WAC.027"
            (
                "1r1r2k1/4pp1p/2p1b1p1/p3R3/RqBP4/4P3/1PQ2PPP/6K1 b - - 0 1",
                "b4e1",
            ), // "WAC.028"
            (
                "r2q2k1/pp1rbppp/4pn2/2P5/1P3B2/6P1/P3QPBP/1R3RK1 w - - 0 1",
                "c5c6",
            ), // "WAC.029"
            (
                "1r3r2/4q1kp/b1pp2p1/5p2/pPn1N3/6P1/P3PPBP/2QRR1K1 w - - 0 1",
                "e4d6",
            ), // "WAC.030"
            (
                "6k1/p4p1p/1p3np1/2q5/4p3/4P1N1/PP3PPP/3Q2K1 w - - 0 1",
                "d1d8",
            ), // "WAC.032"
            (
                "7k/1b1r2p1/p6p/1p2qN2/3bP3/3Q4/P5PP/1B1R3K b - - 0 1",
                "d4g1",
            ), // "WAC.034"
            (
                "r3r2k/2R3pp/pp1q1p2/8/3P3R/7P/PP3PP1/3Q2K1 w - - 0 1",
                "h4h7",
            ), // "WAC.035"
            (
                "3r4/2p1rk2/1pQq1pp1/7p/1P1P4/P4P2/6PP/R1R3K1 b - - 0 1",
                "e7e1",
            ), // "WAC.036"
            (
                "2r5/2rk2pp/1pn1pb2/pN1p4/P2P4/1N2B3/nPR1KPPP/3R4 b - - 0 1",
                "c6d4",
            ), // "WAC.037"
            (
                "r1br2k1/pp2bppp/2nppn2/8/2P1PB2/2N2P2/PqN1B1PP/R2Q1R1K w - - 0 1",
                "c3a4",
            ), // "WAC.039"
            (
                "3r1r1k/1p4pp/p4p2/8/1PQR4/6Pq/P3PP2/2R3K1 b - - 0 1",
                "d8c8",
            ), // "WAC.040"
            (
                "r1b1r1k1/pp1n1pbp/1qp3p1/3p4/1B1P4/Q3PN2/PP2BPPP/R4RK1 w - - 0 1",
                "b4a5",
            ), // "WAC.042"
            (
                "3rb1k1/pq3pbp/4n1p1/3p4/2N5/2P2QB1/PP3PPP/1B1R2K1 b - - 0 1",
                "d5c4",
            ), // "WAC.044"
            (
                "7k/2p1b1pp/8/1p2P3/1P3r2/2P3Q1/1P5P/R4qBK b - - 0 1",
                "f1a1",
            ), // "WAC.045"
            (
                "r1bqr1k1/pp1nb1p1/4p2p/3p1p2/3P4/P1N1PNP1/1PQ2PP1/3RKB1R w K - 0 1",
                "c3b5",
            ), // "WAC.046"
            (
                "r1b2rk1/pp2bppp/2n1pn2/q5B1/2BP4/2N2N2/PP2QPPP/2R2RK1 b - - 0 1",
                "c6d4",
            ), // "WAC.047"
            (
                "1rbq1rk1/p1p1bppp/2p2n2/8/Q1BP4/2N5/PP3PPP/R1B2RK1 b - - 0 1",
                "b8b4",
            ), // "WAC.048"
            (
                "k4r2/1R4pb/1pQp1n1p/3P4/5p1P/3P2P1/r1q1R2K/8 w - - 0 1",
                "b7b6",
            ), // "WAC.050"
            (
                "r1bq1r2/pp4k1/4p2p/3pPp1Q/3N1R1P/2PB4/6P1/6K1 w - - 0 1",
                "f4g4",
            ), // "WAC.051"
            ("6k1/6p1/p7/3Pn3/5p2/4rBqP/P4RP1/5QK1 b - - 0 1", "e3e1"),     // "WAC.053"
            (
                "r3kr2/1pp4p/1p1p4/7q/4P1n1/2PP2Q1/PP4P1/R1BB2K1 b q - 0 1",
                "h5h1",
            ), // "WAC.054"
            (
                "r3r1k1/pp1q1pp1/4b1p1/3p2B1/3Q1R2/8/PPP3PP/4R1K1 w - - 0 1",
                "d4g7",
            ), // "WAC.055"
            (
                "r1bqk2r/pppp1ppp/5n2/2b1n3/4P3/1BP3Q1/PP3PPP/RNB1K1NR b KQkq - 0 1",
                "c5f2",
            ), // "WAC.056"
            (
                "r3q1kr/ppp5/3p2pQ/8/3PP1b1/5R2/PPP3P1/5RK1 w - - 0 1",
                "f3f8",
            ), // "WAC.057"
            ("8/8/2R5/1p2qp1k/1P2r3/2PQ2P1/5K2/8 w - - 0 1", "d3d1"),       // "WAC.058"
            (
                "r1b2rk1/2p1qnbp/p1pp2p1/5p2/2PQP3/1PN2N1P/PB3PP1/3R1RK1 w - - 0 1",
                "c3d5",
            ), // "WAC.059"
            (
                "6r1/3Pn1qk/p1p1P1rp/2Q2p2/2P5/1P4P1/P3R2P/5RK1 b - - 0 1",
                "g6g3",
            ), // "WAC.062"
            (
                "r1brnbk1/ppq2pp1/4p2p/4N3/3P4/P1PB1Q2/3B1PPP/R3R1K1 w - - 0 1",
                "e5f7",
            ), // "WAC.063"
            ("8/6pp/3q1p2/3n1k2/1P6/3NQ2P/5PP1/6K1 w - - 0 1", "g2g4"),     // "WAC.064"
            (
                "1r1r1qk1/p2n1p1p/bp1Pn1pQ/2pNp3/2P2P1N/1P5B/P6P/3R1RK1 w - - 0 1",
                "d5e7",
            ), // "WAC.065"
            (
                "1k1r2r1/ppq5/1bp4p/3pQ3/8/2P2N2/PP4P1/R4R1K b - - 0 1",
                "c7e5",
            ), // "WAC.066"
            (
                "3r2k1/p2q4/1p4p1/3rRp1p/5P1P/6PK/P3R3/3Q4 w - - 0 1",
                "e5d5",
            ), // "WAC.067"
            ("6k1/5ppp/1q6/2b5/8/2R1pPP1/1P2Q2P/7K w - - 0 1", "e2e3"),     // "WAC.068"
            (
                "2kr3r/pppq1ppp/3p1n2/bQ2p3/1n1PP3/1PN1BN1P/1PP2PP1/2KR3R b - - 0 1",
                "b4a2",
            ), // "WAC.070"
            (
                "2kr3r/pp1q1ppp/5n2/1Nb5/2Pp1B2/7Q/P4PPP/1R3RK1 w - - 0 1",
                "b5a7",
            ), // "WAC.071"
            (
                "r3r1k1/pp1n1ppp/2p5/4Pb2/2B2P2/B1P5/P5PP/R2R2K1 w - - 0 1",
                "e5e6",
            ), // "WAC.072"
            (
                "r1q3rk/1ppbb1p1/4Np1p/p3pP2/P3P3/2N4R/1PP1Q1PP/3R2K1 w - - 0 1",
                "e2d2",
            ), // "WAC.073"
            (
                "r3r1k1/pppq1ppp/8/8/1Q4n1/7P/PPP2PP1/RNB1R1K1 b - - 0 1",
                "d7d6",
            ), // "WAC.075"
            (
                "r1b1qrk1/2p2ppp/pb1pnn2/1p2pNB1/3PP3/1BP5/PP2QPPP/RN1R2K1 w - - 0 1",
                "g5f6",
            ), // "WAC.076"
            (
                "3r2k1/ppp2ppp/6q1/b4n2/3nQB2/2p5/P4PPP/RN3RK1 b - - 0 1",
                "f5g3",
            ), // "WAC.077"
            (
                "r2q3r/ppp2k2/4nbp1/5Q1p/2P1NB2/8/PP3P1P/3RR1K1 w - - 0 1",
                "e4g5",
            ), // "WAC.078"
            (
                "r4rk1/p1B1bpp1/1p2pn1p/8/2PP4/3B1P2/qP2QP1P/3R1RK1 w - - 0 1",
                "d1a1",
            ), // "WAC.080"
            (
                "r4rk1/1bR1bppp/4pn2/1p2N3/1P6/P3P3/4BPPP/3R2K1 b - - 0 1",
                "e7d6",
            ), // "WAC.081"
            (
                "3rr1k1/pp3pp1/4b3/8/2P1B2R/6QP/P3q1P1/5R1K w - - 0 1",
                "e4h7",
            ), // "WAC.082"
            (
                "r2q1r1k/2p1b1pp/p1n5/1p1Q1bN1/4n3/1BP1B3/PP3PPP/R4RK1 w - - 0 1",
                "d5g8",
            ), // "WAC.084"
            ("8/p7/1ppk1n2/5ppp/P1PP4/2P1K1P1/5N1P/8 b - - 0 1", "f6g4"),   // "WAC.086"
            ("8/p3k1p1/4r3/2ppNpp1/PP1P4/2P3KP/5P2/8 b - - 0 1", "e6e5"),   // "WAC.087"
            (
                "r6k/p1Q4p/2p1b1rq/4p3/B3P3/4P3/PPP3P1/4RRK1 b - - 0 1",
                "g6g2",
            ), // "WAC.088"
            (
                "3qrrk1/1pp2pp1/1p2bn1p/5N2/2P5/P1P3B1/1P4PP/2Q1RRK1 w - - 0 1",
                "f5g7",
            ), // "WAC.090"
            (
                "2qr2k1/4b1p1/2p2p1p/1pP1p3/p2nP3/PbQNB1PP/1P3PK1/4RB2 b - - 0 1",
                "b3e6",
            ), // "WAC.091"
            (
                "r4rk1/1p2ppbp/p2pbnp1/q7/3BPPP1/2N2B2/PPP4P/R2Q1RK1 b - - 0 1",
                "e6g4",
            ), // "WAC.092"
            (
                "r1b1k1nr/pp3pQp/4pq2/3pn3/8/P1P5/2P2PPP/R1B1KBNR w KQkq - 0 1",
                "c1h6",
            ), // "WAC.093"
            ("8/k7/p7/3Qp2P/n1P5/3KP3/1q6/8 b - - 0 1", "e5e4"),            // "WAC.094"
            ("2r5/1r6/4pNpk/3pP1qp/8/2P1QP2/5PK1/R7 w - - 0 1", "f6g4"),    // "WAC.095"
            ("6k1/5p2/p5np/4B3/3P4/1PP1q3/P3r1QP/6RK w - - 0 1", "g2a8"),   // "WAC.097"
            (
                "1r3rk1/5pb1/p2p2p1/Q1n1q2p/1NP1P3/3p1P1B/PP1R3P/1K2R3 b - - 0 1",
                "c5e4",
            ), // "WAC.098"
            (
                "r1bq1r1k/1pp1Np1p/p2p2pQ/4R3/n7/8/PPPP1PPP/R1B3K1 w - - 0 1",
                "e5h5",
            ), // "WAC.099"
            ("8/k1b5/P4p2/1Pp2p1p/K1P2P1P/8/3B4/8 w - - 0 1", "d2e3 b5b6"), // "WAC.100"
            ("5rk1/p5pp/8/8/2Pbp3/1P4P1/7P/4RN1K b - - 0 1", "d4c3"),       // "WAC.101"
            (
                "6k1/2pb1r1p/3p1PpQ/p1nPp3/1q2P3/2N2P2/PrB5/2K3RR w - - 0 1",
                "h6g6",
            ), // "WAC.103"
            ("5n2/pRrk2p1/P4p1p/4p3/3N4/5P2/6PP/6K1 w - - 0 1", "d4b5"),    // "WAC.107"
            ("r5k1/1q4pp/2p5/p1Q5/2P5/5R2/4RKPP/r7 w - - 0 1", "c5e5"),     // "WAC.108"
            (
                "rn2k1nr/pbp2ppp/3q4/1p2N3/2p5/QP6/PB1PPPPP/R3KB1R b KQkq - 0 1",
                "c4c3",
            ), // "WAC.109"
            (
                "2kr4/bp3p2/p2p2b1/P7/2q5/1N4B1/1PPQ2P1/2KR4 b - - 0 1",
                "a7e3",
            ), // "WAC.110"
            ("6k1/p5p1/5p2/2P2Q2/3pN2p/3PbK1P/7P/6q1 b - - 0 1", "g1f1"),   // "WAC.111"
            (
                "r4kr1/ppp5/4bq1b/7B/2PR1Q1p/2N3P1/PP3P1P/2K1R3 w - - 0 1",
                "e1e6",
            ), // "WAC.112"
            (
                "rnbqkb1r/1p3ppp/5N2/1p2p1B1/2P5/8/PP2PPPP/R2QKB1R b KQkq - 0 1",
                "d8f6",
            ), // "WAC.113"
            (
                "r1b1rnk1/1p4pp/p1p2p2/3pN2n/3P1PPq/2NBPR1P/PPQ5/2R3K1 w - - 0 1",
                "d3h7",
            ), // "WAC.114"
            ("4N2k/5rpp/1Q6/p3q3/8/P5P1/1P3P1P/5K2 w - - 0 1", "e8d6"),     // "WAC.115"
            (
                "r2r2k1/2p2ppp/p7/1p2P1n1/P6q/5P2/1PB1QP1P/R5RK b - - 0 1",
                "d8d2",
            ), // "WAC.116"
            (
                "3r1rk1/q4ppp/p1Rnp3/8/1p6/1N3P2/PP3QPP/3R2K1 b - - 0 1",
                "d6e4",
            ), // "WAC.117"
            (
                "r5k1/pb2rpp1/1p6/2p4q/5R2/2PB2Q1/P1P3PP/5R1K w - - 0 1",
                "f4h4",
            ), // "WAC.118"
            (
                "r2qr1k1/p1p2ppp/2p5/2b5/4nPQ1/3B4/PPP3PP/R1B2R1K b - - 0 1",
                "d8d3",
            ), // "WAC.119"
            (
                "6k1/5p1p/2bP2pb/4p3/2P5/1p1pNPPP/1P1Q1BK1/1q6 b - - 0 1",
                "c6f3",
            ), // "WAC.121"
            (
                "1k6/ppp4p/1n2pq2/1N2Rb2/2P2Q2/8/P4KPP/3r1B2 b - - 0 1",
                "d1f1",
            ), // "WAC.122"
            ("6k1/3r4/2R5/P5P1/1P4p1/8/4rB2/6K1 b - - 0 1", "g4g3"),        // "WAC.124"
            (
                "r1bqr1k1/pp3ppp/1bp5/3n4/3B4/2N2P1P/PPP1B1P1/R2Q1RK1 b - - 0 1",
                "b6d4",
            ), // "WAC.125"
            (
                "r5r1/pQ5p/1qp2R2/2k1p3/4P3/2PP4/P1P3PP/6K1 w - - 0 1",
                "f6c6",
            ), // "WAC.126"
            (
                "2k4r/1pr1n3/p1p1q2p/5pp1/3P1P2/P1P1P3/1R2Q1PP/1RB3K1 w - - 0 1",
                "b2b7",
            ), // "WAC.127"
            (
                "6rk/1pp2Qrp/3p1B2/1pb1p2R/3n1q2/3P4/PPP3PP/R6K w - - 0 1",
                "f7g6",
            ), // "WAC.128"
            (
                "3r1r1k/1b2b1p1/1p5p/2p1Pp2/q1B2P2/4P2P/1BR1Q2K/6R1 b - - 0 1",
                "b7f3",
            ), // "WAC.129"
            (
                "6k1/1pp3q1/5r2/1PPp4/3P1pP1/3Qn2P/3B4/4R1K1 b - - 0 1",
                "g7h6 g7h8",
            ), // "WAC.130"
            (
                "r1b1k2r/1pp1q2p/p1n3p1/3QPp2/8/1BP3B1/P5PP/3R1RK1 w kq - 0 1",
                "g3h4",
            ), // "WAC.133"
            (
                "3r2k1/p6p/2Q3p1/4q3/2P1p3/P3Pb2/1P3P1P/2K2BR1 b - - 0 1",
                "d8d1",
            ), // "WAC.134"
            (
                "3r1r1k/N2qn1pp/1p2np2/2p5/2Q1P2N/3P4/PP4PP/3R1RK1 b - - 0 1",
                "e6d4",
            ), // "WAC.135"
            (
                "3b1rk1/1bq3pp/5pn1/1p2rN2/2p1p3/2P1B2Q/1PB2PPP/R2R2K1 w - - 0 1",
                "d1d7",
            ), // "WAC.137"
            (
                "r1bq3r/ppppR1p1/5n1k/3P4/6pP/3Q4/PP1N1PP1/5K1R w - - 0 1",
                "h4h5",
            ), // "WAC.138"
            (
                "rnb3kr/ppp2ppp/1b6/3q4/3pN3/Q4N2/PPP2KPP/R1B1R3 w - - 0 1",
                "e4f6",
            ), // "WAC.139"
            (
                "r2b1rk1/pq4p1/4ppQP/3pB1p1/3P4/2R5/PP3PP1/5RK1 w - - 0 1",
                "e5c7 c3c7",
            ), // "WAC.140"
            (
                "4r1k1/p1qr1p2/2pb1Bp1/1p5p/3P1n1R/1B3P2/PP3PK1/2Q4R w - - 0 1",
                "c1f4",
            ), // "WAC.141"
            (
                "5b2/pp2r1pk/2pp1pRp/4rP1N/2P1P3/1P4QP/P3q1P1/5R1K w - - 0 1",
                "g6h6",
            ), // "WAC.143"
            (
                "r2q1rk1/pp3ppp/2p2b2/8/B2pPPb1/7P/PPP1N1P1/R2Q1RK1 b - - 0 1",
                "d4d3",
            ), // "WAC.144"
            (
                "r2r2k1/ppqbppbp/2n2np1/2pp4/6P1/1P1PPNNP/PBP2PB1/R2QK2R b KQ - 0 1",
                "f6g4",
            ), // "WAC.147"
            (
                "2r1k3/6pr/p1nBP3/1p3p1p/2q5/2P5/P1R4P/K2Q2R1 w - - 0 1",
                "g1g7",
            ), // "WAC.148"
            ("6k1/6p1/2p4p/4Pp2/4b1qP/2Br4/1P2RQPK/8 b - - 0 1", "e4g2"),   // "WAC.149"
            (
                "8/3b2kp/4p1p1/pr1n4/N1N4P/1P4P1/1K3P2/3R4 w - - 0 1",
                "a4c3",
            ), // "WAC.151"
            (
                "1br2rk1/1pqb1ppp/p3pn2/8/1P6/P1N1PN1P/1B3PP1/1QRR2K1 w - - 0 1",
                "c3e4",
            ), // "WAC.152"
            (
                "r1b2rk1/2p2ppp/p7/1p6/3P3q/1BP3bP/PP3QP1/RNB1R1K1 w - - 0 1",
                "f2f7",
            ), // "WAC.154"
            (
                "5bk1/1rQ4p/5pp1/2pP4/3n1PP1/7P/1q3BB1/4R1K1 w - - 0 1",
                "d5d6",
            ), // "WAC.155"
            (
                "r1b1qN1k/1pp3p1/p2p3n/4p1B1/8/1BP4Q/PP3KPP/8 w - - 0 1",
                "h3h6",
            ), // "WAC.156"
            (
                "5rk1/p4ppp/2p1b3/3Nq3/4P1n1/1p1B2QP/1PPr2P1/1K2R2R w - - 0 1",
                "d5e7",
            ), // "WAC.157"
            (
                "r1b2r2/5P1p/ppn3pk/2p1p1Nq/1bP1PQ2/3P4/PB4BP/1R3RK1 w - - 0 1",
                "g5e6",
            ), // "WAC.159"
            (
                "r3kbnr/p4ppp/2p1p3/8/Q1B3b1/2N1B3/PP3PqP/R3K2R w KQkq - 0 1",
                "c4d5",
            ), // "WAC.162"
            (
                "5rk1/2p4p/2p4r/3P4/4p1b1/1Q2NqPp/PP3P1K/R4R2 b - - 0 1",
                "f3g2",
            ), // "WAC.163"
            ("8/6pp/4p3/1p1n4/1NbkN1P1/P4P1P/1PR3K1/r7 w - - 0 1", "c2c4"), // "WAC.164"
            ("1r5k/p1p3pp/8/8/4p3/P1P1R3/1P1Q1qr1/2KR4 w - - 0 1", "e3e2"), // "WAC.165"
            (
                "r3r1k1/5pp1/p1p4p/2Pp4/8/q1NQP1BP/5PP1/4K2R b K - 0 1",
                "d5d4",
            ), // "WAC.166"
            (
                "r3k2r/pb1q1p2/8/2p1pP2/4p1p1/B1P1Q1P1/P1P3K1/R4R2 b kq - 0 1",
                "d7d2",
            ), // "WAC.168"
            (
                "5rk1/1pp3bp/3p2p1/2PPp3/1P2P3/2Q1B3/4q1PP/R5K1 b - - 0 1",
                "g7h6",
            ), // "WAC.169"
            (
                "5r1k/6Rp/1p2p3/p2pBp2/1qnP4/4P3/Q4PPP/6K1 w - - 0 1",
                "a2c4",
            ), // "WAC.170"
            (
                "2rq4/1b2b1kp/p3p1p1/1p1nNp2/7P/1B2B1Q1/PP3PP1/3R2K1 w - - 0 1",
                "e3h6",
            ), // "WAC.171"
            (
                "2r1b3/1pp1qrk1/p1n1P1p1/7R/2B1p3/4Q1P1/PP3PP1/3R2K1 w - - 0 1",
                "e3h6",
            ), // "WAC.173"
            (
                "r5k1/pppb3p/2np1n2/8/3PqNpP/3Q2P1/PPP5/R4RK1 w - - 0 1",
                "f4h5",
            ), // "WAC.175"
            (
                "3r2k1/p1rn1p1p/1p2pp2/6q1/3PQNP1/5P2/P1P4R/R5K1 w - - 0 1",
                "f4e6",
            ), // "WAC.178"
            (
                "r1q2rk1/p3bppb/3p1n1p/2nPp3/1p2P1P1/6NP/PP2QPB1/R1BNK2R b KQ - 0 1",
                "f6d5",
            ), // "WAC.180"
            (
                "r3k2r/2p2p2/p2p1n2/1p2p3/4P2p/1PPPPp1q/1P5P/R1N2QRK b kq - 0 1",
                "f6g4",
            ), // "WAC.181"
            (
                "r1b2rk1/ppqn1p1p/2n1p1p1/2b3N1/2N5/PP1BP3/1B3PPP/R2QK2R w KQ - 0 1",
                "d1h5",
            ), // "WAC.182"
            (
                "6k1/5p2/p3p3/1p3qp1/2p1Qn2/2P1R3/PP1r1PPP/4R1K1 b - - 0 1",
                "f4h3",
            ), // "WAC.187"
            ("3RNbk1/pp3p2/4rQpp/8/1qr5/7P/P4P2/3R2K1 w - - 0 1", "f6g7"),  // "WAC.188"
            (
                "8/p2b2kp/1q1p2p1/1P1Pp3/4P3/3B2P1/P2Q3P/2Nn3K b - - 0 1",
                "d7h3",
            ), // "WAC.190"
            (
                "r3k3/ppp2Npp/4Bn2/2b5/1n1pp3/N4P2/PPP3qP/R2QKR2 b Qq - 0 1",
                "b4d3",
            ), // "WAC.192"
            (
                "5rk1/ppq2ppp/2p5/4bN2/4P3/6Q1/PPP2PPP/3R2K1 w - - 0 1",
                "f5h6",
            ), // "WAC.194"
            (
                "3r1rk1/1p3p2/p3pnnp/2p3p1/2P2q2/1P5P/PB2QPPN/3RR1K1 w - - 0 1",
                "g2g3",
            ), // "WAC.195"
            (
                "rr4k1/p1pq2pp/Q1n1pn2/2bpp3/4P3/2PP1NN1/PP3PPP/R1B1K2R b KQ - 0 1",
                "c6b4",
            ), // "WAC.196"
            ("7k/1p4p1/7p/3P1n2/4Q3/2P2P2/PP3qRP/7K b - - 0 1", "f2f1"),    // "WAC.197"
            (
                "2br2k1/ppp2p1p/4p1p1/4P2q/2P1Bn2/2Q5/PP3P1P/4R1RK b - - 0 1",
                "d8d3",
            ), // "WAC.198"
            (
                "2rqrn1k/pb4pp/1p2pp2/n2P4/2P3N1/P2B2Q1/1B3PPP/2R1R1K1 w - - 0 1",
                "b2f6",
            ), // "WAC.200"
            (
                "2b2r1k/4q2p/3p2pQ/2pBp3/8/6P1/1PP2P1P/R5K1 w - - 0 1",
                "a1a7",
            ), // "WAC.201"
            (
                "QR2rq1k/2p3p1/3p1pPp/8/4P3/8/P1r3PP/1R4K1 b - - 0 1",
                "c2a2",
            ), // "WAC.202"
            (
                "r4rk1/5ppp/p3q1n1/2p2NQ1/4n3/P3P3/1B3PPP/1R3RK1 w - - 0 1",
                "g5h6",
            ), // "WAC.203"
            (
                "r1b1qrk1/1p3ppp/p1p5/3Nb3/5N2/P7/1P4PQ/K1R1R3 w - - 0 1",
                "e1e5",
            ), // "WAC.204"
            (
                "r3rnk1/1pq2bb1/p4p2/3p1Pp1/3B2P1/1NP4R/P1PQB3/2K4R w - - 0 1",
                "d2g5",
            ), // "WAC.205"
            ("1Qq5/2P1p1kp/3r1pp1/8/8/7P/p4PP1/2R3K1 b - - 0 1", "d6c6"),   // "WAC.206"
            (
                "r1bq2kr/p1pp1ppp/1pn1p3/4P3/2Pb2Q1/BR6/P4PPP/3K1BNR w - - 0 1",
                "g4g7",
            ), // "WAC.207"
            (
                "3r1bk1/ppq3pp/2p5/2P2Q1B/8/1P4P1/P6P/5RK1 w - - 0 1",
                "h5f7",
            ), // "WAC.208"
            (
                "3r1rk1/pp1q1ppp/3pn3/2pN4/5PP1/P5PQ/1PP1B3/1K1R4 w - - 0 1",
                "d1h1",
            ), // "WAC.210"
            (
                "rn1qr2Q/pbppk1p1/1p2pb2/4N3/3P4/2N5/PPP3PP/R4RK1 w - - 0 1",
                "h8g7",
            ), // "WAC.212"
            (
                "3r1r1k/1b4pp/ppn1p3/4Pp1R/Pn5P/3P4/4QP2/1qB1NKR1 w - - 0 1",
                "h5h7",
            ), // "WAC.213"
            (
                "3r2k1/pb1q1pp1/1p2pb1p/8/3N4/P2QB3/1P3PPP/1Br1R1K1 w - - 0 1",
                "d3h7",
            ), // "WAC.215"
            ("7k/p4q1p/1pb5/2p5/4B2Q/2P1B3/P6P/7K b - - 0 1", "f7f1"),      // "WAC.219"
            (
                "3rr1k1/ppp2ppp/8/5Q2/4n3/1B5R/PPP1qPP1/5RK1 b - - 0 1",
                "e2f1",
            ), // "WAC.220"
            (
                "2r1r2k/1q3ppp/p2Rp3/2p1P3/6QB/p3P3/bP3PPP/3R2K1 w - - 0 1",
                "h4f6",
            ), // "WAC.222"
            (
                "2k1rb1r/ppp3pp/2np1q2/5b2/2B2P2/2P1BQ2/PP1N1P1P/2KR3R b - - 0 1",
                "d6d5",
            ), // "WAC.227"
            (
                "r4rk1/1bq1bp1p/4p1p1/p2p4/3BnP2/1N1B3R/PPP3PP/R2Q2K1 w - - 0 1",
                "d3e4",
            ), // "WAC.228"
            (
                "r4rk1/1b1nqp1p/p5p1/1p2PQ2/2p5/5N2/PP3PPP/R1BR2K1 w - - 0 1",
                "c1g5",
            ), // "WAC.231"
            ("1R6/p5pk/4p2p/4P3/8/2r3qP/P3R1b1/4Q1K1 b - - 0 1", "c3c1"),   // "WAC.236"
            (
                "r5k1/pQp2qpp/8/4pbN1/3P4/6P1/PPr4P/1K1R3R b - - 0 1",
                "c2c1",
            ), // "WAC.237"
            (
                "1k1r4/pp1r1pp1/4n1p1/2R5/2Pp1qP1/3P2QP/P4PB1/1R4K1 w - - 0 1",
                "g2b7",
            ), // "WAC.238"
            (
                "2b4k/p1b2p2/2p2q2/3p1PNp/3P2R1/3B4/P1Q2PKP/4r3 w - - 0 1",
                "c2c6",
            ), // "WAC.240"
            (
                "r1b1r1k1/pp1nqp2/2p1p1pp/8/4N3/P1Q1P3/1P3PPP/1BRR2K1 w - - 0 1",
                "d1d7",
            ), // "WAC.242"
            (
                "1b5k/7P/p1p2np1/2P2p2/PP3P2/4RQ1R/q2r3P/6K1 w - - 0 1",
                "e3e8",
            ), // "WAC.250"
            ("k5r1/p4b2/2P5/5p2/3P1P2/4QBrq/P5P1/4R1K1 w - - 0 1", "e3e8"), // "WAC.253"
            (
                "r6k/pp3p1p/2p1bp1q/b3p3/4Pnr1/2PP2NP/PP1Q1PPN/R2B2RK b - - 0 1",
                "f4h3",
            ), // "WAC.254"
            (
                "3r3r/p4pk1/5Rp1/3q4/1p1P2RQ/5N2/P1P4P/2b4K w - - 0 1",
                "f6g6",
            ), // "WAC.255"
            (
                "3r1rk1/1pb1qp1p/2p3p1/p7/P2Np2R/1P5P/1BP2PP1/3Q1BK1 w - - 0 1",
                "d4f5",
            ), // "WAC.256"
            (
                "4r1k1/pq3p1p/2p1r1p1/2Q1p3/3nN1P1/1P6/P1P2P1P/3RR1K1 w - - 0 1",
                "d1d4",
            ), // "WAC.257"
            (
                "r3brkn/1p5p/2p2Ppq/2Pp3B/3Pp2Q/4P1R1/6PP/5R1K w - - 0 1",
                "h5g6",
            ), // "WAC.258"
            (
                "2r2b1r/p1Nk2pp/3p1p2/N2Qn3/4P3/q6P/P4PP1/1R3K1R w - - 0 1",
                "d5e6",
            ), // "WAC.260"
            (
                "6k1/p1B1b2p/2b3r1/2p5/4p3/1PP1N1Pq/P2R1P2/3Q2K1 b - - 0 1",
                "g6h6",
            ), // "WAC.262"
            (
                "rnbqr2k/pppp1Qpp/8/b2NN3/2B1n3/8/PPPP1PPP/R1B1K2R w KQ - 0 1",
                "f7g8",
            ), // "WAC.263"
            (
                "2r1k2r/2pn1pp1/1p3n1p/p3PP2/4q2B/P1P5/2Q1N1PP/R4RK1 w k - 0 1",
                "e5f6",
            ), // "WAC.265"
            (
                "r3q2r/2p1k1p1/p5p1/1p2Nb2/1P2nB2/P7/2PNQbPP/R2R3K b - - 0 1",
                "h8h2",
            ), // "WAC.266"
            (
                "2r1kb1r/pp3ppp/2n1b3/1q1N2B1/1P2Q3/8/P4PPP/3RK1NR w Kk - 0 1",
                "d5c7",
            ), // "WAC.267"
            (
                "2kr2nr/pp1n1ppp/2p1p3/q7/1b1P1B2/P1N2Q1P/1PP1BPP1/R3K2R w KQ - 0 1",
                "a3b4",
            ), // "WAC.269"
            (
                "2r1r1k1/pp1q1ppp/3p1b2/3P4/3Q4/5N2/PP2RPPP/4R1K1 w - - 0 1",
                "d4g4",
            ), // "WAC.270"
            ("2kr4/ppp3Pp/4RP1B/2r5/5P2/1P6/P2p4/3K4 w - - 0 1", "e6d6"),   // "WAC.271"
            (
                "nrq4r/2k1p3/1p1pPnp1/pRpP1p2/P1P2P2/2P1BB2/1R2Q1P1/6K1 w - - 0 1",
                "e3c5",
            ), // "WAC.272"
            (
                "r2qkb1r/pppb2pp/2np1n2/5pN1/2BQP3/2N5/PPP2PPP/R1B1K2R w KQkq - 0 1",
                "c4f7",
            ), // "WAC.278"
            ("2R5/2R4p/5p1k/6n1/8/1P2QPPq/r7/6K1 w - - 0 1", "c7h7"),       // "WAC.281"
            ("6k1/2p3p1/1p1p1nN1/1B1P4/4PK2/8/2r3b1/7R w - - 0 1", "h1h8"), // "WAC.282"
            (
                "3q1rk1/4bp1p/1n2P2Q/3p1p2/6r1/Pp2R2N/1B4PP/7K w - - 0 1",
                "h3g5",
            ), // "WAC.283"
            ("3r1k2/1p6/p4P2/2pP2Qb/8/1P1KB3/P6r/8 b - - 0 1", "d8d5"),     // "WAC.286"
            (
                "r1b2rk1/p4ppp/1p1Qp3/4P2N/1P6/8/P3qPPP/3R1RK1 w - - 0 1",
                "h5f6",
            ), // "WAC.288"
            (
                "2r3k1/5p1p/p3q1p1/2n3P1/1p1QP2P/1P4N1/PK6/2R5 b - - 0 1",
                "e6e5",
            ), // "WAC.289"
            ("4r3/1Q1qk2p/p4pp1/3Pb3/P7/6PP/5P2/4R1K1 w - - 0 1", "d5d6"),  // "WAC.292"
            (
                "1nbq1r1k/3rbp1p/p1p1pp1Q/1p6/P1pPN3/5NP1/1P2PPBP/R4RK1 w - - 0 1",
                "f3g5",
            ), // "WAC.293"
            (
                "4r3/p4r1p/R1p2pp1/1p1bk3/4pNPP/2P1K3/2P2P2/3R4 w - - 0 1",
                "d1d5",
            ), // "WAC.295"
            ("3Q4/p3b1k1/2p2rPp/2q5/4B3/P2P4/7P/6RK w - - 0 1", "d8h8"),    // "WAC.298"
            (
                "b2b1r1k/3R1ppp/4qP2/4p1PQ/4P3/5B2/4N1K1/8 w - - 0 1",
                "g5g6",
            ), // "WAC.300"
        ];

        let mut state: u64 = 17;
        let mut index = obvious_positions.len();

        while index > 1 {
            index -= 1;

            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);

            let j = state % (index as u64 + 1) as u64;

            let temp = obvious_positions[j as usize];
            obvious_positions[j as usize] = obvious_positions[index];
            obvious_positions[index] = temp;
        }

        obvious_positions
    };

    #[test]
    fn win_at_chess() {
        let (sender, receiver) = mpsc::channel();
        let thread_count = thread::available_parallelism().unwrap().get();
        let (divide, remainder) = (
            OBVIOUS_POSITIONS_RANDOMISED.len() / thread_count,
            OBVIOUS_POSITIONS_RANDOMISED.len() % thread_count,
        );

        for i in 0..thread_count {
            let sender = sender.clone();

            let mut end = (i + 1) * divide;
            if i == thread_count - 1 {
                end += remainder;
            }
            let positions = &OBVIOUS_POSITIONS_RANDOMISED[i * divide..end];

            thread::spawn(move || {
                let mut search = Search::new(
                    Board::from_fen(Board::START_POSITION_FEN),
                    megabytes_to_capacity(32),
                );

                for (position, solutions) in positions {
                    let board = Board::from_fen(position);
                    let matches_solution = |answer| {
                        for solution in solutions.split_whitespace() {
                            if solution == uci::encode_move(answer) {
                                return true;
                            }
                        }
                        false
                    };

                    search.new_board(board);
                    search.clear_cache_for_new_game();
                    search.clear_for_new_search();

                    let search_start = Time::now();
                    let result = search.iterative_deepening(&search_start, 2000, 2000, &mut |_| {});

                    sender
                        .send((
                            position,
                            matches_solution(search.pv.root_best_move().decode()),
                            search.quiescence_call_count,
                        ))
                        .unwrap();
                }
            });
        }

        let mut failures = 0;
        let mut successes = 0;
        let mut total_quiescence_call_count = 0;
        println!("Number | Position | Nodes");
        for (position, success, quiescence_call_count) in receiver.iter() {
            if success {
                successes += 1;
                total_quiescence_call_count += quiescence_call_count;
                println!("\x1b[92mSuccess #{successes:<4} {position:<72} {quiescence_call_count:>8}\x1b[0m");
            } else {
                failures += 1;
                println!("\x1b[91mFailure #{failures:<4} {position}\x1b[0m");
            };
            if successes + failures == OBVIOUS_POSITIONS_RANDOMISED.len() {
                break;
            }
        }
        println!("Successes: {successes} Failures: {failures} Total nodes: {total_quiescence_call_count}");
    }
}
