mod encoded_move;
mod eval_data;
mod transposition;

use fnv::FnvHashSet;

use crate::{
    board::{bit_board::BitBoard, piece::Piece, zobrist::Zobrist, Board},
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

use self::{
    encoded_move::EncodedMove,
    transposition::{NodeType, NodeValue, TRANSPOSITION_CAPACITY},
};

pub struct Engine<'a> {
    board: &'a mut Board,
    transposition_table: Vec<Option<NodeValue>>,
    repetition_table: FnvHashSet<Zobrist>,
    best_move: EncodedMove,
    best_score: i32,
}

const CHECKMATE_SCORE: i32 = -i32::MAX + 1;

impl<'a> Engine<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            board,
            transposition_table: vec![None; TRANSPOSITION_CAPACITY],
            repetition_table: FnvHashSet::default(),
            best_move: EncodedMove::NONE,
            best_score: -i32::MAX,
        }
    }

    pub fn board(&self) -> &Board {
        self.board
    }

    fn get_phase(&self) -> i32 {
        let mut phase = 0;
        for piece in Piece::ALL_PIECES {
            let bit_board = *self.board.get_bit_board(piece);
            let piece_index = piece as usize;
            phase += bit_board.count() as i32 * eval_data::PIECE_PHASES[piece_index]
        }
        phase
    }

    fn get_piece_value(piece_index: usize, square_index: usize) -> (i32, i32) {
        let middle_game_piece_score = eval_data::MIDDLE_GAME_PIECE_VALUES[piece_index];
        let end_game_piece_score = eval_data::END_GAME_PIECE_VALUES[piece_index];

        let middle_game_piece_square_score =
            eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLES[piece_index][square_index];

        let end_game_piece_square_score =
            eval_data::END_GAME_PIECE_SQUARE_TABLES[piece_index][square_index];

        (
            middle_game_piece_score + middle_game_piece_square_score,
            end_game_piece_score + end_game_piece_square_score,
        )
    }

    fn calculate_score(phase: i32, middle_game_score: i32, end_game_score: i32) -> i32 {
        let mut middle_game_phase = phase;
        if middle_game_phase > 24 {
            middle_game_phase = 24
        };
        let end_game_phase = 24 - middle_game_phase;
        (middle_game_score * middle_game_phase + end_game_score * end_game_phase) / 24
    }

    fn evaluate(&self) -> i32 {
        let mut middle_game_score_white = 0;
        let mut end_game_score_white = 0;

        for piece in Piece::WHITE_PIECES {
            let mut bit_board = *self.board.get_bit_board(piece);
            let piece_index = piece as usize;
            while !bit_board.is_empty() {
                let square_index = bit_board.pop_square().index() as usize;

                let (middle_game_value, end_game_value) =
                    Self::get_piece_value(piece_index, square_index);

                middle_game_score_white += middle_game_value;
                end_game_score_white += end_game_value;
            }
        }

        let mut middle_game_score_black = 0;
        let mut end_game_score_black = 0;

        for piece in Piece::BLACK_PIECES {
            let mut bit_board = *self.board.get_bit_board(piece);
            let piece_index = piece as usize - 6;
            while !bit_board.is_empty() {
                let square_index = bit_board.pop_square().index() as usize;

                let (middle_game_value, end_game_value) =
                    Self::get_piece_value(piece_index, eval_data::flip(square_index));

                middle_game_score_black += middle_game_value;
                end_game_score_black += end_game_value;
            }
        }

        let phase = self.get_phase();
        Self::calculate_score(
            phase,
            middle_game_score_white - middle_game_score_black,
            end_game_score_white - end_game_score_black,
        ) * if self.board.white_to_move { 1 } else { -1 }
    }

    fn guess_move_value(&self, enemy_pawn_attacks: &BitBoard, move_data: &EncodedMove) -> i32 {
        let mut score = 0;
        match move_data.flag() {
            Flag::EnPassant => score += 0,
            Flag::PawnTwoUp => score += 0,
            Flag::BishopPromotion => score += 200,
            Flag::KnightPromotion => score += 200,
            Flag::RookPromotion => score += 400,
            Flag::QueenPromotion => score += 800,
            Flag::Castle => score += 0,
            Flag::None => score += 0,
        }

        if enemy_pawn_attacks.get(&move_data.to()) {
            score -= 50;
        }

        // This won't take into account en passant
        if let Some(capturing) = self.board.enemy_piece_at(move_data.to()) {
            let (capturing_middle_game_value, capturing_end_game_value) = {
                let capturing_piece_index = capturing as usize % 6;
                let mut capturing_square_index = move_data.to().index() as usize;
                if self.board.white_to_move {
                    capturing_square_index = eval_data::flip(capturing_square_index)
                }

                Self::get_piece_value(capturing_piece_index, capturing_square_index)
            };

            let (moving_middle_game_value, moving_end_game_value) = {
                let moving_piece_index =
                    self.board.friendly_piece_at(move_data.from()).unwrap() as usize % 6;
                let mut moving_from_index = move_data.from().index() as usize;
                if !self.board.white_to_move {
                    moving_from_index = eval_data::flip(moving_from_index)
                }
                Self::get_piece_value(moving_piece_index, moving_from_index)
            };

            let phase = self.get_phase();
            score += Self::calculate_score(
                phase,
                capturing_middle_game_value - moving_middle_game_value,
                capturing_end_game_value - moving_end_game_value,
            );
        }
        score
    }

    fn get_sorted_moves(
        &self,
        move_generator: &MoveGenerator,
        best_move: &EncodedMove,
    ) -> ([EncodedMove; 218], usize) {
        let mut moves = [EncodedMove::NONE; 218];
        let mut index = 0;
        move_generator.gen(
            &mut |move_data| {moves[index] = EncodedMove::new(move_data); index += 1},
            false,
        );

        let actual_moves = &mut moves[0..index];
        // Best moves will be at the back, so iterate in reverse later.
        actual_moves.sort_by_cached_key(|move_data| {
            if *move_data == *best_move {
                return 10000;
            }
            self.guess_move_value(&move_generator.enemy_pawn_attacks(), move_data)
        });

        (moves, index)
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = self.evaluate();
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
        self.repetition_table.insert(self.board.zobrist_key());
        self.board.make_move(move_data);
    }
    pub fn unmake_move(&mut self, move_data: &Move) {
        self.board.unmake_move(move_data);
        assert!(
            self.repetition_table.remove(&self.board.zobrist_key())
        );
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
            if saved.key != zobrist_key {
                // println!("Collision!")
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
                key: zobrist_key,
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
        let (moves, move_count) = self.get_sorted_moves(&move_generator, hash_move);

        if move_count == 0 {
            if move_generator.is_in_check() {
                if ply == 0 {
                    self.best_score = CHECKMATE_SCORE;
                }
                return CHECKMATE_SCORE
            }
            if ply == 0 {
                self.best_score = 0
            }
            return 0;
        }

        let mut node_type = NodeType::Alpha;
        let mut best_move = EncodedMove::NONE;
        for index in (0..move_count).rev() {
            let move_data = moves[index];

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
                    key: zobrist_key,
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
        }
        self.transposition_table[zobrist_index] = Some(NodeValue {
            key: zobrist_key,
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
    use crate::{board::Board, engine::Engine};

    #[test]
    fn quiescence_search_works() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        let mut quiet =
            Board::from_fen("rnb1kbnr/ppp1pppp/8/3q4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            Engine::new(&mut board).quiescence_search(-i32::MAX, i32::MAX),
            Engine::new(&mut quiet).evaluate()
        )
    }
}
