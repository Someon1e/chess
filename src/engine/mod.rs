mod piece_square_table;

use crate::{
    board::{
        piece::{self, Piece},
        Board,
    },
    move_generator::{move_data::Move, MoveGenerator},
};

pub struct Engine<'a> {
    move_generator: &'a mut MoveGenerator<'a>,
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
    pub fn new(move_generator: &'a mut MoveGenerator<'a>) -> Self {
        Self {
            move_generator,
            transposition_table: vec![None; TRANSPOSITION_CAPACITY as usize],
        }
    }
    pub fn board_mut(&mut self) -> &mut Board {
        self.move_generator.board_mut()
    }
    pub fn board(&self) -> &Board {
        self.move_generator.board()
    }
    pub fn move_generator(&self) -> &MoveGenerator {
        self.move_generator
    }

    fn get_absolute_white_piece_value(&self, piece: &Piece, square_index: usize) -> i32 {
        match piece {
            Piece::WhitePawn => 100 + piece_square_table::PAWN[square_index],
            Piece::WhiteKnight => 320 + piece_square_table::KNIGHT[square_index],
            Piece::WhiteBishop => 330 + piece_square_table::BISHOP[square_index],
            Piece::WhiteRook => 500 + piece_square_table::ROOK[square_index],
            Piece::WhiteQueen => 900 + piece_square_table::QUEEN[square_index],
            Piece::WhiteKing => 20000 + piece_square_table::KING[square_index],
            _ => unreachable!(),
        }
    }
    fn get_absolute_black_piece_value(&self, piece: &Piece, square_index: usize) -> i32 {
        match piece {
            Piece::BlackPawn => {
                100 + piece_square_table::PAWN[piece_square_table::FLIP[square_index]]
            }
            Piece::BlackKnight => {
                320 + piece_square_table::KNIGHT[piece_square_table::FLIP[square_index]]
            }
            Piece::BlackBishop => {
                330 + piece_square_table::BISHOP[piece_square_table::FLIP[square_index]]
            }
            Piece::BlackRook => {
                500 + piece_square_table::ROOK[piece_square_table::FLIP[square_index]]
            }
            Piece::BlackQueen => {
                900 + piece_square_table::QUEEN[piece_square_table::FLIP[square_index]]
            }
            Piece::BlackKing => {
                20000 + piece_square_table::KING[piece_square_table::FLIP[square_index]]
            }
            _ => unreachable!(),
        }
    }

    fn evaluate(&mut self) -> i32 {
        let mut score = 0;
        for piece in piece::WHITE_PIECES {
            let mut bit_board = *self.board().get_bit_board(piece);
            while !bit_board.is_empty() {
                let square = bit_board.pop_square();
                score += self.get_absolute_white_piece_value(&piece, square.index() as usize);
            }
        }
        for piece in piece::BLACK_PIECES {
            let mut bit_board = *self.board().get_bit_board(piece);
            while !bit_board.is_empty() {
                let square = bit_board.pop_square();
                score -= self.get_absolute_black_piece_value(&piece, square.index() as usize);
            }
        }
        score
    }

    fn guess_move_value(&self, move_data: &Move) -> i32 {
        let capturing = self.board().enemy_piece_at(move_data.to());
        // This won't take into account en passant
        if let Some(capturing) = capturing {
            if self.board().white_to_move {
                self.get_absolute_black_piece_value(&capturing, move_data.to().index() as usize)
            } else {
                self.get_absolute_white_piece_value(&capturing, move_data.to().index() as usize)
            }
        } else {
            0
        }
    }

    fn sort_moves(&self, moves: &mut Vec<Move>, hash_move: &Move) {
        moves.sort_by_cached_key(|move_data| {
            i32::MAX - {
                if *move_data == *hash_move {
                    return 100000;
                }
                self.guess_move_value(move_data)
            }
        });
    }

    pub fn negamax(&mut self, depth: u16, mut alpha: i32, beta: i32) -> i32 {
        let zobrist_key = self.board().zobrist_key();

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
            let evaluation = if self.board().white_to_move {
                self.evaluate()
            } else {
                -self.evaluate()
            };
            self.transposition_table[zobrist_key.index(TRANSPOSITION_CAPACITY)] = Some(NodeValue {
                depth,
                node_type,
                value: evaluation,
                best_move: Move::none(),
            });
            return evaluation;
        };

        let mut moves = Vec::with_capacity(10);
        self.move_generator.gen(&mut moves);
        self.sort_moves(&mut moves, &hash_move);

        let mut best_move = Move::none();
        for move_data in moves.iter().rev() {
            self.board_mut().make_move(&move_data);
            let score = -self.negamax(depth - 1, -beta, -alpha);
            self.board_mut().unmake_move(&move_data);
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
        let mut moves = Vec::with_capacity(10);
        self.move_generator.gen(&mut moves);
        self.sort_moves(&mut moves, &best_move);

        let (mut best_move, mut best_score) = (Move::none(), -i32::MAX);
        for move_data in moves {
            if should_cancel() {
                break;
            }
            self.board_mut().make_move(&move_data);
            let score = -self.negamax(depth - 1, -i32::MAX, i32::MAX);
            println!("{} {}", move_data, score);
            self.board_mut().unmake_move(&move_data);
            if score > best_score {
                (best_move, best_score) = (move_data, score);
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
