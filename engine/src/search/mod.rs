mod encoded_move;
mod eval;
mod eval_data;
mod move_ordering;
mod transposition;

use crate::{
    board::{zobrist::Zobrist, Board},
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

use self::{
    encoded_move::EncodedMove,
    eval::Eval,
    move_ordering::MoveOrderer,
    transposition::{NodeType, NodeValue, TRANSPOSITION_CAPACITY},
};

const CHECKMATE_SCORE: i32 = -i32::MAX + 1;

const NOT_LATE_MOVES: usize = 3;
const NULL_MOVE_R: u16 = 3;

pub struct Search<'a> {
    board: &'a mut Board,
    transposition_table: Vec<Option<NodeValue>>,
    repetition_table: Vec<Zobrist>,
    killer_moves: [EncodedMove; 32],
    history_heuristic: [[[u16; 64]; 64]; 2],
    best_move: EncodedMove,
    best_score: i32,

    #[cfg(test)]
    times_evaluation_was_called: u32,
}

impl<'a> Search<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            board,
            transposition_table: vec![None; TRANSPOSITION_CAPACITY],
            repetition_table: Vec::with_capacity(5),
            killer_moves: [EncodedMove::NONE; 32],
            history_heuristic: [[[0; 64]; 64]; 2],
            best_move: EncodedMove::NONE,
            best_score: -i32::MAX,

            #[cfg(test)]
            times_evaluation_was_called: 0,
        }
    }

    pub fn board(&self) -> &Board {
        self.board
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        #[cfg(test)]
        {
            self.times_evaluation_was_called += 1
        }

        let stand_pat = Eval::evaluate(self);

        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let move_generator = MoveGenerator::new(self.board);
        let (mut move_guesses, move_count) =
            MoveOrderer::get_sorted_moves_captures_only(self, &move_generator);
        if move_count == 0 {
            return alpha;
        }
        let mut index = move_count - 1;
        loop {
            let move_data = MoveOrderer::put_highest_guessed_move_on_top(&mut move_guesses, index)
                .move_data
                .decode();

            self.board.make_move(&move_data);
            let score = -self.quiescence_search(-beta, -alpha);
            self.board.unmake_move(&move_data);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }

            if index == 0 {
                break;
            }
            index -= 1;
        }
        alpha
    }
    pub fn make_move(&mut self, move_data: &Move) {
        self.repetition_table.push(self.board.zobrist_key());
        self.board.make_move(move_data);
    }
    pub fn unmake_move(&mut self, move_data: &Move) {
        self.board.unmake_move(move_data);

        let zobrist_key = self.board.zobrist_key();
        for (index, other_key) in self.repetition_table.iter().enumerate().rev() {
            if *other_key == zobrist_key {
                self.repetition_table.swap_remove(index);
                break;
            }
        }
    }
    fn negamax(
        &mut self,

        allow_null_move: bool,
        ply: u16,
        depth: u16,

        should_cancel: &mut dyn FnMut() -> bool,

        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        if should_cancel() {
            return 0;
        }

        let zobrist_key = self.board.zobrist_key();

        if ply != 0 && self.repetition_table.contains(&zobrist_key) {
            return 0;
        }

        let ply_remaining = depth - ply;

        let zobrist_index = zobrist_key % self.transposition_table.len();

        let mut hash_move = EncodedMove::NONE;

        if let Some(saved) = self.transposition_table[zobrist_index] {
            if saved.zobrist_key != zobrist_key {
                // eprintln!("Collision!")
            } else {
                if saved.ply_remaining >= ply_remaining {
                    let node_type = &saved.node_type;
                    match node_type {
                        NodeType::Exact => return saved.value,
                        NodeType::Beta => {
                            if saved.value >= beta {
                                return saved.value;
                            }
                        }
                        NodeType::Alpha => {
                            if saved.value <= alpha {
                                return saved.value;
                            }
                        }
                    }
                }
                hash_move = saved.best_move
            }
        }

        if ply_remaining == 0 {
            let evaluation = self.quiescence_search(alpha, beta);
            self.transposition_table[zobrist_index] = Some(NodeValue {
                zobrist_key,
                ply_remaining,
                node_type: NodeType::Exact,
                value: evaluation,
                best_move: EncodedMove::NONE,
            });
            return evaluation;
        };

        let move_generator = MoveGenerator::new(self.board);

        if allow_null_move && ply_remaining > NULL_MOVE_R && !move_generator.is_in_check() {
            // TODO: thoroughly test this works

            self.board.make_null_move();
            let score = -self.negamax(
                false,
                ply + 1,
                depth - NULL_MOVE_R + 1,
                should_cancel,
                -beta,
                -beta + 1,
            );
            self.board.unmake_null_move();
            if should_cancel() {
                return 0;
            }
            if score >= beta {
                return beta;
            }
        }

        if ply == 0 {
            hash_move = self.best_move;
        }
        let (mut move_guesses, move_count) = MoveOrderer::get_sorted_moves(
            self,
            &move_generator,
            hash_move,
            if (ply as usize) < self.killer_moves.len() {
                self.killer_moves[ply as usize]
            } else {
                EncodedMove::NONE
            },
        );

        if move_count == 0 {
            if move_generator.is_in_check() {
                if ply == 0 {
                    self.best_score = CHECKMATE_SCORE;
                }
                return CHECKMATE_SCORE;
            }
            if ply == 0 {
                self.best_score = 0
            }
            return 0;
        }

        let mut node_type = NodeType::Alpha;
        let mut best_move = EncodedMove::NONE;
        let mut index = move_count - 1;
        loop {
            let encoded_move_data =
                MoveOrderer::put_highest_guessed_move_on_top(&mut move_guesses, index).move_data;
            let move_data = encoded_move_data.decode();

            let is_capture = move_generator.enemy_piece_bit_board().get(&move_data.to);
            self.make_move(&move_data);

            let check_extension = MoveGenerator::calculate_is_in_check(self.board);

            let mut normal_search =
                check_extension || is_capture || index < NOT_LATE_MOVES || (ply_remaining) < 3;
            let mut score = 0;
            if !normal_search {
                score = -self.negamax(true, ply + 1, depth - 1, should_cancel, -alpha - 1, -alpha);
                if score > alpha {
                    normal_search = true;
                }
            }

            if normal_search {
                score = -self.negamax(
                    true,
                    ply + 1,
                    depth + (check_extension as u16),
                    should_cancel,
                    -beta,
                    -alpha,
                );
            }
            self.unmake_move(&move_data);
            if should_cancel() {
                return 0;
            }
            if score >= beta {
                if !is_capture {
                    if move_data.flag == Flag::None && (ply as usize) < self.killer_moves.len() {
                        self.killer_moves[ply as usize] = encoded_move_data
                    }

                    self.history_heuristic[self.board.white_to_move as usize]
                        [move_data.from.usize()][move_data.to.usize()] +=
                        ply_remaining * ply_remaining;
                }
                node_type = NodeType::Beta;
                best_move = encoded_move_data;
                self.transposition_table[zobrist_index] = Some(NodeValue {
                    zobrist_key,
                    ply_remaining,
                    node_type,
                    value: score,
                    best_move,
                });
                return beta;
            }
            if score > alpha {
                node_type = NodeType::Exact;
                alpha = score;
                best_move = encoded_move_data;

                if ply == 0 {
                    self.best_move = best_move;
                    self.best_score = score;
                }
            }
            if index == 0 {
                break;
            }
            index -= 1;
        }
        self.transposition_table[zobrist_index] = Some(NodeValue {
            zobrist_key,
            ply_remaining,
            node_type,
            value: alpha,
            best_move,
        });

        alpha
    }
    pub fn depth_by_depth(
        &mut self,
        depth_completed: &mut dyn FnMut(u16, (EncodedMove, i32)) -> bool,
    ) -> (u16, EncodedMove, i32) {
        let mut depth = 0;
        loop {
            depth += 1;
            self.negamax(false, 0, depth, &mut || false, -i32::MAX, i32::MAX);

            if self.best_move.is_none() || self.best_score.abs() == CHECKMATE_SCORE.abs() {
                return (depth, self.best_move, self.best_score);
            }
            let stop = depth_completed(depth, (self.best_move, self.best_score));
            if stop {
                break;
            }
        }
        (depth, self.best_move, self.best_score)
    }
    pub fn iterative_deepening(
        &mut self,
        depth_completed: &mut dyn FnMut(u16, (EncodedMove, i32)),
        should_cancel: &mut dyn FnMut() -> bool,
    ) -> (u16, EncodedMove, i32) {
        let mut depth = 0;
        while !should_cancel() {
            depth += 1;
            self.negamax(false, 0, depth, should_cancel, -i32::MAX, i32::MAX);
            if should_cancel() {
                break;
            }

            if self.best_move.is_none() || self.best_score.abs() == CHECKMATE_SCORE.abs() {
                break;
            }
            depth_completed(depth, (self.best_move, self.best_score));
        }
        (depth, self.best_move, self.best_score)
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread};

    use crate::{
        board::Board,
        search::{eval::Eval, Search},
        timer::timer::Time,
        uci,
    };

    #[test]
    fn quiescence_search_works() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        let mut quiet =
            Board::from_fen("rnb1kbnr/ppp1pppp/8/3q4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            Search::new(&mut board).quiescence_search(-i32::MAX, i32::MAX),
            Eval::evaluate(&Search::new(&mut quiet))
        )
    }

    #[test]
    fn test_evaluation() {
        let mut starting_rank_pawn = Board::from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1");
        let mut one_step_from_promoting_pawn = Board::from_fen("8/4P3/8/8/8/8/8/8 w - - 0 1");
        assert!(
            Eval::evaluate(&Search::new(&mut one_step_from_promoting_pawn))
                > Eval::evaluate(&Search::new(&mut starting_rank_pawn))
        )
    }

    const OBVIOUS_POSITIONS: [(&str, &str); 201] = [
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
            "b4d6",
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
    #[test]
    fn win_at_chess() {
        let (sender, receiver) = mpsc::channel();
        let mut threads = vec![];
        const THREAD_COUNT: usize = 8;
        let (divide, remainder) = (
            OBVIOUS_POSITIONS.len() / THREAD_COUNT,
            OBVIOUS_POSITIONS.len() % THREAD_COUNT,
        );
        for i in 0..THREAD_COUNT {
            let sender = sender.clone();

            let mut end = (i + 1) * divide;
            if i == THREAD_COUNT - 1 {
                end += remainder;
            }
            let positions = &OBVIOUS_POSITIONS[i * divide..end];

            threads.push(thread::spawn(move || {
                for (position, solution) in positions {
                    let mut board = Board::from_fen(position);
                    let solution = uci::decode_move(&board, &solution[0..4]);
                    let mut search = Search::new(&mut board);
                    let search_start = Time::now();
                    let result = search.depth_by_depth(&mut |depth, answer| {
                        if answer.0.decode() == solution {
                            true
                        } else {
                            2000 < search_start.miliseconds()
                        }
                    });

                    sender
                        .send((
                            position,
                            result.1.decode() == solution,
                            search.times_evaluation_was_called,
                        ))
                        .unwrap();
                }
            }));
        }

        let mut failures = 0;
        let mut successes = 0;
        let mut total_times_evaluation_was_called = 0;
        for (position, success, times_evaluation_was_called) in receiver.iter() {
            if success {
                successes += 1;
                total_times_evaluation_was_called += times_evaluation_was_called;
                println!("Success #{successes} {position}");
            } else {
                failures += 1;
                println!("Failure #{failures} {position}");
            };
            if successes + failures == OBVIOUS_POSITIONS.len() {
                break;
            }
        }
        println!("Successes: {successes} Failures: {failures} Times eval was called: {total_times_evaluation_was_called}")
    }
}
