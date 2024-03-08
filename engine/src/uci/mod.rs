use core::str::SplitWhitespace;

mod go_params;

use crate::{
    board::{piece::Piece, square::Square, Board},
    move_generator::move_data::{Flag, Move},
    perft::perft_root,
    search::Search,
    timer::inner::Time,
};

use self::go_params::GoParameters;

#[must_use]
pub fn encode_move(move_data: Move) -> String {
    let mut encoded = String::with_capacity(4);
    encoded.push_str(&move_data.from.to_notation());
    encoded.push_str(&move_data.to.to_notation());

    match move_data.flag {
        Flag::QueenPromotion => encoded.push('q'),
        Flag::RookPromotion => encoded.push('r'),
        Flag::KnightPromotion => encoded.push('n'),
        Flag::BishopPromotion => encoded.push('b'),
        _ => {}
    };
    encoded
}

#[must_use]
pub fn decode_move(board: &Board, uci_move: &str) -> Move {
    let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
    let (from, to) = (Square::from_notation(from), Square::from_notation(to));
    let piece = board
        .friendly_piece_at(from)
        .expect("Tried to play illegal move {uci_move} on {board}");

    let mut flag = Flag::None;
    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
        if from.rank().abs_diff(to.rank()) == 2 {
            flag = Flag::PawnTwoUp;
        } else if board.game_state.en_passant_square == Some(to) {
            flag = Flag::EnPassant;
        } else if let Some(promotion) = uci_move.chars().nth(4) {
            flag = match promotion {
                'q' => Flag::QueenPromotion,
                'r' => Flag::RookPromotion,
                'n' => Flag::KnightPromotion,
                'b' => Flag::BishopPromotion,
                _ => {
                    panic!("Invalid promotion notation in {uci_move}")
                }
            }
        }
    } else if (piece == Piece::BlackKing || piece == Piece::WhiteKing)
        && from.file().abs_diff(to.file()) > 1
    {
        flag = Flag::Castle;
    }

    Move { from, to, flag }
}

pub struct UCIProcessor {
    pub max_thinking_time: u128,

    pub moves: Vec<String>,
    pub fen: Option<String>,

    pub search: Option<Search>,

    pub out: fn(&str),
}

impl UCIProcessor {
    pub fn uci(&self) {
        (self.out)(
            "id name chess
id author someone
uciok",
        );
    }
    pub fn isready(&self) {
        (self.out)("readyok");
    }
    pub fn position(&mut self, args: &mut SplitWhitespace) {
        self.moves.clear();

        let mut startpos = true;
        let mut building_fen = String::new();
        while let Some(label) = args.next() {
            match label {
                "startpos" => startpos = true,
                "fen" => startpos = false,
                "moves" => {
                    for uci_move in args.by_ref() {
                        self.moves.push(uci_move.to_owned());
                    }
                }
                _ => {
                    if !startpos {
                        building_fen.push_str(label);
                        building_fen.push(' ');
                    }
                }
            }
        }
        self.fen = Some(if startpos {
            Board::START_POSITION_FEN.to_owned()
        } else {
            building_fen
        });
    }

    pub fn go(&mut self, args: &mut SplitWhitespace) {
        let mut parameters = GoParameters::empty();
        parameters.parse(args);

        let mut board = Board::from_fen(self.fen.as_ref().unwrap());

        if parameters.perft {
            for uci_move in &self.moves {
                board.make_move(&decode_move(&board, uci_move));
            }
            (self.out)(&format!(
                "Nodes searched: {}",
                perft_root(&mut board, parameters.depth.unwrap(), self.out)
            ));
            return;
        }

        let search = if self.search.is_none() {
            // First time making search
            let search = Search::new(board);
            self.search = Some(search);
            self.search.as_mut().unwrap()
        } else {
            // Using cached search
            let search = self.search.as_mut().unwrap();
            search.new_board(board);
            search.clear_for_new_search();
            search
        };
        for uci_move in &self.moves {
            search.make_move(&decode_move(search.board(), uci_move));
        }

        let think_time = if parameters.infinite {
            self.max_thinking_time
        } else {
            parameters.move_time_in_ms.unwrap_or_else(|| {
                let clock_time = (if search.board().white_to_move {
                    parameters.white_time
                } else {
                    parameters.black_time
                })
                .unwrap();
                let increment = (if search.board().white_to_move {
                    parameters.white_increment
                } else {
                    parameters.black_increment
                })
                .unwrap_or(0);
                (clock_time / 20 + increment / 2).min(self.max_thinking_time)
            })
        };

        let search_start = Time::now();
        let (depth, best_move, evaluation) = search.iterative_deepening(
            &mut |depth, (best_move, evaluation)| {
                (self.out)(&format!(
                    "info depth {depth} score cp {evaluation} time {} pv {}",
                    search_start.miliseconds(),
                    encode_move(best_move.decode())
                ));
            },
            &mut || search_start.miliseconds() > think_time,
        );
        (self.out)(&format!(
            "info depth {depth} score cp {evaluation} time {} pv {}",
            search_start.miliseconds(),
            encode_move(best_move.decode())
        ));
        (self.out)(&format!("bestmove {}", encode_move(best_move.decode())));
    }

    pub fn stop(&self) {}
    pub fn ucinewgame(&mut self) {
        // New game, so old data like transposition table will not help
        if let Some(search) = &mut self.search {
            search.clear_cache_for_new_game();
        }
    }
}
