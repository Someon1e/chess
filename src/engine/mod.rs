mod encoded_move;
mod eval;
mod eval_data;
mod move_ordering;
mod transposition;

use crate::{
    board::{zobrist::Zobrist, Board},
    move_generator::{move_data::Move, MoveGenerator},
};

use self::{
    encoded_move::EncodedMove,
    eval::Eval,
    move_ordering::MoveOrderer,
    transposition::{NodeType, NodeValue, TRANSPOSITION_CAPACITY},
};

pub struct Engine<'a> {
    board: &'a mut Board,
    transposition_table: Vec<Option<NodeValue>>,
    repetition_table: Vec<Zobrist>,
    best_move: EncodedMove,
    best_score: i32,
}

const CHECKMATE_SCORE: i32 = -i32::MAX + 1;

impl<'a> Engine<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            board,
            transposition_table: vec![None; TRANSPOSITION_CAPACITY],
            repetition_table: Vec::with_capacity(5),
            best_move: EncodedMove::NONE,
            best_score: -i32::MAX,
        }
    }

    pub fn board(&self) -> &Board {
        self.board
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = Eval::evaluate(self);
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let mut return_value = None;
        MoveGenerator::new(self.board).gen(
            &mut |capture| {
                if return_value.is_some() {
                    return;
                }

                self.board.make_move(&capture);
                let score = -self.quiescence_search(-beta, -alpha);
                self.board.unmake_move(&capture);

                if score >= beta {
                    return_value = Some(beta);
                    return;
                }
                if score > alpha {
                    alpha = score;
                }
            },
            true,
        );
        return_value.unwrap_or(alpha)
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
    pub fn negamax(
        &mut self,
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

        let zobrist_index = zobrist_key.index(TRANSPOSITION_CAPACITY);

        let mut hash_move = &EncodedMove::NONE;

        // TODO: thoroughly test this works
        if let Some(saved) = &self.transposition_table[zobrist_index] {
            if saved.zobrist_key != zobrist_key {
                // eprintln!("Collision!")
            } else {
                if saved.ply_remaining >= depth - ply {
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
                hash_move = &saved.best_move
            }
        }

        if ply == depth {
            let evaluation = self.quiescence_search(alpha, beta);
            self.transposition_table[zobrist_index] = Some(NodeValue {
                zobrist_key,
                ply_remaining: depth - ply,
                node_type: NodeType::Exact,
                value: evaluation,
                best_move: EncodedMove::NONE,
            });
            return evaluation;
        };

        let move_generator = MoveGenerator::new(self.board);

        if ply == 0 {
            hash_move = &self.best_move;
        }
        let (mut moves, mut move_guesses, move_count) =
            MoveOrderer::get_sorted_moves(self, &move_generator, hash_move);

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
            let move_data =
                MoveOrderer::put_highest_guessed_move_on_top(&mut moves, &mut move_guesses, index);

            self.make_move(&move_data.decode());

            let mut normal_search = index < 3 || (depth - ply) < 3 || move_generator.is_in_check();
            let mut score = 0;
            if !normal_search {
                score = -self.negamax(ply + 2, depth, should_cancel, -beta, -alpha);
                if score > alpha {
                    normal_search = true;
                }
            }

            if normal_search {
                score = -self.negamax(ply + 1, depth, should_cancel, -beta, -alpha);
            }
            self.unmake_move(&move_data.decode());
            if should_cancel() {
                return 0;
            }
            if score >= beta {
                node_type = NodeType::Beta;
                best_move = move_data;
                self.transposition_table[zobrist_index] = Some(NodeValue {
                    zobrist_key,
                    ply_remaining: depth - ply,
                    node_type,
                    value: score,
                    best_move,
                });
                return beta;
            }
            if score > alpha {
                node_type = NodeType::Exact;
                alpha = score;
                best_move = move_data;

                if ply == 0 {
                    self.best_move = best_move;
                    self.best_score = score;
                }
            }
            if index == 0 {
                break;
            }
            index = index - 1;
        }
        self.transposition_table[zobrist_index] = Some(NodeValue {
            zobrist_key,
            ply_remaining: depth - ply,
            node_type,
            value: alpha,
            best_move,
        });

        alpha
    }
    pub fn iterative_deepening(
        &mut self,
        depth_completed: &mut dyn FnMut(u16, (EncodedMove, i32)),
        should_cancel: &mut dyn FnMut() -> bool,
    ) -> (EncodedMove, i32) {
        let mut depth = 0;
        while !should_cancel() {
            depth += 1;
            self.negamax(0, depth, should_cancel, -i32::MAX, i32::MAX);
            if should_cancel() {
                break;
            }

            if self.best_move.is_none() || self.best_score.abs() == CHECKMATE_SCORE.abs() {
                break;
            }
            depth_completed(depth, (self.best_move, self.best_score));
        }
        (self.best_move, self.best_score)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        engine::{eval::Eval, Engine},
    };

    #[test]
    fn quiescence_search_works() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        let mut quiet =
            Board::from_fen("rnb1kbnr/ppp1pppp/8/3q4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            Engine::new(&mut board).quiescence_search(-i32::MAX, i32::MAX),
            Eval::evaluate(&Engine::new(&mut quiet))
        )
    }

    #[test]
    fn test_evaluation() {
        let mut starting_rank_pawn = Board::from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1");
        let mut one_step_from_promoting_pawn = Board::from_fen("8/4P3/8/8/8/8/8/8 w - - 0 1");
        assert!(
            Eval::evaluate(&Engine::new(&mut one_step_from_promoting_pawn))
                > Eval::evaluate(&Engine::new(&mut starting_rank_pawn))
        )
    }
}
