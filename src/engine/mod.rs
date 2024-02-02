mod eval_data;

use crate::{
    board::{
        piece::{self, Piece},
        Board,
    },
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

pub struct Engine<'a> {
    board: &'a mut Board,
    transposition_table: Vec<Option<NodeValue>>,
}

#[derive(Clone, Copy)]
struct NodeValue {
    depth: u16,
    node_type: NodeType,
    value: i32,
    best_move: Move,
}

#[derive(Clone, Copy)]
enum NodeType {
    Exact,
    Beta,
    Alpha,
}

const TRANSPOSITION_CAPACITY: usize = 500000;

impl<'a> Engine<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            board,
            transposition_table: vec![None; TRANSPOSITION_CAPACITY as usize],
        }
    }

    fn get_phase(&self) -> i32 {
        let mut phase = 0;
        for piece in piece::ALL_PIECES {
            let mut bit_board = *self.board.get_bit_board(piece);
            let piece_index = piece as usize;
            while !bit_board.is_empty() {
                bit_board.pop_square();

                phase += eval_data::PIECE_PHASES[piece_index]
            }
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

        for piece in piece::WHITE_PIECES {
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

        for piece in piece::BLACK_PIECES {
            let mut bit_board = *self.board.get_bit_board(piece);
            let piece_index = piece as usize - 6;
            while !bit_board.is_empty() {
                let square_index = bit_board.pop_square().index() as usize;

                let (middle_game_value, end_game_value) =
                    Self::get_piece_value(piece_index, eval_data::FLIP[square_index]);

                middle_game_score_black += middle_game_value;
                end_game_score_black += end_game_value;
            }
        }

        let phase = self.get_phase();
        Self::calculate_score(
            phase,
            middle_game_score_white - middle_game_score_black,
            end_game_score_white - end_game_score_black,
        )
    }

    fn guess_move_value(&self, move_data: &Move) -> i32 {
        let capturing = self.board.enemy_piece_at(move_data.to());
        // This won't take into account en passant
        if let Some(capturing) = capturing {
            let capturing_index = capturing as usize % 6;
            let moving_index = self.board.friendly_piece_at(move_data.from()).unwrap() as usize % 6;

            let (moving_middle_game_value, moving_end_game_value) =
                Self::get_piece_value(moving_index, move_data.to().index() as usize);

            let (capturing_middle_game_value, capturing_end_game_value) =
                Self::get_piece_value(capturing_index, move_data.to().index() as usize);

            let phase = self.get_phase();
            Self::calculate_score(
                phase,
                capturing_middle_game_value - moving_middle_game_value,
                capturing_end_game_value - moving_end_game_value,
            )
        } else {
            0
        }
    }

    fn sort_moves_ascending(&self, moves: &mut Vec<Move>, hash_move: &Move) {
        // Best moves will be at the back, so iterate in reverse later.
        moves.sort_by_cached_key(|move_data| {
            if *move_data == *hash_move {
                return 100000;
            }
            self.guess_move_value(move_data)
        });
    }

    fn quiescence_search(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = self.evaluate();
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let mut captures = Vec::with_capacity(10);
        MoveGenerator::new(&self.board).gen(&mut |move_data| {
            // TODO: This is extremely inefficient, the move generator should have a parameter to only generate captures.
            if *move_data.flag() == Flag::EnPassant {
                captures.push(move_data)
            } else {
                let capturing = self.board.enemy_piece_at(move_data.to());
                if capturing.is_some() {
                    captures.push(move_data)
                }
            }
        });
        for capture in captures {
            self.board.make_move(&capture);
            let score = -self.quiescence_search(-beta, -alpha);
            self.board.unmake_move(&capture);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    pub fn negamax(&mut self, depth: u16, mut alpha: i32, beta: i32) -> i32 {
        let zobrist_key = self.board.zobrist_key();

        let mut hash_move = &Move::none();

        // TODO: thoroughly test this works
        if let Some(saved) =
            &self.transposition_table[(zobrist_key.index(TRANSPOSITION_CAPACITY)) as usize]
        {
            if saved.depth >= depth {
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

        let mut node_type = NodeType::Alpha;

        if depth == 0 {
            let evaluation = if self.board.white_to_move {
                self.quiescence_search(alpha, beta)
            } else {
                -self.quiescence_search(alpha, beta)
            };
            self.transposition_table[zobrist_key.index(TRANSPOSITION_CAPACITY)] = Some(NodeValue {
                depth,
                node_type,
                value: evaluation,
                best_move: Move::none(),
            });
            return evaluation;
        };

        let mut moves = Vec::with_capacity(30);
        MoveGenerator::new(&self.board).gen(&mut |move_data| moves.push(move_data));
        self.sort_moves_ascending(&mut moves, &hash_move);

        let mut best_move = Move::none();
        for move_data in moves.iter().rev() {
            self.board.make_move(&move_data);
            let score = -self.negamax(depth - 1, -beta, -alpha);
            self.board.unmake_move(&move_data);
            if score >= beta {
                node_type = NodeType::Beta;
                best_move = *move_data;
                self.transposition_table[zobrist_key.index(TRANSPOSITION_CAPACITY)] =
                    Some(NodeValue {
                        depth,
                        node_type,
                        value: score,
                        best_move,
                    });
                return beta;
            }
            if score > alpha {
                node_type = NodeType::Exact;
                alpha = score;
                best_move = *move_data;
            }
        }
        self.transposition_table[zobrist_key.index(TRANSPOSITION_CAPACITY)] = Some(NodeValue {
            depth,
            node_type,
            value: alpha,
            best_move,
        });

        alpha
    }
    pub fn best_move(
        &mut self,
        depth: u16,
        should_cancel: &mut dyn FnMut() -> bool,
        best_move: Move,
    ) -> (Move, i32) {
        let mut moves = Vec::with_capacity(30);
        MoveGenerator::new(&self.board).gen(&mut |move_data| moves.push(move_data));
        self.sort_moves_ascending(&mut moves, &best_move);

        let (mut best_move, mut best_score) = (Move::none(), -i32::MAX);
        for move_data in moves.iter().rev() {
            if should_cancel() {
                break;
            }
            self.board.make_move(move_data);
            let score = -self.negamax(depth - 1, -i32::MAX, i32::MAX);
            self.board.unmake_move(move_data);
            if score > best_score {
                (best_move, best_score) = (*move_data, score);
            }
        }
        (best_move, best_score)
    }
    pub fn iterative_deepening(
        &mut self,
        depth_completed: &mut dyn FnMut(u16, (Move, i32)),
        should_cancel: &mut dyn FnMut() -> bool,
    ) -> (Move, i32) {
        let mut depth = 0;
        let (mut best_move, mut best_score) = (Move::none(), -i32::MAX);
        while !should_cancel() {
            depth += 1;
            let (new_best_move, new_best_score) = self.best_move(depth, should_cancel, best_move);
            if should_cancel() {
                if !new_best_move.is_none() {
                    best_move = new_best_move;
                    best_score = new_best_score;
                }
            } else {
                best_move = new_best_move;
                best_score = new_best_score;
                depth_completed(depth, (new_best_move, new_best_score));
            }
        }
        (best_move, best_score)
    }
}
