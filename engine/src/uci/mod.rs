use core::str::SplitWhitespace;

mod go_params;

use crate::{
    board::{piece::Piece, square::Square, Board},
    move_generator::move_data::{Flag, Move},
    perft::perft_root,
    search::{Search, IMMEDIATE_CHECKMATE_SCORE},
    timer::inner::Time,
};

use self::go_params::GoParameters;

/// Encodes a move in algebraic notation.
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

/// # Panics
///
/// Will panic if there is no friendly piece at `from`.
#[must_use]
pub fn decode_move(board: &Board, from: Square, to: Square, promotion: Flag) -> Move {
    let piece = board
        .friendly_piece_at(from)
        .expect("Tried to play illegal move {uci_move} on {board}");

    let mut flag = promotion;
    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
        if from.rank().abs_diff(to.rank()) == 2 {
            flag = Flag::PawnTwoUp;
        } else if board.game_state.en_passant_square == Some(to) {
            flag = Flag::EnPassant;
        }
    } else if (piece == Piece::BlackKing || piece == Piece::WhiteKing)
        && from.file().abs_diff(to.file()) > 1
    {
        flag = Flag::Castle;
    }

    Move { from, to, flag }
}

/// Handles UCI input and output.
pub struct UCIProcessor {
    /// Maximum time to search, in milliseconds.
    pub max_thinking_time: u128,

    /// FEN to be used.
    pub fen: Option<String>,

    /// Moves to be played after FEN.
    pub moves: Vec<(Square, Square, Flag)>,

    /// Search instance.
    pub search: Option<Search>,

    /// Called with UCI output.
    pub out: fn(&str),
}

impl UCIProcessor {
    /// Outputs `id` command, `option` commands, and `uciok`
    pub fn uci(&self) {
        (self.out)(
            "id name chess
id author someone
option name Hash type spin default 32 min 32 max 32
option name Threads type spin default 1 min 1 max 1
uciok",
        );
    }

    /// This should output `readyok`.
    pub fn isready(&self) {
        (self.out)("readyok");
    }

    /// # Panics
    ///
    /// Will panic if there are invalid moves.
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
                        let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
                        let (from, to) = (Square::from_notation(from), Square::from_notation(to));
                        let promotion = match uci_move.chars().nth(4) {
                            None => Flag::None,
                            Some('q') => Flag::QueenPromotion,
                            Some('r') => Flag::RookPromotion,
                            Some('n') => Flag::KnightPromotion,
                            Some('b') => Flag::BishopPromotion,
                            _ => {
                                panic!("Invalid promotion notation in {uci_move}")
                            }
                        };
                        self.moves.push((from, to, promotion));
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

    /// # Panics
    ///
    /// Will panic if there are missing parameters.
    pub fn go(&mut self, args: &mut SplitWhitespace) {
        let mut parameters = GoParameters::empty();
        parameters.parse(args);

        let mut board = Board::from_fen(self.fen.as_ref().unwrap());

        if parameters.perft {
            for (from, to, promotion) in &self.moves {
                board.make_move(&decode_move(&board, *from, *to, *promotion));
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
        for (from, to, promotion) in &self.moves {
            search.make_move(&decode_move(search.board(), *from, *to, *promotion));
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
        let output_info = |depth, best_move, evaluation| {
            if Search::score_is_checkmate(evaluation) {
                (self.out)(&format!(
                    "info depth {depth} score mate {} time {} pv {}",
                    (evaluation - IMMEDIATE_CHECKMATE_SCORE.abs()) * -evaluation.signum(),
                    search_start.miliseconds(),
                    encode_move(best_move)
                ));
            } else {
                (self.out)(&format!(
                    "info depth {depth} score cp {evaluation} time {} pv {}",
                    search_start.miliseconds(),
                    encode_move(best_move)
                ));
            }
        };
        let (depth, best_move, evaluation) = search.iterative_deepening(
            &mut |depth, (best_move, evaluation)| {
                output_info(depth, best_move.decode(), evaluation);
            },
            &mut || search_start.miliseconds() > think_time,
        );
        output_info(depth, best_move.decode(), evaluation);
        (self.out)(&format!("bestmove {}", encode_move(best_move.decode())));
    }

    /// Stop calculating as soon as possible.
    pub fn stop(&self) {
        todo!("Stop search immediately")
    }

    /// This is sent to the engine when the next search (started with "position" and "go") will be from
    /// a different game. This can be a new game the engine should play or a new game it should analyse but
    /// also the next position from a testsuite with positions only.
    pub fn ucinewgame(&mut self) {
        // New game, so old data like transposition table will not help
        if let Some(search) = &mut self.search {
            search.clear_cache_for_new_game();
        }
    }
}
